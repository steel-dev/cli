//! Session map: routes MCP tool calls to `steel browser` daemons.
//!
//! Each MCP-visible `sessionId` corresponds to a daemon process owning one
//! Steel cloud session. The map caches `DaemonClient` connections so tool
//! calls don't repay the socket-connect cost on every invocation.
//!
//! The default session (`mcp-default`) is auto-created the first time a
//! browser tool is invoked without an explicit `sessionId` — this matches the
//! hybrid lifecycle model: simple by default, explicit when you need it.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Result, bail};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::browser::daemon::client::DaemonClient;
use crate::browser::daemon::process;
use crate::browser::daemon::protocol::{DaemonCommand, DaemonCreateParams};
use crate::util::api;

pub const DEFAULT_SESSION: &str = "mcp-default";

/// Options accepted by `session_create`. Mirrors the subset of
/// `DaemonCreateParams` that makes sense to expose over MCP.
#[derive(Debug, Default, Clone)]
pub struct CreateOptions {
    pub stealth: bool,
    pub proxy_url: Option<String>,
    pub timeout_ms: Option<u64>,
    pub region: Option<String>,
    pub solve_captcha: bool,
    pub profile_id: Option<String>,
    pub persist_profile: bool,
    pub namespace: Option<String>,
    pub credentials: bool,
}

/// Cached `DaemonClient` connections keyed by session name (= MCP sessionId).
///
/// Cloning is cheap: the inner state is `Arc`. Tool handlers hold a clone for
/// the duration of each request.
///
/// Each `DaemonClient` lives behind its own `Mutex` so tool calls against
/// different sessions can run concurrently — the outer lock is only held for
/// the duration of a HashMap lookup or insert.
#[derive(Clone, Default)]
pub struct SessionMap {
    inner: Arc<Mutex<HashMap<String, Arc<Mutex<DaemonClient>>>>>,
}

impl SessionMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Spawn a new daemon (or attach to an existing one) and return its name.
    pub async fn create(&self, name: Option<String>, opts: CreateOptions) -> Result<String> {
        let name = name.unwrap_or_else(|| format!("mcp-{}", uuid::Uuid::new_v4()));
        if let Some(err) = process::validate_session_name(&name) {
            bail!("{err}");
        }
        self.ensure(&name, opts).await?;
        Ok(name)
    }

    /// Send a `DaemonCommand` to the given session, auto-creating the default
    /// session if none exists yet.
    pub async fn send(&self, session_id: Option<&str>, cmd: DaemonCommand) -> Result<Value> {
        let name = session_id.unwrap_or(DEFAULT_SESSION).to_string();
        self.ensure(&name, CreateOptions::default()).await?;
        let client = {
            let map = self.inner.lock().await;
            map.get(&name)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Session '{name}' disappeared"))?
        };
        let mut guard = client.lock().await;
        guard.send(cmd).await
    }

    /// Release a session: drop the cached client connection, then ask the
    /// daemon to shut down cleanly. Best-effort — daemon may already be gone.
    pub async fn release(&self, session_id: &str) -> Result<()> {
        {
            let mut map = self.inner.lock().await;
            map.remove(session_id);
        }
        let _ = process::stop_daemon(session_id).await;
        Ok(())
    }

    /// List all sessions currently tracked by this server instance.
    pub async fn list(&self) -> Vec<String> {
        let map = self.inner.lock().await;
        map.keys().cloned().collect()
    }

    /// Ensure a daemon exists for `name`. If not, spawn one with `opts` and
    /// cache the resulting client connection.
    async fn ensure(&self, name: &str, opts: CreateOptions) -> Result<()> {
        if self.inner.lock().await.contains_key(name) {
            return Ok(());
        }

        // Try to attach to an existing daemon socket first — the user may
        // have started a named session via the CLI in another terminal.
        if let Ok(client) = DaemonClient::connect(name).await {
            self.inner
                .lock()
                .await
                .insert(name.to_string(), Arc::new(Mutex::new(client)));
            return Ok(());
        }

        let (mode, base_url, auth) = api::resolve_with_auth();
        let params = DaemonCreateParams {
            api_key: auth.api_key,
            base_url,
            mode,
            session_name: name.to_string(),
            stealth: opts.stealth,
            proxy_url: opts.proxy_url,
            timeout_ms: opts.timeout_ms,
            headless: None,
            region: opts.region,
            solve_captcha: opts.solve_captcha,
            profile_id: opts.profile_id,
            persist_profile: opts.persist_profile,
            namespace: opts.namespace,
            credentials: opts.credentials,
        };

        let child = process::spawn_daemon(name, &params)?;
        process::wait_for_daemon(name, child, Duration::from_secs(30)).await?;
        let client = DaemonClient::connect(name).await?;
        self.inner
            .lock()
            .await
            .insert(name.to_string(), Arc::new(Mutex::new(client)));
        Ok(())
    }
}
