//! Thin wrapper exposing `crate::mcp` as the `steel mcp` subcommand.

pub use crate::mcp::McpArgs as Args;

pub async fn run(args: Args) -> anyhow::Result<()> {
    crate::mcp::run(args).await
}
