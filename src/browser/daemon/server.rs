use std::time::Duration;

use anyhow::Result;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;

use crate::api::client::SteelClient;
use crate::browser::engine::BrowserEngine;
use crate::browser::lifecycle::{
    get_session_created_at_ms, get_session_timeout, to_session_summary,
};
use crate::config::auth::{Auth, AuthSource};

use super::process;
use super::protocol::*;

const IDLE_TIMEOUT: Duration = Duration::from_secs(300);
const HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(30);
/// Shut down proactively this long before the cloud session is expected to expire,
/// giving us time to release cleanly rather than having the server kill the CDP socket.
const EXPIRY_BUFFER: Duration = Duration::from_secs(30);

pub async fn run(session_name: String, params: DaemonCreateParams) -> Result<()> {
    let pid_path = process::pid_path(&session_name);
    std::fs::create_dir_all(pid_path.parent().unwrap())?;
    std::fs::write(&pid_path, std::process::id().to_string())?;

    // Build API client + auth from params
    let api_client = SteelClient::new()?;
    let auth = Auth {
        api_key: params.api_key.clone(),
        source: AuthSource::Env,
    };
    let options = params.to_create_options();

    // Create the cloud session
    let session = match api_client
        .create_session(&params.base_url, params.mode, &options, &auth)
        .await
    {
        Ok(s) => s,
        Err(e) => {
            cleanup(&session_name);
            return Err(anyhow::anyhow!("{e}"));
        }
    };

    let summary = match to_session_summary(&session, params.mode, Some(&session_name), &auth) {
        Ok(s) => s,
        Err(e) => {
            cleanup(&session_name);
            return Err(e);
        }
    };

    let cdp_url = match summary.connect_url {
        Some(ref url) => url.clone(),
        None => {
            cleanup(&session_name);
            return Err(anyhow::anyhow!("Session has no CDP connect URL"));
        }
    };

    // Prefer API-reported timeout over what we requested — the server may apply defaults
    let effective_timeout = get_session_timeout(&session).or(params.timeout_ms);
    // Prefer API-reported createdAt; fall back to local clock
    let created_at_ms = get_session_created_at_ms(&session).or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .ok()
    });

    let session_info = SessionInfo {
        session_id: summary.id.clone(),
        session_name: session_name.clone(),
        mode: params.mode,
        status: summary.status,
        connect_url: Some(cdp_url.clone()),
        viewer_url: summary.viewer_url,
        profile_id: summary.profile_id,
        timeout_ms: effective_timeout,
        created_at_ms,
    };

    // Compute when the session will expire (if timeout is known).
    // Use wall-clock math: `created_at_ms + timeout - buffer` converted to a tokio Instant.
    let expires_at = match (effective_timeout, created_at_ms) {
        (Some(timeout), Some(created)) => {
            let now_epoch = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            let expire_epoch = created.saturating_add(timeout);
            let buffer = EXPIRY_BUFFER.as_millis() as u64;
            if now_epoch >= expire_epoch.saturating_sub(buffer) {
                // Already past expiry — will exit on the first loop iteration
                Some(tokio::time::Instant::now())
            } else {
                let remaining = expire_epoch
                    .saturating_sub(buffer)
                    .saturating_sub(now_epoch);
                Some(tokio::time::Instant::now() + Duration::from_millis(remaining))
            }
        }
        _ => None,
    };

    let mut engine = match BrowserEngine::connect(&cdp_url).await {
        Ok(e) => e,
        Err(e) => {
            // Best-effort release
            let _ = api_client
                .release_session(
                    &params.base_url,
                    params.mode,
                    &session_info.session_id,
                    &auth,
                )
                .await;
            cleanup(&session_name);
            return Err(e);
        }
    };

    let socket_path = process::socket_path(&session_name);
    let _ = std::fs::remove_file(&socket_path);
    let listener = match UnixListener::bind(&socket_path) {
        Ok(l) => l,
        Err(e) => {
            let _ = api_client
                .release_session(
                    &params.base_url,
                    params.mode,
                    &session_info.session_id,
                    &auth,
                )
                .await;
            cleanup(&session_name);
            return Err(e.into());
        }
    };

    let mut health_interval = tokio::time::interval(HEALTH_CHECK_INTERVAL);
    health_interval.tick().await; // discard immediate first tick

    let idle_sleep = tokio::time::sleep(IDLE_TIMEOUT);
    tokio::pin!(idle_sleep);

    let expiry_sleep = async {
        match expires_at {
            Some(deadline) => tokio::time::sleep_until(deadline).await,
            None => std::future::pending().await,
        }
    };
    tokio::pin!(expiry_sleep);

    loop {
        tokio::select! {
            result = listener.accept() => {
                idle_sleep.as_mut().reset(tokio::time::Instant::now() + IDLE_TIMEOUT);
                match result {
                    Ok((stream, _)) => {
                        match handle_connection(&mut engine, &session_info, stream).await {
                            ConnectionResult::Continue => {}
                            ConnectionResult::Shutdown => break,
                            ConnectionResult::Disconnected => {
                                eprintln!("[daemon] CDP disconnected during command, shutting down");
                                break;
                            }
                        }
                    }
                    Err(_) => continue,
                }
            }
            _ = health_interval.tick() => {
                if !engine.is_alive().await {
                    eprintln!("[daemon] CDP connection lost, shutting down");
                    break;
                }
            }
            _ = &mut idle_sleep => {
                eprintln!("[daemon] Idle timeout ({IDLE_TIMEOUT:?}), shutting down");
                break;
            }
            _ = &mut expiry_sleep => {
                eprintln!("[daemon] Session timeout reached, shutting down");
                break;
            }
        }
    }

    engine.close().await.ok();

    // Best-effort release the API session
    let _ = api_client
        .release_session(
            &params.base_url,
            params.mode,
            &session_info.session_id,
            &auth,
        )
        .await;

    cleanup(&session_name);
    Ok(())
}

enum ConnectionResult {
    Continue,
    Shutdown,
    Disconnected,
}

async fn handle_connection(
    engine: &mut BrowserEngine,
    session_info: &SessionInfo,
    stream: tokio::net::UnixStream,
) -> ConnectionResult {
    let (read_half, mut write_half) = tokio::io::split(stream);
    let mut reader = BufReader::new(read_half);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Err(_) | Ok(0) => return ConnectionResult::Continue,
            Ok(_) => {
                let request: DaemonRequest = match serde_json::from_str(&line) {
                    Ok(r) => r,
                    Err(e) => {
                        let resp = DaemonResponse {
                            id: 0,
                            result: DaemonResult::Error {
                                message: e.to_string(),
                            },
                        };
                        let json = serde_json::to_string(&resp).unwrap() + "\n";
                        let _ = write_half.write_all(json.as_bytes()).await;
                        continue;
                    }
                };

                let is_shutdown = matches!(
                    request.command,
                    DaemonCommand::Shutdown | DaemonCommand::Close
                );
                let result = dispatch(engine, session_info, request.command).await;

                // Check if CDP connection died during dispatch
                let cdp_dead =
                    matches!(&result, DaemonResult::Error { .. }) && !engine.is_alive().await;

                let resp = DaemonResponse {
                    id: request.id,
                    result,
                };
                let json = serde_json::to_string(&resp).unwrap() + "\n";
                let _ = write_half.write_all(json.as_bytes()).await;
                let _ = write_half.flush().await;

                if is_shutdown {
                    return ConnectionResult::Shutdown;
                }
                if cdp_dead {
                    return ConnectionResult::Disconnected;
                }
            }
        }
    }
}

async fn dispatch(
    engine: &mut BrowserEngine,
    session_info: &SessionInfo,
    cmd: DaemonCommand,
) -> DaemonResult {
    match dispatch_inner(engine, session_info, cmd).await {
        Ok(data) => DaemonResult::Ok { data },
        Err(e) => DaemonResult::Error {
            message: e.to_string(),
        },
    }
}

use browser_engine::native::screenshot::ScreenshotOptions;
use browser_engine::native::snapshot::SnapshotOptions;

fn build_snapshot_options(
    interactive_only: bool,
    selector: Option<String>,
    compact: bool,
    max_depth: Option<usize>,
    cursor: bool,
) -> SnapshotOptions {
    SnapshotOptions {
        selector,
        interactive: interactive_only,
        compact,
        depth: max_depth,
        cursor,
    }
}

fn build_screenshot_options(
    full_page: bool,
    selector: Option<String>,
    format: Option<String>,
    quality: Option<i32>,
    annotate: bool,
    path: Option<String>,
    screenshot_dir: Option<String>,
) -> ScreenshotOptions {
    ScreenshotOptions {
        selector,
        path,
        full_page,
        format: format.unwrap_or_else(|| "png".to_string()),
        quality,
        annotate,
        output_dir: screenshot_dir,
    }
}

async fn dispatch_inner(
    engine: &mut BrowserEngine,
    session_info: &SessionInfo,
    cmd: DaemonCommand,
) -> Result<Value> {
    match cmd {
        DaemonCommand::Navigate {
            url,
            wait_until,
            headers,
        } => {
            let result = engine
                .navigate(&url, wait_until.as_deref(), headers.as_ref())
                .await?;
            Ok(result)
        }
        DaemonCommand::Click {
            selector,
            button,
            click_count,
            new_tab,
        } => {
            engine
                .click(&selector, button.as_deref(), click_count, new_tab)
                .await?;
            if new_tab {
                Ok(json!({"clicked": selector, "newTab": true}))
            } else {
                Ok(json!({"clicked": selector}))
            }
        }
        DaemonCommand::Fill { selector, value } => {
            engine.fill(&selector, &value).await?;
            Ok(json!({"filled": selector}))
        }
        DaemonCommand::TypeText {
            selector,
            text,
            clear,
            delay_ms,
        } => {
            engine.type_text(&selector, &text, clear, delay_ms).await?;
            Ok(json!({"typed": text}))
        }
        DaemonCommand::Snapshot {
            interactive_only,
            selector,
            compact,
            max_depth,
            cursor,
        } => {
            let options =
                build_snapshot_options(interactive_only, selector, compact, max_depth, cursor);
            let text = engine.snapshot(options).await?;
            Ok(Value::String(text))
        }
        DaemonCommand::Screenshot {
            full_page,
            selector,
            format,
            quality,
            annotate,
            path,
            screenshot_dir,
        } => {
            let options = build_screenshot_options(
                full_page,
                selector,
                format,
                quality,
                annotate,
                path,
                screenshot_dir,
            );
            let result = engine.take_screenshot(options).await?;
            Ok(json!({ "path": result.path }))
        }
        DaemonCommand::Press { key } => {
            engine.press(&key).await?;
            Ok(json!({"pressed": key}))
        }
        DaemonCommand::Hover { selector } => {
            engine.hover(&selector).await?;
            Ok(json!({"hovered": selector}))
        }
        DaemonCommand::Scroll {
            selector,
            delta_x,
            delta_y,
        } => {
            engine.scroll(selector.as_deref(), delta_x, delta_y).await?;
            Ok(json!({"scrolled": true}))
        }
        DaemonCommand::Select { selector, values } => {
            engine.select(&selector, &values).await?;
            Ok(json!({"selected": values}))
        }
        DaemonCommand::Check { selector } => {
            engine.check(&selector).await?;
            Ok(json!({"checked": selector}))
        }
        DaemonCommand::Uncheck { selector } => {
            engine.uncheck(&selector).await?;
            Ok(json!({"unchecked": selector}))
        }
        DaemonCommand::Eval { script } => {
            let val = engine.evaluate(&script).await?;
            Ok(val)
        }
        DaemonCommand::GetText { selector } => {
            let text = engine.get_text(&selector).await?;
            Ok(json!({ "text": text }))
        }
        DaemonCommand::GetAttribute {
            selector,
            attribute,
        } => {
            let value = engine.get_attribute(&selector, &attribute).await?;
            Ok(json!({ "value": value }))
        }
        DaemonCommand::IsVisible { selector } => {
            let visible = engine.is_visible(&selector).await?;
            Ok(json!({ "visible": visible }))
        }
        DaemonCommand::IsEnabled { selector } => {
            let enabled = engine.is_enabled(&selector).await?;
            Ok(json!({ "enabled": enabled }))
        }
        DaemonCommand::IsChecked { selector } => {
            let checked = engine.is_checked(&selector).await?;
            Ok(json!({ "checked": checked }))
        }
        DaemonCommand::Wait {
            timeout,
            text,
            selector,
            state,
            url,
            function,
            load_state,
        } => {
            let result = engine
                .wait(
                    timeout.unwrap_or(30000),
                    text.as_deref(),
                    selector.as_deref(),
                    state.as_deref(),
                    url.as_deref(),
                    function.as_deref(),
                    load_state.as_deref(),
                )
                .await?;
            Ok(result)
        }
        DaemonCommand::Url => {
            let url = engine.url().await?;
            Ok(Value::String(url))
        }
        DaemonCommand::Title => {
            let title = engine.title().await?;
            Ok(Value::String(title))
        }
        DaemonCommand::Back => {
            engine.evaluate("history.back()").await?;
            Ok(json!({"url": engine.url().await.unwrap_or_default()}))
        }
        DaemonCommand::Forward => {
            engine.evaluate("history.forward()").await?;
            Ok(json!({"url": engine.url().await.unwrap_or_default()}))
        }
        DaemonCommand::Reload => {
            engine.evaluate("location.reload()").await?;
            Ok(json!({"url": engine.url().await.unwrap_or_default()}))
        }
        DaemonCommand::Focus { selector } => {
            engine.focus(&selector).await?;
            Ok(json!({"focused": selector}))
        }
        DaemonCommand::Clear { selector } => {
            engine.clear(&selector).await?;
            Ok(json!({"cleared": selector}))
        }
        DaemonCommand::SelectAll { selector } => {
            engine.select_all(&selector).await?;
            Ok(json!({"selected": selector}))
        }
        DaemonCommand::ScrollIntoView { selector } => {
            engine.scroll_into_view(&selector).await?;
            Ok(json!({"scrolled": selector}))
        }
        DaemonCommand::DblClick { selector } => {
            engine.dblclick(&selector).await?;
            Ok(json!({"clicked": selector}))
        }
        DaemonCommand::InnerText { selector } => {
            let text = engine.inner_text(&selector).await?;
            Ok(json!({ "text": text }))
        }
        DaemonCommand::InnerHtml { selector } => {
            let html = engine.inner_html(&selector).await?;
            Ok(json!({ "html": html }))
        }
        DaemonCommand::InputValue { selector } => {
            let value = engine.input_value(&selector).await?;
            Ok(json!({ "value": value }))
        }
        DaemonCommand::SetValue { selector, value } => {
            engine.set_value(&selector, &value).await?;
            Ok(json!({"set": selector, "value": value}))
        }
        DaemonCommand::Count { selector } => {
            let count = engine.count(&selector).await?;
            Ok(json!({ "count": count, "selector": selector }))
        }
        DaemonCommand::BoundingBox { selector } => engine.bounding_box(&selector).await,
        DaemonCommand::Styles {
            selector,
            properties,
        } => engine.styles(&selector, properties).await,
        DaemonCommand::Content => {
            let html = engine.content().await?;
            Ok(Value::String(html))
        }
        DaemonCommand::Find { selector } => engine.find(&selector).await,
        DaemonCommand::BringToFront => {
            engine.bring_to_front().await?;
            Ok(json!({"broughtToFront": true}))
        }
        DaemonCommand::TabList => {
            let data = engine.tab_list();
            Ok(data)
        }
        DaemonCommand::TabNew { url } => {
            let result = engine.tab_new(url.as_deref()).await?;
            Ok(result)
        }
        DaemonCommand::TabSwitch { index } => {
            let result = engine.tab_switch(index).await?;
            Ok(result)
        }
        DaemonCommand::TabClose { index } => {
            let result = engine.tab_close(index).await?;
            Ok(result)
        }
        // ── Cookies ──
        DaemonCommand::CookiesGet { urls } => {
            let data = engine.cookies_get(urls.as_deref()).await?;
            Ok(data)
        }
        DaemonCommand::CookiesSet {
            name,
            value,
            domain,
            path,
            secure,
            http_only,
        } => {
            engine
                .cookies_set(
                    &name,
                    &value,
                    domain.as_deref(),
                    path.as_deref(),
                    secure,
                    http_only,
                )
                .await?;
            Ok(json!({"set": true}))
        }
        DaemonCommand::CookiesClear => {
            engine.cookies_clear().await?;
            Ok(json!({"cleared": true}))
        }

        // ── Storage ──
        DaemonCommand::StorageGet { storage_type, key } => {
            let data = engine.storage_get(&storage_type, key.as_deref()).await?;
            Ok(data)
        }
        DaemonCommand::StorageSet {
            storage_type,
            key,
            value,
        } => {
            engine.storage_set(&storage_type, &key, &value).await?;
            Ok(json!({"set": true}))
        }
        DaemonCommand::StorageClear { storage_type } => {
            engine.storage_clear(&storage_type).await?;
            Ok(json!({"cleared": true}))
        }

        // ── Drag & drop ──
        DaemonCommand::Drag { source, target } => {
            engine.drag(&source, &target).await?;
            Ok(json!({"dragged": true, "source": source, "target": target}))
        }

        // ── File upload ──
        DaemonCommand::Upload { selector, files } => {
            let count = files.len();
            engine.upload_files(&selector, &files).await?;
            Ok(json!({"uploaded": count, "selector": selector}))
        }

        // ── Visual ──
        DaemonCommand::Highlight { selector } => {
            engine.highlight(&selector).await?;
            Ok(json!({"highlighted": selector}))
        }

        // ── Browser settings ──
        DaemonCommand::SetGeolocation {
            latitude,
            longitude,
            accuracy,
        } => {
            engine
                .set_geolocation(latitude, longitude, accuracy)
                .await?;
            Ok(json!({"latitude": latitude, "longitude": longitude}))
        }
        DaemonCommand::SetViewport {
            width,
            height,
            device_scale_factor,
            mobile,
        } => {
            engine
                .set_viewport(width, height, device_scale_factor, mobile)
                .await?;
            Ok(
                json!({"width": width, "height": height, "deviceScaleFactor": device_scale_factor.unwrap_or(1.0), "mobile": mobile.unwrap_or(false)}),
            )
        }
        DaemonCommand::SetUserAgent { user_agent } => {
            engine.set_user_agent(&user_agent).await?;
            Ok(json!({"userAgent": user_agent}))
        }
        DaemonCommand::SetHeaders { headers } => {
            engine.set_extra_headers(&headers).await?;
            Ok(json!({"set": true}))
        }
        DaemonCommand::SetOffline { offline } => {
            engine.set_offline(offline).await?;
            Ok(json!({"offline": offline}))
        }

        // ── Diff ──
        DaemonCommand::DiffSnapshot {
            baseline,
            selector,
            compact,
            max_depth,
        } => {
            let options = build_snapshot_options(false, selector, compact, max_depth, false);
            engine.diff_snapshot(baseline.as_deref(), options).await
        }
        DaemonCommand::DiffScreenshot {
            baseline,
            threshold,
            selector,
            full_page,
            output,
        } => {
            let options =
                build_screenshot_options(full_page, selector, None, None, false, None, None);
            engine
                .diff_screenshot(
                    &baseline,
                    threshold.unwrap_or(0.1),
                    options,
                    output.as_deref(),
                )
                .await
        }

        DaemonCommand::Close | DaemonCommand::Shutdown => Ok(json!({"closed": true})),
        DaemonCommand::Ping => Ok(json!("pong")),
        DaemonCommand::GetSessionInfo => {
            let info_json = serde_json::to_value(session_info)?;
            Ok(info_json)
        }
    }
}

fn cleanup(session_name: &str) {
    let _ = std::fs::remove_file(process::socket_path(session_name));
    let _ = std::fs::remove_file(process::pid_path(session_name));
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── build_snapshot_options field mapping ────────────────────────

    #[test]
    fn snapshot_options_interactive_only_maps_to_interactive() {
        let opts = build_snapshot_options(true, None, false, None, false);
        assert!(opts.interactive);
    }

    #[test]
    fn snapshot_options_interactive_false() {
        let opts = build_snapshot_options(false, None, false, None, false);
        assert!(!opts.interactive);
    }

    #[test]
    fn snapshot_options_max_depth_maps_to_depth() {
        let opts = build_snapshot_options(false, None, false, Some(5), false);
        assert_eq!(opts.depth, Some(5));
    }

    #[test]
    fn snapshot_options_max_depth_none() {
        let opts = build_snapshot_options(false, None, false, None, false);
        assert_eq!(opts.depth, None);
    }

    #[test]
    fn snapshot_options_selector_passthrough() {
        let opts = build_snapshot_options(false, Some("div.main".into()), false, None, false);
        assert_eq!(opts.selector.as_deref(), Some("div.main"));
    }

    #[test]
    fn snapshot_options_compact_and_cursor() {
        let opts = build_snapshot_options(false, None, true, None, true);
        assert!(opts.compact);
        assert!(opts.cursor);
    }

    #[test]
    fn snapshot_options_all_fields() {
        let opts = build_snapshot_options(true, Some("#app".into()), true, Some(3), true);
        assert!(opts.interactive);
        assert_eq!(opts.selector.as_deref(), Some("#app"));
        assert!(opts.compact);
        assert_eq!(opts.depth, Some(3));
        assert!(opts.cursor);
    }

    // ── build_screenshot_options field mapping ─────────────────────

    #[test]
    fn screenshot_options_format_defaults_to_png() {
        let opts = build_screenshot_options(false, None, None, None, false, None, None);
        assert_eq!(opts.format, "png");
    }

    #[test]
    fn screenshot_options_format_passthrough() {
        let opts =
            build_screenshot_options(false, None, Some("jpeg".into()), None, false, None, None);
        assert_eq!(opts.format, "jpeg");
    }

    #[test]
    fn screenshot_options_dir_maps_to_output_dir() {
        let opts = build_screenshot_options(
            false,
            None,
            None,
            None,
            false,
            None,
            Some("/tmp/shots".into()),
        );
        assert_eq!(opts.output_dir.as_deref(), Some("/tmp/shots"));
    }

    #[test]
    fn screenshot_options_full_page_and_annotate() {
        let opts = build_screenshot_options(true, None, None, None, true, None, None);
        assert!(opts.full_page);
        assert!(opts.annotate);
    }

    #[test]
    fn screenshot_options_quality_passthrough() {
        let opts = build_screenshot_options(false, None, None, Some(80), false, None, None);
        assert_eq!(opts.quality, Some(80));
    }

    #[test]
    fn screenshot_options_path_and_selector() {
        let opts = build_screenshot_options(
            false,
            Some("canvas".into()),
            None,
            None,
            false,
            Some("/out/img.png".into()),
            None,
        );
        assert_eq!(opts.selector.as_deref(), Some("canvas"));
        assert_eq!(opts.path.as_deref(), Some("/out/img.png"));
    }

    #[test]
    fn screenshot_options_all_fields() {
        let opts = build_screenshot_options(
            true,
            Some("body".into()),
            Some("webp".into()),
            Some(90),
            true,
            Some("/tmp/shot.webp".into()),
            Some("/tmp".into()),
        );
        assert!(opts.full_page);
        assert_eq!(opts.selector.as_deref(), Some("body"));
        assert_eq!(opts.format, "webp");
        assert_eq!(opts.quality, Some(90));
        assert!(opts.annotate);
        assert_eq!(opts.path.as_deref(), Some("/tmp/shot.webp"));
        assert_eq!(opts.output_dir.as_deref(), Some("/tmp"));
    }
}
