//! Session lifecycle: routes MCP tool calls to `steel browser` daemons.
//!
//! Each MCP-visible `sessionId` corresponds to a daemon process owning one
//! Steel cloud session. The default session (`mcp-default`) is auto-created
//! the first time a browser tool runs without an explicit `sessionId`.
//!
//! **Connection model.** The daemon's per-connection handler tears down the
//! Unix socket after 5 s of idle (it was designed for one-command CLI
//! invocations — see `handle_connection` in `browser::daemon::server`). MCP
//! tool calls arrive seconds-to-minutes apart, so caching a `DaemonClient`
//! across calls would write into a half-closed socket and surface as
//! `Broken pipe`. We follow the CLI's `ensure_daemon` pattern instead:
//! connect fresh on every call. The daemon itself stays alive; only the
//! per-call sockets are short-lived.

use std::time::Duration;

use anyhow::{Result, bail};
use serde_json::Value;

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

/// Stateless session router. Daemons own all session state (PID, socket,
/// log) on the filesystem; we just dial them on demand. Cloning is free.
#[derive(Clone, Default)]
pub struct SessionMap;

impl SessionMap {
    pub const fn new() -> Self {
        Self
    }

    /// Spawn a new daemon (or attach to an existing one with the same name).
    /// Returns the session name (= MCP sessionId).
    pub async fn create(&self, name: Option<String>, opts: CreateOptions) -> Result<String> {
        let name = name.unwrap_or_else(|| format!("mcp-{}", uuid::Uuid::new_v4()));
        if let Some(err) = process::validate_session_name(&name) {
            bail!("{err}");
        }
        ensure_daemon(&name, opts).await?;
        Ok(name)
    }

    /// Send a `DaemonCommand`. Connects fresh; the daemon expects
    /// one-command-per-connection. For the default session, auto-spawns on
    /// first use and respawns if the daemon died.
    pub async fn send(&self, session_id: Option<&str>, cmd: DaemonCommand) -> Result<Value> {
        let name = session_id.unwrap_or(DEFAULT_SESSION).to_string();
        let mut client = match DaemonClient::connect(&name).await {
            Ok(c) => c,
            Err(_) => {
                if name == DEFAULT_SESSION {
                    // Default session: auto-create on first call, respawn if dead.
                    ensure_daemon(&name, CreateOptions::default()).await?;
                    DaemonClient::connect(&name).await?
                } else {
                    bail!(
                        "Session '{name}' is not running. Call session_create first, or omit session_id to use the default session."
                    );
                }
            }
        };
        client.send(cmd).await
    }

    /// Release a session: ask the daemon to shut down cleanly. Best-effort.
    pub async fn release(&self, session_id: &str) -> Result<()> {
        process::stop_daemon(session_id).await
    }

    /// List MCP-managed sessions visible on the local filesystem.
    pub fn list(&self) -> Vec<String> {
        process::list_daemon_names()
            .into_iter()
            .filter(|n| n.starts_with("mcp-"))
            .collect()
    }
}

/// Connect to an existing daemon if alive, otherwise spawn a new one.
async fn ensure_daemon(name: &str, opts: CreateOptions) -> Result<()> {
    if DaemonClient::connect(name).await.is_ok() {
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
    Ok(())
}
