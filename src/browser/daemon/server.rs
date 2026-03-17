use std::time::Duration;

use anyhow::Result;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;

use crate::browser::engine::BrowserEngine;

use super::process;
use super::protocol::*;

const IDLE_TIMEOUT: Duration = Duration::from_secs(300);

pub async fn run(session_id: String, cdp_url: String) -> Result<()> {
    let pid_path = process::pid_path(&session_id);
    std::fs::create_dir_all(pid_path.parent().unwrap())?;
    std::fs::write(&pid_path, std::process::id().to_string())?;

    let mut engine = match BrowserEngine::connect(&cdp_url).await {
        Ok(e) => e,
        Err(e) => {
            cleanup(&session_id);
            return Err(e);
        }
    };

    let socket_path = process::socket_path(&session_id);
    let _ = std::fs::remove_file(&socket_path);
    let listener = UnixListener::bind(&socket_path)?;

    loop {
        match tokio::time::timeout(IDLE_TIMEOUT, listener.accept()).await {
            Err(_) => break,
            Ok(Err(_)) => continue,
            Ok(Ok((stream, _))) => {
                if handle_connection(&mut engine, stream).await {
                    break;
                }
            }
        }
    }

    engine.close().await.ok();
    cleanup(&session_id);
    Ok(())
}

/// Returns true if the daemon should shut down.
async fn handle_connection(
    engine: &mut BrowserEngine,
    stream: tokio::net::UnixStream,
) -> bool {
    let (read_half, mut write_half) = tokio::io::split(stream);
    let mut reader = BufReader::new(read_half);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Err(_) | Ok(0) => return false,
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

                let is_shutdown =
                    matches!(request.command, DaemonCommand::Shutdown | DaemonCommand::Close);
                let result = dispatch(engine, request.command).await;
                let resp = DaemonResponse {
                    id: request.id,
                    result,
                };
                let json = serde_json::to_string(&resp).unwrap() + "\n";
                let _ = write_half.write_all(json.as_bytes()).await;
                let _ = write_half.flush().await;

                if is_shutdown {
                    return true;
                }
            }
        }
    }
}

async fn dispatch(engine: &mut BrowserEngine, cmd: DaemonCommand) -> DaemonResult {
    match dispatch_inner(engine, cmd).await {
        Ok(data) => DaemonResult::Ok { data },
        Err(e) => DaemonResult::Error {
            message: e.to_string(),
        },
    }
}

async fn dispatch_inner(engine: &mut BrowserEngine, cmd: DaemonCommand) -> Result<Value> {
    use browser_engine::native::screenshot::ScreenshotOptions;
    use browser_engine::native::snapshot::SnapshotOptions;

    match cmd {
        DaemonCommand::Navigate {
            url,
            wait_until,
            headers,
        } => {
            engine
                .navigate(&url, wait_until.as_deref(), headers.as_ref())
                .await?;
            Ok(Value::Null)
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
            Ok(Value::Null)
        }
        DaemonCommand::Fill { selector, value } => {
            engine.fill(&selector, &value).await?;
            Ok(Value::Null)
        }
        DaemonCommand::TypeText {
            selector,
            text,
            clear,
            delay_ms,
        } => {
            engine.type_text(&selector, &text, clear, delay_ms).await?;
            Ok(Value::Null)
        }
        DaemonCommand::Snapshot {
            interactive_only,
            selector,
            compact,
            max_depth,
            cursor,
        } => {
            let options = SnapshotOptions {
                selector: selector,
                interactive: interactive_only,
                compact,
                depth: max_depth,
                cursor,
            };
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
            let options = ScreenshotOptions {
                selector,
                path,
                full_page,
                format: format.unwrap_or_else(|| "png".to_string()),
                quality,
                annotate,
                output_dir: screenshot_dir,
            };
            let result = engine.take_screenshot(options).await?;
            Ok(json!({ "path": result.path }))
        }
        DaemonCommand::Press { key } => {
            engine.press(&key).await?;
            Ok(Value::Null)
        }
        DaemonCommand::Hover { selector } => {
            engine.hover(&selector).await?;
            Ok(Value::Null)
        }
        DaemonCommand::Scroll {
            selector,
            delta_x,
            delta_y,
        } => {
            engine
                .scroll(selector.as_deref(), delta_x, delta_y)
                .await?;
            Ok(Value::Null)
        }
        DaemonCommand::Select { selector, values } => {
            engine.select(&selector, &values).await?;
            Ok(Value::Null)
        }
        DaemonCommand::Check { selector } => {
            engine.check(&selector).await?;
            Ok(Value::Null)
        }
        DaemonCommand::Uncheck { selector } => {
            engine.uncheck(&selector).await?;
            Ok(Value::Null)
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
            Ok(Value::Null)
        }
        DaemonCommand::Forward => {
            engine.evaluate("history.forward()").await?;
            Ok(Value::Null)
        }
        DaemonCommand::Reload => {
            engine.evaluate("location.reload()").await?;
            Ok(Value::Null)
        }
        DaemonCommand::Focus { selector } => {
            engine.focus(&selector).await?;
            Ok(Value::Null)
        }
        DaemonCommand::Clear { selector } => {
            engine.clear(&selector).await?;
            Ok(Value::Null)
        }
        DaemonCommand::SelectAll { selector } => {
            engine.select_all(&selector).await?;
            Ok(Value::Null)
        }
        DaemonCommand::ScrollIntoView { selector } => {
            engine.scroll_into_view(&selector).await?;
            Ok(Value::Null)
        }
        DaemonCommand::DblClick { selector } => {
            engine.dblclick(&selector).await?;
            Ok(Value::Null)
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
            Ok(Value::Null)
        }
        DaemonCommand::Count { selector } => {
            let count = engine.count(&selector).await?;
            Ok(json!({ "count": count, "selector": selector }))
        }
        DaemonCommand::BoundingBox { selector } => {
            engine.bounding_box(&selector).await
        }
        DaemonCommand::Styles { selector, properties } => {
            engine.styles(&selector, properties).await
        }
        DaemonCommand::Content => {
            let html = engine.content().await?;
            Ok(Value::String(html))
        }
        DaemonCommand::Find { selector } => {
            engine.find(&selector).await
        }
        DaemonCommand::BringToFront => {
            engine.bring_to_front().await?;
            Ok(Value::Null)
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
        DaemonCommand::Close | DaemonCommand::Shutdown => {
            // Engine is closed by the main loop after handle_connection returns true.
            Ok(Value::Null)
        }
        DaemonCommand::Ping => Ok(json!("pong")),
    }
}

fn cleanup(session_id: &str) {
    let _ = std::fs::remove_file(process::socket_path(session_id));
    let _ = std::fs::remove_file(process::pid_path(session_id));
}
