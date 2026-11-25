use std::io::{self, IsTerminal, Read};

use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use reqwest::Method;
use serde_json::{Value, json};

use crate::api::client::{ApiError, SteelClient};
use crate::util::output;

#[derive(Args)]
pub struct Project {
    /// Project ID (defaults to the current API principal's project)
    #[arg(long)]
    project_id: Option<String>,
}

#[derive(Args)]
pub struct Resource {
    /// Resource ID
    id: String,
    #[command(flatten)]
    project: Project,
}

#[derive(Args)]
pub struct VaultFields {
    /// Vault description
    #[arg(long)]
    description: Option<String>,
    /// Remove the Vault description
    #[arg(long, conflicts_with = "description")]
    clear_description: bool,
}

#[derive(Subcommand)]
pub enum VaultCommand {
    /// List Steel Vaults
    List(Project),
    /// Get a Steel Vault by ID
    Get(Resource),
    /// Create a Steel Vault
    Create {
        name: String,
        #[arg(long)]
        description: Option<String>,
        #[command(flatten)]
        project: Project,
    },
    /// Rename a Steel Vault or update its description
    Update {
        #[command(flatten)]
        resource: Resource,
        #[arg(long)]
        name: Option<String>,
        #[command(flatten)]
        fields: VaultFields,
    },
    /// Delete an empty Steel Vault
    Delete(Resource),
}

#[derive(Subcommand)]
pub enum ConnectionCommand {
    /// List saved Secret Connections
    List(Project),
    /// Get a saved Secret Connection by ID
    Get(Resource),
    /// Create a connection from JSON on stdin (name, provider, auth, optional config and tokenBudget)
    Create(Project),
    /// Update a connection from JSON on stdin (name, enabled, auth, config, or tokenBudget)
    Update(Resource),
    /// Delete a saved Secret Connection
    Delete(Resource),
    /// Test connectivity of a saved connection, not credential access
    Test(Resource),
}

impl VaultCommand {
    pub const fn telemetry_name(&self) -> &'static str {
        match self {
            Self::List(_) => "list",
            Self::Get(_) => "get",
            Self::Create { .. } => "create",
            Self::Update { .. } => "update",
            Self::Delete(_) => "delete",
        }
    }
}

impl ConnectionCommand {
    pub const fn telemetry_name(&self) -> &'static str {
        match self {
            Self::List(_) => "list",
            Self::Get(_) => "get",
            Self::Create(_) => "create",
            Self::Update(_) => "update",
            Self::Delete(_) => "delete",
            Self::Test(_) => "test",
        }
    }
}

pub(super) fn id_segment(id: &str) -> Result<&str> {
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
        bail!("Invalid resource ID.");
    }
    Ok(id)
}

fn project_path(path: &str, project: Project) -> String {
    match project.project_id {
        Some(id) => format!("{path}?projectId={}", urlencoding::encode(&id)),
        None => path.to_owned(),
    }
}

fn resource_path(collection: &str, resource: Resource, suffix: &str) -> Result<String> {
    Ok(project_path(
        &format!("{collection}/{}{suffix}", id_segment(&resource.id)?),
        resource.project,
    ))
}

pub(super) async fn send(method: Method, path: &str, body: Option<Value>) -> Result<()> {
    let result = SteelClient::new()?.request(method, path, body).await?;
    if result.is_null() {
        output::success_silent();
    } else {
        output::success_data(result);
    }
    Ok(())
}

pub async fn run_vault(command: VaultCommand) -> Result<()> {
    const PATH: &str = "/vaults";
    let (method, path, body) = match command {
        VaultCommand::List(project) => (Method::GET, project_path(PATH, project), None),
        VaultCommand::Get(resource) => (Method::GET, resource_path(PATH, resource, "")?, None),
        VaultCommand::Delete(resource) => {
            (Method::DELETE, resource_path(PATH, resource, "")?, None)
        }
        VaultCommand::Create {
            name,
            description,
            project,
        } => {
            let mut body = json!({ "name": name });
            if let Some(description) = description {
                body["description"] = json!(description);
            }
            if let Some(project_id) = project.project_id {
                body["projectId"] = json!(project_id);
            }
            (Method::POST, PATH.to_owned(), Some(body))
        }
        VaultCommand::Update {
            resource,
            name,
            fields,
        } => {
            let mut body = json!({});
            if let Some(name) = name {
                body["name"] = json!(name);
            }
            if fields.clear_description {
                body["description"] = Value::Null;
            }
            if let Some(description) = fields.description {
                body["description"] = json!(description);
            }
            if body == json!({}) {
                bail!("Specify a name or description to update.");
            }
            (
                Method::PATCH,
                resource_path(PATH, resource, "")?,
                Some(body),
            )
        }
    };
    send(method, &path, body).await
}

fn connection_input(allowed: &[&str]) -> Result<Value> {
    const MAX_BYTES: usize = 1024 * 1024;
    let stdin = io::stdin();
    if stdin.is_terminal() {
        bail!(
            "Pipe connection JSON on stdin from a trusted source; do not put provider authentication on the command line."
        );
    }
    let mut bytes = Vec::new();
    stdin
        .lock()
        .take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .context("Could not read connection JSON from stdin.")?;
    if bytes.len() > MAX_BYTES {
        bail!("Connection JSON exceeds 1 MiB.");
    }
    let body: Value = serde_json::from_slice(&bytes)
        .map_err(|_| anyhow::anyhow!("Expected a JSON object on stdin."))?;
    let fields = body
        .as_object()
        .filter(|fields| !fields.is_empty())
        .context("Expected a nonempty JSON object on stdin.")?;
    if fields.keys().any(|key| !allowed.contains(&key.as_str())) {
        bail!(
            "Unsupported field in connection JSON. Supported fields: {}.",
            allowed.join(", ")
        );
    }
    Ok(body)
}

async fn write_connection(method: Method, path: &str, body: Value) -> Result<()> {
    let client = SteelClient::new()?;
    // Do not print upstream error bodies from requests carrying provider authentication.
    let response = client.request(method, path, Some(body)).await.map_err(|error| match error {
        ApiError::RequestFailed { status, .. } => anyhow::anyhow!("Secret Connection request failed (HTTP {status}). Check the connection configuration and project access."),
        ApiError::Redirected { .. } => anyhow::anyhow!("Secret Connection request was redirected. Use the final API URL."),
        other => other.into(),
    })?;
    output::success_data(response);
    Ok(())
}

pub async fn run_connection(command: ConnectionCommand) -> Result<()> {
    const PATH: &str = "/secret-connections";
    match command {
        ConnectionCommand::List(project) => {
            send(Method::GET, &project_path(PATH, project), None).await
        }
        ConnectionCommand::Get(resource) => {
            send(Method::GET, &resource_path(PATH, resource, "")?, None).await
        }
        ConnectionCommand::Delete(resource) => {
            send(Method::DELETE, &resource_path(PATH, resource, "")?, None).await
        }
        ConnectionCommand::Test(resource) => {
            send(Method::POST, &resource_path(PATH, resource, "/test")?, None).await
        }
        ConnectionCommand::Create(project) => {
            let mut body =
                connection_input(&["name", "provider", "auth", "config", "tokenBudget"])?;
            if let Some(id) = project.project_id {
                body["projectId"] = json!(id);
            }
            write_connection(Method::POST, PATH, body).await
        }
        ConnectionCommand::Update(resource) => {
            let body = connection_input(&["name", "enabled", "auth", "config", "tokenBudget"])?;
            write_connection(Method::PATCH, &resource_path(PATH, resource, "")?, body).await
        }
    }
}

#[derive(Args)]
pub struct Bindings {
    /// ID of a Steel-stored credential (repeatable)
    #[arg(long = "credential-id")]
    credential_ids: Vec<String>,
    /// Item or Vault reference, or a JSON binding with allowedOrigins (repeatable)
    #[arg(long = "binding", value_parser = parse_binding)]
    bindings: Vec<Value>,
}

fn parse_binding(input: &str) -> Result<Value, String> {
    if input.starts_with("steel://") {
        return Ok(json!(input));
    }
    #[derive(serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct OriginBinding {
        reference: String,
        allowed_origins: Vec<String>,
    }
    let binding = serde_json::from_str::<OriginBinding>(input).map_err(
        |_| "Expected a steel:// reference or a JSON object with reference and allowedOrigins.",
    )?;
    serde_json::to_value(binding).map_err(|_| "Invalid binding.".to_owned())
}

#[derive(Subcommand)]
pub enum SessionCommand {
    /// List a session's credential bindings and lookup mode (no credential values)
    List { session_id: String },
    /// Attach bindings and switch the session to explicit lookup
    Attach {
        session_id: String,
        #[command(flatten)]
        targets: Bindings,
    },
    /// Replace all bindings; --clear leaves lookup explicit with no bindings
    Replace {
        session_id: String,
        #[command(flatten)]
        targets: Bindings,
        #[arg(long, conflicts_with_all = ["credential_ids", "bindings"])]
        clear: bool,
    },
    /// Detach an individual credential by ID
    Detach {
        session_id: String,
        credential_id: String,
    },
    /// Detach a Vault or item reference without changing other bindings
    DetachBinding {
        session_id: String,
        reference: String,
    },
}

impl SessionCommand {
    pub const fn telemetry_name(&self) -> &'static str {
        match self {
            Self::List { .. } => "credentials.list",
            Self::Attach { .. } => "credentials.attach",
            Self::Replace { .. } => "credentials.replace",
            Self::Detach { .. } => "credentials.detach",
            Self::DetachBinding { .. } => "credentials.detach-binding",
        }
    }
}

fn session_path(session_id: &str) -> Result<String> {
    Ok(format!("/sessions/{}/credentials", id_segment(session_id)?))
}

fn binding_body(targets: Bindings, clear: bool) -> Result<Value> {
    if targets.credential_ids.is_empty() && targets.bindings.is_empty() && !clear {
        bail!("Supply at least one --credential-id or --binding.");
    }
    Ok(json!({ "credentialIds": targets.credential_ids, "bindings": targets.bindings }))
}

pub async fn run_session(command: SessionCommand) -> Result<()> {
    let (method, path, body) = match command {
        SessionCommand::List { session_id } => (Method::GET, session_path(&session_id)?, None),
        SessionCommand::Attach {
            session_id,
            targets,
        } => (
            Method::POST,
            session_path(&session_id)?,
            Some(binding_body(targets, false)?),
        ),
        SessionCommand::Replace {
            session_id,
            targets,
            clear,
        } => (
            Method::PUT,
            session_path(&session_id)?,
            Some(binding_body(targets, clear)?),
        ),
        SessionCommand::Detach {
            session_id,
            credential_id,
        } => (
            Method::DELETE,
            format!(
                "{}/{}",
                session_path(&session_id)?,
                id_segment(&credential_id)?
            ),
            None,
        ),
        SessionCommand::DetachBinding {
            session_id,
            reference,
        } => (
            Method::DELETE,
            format!(
                "{}/bindings?reference={}",
                session_path(&session_id)?,
                urlencoding::encode(&reference)
            ),
            None,
        ),
    };
    send(method, &path, body).await
}
