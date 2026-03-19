//! Browser engine: direct integration with agent-browser native modules.
//!
//! Replaces the subprocess-spawning passthrough model with in-process
//! browser automation via CDP.

use std::collections::HashMap;

use anyhow::Result;

use browser_engine::native::browser::{BrowserManager, WaitUntil};
use browser_engine::native::cdp::client::CdpClient;
use browser_engine::native::diff;
use browser_engine::native::element::{self, RefMap};
use browser_engine::native::interaction;
use browser_engine::native::network;
use browser_engine::native::screenshot::{self, ScreenshotOptions, ScreenshotResult};
use browser_engine::native::snapshot::{self, SnapshotOptions};

/// Holds the browser state for a Steel session.
/// This replaces the daemon model — state is owned by the CLI process.
pub struct BrowserEngine {
    pub manager: BrowserManager,
    pub ref_map: RefMap,
}

impl BrowserEngine {
    /// Connect to an existing browser via CDP endpoint.
    pub async fn connect(cdp_url: &str) -> Result<Self> {
        let manager = BrowserManager::connect_cdp(cdp_url)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(Self {
            manager,
            ref_map: RefMap::new(),
        })
    }

    /// Get the active CDP client and session ID, or bail.
    fn active_client_and_session(&self) -> Result<(&CdpClient, &str)> {
        let client = &*self.manager.client;
        let session_id = self
            .manager
            .active_session_id()
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok((client, session_id))
    }

    /// Take an accessibility snapshot of the current page.
    pub async fn snapshot(&mut self, options: SnapshotOptions) -> Result<String> {
        let client = &*self.manager.client;
        let session_id = self
            .manager
            .active_session_id()
            .map_err(|e| anyhow::anyhow!(e))?;
        snapshot::take_snapshot(client, session_id, &options, &mut self.ref_map)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Navigate to a URL.
    pub async fn navigate(
        &mut self,
        url: &str,
        wait_until: Option<&str>,
        headers: Option<&HashMap<String, String>>,
    ) -> Result<serde_json::Value> {
        self.ref_map.clear();
        let wu = wait_until
            .map(WaitUntil::from_str)
            .unwrap_or(WaitUntil::Load);

        // Set scoped headers if provided
        let has_headers = headers.is_some_and(|h| !h.is_empty());
        if let Some(h) = headers
            && !h.is_empty()
        {
            let client = &*self.manager.client;
            let session_id = self
                .manager
                .active_session_id()
                .map_err(|e| anyhow::anyhow!(e))?;
            network::set_extra_headers(client, session_id, h)
                .await
                .map_err(|e| anyhow::anyhow!(e))?;
        }

        let result = self
            .manager
            .navigate(url, wu)
            .await
            .map_err(|e| anyhow::anyhow!(e));

        // Clear scoped headers after navigation
        if has_headers {
            let client = &*self.manager.client;
            if let Ok(session_id) = self.manager.active_session_id() {
                let empty = HashMap::new();
                let _ = network::set_extra_headers(client, session_id, &empty).await;
            }
        }

        result
    }

    /// Click an element by ref or CSS selector.
    pub async fn click(
        &mut self,
        selector: &str,
        button: Option<&str>,
        click_count: Option<i32>,
        new_tab: bool,
    ) -> Result<()> {
        let client = &*self.manager.client;
        let session_id = self
            .manager
            .active_session_id()
            .map_err(|e| anyhow::anyhow!(e))?;

        if new_tab {
            // Resolve element's href and open in new tab
            let object_id =
                element::resolve_element_object_id(client, session_id, &self.ref_map, selector)
                    .await
                    .map_err(|e| anyhow::anyhow!(e))?;
            let call_params = serde_json::json!({
                "objectId": object_id,
                "functionDeclaration": "function() { var h = this.getAttribute('href'); if (!h) return null; try { return new URL(h, document.baseURI).toString(); } catch(e) { return null; } }",
                "returnByValue": true
            });
            let call_result = client
                .send_command(
                    "Runtime.callFunctionOn",
                    Some(call_params),
                    Some(session_id),
                )
                .await
                .map_err(|e| anyhow::anyhow!(e))?;
            let href = call_result
                .get("result")
                .and_then(|r| r.get("value"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Element '{}' does not have an href attribute. --new-tab only works on links.",
                        selector
                    )
                })?;
            self.ref_map.clear();
            self.manager
                .tab_new(Some(href))
                .await
                .map_err(|e| anyhow::anyhow!(e))?;
            return Ok(());
        }

        interaction::click(
            client,
            session_id,
            &self.ref_map,
            selector,
            button.unwrap_or("left"),
            click_count.unwrap_or(1),
        )
        .await
        .map_err(|e| anyhow::anyhow!(e))
    }

    /// Fill an input field (clears first, then inserts text).
    pub async fn fill(&self, selector: &str, value: &str) -> Result<()> {
        let (client, session_id) = self.active_client_and_session()?;
        interaction::fill(client, session_id, &self.ref_map, selector, value)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Type text character by character (simulates keystrokes).
    pub async fn type_text(
        &self,
        selector: &str,
        text: &str,
        clear: bool,
        delay_ms: Option<u64>,
    ) -> Result<()> {
        let (client, session_id) = self.active_client_and_session()?;
        interaction::type_text(
            client,
            session_id,
            &self.ref_map,
            selector,
            text,
            clear,
            delay_ms,
        )
        .await
        .map_err(|e| anyhow::anyhow!(e))
    }

    /// Take a screenshot.
    pub async fn take_screenshot(
        &mut self,
        options: ScreenshotOptions,
    ) -> Result<ScreenshotResult> {
        let client = &*self.manager.client;
        let session_id = self
            .manager
            .active_session_id()
            .map_err(|e| anyhow::anyhow!(e))?;
        if options.annotate {
            self.ref_map.clear();
            let snap_opts = SnapshotOptions {
                interactive: true,
                ..SnapshotOptions::default()
            };
            let _ =
                snapshot::take_snapshot(client, session_id, &snap_opts, &mut self.ref_map).await;
        }
        screenshot::take_screenshot(client, session_id, &self.ref_map, &options)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Hover over an element.
    pub async fn hover(&self, selector: &str) -> Result<()> {
        let (client, session_id) = self.active_client_and_session()?;
        interaction::hover(client, session_id, &self.ref_map, selector)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Scroll the page or a specific element.
    pub async fn scroll(&self, selector: Option<&str>, delta_x: f64, delta_y: f64) -> Result<()> {
        let (client, session_id) = self.active_client_and_session()?;
        interaction::scroll(
            client,
            session_id,
            &self.ref_map,
            selector,
            delta_x,
            delta_y,
        )
        .await
        .map_err(|e| anyhow::anyhow!(e))
    }

    /// Select dropdown option(s).
    pub async fn select(&self, selector: &str, values: &[String]) -> Result<()> {
        let (client, session_id) = self.active_client_and_session()?;
        interaction::select_option(client, session_id, &self.ref_map, selector, values)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Press a key (e.g. "Enter", "Tab", "Escape").
    pub async fn press(&self, key: &str) -> Result<()> {
        let (client, session_id) = self.active_client_and_session()?;
        interaction::press_key(client, session_id, key)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Check a checkbox.
    pub async fn check(&self, selector: &str) -> Result<()> {
        let (client, session_id) = self.active_client_and_session()?;
        interaction::check(client, session_id, &self.ref_map, selector)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Uncheck a checkbox.
    pub async fn uncheck(&self, selector: &str) -> Result<()> {
        let (client, session_id) = self.active_client_and_session()?;
        interaction::uncheck(client, session_id, &self.ref_map, selector)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Get the text content of an element.
    pub async fn get_text(&self, selector: &str) -> Result<String> {
        let (client, session_id) = self.active_client_and_session()?;
        element::get_element_text(client, session_id, &self.ref_map, selector)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Get an attribute value of an element.
    pub async fn get_attribute(
        &self,
        selector: &str,
        attribute: &str,
    ) -> Result<serde_json::Value> {
        let (client, session_id) = self.active_client_and_session()?;
        element::get_element_attribute(client, session_id, &self.ref_map, selector, attribute)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Check if an element is visible.
    pub async fn is_visible(&self, selector: &str) -> Result<bool> {
        let (client, session_id) = self.active_client_and_session()?;
        element::is_element_visible(client, session_id, &self.ref_map, selector)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Check if an element is enabled.
    pub async fn is_enabled(&self, selector: &str) -> Result<bool> {
        let (client, session_id) = self.active_client_and_session()?;
        element::is_element_enabled(client, session_id, &self.ref_map, selector)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Check if a checkbox/radio is checked.
    pub async fn is_checked(&self, selector: &str) -> Result<bool> {
        let (client, session_id) = self.active_client_and_session()?;
        element::is_element_checked(client, session_id, &self.ref_map, selector)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Evaluate JavaScript in the page.
    pub async fn evaluate(&self, script: &str) -> Result<serde_json::Value> {
        self.manager
            .evaluate(script, None)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Get current page URL.
    pub async fn url(&self) -> Result<String> {
        self.manager.get_url().await.map_err(|e| anyhow::anyhow!(e))
    }

    /// Get current page title.
    pub async fn title(&self) -> Result<String> {
        self.manager
            .get_title()
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Close the browser.
    pub async fn close(&mut self) -> Result<()> {
        self.ref_map.clear();
        self.manager.close().await.map_err(|e| anyhow::anyhow!(e))
    }

    /// Wait for a condition to be met.
    pub async fn wait(
        &self,
        timeout_ms: u64,
        text: Option<&str>,
        selector: Option<&str>,
        state: Option<&str>,
        url: Option<&str>,
        function: Option<&str>,
        load_state: Option<&str>,
    ) -> Result<serde_json::Value> {
        if let Some(text) = text {
            let expr = format!(
                "(document.body.innerText || '').includes({})",
                serde_json::to_string(text).unwrap_or_default()
            );
            self.poll_until_true(&expr, timeout_ms).await?;
            return Ok(serde_json::json!({ "waited": "text", "text": text }));
        }

        if let Some(sel) = selector {
            let state_str = state.unwrap_or("visible");
            let sel_json = serde_json::to_string(sel).unwrap_or_default();
            let expr = match state_str {
                "attached" => format!("!!document.querySelector({sel_json})"),
                "detached" => format!("!document.querySelector({sel_json})"),
                "hidden" => format!(
                    r#"(() => {{
                        const el = document.querySelector({sel_json});
                        if (!el) return true;
                        const s = window.getComputedStyle(el);
                        return s.display === 'none' || s.visibility === 'hidden' || parseFloat(s.opacity) === 0;
                    }})()"#
                ),
                _ => format!(
                    r#"(() => {{
                        const el = document.querySelector({sel_json});
                        if (!el) return false;
                        const r = el.getBoundingClientRect();
                        const s = window.getComputedStyle(el);
                        return r.width > 0 && r.height > 0 && s.visibility !== 'hidden' && s.display !== 'none';
                    }})()"#
                ),
            };
            self.poll_until_true(&expr, timeout_ms).await?;
            return Ok(serde_json::json!({ "waited": "selector", "selector": sel }));
        }

        if let Some(url_pattern) = url {
            let expr = format!(
                "location.href.includes({})",
                serde_json::to_string(url_pattern).unwrap_or_default()
            );
            self.poll_until_true(&expr, timeout_ms).await?;
            return Ok(serde_json::json!({ "waited": "url", "url": url_pattern }));
        }

        if let Some(fn_str) = function {
            let expr = format!("!!({})", fn_str);
            self.poll_until_true(&expr, timeout_ms).await?;
            return Ok(serde_json::json!({ "waited": "function" }));
        }

        if let Some(ls) = load_state {
            let wu = WaitUntil::from_str(ls);
            let session_id = self
                .manager
                .active_session_id()
                .map_err(|e| anyhow::anyhow!(e))?;
            self.manager
                .wait_for_lifecycle_external(wu, session_id)
                .await
                .map_err(|e| anyhow::anyhow!(e))?;
            return Ok(serde_json::json!({ "waited": "load", "state": ls }));
        }

        // Plain timeout wait
        tokio::time::sleep(tokio::time::Duration::from_millis(timeout_ms)).await;
        Ok(serde_json::json!({ "waited": "timeout", "ms": timeout_ms }))
    }

    /// Poll a JS expression until it returns true.
    async fn poll_until_true(&self, expression: &str, timeout_ms: u64) -> Result<()> {
        let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_millis(timeout_ms);
        loop {
            let result = self.evaluate(expression).await?;
            if result.as_bool().unwrap_or(false) {
                return Ok(());
            }
            if tokio::time::Instant::now() >= deadline {
                anyhow::bail!("Wait timed out after {timeout_ms}ms");
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    /// Double-click an element.
    pub async fn dblclick(&self, selector: &str) -> Result<()> {
        let (client, session_id) = self.active_client_and_session()?;
        interaction::dblclick(client, session_id, &self.ref_map, selector)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Focus an element.
    pub async fn focus(&self, selector: &str) -> Result<()> {
        let (client, session_id) = self.active_client_and_session()?;
        interaction::focus(client, session_id, &self.ref_map, selector)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Clear an input field.
    pub async fn clear(&self, selector: &str) -> Result<()> {
        let (client, session_id) = self.active_client_and_session()?;
        interaction::clear(client, session_id, &self.ref_map, selector)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Select all text in an input.
    pub async fn select_all(&self, selector: &str) -> Result<()> {
        let (client, session_id) = self.active_client_and_session()?;
        interaction::select_all(client, session_id, &self.ref_map, selector)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Scroll an element into view.
    pub async fn scroll_into_view(&self, selector: &str) -> Result<()> {
        let (client, session_id) = self.active_client_and_session()?;
        interaction::scroll_into_view(client, session_id, &self.ref_map, selector)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Get inner text of an element.
    pub async fn inner_text(&self, selector: &str) -> Result<String> {
        let (client, session_id) = self.active_client_and_session()?;
        element::get_element_inner_text(client, session_id, &self.ref_map, selector)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Get inner HTML of an element.
    pub async fn inner_html(&self, selector: &str) -> Result<String> {
        let (client, session_id) = self.active_client_and_session()?;
        element::get_element_inner_html(client, session_id, &self.ref_map, selector)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Get the value of an input/textarea.
    pub async fn input_value(&self, selector: &str) -> Result<String> {
        let (client, session_id) = self.active_client_and_session()?;
        element::get_element_input_value(client, session_id, &self.ref_map, selector)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Set the value of an input (without events).
    pub async fn set_value(&self, selector: &str, value: &str) -> Result<()> {
        let (client, session_id) = self.active_client_and_session()?;
        element::set_element_value(client, session_id, &self.ref_map, selector, value)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Count elements matching a selector.
    pub async fn count(&self, selector: &str) -> Result<i64> {
        let (client, session_id) = self.active_client_and_session()?;
        element::get_element_count(client, session_id, selector)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Get the bounding box of an element.
    pub async fn bounding_box(&self, selector: &str) -> Result<serde_json::Value> {
        let (client, session_id) = self.active_client_and_session()?;
        element::get_element_bounding_box(client, session_id, &self.ref_map, selector)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Get CSS styles of an element.
    pub async fn styles(
        &self,
        selector: &str,
        properties: Option<Vec<String>>,
    ) -> Result<serde_json::Value> {
        let (client, session_id) = self.active_client_and_session()?;
        element::get_element_styles(client, session_id, &self.ref_map, selector, properties)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Get the page HTML content.
    pub async fn content(&self) -> Result<String> {
        self.manager
            .get_content()
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Find all elements matching a selector.
    pub async fn find(&self, selector: &str) -> Result<serde_json::Value> {
        let sel_json = serde_json::to_string(selector).unwrap_or_default();
        let script = format!(
            r#"(() => {{
                const els = document.querySelectorAll({sel_json});
                return Array.from(els).map((el, i) => ({{
                    index: i,
                    tagName: el.tagName.toLowerCase(),
                    text: el.textContent?.trim().substring(0, 100) || '',
                    visible: el.offsetWidth > 0 && el.offsetHeight > 0,
                }}));
            }})()"#
        );
        let result = self.evaluate(&script).await?;
        Ok(serde_json::json!({ "elements": result, "selector": selector }))
    }

    /// Bring the browser window to the foreground.
    pub async fn bring_to_front(&self) -> Result<()> {
        self.manager
            .bring_to_front()
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// List open tabs.
    pub fn tab_list(&self) -> serde_json::Value {
        let tabs = self.manager.tab_list();
        serde_json::json!({ "tabs": tabs })
    }

    /// Open a new tab.
    pub async fn tab_new(&mut self, url: Option<&str>) -> Result<serde_json::Value> {
        self.ref_map.clear();
        self.manager
            .tab_new(url)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Switch to a tab by index.
    pub async fn tab_switch(&mut self, index: usize) -> Result<serde_json::Value> {
        self.ref_map.clear();
        self.manager
            .tab_switch(index)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Close a tab by index (or the active tab if None).
    pub async fn tab_close(&mut self, index: Option<usize>) -> Result<serde_json::Value> {
        self.ref_map.clear();
        self.manager
            .tab_close(index)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Check if browser connection is alive.
    pub async fn is_alive(&self) -> bool {
        self.manager.is_connection_alive().await
    }

    // ── Cookies ──────────────────────────────────────────────────────

    /// Get all cookies, optionally filtered by URLs.
    pub async fn cookies_get(&self, urls: Option<&[String]>) -> Result<serde_json::Value> {
        let (client, session_id) = self.active_client_and_session()?;
        let params = match urls {
            Some(urls) if !urls.is_empty() => serde_json::json!({ "urls": urls }),
            _ => serde_json::json!({}),
        };
        let result = client
            .send_command("Network.getCookies", Some(params), Some(session_id))
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(result
            .get("cookies")
            .cloned()
            .unwrap_or(serde_json::Value::Array(vec![])))
    }

    /// Set a cookie.
    pub async fn cookies_set(
        &self,
        name: &str,
        value: &str,
        domain: Option<&str>,
        path: Option<&str>,
        secure: bool,
        http_only: bool,
    ) -> Result<()> {
        let (client, session_id) = self.active_client_and_session()?;
        let mut params = serde_json::json!({ "name": name, "value": value });
        if let Some(d) = domain {
            params["domain"] = serde_json::json!(d);
        } else {
            let url = self.manager.get_url().await.unwrap_or_default();
            params["url"] = serde_json::json!(url);
        }
        if let Some(p) = path {
            params["path"] = serde_json::json!(p);
        }
        if secure {
            params["secure"] = serde_json::json!(true);
        }
        if http_only {
            params["httpOnly"] = serde_json::json!(true);
        }
        client
            .send_command("Network.setCookie", Some(params), Some(session_id))
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(())
    }

    /// Clear all browser cookies.
    pub async fn cookies_clear(&self) -> Result<()> {
        let (client, session_id) = self.active_client_and_session()?;
        client
            .send_command("Network.clearBrowserCookies", None, Some(session_id))
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(())
    }

    // ── Storage ──────────────────────────────────────────────────────

    /// Get storage values (localStorage or sessionStorage).
    pub async fn storage_get(
        &self,
        storage_type: &str,
        key: Option<&str>,
    ) -> Result<serde_json::Value> {
        let st = if storage_type == "session" {
            "sessionStorage"
        } else {
            "localStorage"
        };
        match key {
            Some(k) => {
                let k_json = serde_json::to_string(k).unwrap_or_default();
                self.evaluate(&format!("{st}.getItem({k_json})")).await
            }
            None => {
                let script = format!(
                    r#"(() => {{ const s = {st}; const r = {{}}; for (let i = 0; i < s.length; i++) {{ const k = s.key(i); r[k] = s.getItem(k); }} return r; }})()"#
                );
                self.evaluate(&script).await
            }
        }
    }

    /// Set a storage value.
    pub async fn storage_set(&self, storage_type: &str, key: &str, value: &str) -> Result<()> {
        let st = if storage_type == "session" {
            "sessionStorage"
        } else {
            "localStorage"
        };
        let k_json = serde_json::to_string(key).unwrap_or_default();
        let v_json = serde_json::to_string(value).unwrap_or_default();
        self.evaluate(&format!("{st}.setItem({k_json}, {v_json})"))
            .await?;
        Ok(())
    }

    /// Clear all values in a storage type.
    pub async fn storage_clear(&self, storage_type: &str) -> Result<()> {
        let st = if storage_type == "session" {
            "sessionStorage"
        } else {
            "localStorage"
        };
        self.evaluate(&format!("{st}.clear()")).await?;
        Ok(())
    }

    // ── Drag ─────────────────────────────────────────────────────────

    /// Drag and drop from one element to another.
    pub async fn drag(&self, source: &str, target: &str) -> Result<()> {
        let (client, session_id) = self.active_client_and_session()?;
        let src_box = element::get_element_bounding_box(client, session_id, &self.ref_map, source)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        let tgt_box = element::get_element_bounding_box(client, session_id, &self.ref_map, target)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        let src_x =
            src_box["x"].as_f64().unwrap_or(0.0) + src_box["width"].as_f64().unwrap_or(0.0) / 2.0;
        let src_y =
            src_box["y"].as_f64().unwrap_or(0.0) + src_box["height"].as_f64().unwrap_or(0.0) / 2.0;
        let tgt_x =
            tgt_box["x"].as_f64().unwrap_or(0.0) + tgt_box["width"].as_f64().unwrap_or(0.0) / 2.0;
        let tgt_y =
            tgt_box["y"].as_f64().unwrap_or(0.0) + tgt_box["height"].as_f64().unwrap_or(0.0) / 2.0;

        // Mouse down on source
        client
            .send_command(
                "Input.dispatchMouseEvent",
                Some(serde_json::json!({
                    "type": "mousePressed", "x": src_x, "y": src_y,
                    "button": "left", "clickCount": 1
                })),
                Some(session_id),
            )
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        // Interpolate mouse moves
        let steps = 10u32;
        for i in 1..=steps {
            let t = i as f64 / steps as f64;
            let x = src_x + (tgt_x - src_x) * t;
            let y = src_y + (tgt_y - src_y) * t;
            client
                .send_command(
                    "Input.dispatchMouseEvent",
                    Some(serde_json::json!({
                        "type": "mouseMoved", "x": x, "y": y, "button": "left"
                    })),
                    Some(session_id),
                )
                .await
                .map_err(|e| anyhow::anyhow!(e))?;
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Mouse up on target
        client
            .send_command(
                "Input.dispatchMouseEvent",
                Some(serde_json::json!({
                    "type": "mouseReleased", "x": tgt_x, "y": tgt_y,
                    "button": "left", "clickCount": 1
                })),
                Some(session_id),
            )
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        Ok(())
    }

    // ── Upload ───────────────────────────────────────────────────────

    /// Upload files to a file input element.
    pub async fn upload_files(&self, selector: &str, files: &[String]) -> Result<()> {
        let (client, session_id) = self.active_client_and_session()?;
        let object_id =
            element::resolve_element_object_id(client, session_id, &self.ref_map, selector)
                .await
                .map_err(|e| anyhow::anyhow!(e))?;
        client
            .send_command(
                "DOM.setFileInputFiles",
                Some(serde_json::json!({ "objectId": object_id, "files": files })),
                Some(session_id),
            )
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(())
    }

    // ── Highlight ────────────────────────────────────────────────────

    /// Visually highlight an element.
    pub async fn highlight(&self, selector: &str) -> Result<()> {
        let (client, session_id) = self.active_client_and_session()?;
        interaction::highlight(client, session_id, &self.ref_map, selector)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    // ── Browser settings ─────────────────────────────────────────────

    /// Set browser geolocation.
    pub async fn set_geolocation(
        &self,
        latitude: f64,
        longitude: f64,
        accuracy: Option<f64>,
    ) -> Result<()> {
        let (client, session_id) = self.active_client_and_session()?;
        let params = serde_json::json!({
            "latitude": latitude,
            "longitude": longitude,
            "accuracy": accuracy.unwrap_or(1.0),
        });
        client
            .send_command(
                "Emulation.setGeolocationOverride",
                Some(params),
                Some(session_id),
            )
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(())
    }

    /// Set browser viewport size.
    pub async fn set_viewport(
        &self,
        width: u32,
        height: u32,
        device_scale_factor: Option<f64>,
        mobile: Option<bool>,
    ) -> Result<()> {
        let (client, session_id) = self.active_client_and_session()?;
        let params = serde_json::json!({
            "width": width,
            "height": height,
            "deviceScaleFactor": device_scale_factor.unwrap_or(1.0),
            "mobile": mobile.unwrap_or(false),
        });
        client
            .send_command(
                "Emulation.setDeviceMetricsOverride",
                Some(params),
                Some(session_id),
            )
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(())
    }

    /// Set browser user agent string.
    pub async fn set_user_agent(&self, user_agent: &str) -> Result<()> {
        let (client, session_id) = self.active_client_and_session()?;
        client
            .send_command(
                "Emulation.setUserAgentOverride",
                Some(serde_json::json!({ "userAgent": user_agent })),
                Some(session_id),
            )
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(())
    }

    /// Set extra HTTP headers for all requests.
    pub async fn set_extra_headers(&self, headers: &HashMap<String, String>) -> Result<()> {
        let (client, session_id) = self.active_client_and_session()?;
        network::set_extra_headers(client, session_id, headers)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    // ── Diff ─────────────────────────────────────────────────────────

    /// Diff current snapshot against a baseline string or file path.
    pub async fn diff_snapshot(
        &mut self,
        baseline: Option<&str>,
        options: SnapshotOptions,
    ) -> Result<serde_json::Value> {
        let current = self.snapshot(options).await?;
        let baseline_text = match baseline {
            Some(b) if std::path::Path::new(b).exists() => std::fs::read_to_string(b)
                .map_err(|e| anyhow::anyhow!("Failed to read baseline: {e}"))?,
            Some(b) => b.to_string(),
            None => String::new(),
        };
        let result = diff::diff_snapshots(&baseline_text, &current);
        Ok(serde_json::json!({
            "diff": result.diff,
            "additions": result.additions,
            "removals": result.removals,
            "unchanged": result.unchanged,
            "changed": result.changed,
        }))
    }

    /// Diff current screenshot against a baseline image file.
    pub async fn diff_screenshot(
        &mut self,
        baseline_path: &str,
        threshold: f64,
        options: ScreenshotOptions,
        output_path: Option<&str>,
    ) -> Result<serde_json::Value> {
        let result = self.take_screenshot(options).await?;

        let current_bytes =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &result.base64)
                .map_err(|e| anyhow::anyhow!("Failed to decode screenshot: {e}"))?;

        let baseline_bytes = std::fs::read(baseline_path)
            .map_err(|e| anyhow::anyhow!("Failed to read baseline: {e}"))?;

        let diff_result = diff::diff_screenshot(&baseline_bytes, &current_bytes, threshold)
            .map_err(|e| anyhow::anyhow!(e))?;

        if let (Some(out), Some(diff_data)) = (output_path, &diff_result.diff_image) {
            std::fs::write(out, diff_data)
                .map_err(|e| anyhow::anyhow!("Failed to write diff image: {e}"))?;
        }

        Ok(serde_json::json!({
            "match": diff_result.matched,
            "mismatchPercentage": diff_result.mismatch_percentage,
            "totalPixels": diff_result.total_pixels,
            "differentPixels": diff_result.different_pixels,
            "diffPath": output_path,
            "dimensionMismatch": diff_result.dimension_mismatch,
        }))
    }

    /// Set offline mode.
    pub async fn set_offline(&self, offline: bool) -> Result<()> {
        let (client, session_id) = self.active_client_and_session()?;
        let params = serde_json::json!({
            "offline": offline,
            "latency": 0,
            "downloadThroughput": -1,
            "uploadThroughput": -1,
        });
        client
            .send_command(
                "Network.emulateNetworkConditions",
                Some(params),
                Some(session_id),
            )
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(())
    }
}
