//! Transport wiring. Two modes:
//!
//! - `stdio` — for desktop MCP clients that spawn `steel mcp` as a
//!   subprocess (Claude Desktop, Cursor, Windsurf, Claude Code).
//! - `streamable HTTP` — for remote clients and the hosted `mcp.steel.dev`
//!   endpoint. Mounted at `/mcp` on an axum router.
//!
//! Both transports share the same `SteelMcp` service definition.

use anyhow::Result;
use rmcp::ServiceExt;
use rmcp::transport::stdio;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use tracing_subscriber::EnvFilter;

use crate::mcp::server::SteelMcp;

/// Run the MCP server over stdio. Logging is redirected to stderr so it
/// doesn't clobber the JSON-RPC stream on stdout.
pub async fn run_stdio() -> Result<()> {
    init_logging_to_stderr();
    let service = SteelMcp::new().serve(stdio()).await.inspect_err(|e| {
        tracing::error!("mcp stdio service error: {e:?}");
    })?;
    service.waiting().await?;
    Ok(())
}

/// Run the MCP server over streamable HTTP, mounted at `/mcp`.
pub async fn run_http(bind: &str, port: u16) -> Result<()> {
    init_logging_to_stdout();
    let cancel = tokio_util::sync::CancellationToken::new();

    let service = StreamableHttpService::new(
        || Ok(SteelMcp::new()),
        std::sync::Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default().with_cancellation_token(cancel.child_token()),
    );

    let router = axum::Router::new().nest_service("/mcp", service);
    let addr = format!("{bind}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("steel-mcp listening on http://{addr}/mcp");

    let serve_fut = axum::serve(listener, router).with_graceful_shutdown(async move {
        let _ = tokio::signal::ctrl_c().await;
        cancel.cancel();
    });
    serve_fut.await?;
    Ok(())
}

fn init_logging_to_stderr() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .try_init();
}

fn init_logging_to_stdout() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .try_init();
}
