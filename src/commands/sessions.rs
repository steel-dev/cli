use std::collections::HashSet;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use futures_util::StreamExt;
use serde_json::{Value, json};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use crate::api::client::SteelClient;
use crate::api::generated::{
    GetSessionAgentLogsQuery, GetSessionAgentTracesQuery, GetSessionLogsQuery, GetSessionsQuery,
};
use crate::browser::daemon::client::DaemonClient;
use crate::browser::daemon::protocol::{DaemonCommand, SessionInfo};
use crate::config::auth::Auth;
use crate::config::settings::ApiMode;
use crate::status;
use crate::util::{api, output, style};

#[derive(Subcommand)]
pub enum Command {
    /// List cloud sessions
    List(ListArgs),

    /// Get one cloud session
    Get(GetArgs),

    /// Release a cloud session
    Release(ReleaseArgs),

    /// Read browser event logs for a session
    Logs(LogsArgs),

    /// Read raw CDP-derived agent events for a session
    AgentLogs(LogsArgs),

    /// Read semantic agent trace timeline for a session
    Traces(TracesArgs),
}

impl Command {
    pub const fn telemetry_name(&self) -> &'static str {
        match self {
            Self::List(_) => "list",
            Self::Get(_) => "get",
            Self::Release(_) => "release",
            Self::Logs(_) => "logs",
            Self::AgentLogs(_) => "agent-logs",
            Self::Traces(_) => "traces",
        }
    }
}

#[derive(Parser)]
pub struct ListArgs {
    /// Number of sessions to return
    #[arg(long)]
    pub limit: Option<u16>,

    /// Cursor ID for pagination
    #[arg(long)]
    pub cursor_id: Option<String>,

    /// Filter by status: live, released, or failed
    #[arg(long)]
    pub status: Option<String>,
}

#[derive(Parser)]
pub struct GetArgs {
    /// Cloud session ID
    pub session_id: Option<String>,

    /// Resolve session ID from a local daemon session name
    #[arg(long)]
    pub session: Option<String>,
}

#[derive(Parser)]
pub struct ReleaseArgs {
    /// Cloud session ID
    pub session_id: Option<String>,

    /// Release all active cloud sessions
    #[arg(short, long)]
    pub all: bool,

    /// Resolve session ID from a local daemon session name
    #[arg(long)]
    pub session: Option<String>,
}

#[derive(Parser)]
pub struct LogsArgs {
    /// Cloud session ID
    pub session_id: Option<String>,

    /// Resolve session ID from a local daemon session name
    #[arg(long)]
    pub session: Option<String>,

    /// Stream live frames over WebSocket after printing historical data
    #[arg(short, long)]
    pub follow: bool,

    /// Filter by namespace
    #[arg(long)]
    pub namespace: Option<String>,

    /// ISO timestamp lower bound
    #[arg(long)]
    pub start_time: Option<String>,

    /// Alias for --start-time
    #[arg(long, alias = "since")]
    pub since: Option<String>,

    /// ISO timestamp upper bound
    #[arg(long)]
    pub end_time: Option<String>,

    /// Event type filter, repeatable or comma-separated
    #[arg(long, value_delimiter = ',')]
    pub event_type: Vec<String>,

    /// Number of events to return
    #[arg(long)]
    pub limit: Option<u16>,

    /// Pagination offset
    #[arg(long)]
    pub offset: Option<u32>,
}

#[derive(Parser)]
pub struct TracesArgs {
    /// Cloud session ID
    pub session_id: Option<String>,

    /// Resolve session ID from a local daemon session name
    #[arg(long)]
    pub session: Option<String>,

    /// Stream live trace frames over WebSocket after printing historical data
    #[arg(short, long)]
    pub follow: bool,

    /// Filter by namespace
    #[arg(long)]
    pub namespace: Option<String>,

    /// ISO timestamp lower bound
    #[arg(long)]
    pub start_time: Option<String>,

    /// Alias for --start-time
    #[arg(long, alias = "since")]
    pub since: Option<String>,

    /// ISO timestamp upper bound
    #[arg(long)]
    pub end_time: Option<String>,

    /// Event type filter, repeatable or comma-separated
    #[arg(long, value_delimiter = ',')]
    pub event_type: Vec<String>,
}

#[derive(Clone, Copy)]
enum StreamKind {
    Logs,
    Traces,
}

pub async fn run(command: Command) -> Result<()> {
    match command {
        Command::List(args) => run_list(args).await,
        Command::Get(args) => run_get(args).await,
        Command::Release(args) => run_release(args).await,
        Command::Logs(args) => run_logs(args, StreamKind::Logs, "logs").await,
        Command::AgentLogs(args) => run_agent_logs(args).await,
        Command::Traces(args) => run_traces(args).await,
    }
}

async fn run_list(args: ListArgs) -> Result<()> {
    let (mode, base_url, auth) = api::resolve_with_auth();
    let client = SteelClient::new()?;
    let data = client
        .cli_get_sessions(
            &base_url,
            mode,
            &auth,
            &GetSessionsQuery {
                cursor_id: args.cursor_id,
                limit: args.limit,
                status: args.status,
            },
        )
        .await?;

    if output::is_json() {
        output::success_data(data);
    } else {
        print_sessions(&data);
    }
    Ok(())
}

async fn run_get(args: GetArgs) -> Result<()> {
    let session_id =
        resolve_session_id(args.session_id.as_deref(), args.session.as_deref()).await?;
    let (mode, base_url, auth) = api::resolve_with_auth();
    let client = SteelClient::new()?;
    let data = client
        .cli_get_session(&base_url, mode, &auth, &session_id)
        .await?;

    output::success_data(data);
    Ok(())
}

async fn run_release(args: ReleaseArgs) -> Result<()> {
    let (mode, base_url, auth) = api::resolve_with_auth();
    let client = SteelClient::new()?;

    if args.all {
        let data = client
            .cli_release_all_sessions(&base_url, mode, &auth)
            .await?;
        if output::is_json() {
            output::success_data(data);
        } else {
            println!("Released all active cloud sessions.");
        }
        return Ok(());
    }

    let session_id =
        resolve_session_id(args.session_id.as_deref(), args.session.as_deref()).await?;
    let data = client
        .cli_release_session(&base_url, mode, &auth, &session_id)
        .await?;

    if output::is_json() {
        output::success_data(data);
    } else {
        println!("Released session {session_id}.");
    }
    Ok(())
}

async fn run_logs(args: LogsArgs, stream_kind: StreamKind, label: &str) -> Result<()> {
    let session_id =
        resolve_session_id(args.session_id.as_deref(), args.session.as_deref()).await?;
    let query = GetSessionLogsQuery {
        namespace: args.namespace,
        start_time: args.start_time.or(args.since),
        end_time: args.end_time,
        event_types: args.event_type,
        limit: args.limit,
        offset: args.offset,
    };
    let (mode, base_url, auth) = api::resolve_with_auth();
    let client = SteelClient::new()?;
    let data = client
        .cli_get_session_logs(&base_url, mode, &auth, &session_id, &query)
        .await?;

    print_event_response(&data, label);
    if args.follow {
        follow_if_live(
            &client,
            &base_url,
            mode,
            &auth,
            &session_id,
            stream_kind,
            &data,
        )
        .await?;
    }
    Ok(())
}

async fn run_agent_logs(args: LogsArgs) -> Result<()> {
    let session_id =
        resolve_session_id(args.session_id.as_deref(), args.session.as_deref()).await?;
    let query = GetSessionAgentLogsQuery {
        namespace: args.namespace,
        start_time: args.start_time.or(args.since),
        end_time: args.end_time,
        event_types: args.event_type,
        limit: args.limit,
        offset: args.offset,
    };
    let (mode, base_url, auth) = api::resolve_with_auth();
    let client = SteelClient::new()?;
    let data = client
        .cli_get_session_agent_logs(&base_url, mode, &auth, &session_id, &query)
        .await?;

    print_event_response(&data, "agent logs");
    Ok(())
}

async fn run_traces(args: TracesArgs) -> Result<()> {
    let session_id =
        resolve_session_id(args.session_id.as_deref(), args.session.as_deref()).await?;
    let query = GetSessionAgentTracesQuery {
        namespace: args.namespace,
        start_time: args.start_time.or(args.since),
        end_time: args.end_time,
        event_types: args.event_type,
    };
    let (mode, base_url, auth) = api::resolve_with_auth();
    let client = SteelClient::new()?;
    let data = client
        .cli_get_session_agent_traces(&base_url, mode, &auth, &session_id, &query)
        .await?;

    print_event_response(&data, "traces");
    if args.follow {
        follow_if_live(
            &client,
            &base_url,
            mode,
            &auth,
            &session_id,
            StreamKind::Traces,
            &data,
        )
        .await?;
    }
    Ok(())
}

async fn resolve_session_id(
    session_id: Option<&str>,
    session_name: Option<&str>,
) -> Result<String> {
    if let Some(id) = session_id.map(str::trim).filter(|id| !id.is_empty()) {
        return Ok(id.to_string());
    }

    let Some(name) = session_name.map(str::trim).filter(|name| !name.is_empty()) else {
        anyhow::bail!("Missing session ID. Pass <session-id> or --session <name>.");
    };

    let mut client = DaemonClient::connect(name)
        .await
        .with_context(|| format!("No running local browser session named \"{name}\"."))?;
    let data = client.send(DaemonCommand::GetSessionInfo).await?;
    let info: SessionInfo = serde_json::from_value(data)?;
    Ok(info.session_id)
}

async fn follow_if_live(
    client: &SteelClient,
    base_url: &str,
    mode: ApiMode,
    auth: &Auth,
    session_id: &str,
    stream_kind: StreamKind,
    historical: &Value,
) -> Result<()> {
    let session = client
        .cli_get_session(base_url, mode, auth, session_id)
        .await
        .unwrap_or_else(|_| json!({}));

    if let Some(status) = session_status(&session)
        && status != "live"
    {
        status!("Session {session_id} is {status}; showing historical data only.");
        return Ok(());
    }

    let mut seen = event_ids(historical);
    let url = build_ws_url(base_url, mode, auth, stream_kind, session_id)?;
    status!("Following live session {session_id}. Press Ctrl-C to stop.");

    let (ws_stream, _) = connect_async(url.as_str()).await.with_context(|| {
        format!(
            "Failed to connect to live {} stream",
            stream_label(stream_kind)
        )
    })?;
    let (_, mut read) = ws_stream.split();

    while let Some(message) = read.next().await {
        let message = message?;
        match message {
            Message::Text(text) => print_live_payload(&text, &mut seen)?,
            Message::Binary(bytes) => {
                if let Ok(text) = String::from_utf8(bytes.to_vec()) {
                    print_live_payload(&text, &mut seen)?;
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    Ok(())
}

fn build_ws_url(
    base_url: &str,
    mode: ApiMode,
    auth: &Auth,
    stream_kind: StreamKind,
    session_id: &str,
) -> Result<String> {
    let ws_base_url = std::env::var("STEEL_WS_URL").unwrap_or_else(|_| {
        if mode == ApiMode::Cloud {
            "wss://connect.steel.dev".to_string()
        } else {
            base_url.to_string()
        }
    });
    let mut url = url::Url::parse(&ws_base_url)
        .with_context(|| format!("Invalid WebSocket URL for follow: {ws_base_url}"))?;
    let scheme = match url.scheme() {
        "wss" | "ws" => url.scheme().to_string(),
        "https" => "wss".to_string(),
        "http" => "ws".to_string(),
        other => anyhow::bail!("Unsupported API URL scheme for WebSocket follow: {other}"),
    };
    url.set_scheme(&scheme)
        .map_err(|_| anyhow::anyhow!("Failed to set WebSocket URL scheme"))?;

    let base_path = url.path().trim_end_matches('/');
    let base_path = if base_path.ends_with("/v1") {
        base_path.to_string()
    } else {
        format!("{base_path}/v1")
    };
    let suffix = match stream_kind {
        StreamKind::Logs => "logs",
        StreamKind::Traces => "agent-traces",
    };
    url.set_path(&format!("{base_path}/sessions/{session_id}/{suffix}"));
    url.set_query(None);
    if let Some(key) = auth.api_key.as_deref().filter(|key| !key.trim().is_empty()) {
        url.query_pairs_mut().append_pair("apiKey", key.trim());
    }
    Ok(url.to_string())
}

fn print_live_payload(text: &str, seen: &mut HashSet<String>) -> Result<()> {
    let value: Value = serde_json::from_str(text)?;
    for event in event_values(&value) {
        if let Some(id) = event_id(&event)
            && !seen.insert(id)
        {
            continue;
        }
        if output::is_json() {
            output::success_data(event);
        } else {
            println!("{}", format_event_line(&event));
        }
    }
    Ok(())
}

fn print_event_response(data: &Value, label: &str) {
    if output::is_json() {
        output::success_data(data.clone());
        return;
    }

    let events = event_values(data);
    if events.is_empty() {
        println!("No {label} found.");
        return;
    }

    for event in events {
        println!("{}", format_event_line(&event));
    }
}

fn print_sessions(data: &Value) {
    let sessions = session_values(data);
    if sessions.is_empty() {
        println!("No cloud sessions found.");
        return;
    }

    let max_id = sessions
        .iter()
        .filter_map(|s| s.get("id").and_then(Value::as_str))
        .map(str::len)
        .max()
        .unwrap_or(2)
        .max(2);
    println!(
        "{}",
        style::bold(&format!("{:<max_id$}  {:<10}  CREATED", "ID", "STATUS"))
    );
    for session in sessions {
        let id = session.get("id").and_then(Value::as_str).unwrap_or("-");
        let status = session.get("status").and_then(Value::as_str).unwrap_or("-");
        let created = session
            .get("createdAt")
            .or_else(|| session.get("created_at"))
            .and_then(Value::as_str)
            .unwrap_or("-");
        let status_cell = colorize_status(status, &format!("{status:<10}"));
        println!("{id:<max_id$}  {status_cell}  {created}");
    }
}

/// Color a session status cell by lifecycle: live (green), failed (red),
/// released (dim). The cell is pre-padded so alignment is preserved regardless
/// of whether color is emitted.
fn colorize_status(status: &str, padded: &str) -> String {
    match status {
        "live" => style::green(padded),
        "failed" => style::red(padded),
        "released" => style::dim(padded),
        _ => padded.to_string(),
    }
}

fn session_values(data: &Value) -> Vec<Value> {
    if let Some(arr) = data.as_array() {
        return arr.clone();
    }
    if let Some(arr) = data.get("sessions").and_then(Value::as_array) {
        return arr.clone();
    }
    if let Some(session) = data.get("session").filter(|v| v.is_object()) {
        return vec![session.clone()];
    }
    if data.get("id").is_some() {
        return vec![data.clone()];
    }
    Vec::new()
}

fn event_values(data: &Value) -> Vec<Value> {
    if let Some(arr) = data.as_array() {
        return arr.clone();
    }
    if let Some(arr) = data.get("events").and_then(Value::as_array) {
        return arr.clone();
    }
    if data.is_object() {
        return vec![data.clone()];
    }
    Vec::new()
}

fn event_ids(data: &Value) -> HashSet<String> {
    event_values(data)
        .into_iter()
        .filter_map(|event| event_id(&event))
        .collect()
}

fn event_id(event: &Value) -> Option<String> {
    event
        .get("id")
        .or_else(|| event.get("eventId"))
        .or_else(|| event.get("traceId"))
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn session_status(data: &Value) -> Option<&str> {
    data.get("status")
        .or_else(|| {
            data.get("session")
                .and_then(|session| session.get("status"))
        })
        .and_then(Value::as_str)
}

fn format_event_line(event: &Value) -> String {
    let timestamp = event
        .get("timestamp")
        .or_else(|| event.get("time"))
        .or_else(|| event.get("createdAt"))
        .and_then(Value::as_str)
        .unwrap_or("-");
    let kind = event
        .get("type")
        .or_else(|| event.get("action"))
        .or_else(|| event.get("method"))
        .or_else(|| event.get("name"))
        .and_then(Value::as_str)
        .unwrap_or("event");
    let detail = event
        .get("log")
        .or_else(|| event.get("message"))
        .or_else(|| event.get("text"))
        .or_else(|| event.get("url"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| compact_json(event));

    let timestamp = style::dim(timestamp);
    let kind = colorize_kind(kind, &format!("{kind:<18}"));
    format!("{timestamp}  {kind}  {detail}")
}

/// Color an event/trace type by semantics so failures and warnings stand out in
/// a fast-scrolling stream. The label is pre-padded so columns stay aligned
/// whether or not color is emitted.
fn colorize_kind(kind: &str, padded: &str) -> String {
    let lower = kind.to_ascii_lowercase();
    if lower.contains("error") || lower.contains("fail") || lower.contains("exception") {
        style::red(padded)
    } else if lower.contains("warn") {
        style::yellow(padded)
    } else if lower.contains("nav")
        || lower.contains("request")
        || lower.contains("response")
        || lower.contains("network")
        || lower.contains("goto")
    {
        style::blue(padded)
    } else {
        style::cyan(padded)
    }
}

fn compact_json(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "<unprintable event>".to_string())
}

const fn stream_label(stream_kind: StreamKind) -> &'static str {
    match stream_kind {
        StreamKind::Logs => "logs",
        StreamKind::Traces => "traces",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ws_url_uses_cloud_connect_host_and_adds_api_key() {
        let url = build_ws_url(
            "https://api.steel.dev/v1",
            ApiMode::Cloud,
            &Auth {
                api_key: Some("sk_test".into()),
                source: crate::config::auth::AuthSource::Env,
            },
            StreamKind::Traces,
            "sess",
        )
        .unwrap();
        assert_eq!(
            url,
            "wss://connect.steel.dev/v1/sessions/sess/agent-traces?apiKey=sk_test"
        );
    }

    #[test]
    fn ws_url_preserves_local_api_host() {
        let url = build_ws_url(
            "http://localhost:3000/v1",
            ApiMode::Local,
            &Auth {
                api_key: Some("sk_test".into()),
                source: crate::config::auth::AuthSource::Env,
            },
            StreamKind::Logs,
            "sess",
        )
        .unwrap();
        assert_eq!(
            url,
            "ws://localhost:3000/v1/sessions/sess/logs?apiKey=sk_test"
        );
    }

    #[test]
    fn event_values_extracts_events_array() {
        let values = event_values(&json!({"events": [{"id": "1"}, {"id": "2"}]}));
        assert_eq!(values.len(), 2);
    }
}
