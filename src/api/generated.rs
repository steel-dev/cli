//! Generated bindings for CLI-tagged Steel API operations.
//!
//! Source: OpenAPI operations with `x-steel-cli`.
//! Regenerate with `npm run api:generate`.
//! Keep command UX in `commands/`; this module only owns request shape,
//! endpoint paths, and operation metadata.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::client::{ApiError, SteelClient};
use crate::config::auth::Auth;
use crate::config::settings::ApiMode;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OperationMetadata {
    pub id: &'static str,
    pub command: &'static str,
    pub status: &'static str,
    pub method: &'static str,
    pub path: &'static str,
    pub summary: &'static str,
    pub example: &'static str,
    pub streaming: Option<StreamingMetadata>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StreamingMetadata {
    pub transport: &'static str,
    pub path: &'static str,
}

pub const CLI_OPERATION_METADATA: &[OperationMetadata] = &[
    OperationMetadata {
        id: "get_session_agent_logs",
        command: "sessions agent-logs",
        status: "implemented",
        method: "GET",
        path: "/v1/sessions/{id}/agent-logs",
        summary: "Get session agent logs",
        example: "steel sessions agent-logs <session-id> --limit 100",
        streaming: None,
    },
    OperationMetadata {
        id: "get_session",
        command: "sessions get",
        status: "implemented",
        method: "GET",
        path: "/v1/sessions/{id}",
        summary: "Get session details",
        example: "steel sessions get <session-id>",
        streaming: None,
    },
    OperationMetadata {
        id: "get_sessions",
        command: "sessions list",
        status: "implemented",
        method: "GET",
        path: "/v1/sessions",
        summary: "List all sessions",
        example: "steel sessions list --status live --limit 20",
        streaming: None,
    },
    OperationMetadata {
        id: "get_session_logs",
        command: "sessions logs",
        status: "implemented",
        method: "GET",
        path: "/v1/sessions/{id}/logs",
        summary: "Get session logs",
        example: "steel sessions logs <session-id> --follow",
        streaming: Some(StreamingMetadata {
            transport: "websocket",
            path: "/v1/sessions/{id}/logs",
        }),
    },
    OperationMetadata {
        id: "release_session",
        command: "sessions release",
        status: "implemented",
        method: "POST",
        path: "/v1/sessions/{id}/release",
        summary: "Release a session",
        example: "steel sessions release <session-id>",
        streaming: None,
    },
    OperationMetadata {
        id: "release_all_sessions",
        command: "sessions release --all",
        status: "implemented",
        method: "POST",
        path: "/v1/sessions/release",
        summary: "Release all sessions",
        example: "steel sessions release --all",
        streaming: None,
    },
    OperationMetadata {
        id: "get_session_agent_traces",
        command: "sessions traces",
        status: "implemented",
        method: "GET",
        path: "/v1/sessions/{id}/agent-traces",
        summary: "Get session agent traces",
        example: "steel sessions traces <session-id> --follow",
        streaming: Some(StreamingMetadata {
            transport: "websocket",
            path: "/v1/sessions/{id}/agent-traces",
        }),
    },
];

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSessionAgentLogsQuery {
    pub namespace: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub event_types: Vec<String>,
    pub limit: Option<u16>,
    pub offset: Option<u32>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSessionsQuery {
    pub cursor_id: Option<String>,
    pub limit: Option<u16>,
    pub status: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSessionLogsQuery {
    pub namespace: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub event_types: Vec<String>,
    pub limit: Option<u16>,
    pub offset: Option<u32>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSessionAgentTracesQuery {
    pub namespace: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub event_types: Vec<String>,
}

fn push_query(path: &mut String, key: &str, value: &str) {
    if path.contains('?') {
        path.push('&');
    } else {
        path.push('?');
    }
    path.push_str(key);
    path.push('=');
    path.push_str(&urlencoding::encode(value));
}

fn push_query_number(path: &mut String, key: &str, value: impl std::fmt::Display) {
    push_query(path, key, &value.to_string());
}

fn build_get_session_agent_logs_path(session_id: &str, query: &GetSessionAgentLogsQuery) -> String {
    let mut path = format!("/sessions/{session_id}/agent-logs");
    if let Some(value) = query
        .namespace
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        push_query(&mut path, "namespace", value.trim());
    }
    if let Some(value) = query
        .start_time
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        push_query(&mut path, "startTime", value.trim());
    }
    if let Some(value) = query
        .end_time
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        push_query(&mut path, "endTime", value.trim());
    }
    for value in query
        .event_types
        .iter()
        .filter(|value| !value.trim().is_empty())
    {
        push_query(&mut path, "eventTypes", value.trim());
    }
    if let Some(value) = query.limit {
        push_query_number(&mut path, "limit", value);
    }
    if let Some(value) = query.offset {
        push_query_number(&mut path, "offset", value);
    }
    path
}

fn build_get_session_path(session_id: &str) -> String {
    format!("/sessions/{session_id}")
}

fn build_get_sessions_path(query: &GetSessionsQuery) -> String {
    let mut path = "/sessions".to_string();
    if let Some(value) = query
        .cursor_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        push_query(&mut path, "cursorId", value.trim());
    }
    if let Some(value) = query.limit {
        push_query_number(&mut path, "limit", value);
    }
    if let Some(value) = query
        .status
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        push_query(&mut path, "status", value.trim());
    }
    path
}

fn build_get_session_logs_path(session_id: &str, query: &GetSessionLogsQuery) -> String {
    let mut path = format!("/sessions/{session_id}/logs");
    if let Some(value) = query
        .namespace
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        push_query(&mut path, "namespace", value.trim());
    }
    if let Some(value) = query
        .start_time
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        push_query(&mut path, "startTime", value.trim());
    }
    if let Some(value) = query
        .end_time
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        push_query(&mut path, "endTime", value.trim());
    }
    for value in query
        .event_types
        .iter()
        .filter(|value| !value.trim().is_empty())
    {
        push_query(&mut path, "eventTypes", value.trim());
    }
    if let Some(value) = query.limit {
        push_query_number(&mut path, "limit", value);
    }
    if let Some(value) = query.offset {
        push_query_number(&mut path, "offset", value);
    }
    path
}

fn build_release_session_path(session_id: &str) -> String {
    format!("/sessions/{session_id}/release")
}

fn build_get_session_agent_traces_path(
    session_id: &str,
    query: &GetSessionAgentTracesQuery,
) -> String {
    let mut path = format!("/sessions/{session_id}/agent-traces");
    if let Some(value) = query
        .namespace
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        push_query(&mut path, "namespace", value.trim());
    }
    if let Some(value) = query
        .start_time
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        push_query(&mut path, "startTime", value.trim());
    }
    if let Some(value) = query
        .end_time
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        push_query(&mut path, "endTime", value.trim());
    }
    for value in query
        .event_types
        .iter()
        .filter(|value| !value.trim().is_empty())
    {
        push_query(&mut path, "eventTypes", value.trim());
    }
    path
}

impl SteelClient {
    pub async fn cli_get_session_agent_logs(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
        session_id: &str,
        query: &GetSessionAgentLogsQuery,
    ) -> Result<Value, ApiError> {
        self.request(
            base_url,
            mode,
            reqwest::Method::GET,
            &build_get_session_agent_logs_path(session_id, query),
            None,
            auth,
        )
        .await
    }

    pub async fn cli_get_session(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
        session_id: &str,
    ) -> Result<Value, ApiError> {
        self.request(
            base_url,
            mode,
            reqwest::Method::GET,
            &build_get_session_path(session_id),
            None,
            auth,
        )
        .await
    }

    pub async fn cli_get_sessions(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
        query: &GetSessionsQuery,
    ) -> Result<Value, ApiError> {
        self.request(
            base_url,
            mode,
            reqwest::Method::GET,
            &build_get_sessions_path(query),
            None,
            auth,
        )
        .await
    }

    pub async fn cli_get_session_logs(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
        session_id: &str,
        query: &GetSessionLogsQuery,
    ) -> Result<Value, ApiError> {
        self.request(
            base_url,
            mode,
            reqwest::Method::GET,
            &build_get_session_logs_path(session_id, query),
            None,
            auth,
        )
        .await
    }

    pub async fn cli_release_session(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
        session_id: &str,
    ) -> Result<Value, ApiError> {
        self.request(
            base_url,
            mode,
            reqwest::Method::POST,
            &build_release_session_path(session_id),
            None,
            auth,
        )
        .await
    }

    pub async fn cli_release_all_sessions(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
    ) -> Result<Value, ApiError> {
        self.request(
            base_url,
            mode,
            reqwest::Method::POST,
            "/sessions/release",
            None,
            auth,
        )
        .await
    }

    pub async fn cli_get_session_agent_traces(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
        session_id: &str,
        query: &GetSessionAgentTracesQuery,
    ) -> Result<Value, ApiError> {
        self.request(
            base_url,
            mode,
            reqwest::Method::GET,
            &build_get_session_agent_traces_path(session_id, query),
            None,
            auth,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_list_query_uses_openapi_names() {
        let path = build_get_sessions_path(&GetSessionsQuery {
            cursor_id: Some("abc".into()),
            limit: Some(25),
            status: Some("live".into()),
        });
        assert_eq!(path, "/sessions?cursorId=abc&limit=25&status=live");
    }

    #[test]
    fn logs_query_repeats_event_types() {
        let path = build_get_session_logs_path(
            "sess",
            &GetSessionLogsQuery {
                namespace: Some("prod".into()),
                event_types: vec!["console".into(), "error".into()],
                limit: Some(10),
                offset: Some(5),
                ..Default::default()
            },
        );
        assert_eq!(
            path,
            "/sessions/sess/logs?namespace=prod&eventTypes=console&eventTypes=error&limit=10&offset=5"
        );
    }
}
