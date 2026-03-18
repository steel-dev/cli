use clap::{Parser, Subcommand};
use serde_json::json;

use crate::api::client::SteelClient;
use crate::util::{api, output};

#[derive(Subcommand)]
pub enum Command {
    /// List stored credentials
    List(ListArgs),

    /// Create a new credential
    Create(CreateArgs),

    /// Update an existing credential
    Update(UpdateArgs),

    /// Delete a credential
    Delete(DeleteArgs),
}

#[derive(Parser)]
pub struct ListArgs {
    /// Filter by namespace
    #[arg(short, long)]
    pub namespace: Option<String>,

    /// Filter by origin URL
    #[arg(long)]
    pub origin: Option<String>,
}

#[derive(Parser)]
pub struct CreateArgs {
    /// Origin URL to associate the credential with
    #[arg(long)]
    pub origin: Option<String>,

    /// Username
    #[arg(short, long)]
    pub username: Option<String>,

    /// Password
    #[arg(short, long)]
    pub password: Option<String>,

    /// TOTP secret for two-factor authentication
    #[arg(long)]
    pub totp_secret: Option<String>,

    /// Credential namespace
    #[arg(short, long)]
    pub namespace: Option<String>,

    /// Human-readable label
    #[arg(long)]
    pub label: Option<String>,
}

#[derive(Parser)]
pub struct UpdateArgs {
    /// Origin URL of the credential to update
    #[arg(long)]
    pub origin: Option<String>,

    /// New username
    #[arg(short, long)]
    pub username: Option<String>,

    /// New password
    #[arg(short, long)]
    pub password: Option<String>,

    /// New TOTP secret
    #[arg(long)]
    pub totp_secret: Option<String>,

    /// Credential namespace
    #[arg(short, long)]
    pub namespace: Option<String>,

    /// New human-readable label
    #[arg(long)]
    pub label: Option<String>,
}

#[derive(Parser)]
pub struct DeleteArgs {
    /// Origin URL of the credential to delete
    #[arg(long)]
    pub origin: Option<String>,

    /// Credential namespace
    #[arg(short, long)]
    pub namespace: Option<String>,
}

pub async fn run(command: Command) -> anyhow::Result<()> {
    match command {
        Command::List(args) => run_list(args).await,
        Command::Create(args) => run_create(args).await,
        Command::Update(args) => run_update(args).await,
        Command::Delete(args) => run_delete(args).await,
    }
}

async fn run_list(args: ListArgs) -> anyhow::Result<()> {
    let (mode, base_url, auth_info) = api::resolve_with_auth();
    let client = SteelClient::new()?;

    let mut query_params = Vec::new();
    if let Some(ref ns) = args.namespace {
        query_params.push(format!("namespace={}", urlencoding::encode(ns)));
    }
    if let Some(ref origin) = args.origin {
        query_params.push(format!("origin={}", urlencoding::encode(origin)));
    }

    let path = if query_params.is_empty() {
        "/credentials".to_string()
    } else {
        format!("/credentials?{}", query_params.join("&"))
    };

    let data = client
        .request(
            &base_url,
            mode,
            reqwest::Method::GET,
            &path,
            None,
            &auth_info,
        )
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // Normalize response — could be array or { credentials: [...] }
    let credentials = if let Some(arr) = data.as_array() {
        arr.clone()
    } else if let Some(arr) = data.get("credentials").and_then(|v| v.as_array()) {
        arr.clone()
    } else {
        vec![data]
    };

    output::success_data(json!(credentials));

    Ok(())
}

async fn run_create(args: CreateArgs) -> anyhow::Result<()> {
    let origin = args
        .origin
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow::anyhow!("Missing required flag: --origin"))?;
    let username = args
        .username
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow::anyhow!("Missing required flag: --username"))?;
    let password = args
        .password
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow::anyhow!("Missing required flag: --password"))?;

    let (mode, base_url, auth_info) = api::resolve_with_auth();
    let client = SteelClient::new()?;

    let mut value = json!({
        "username": username,
        "password": password,
    });
    if let Some(ref secret) = args.totp_secret {
        let trimmed = secret.trim();
        if !trimmed.is_empty() {
            value["totpSecret"] = json!(trimmed);
        }
    }

    let mut body = json!({
        "origin": origin,
        "value": value,
    });
    if let Some(ref ns) = args.namespace {
        let trimmed = ns.trim();
        if !trimmed.is_empty() {
            body["namespace"] = json!(trimmed);
        }
    }
    if let Some(ref label) = args.label {
        let trimmed = label.trim();
        if !trimmed.is_empty() {
            body["label"] = json!(trimmed);
        }
    }

    let result = client
        .request(
            &base_url,
            mode,
            reqwest::Method::POST,
            "/credentials",
            Some(body),
            &auth_info,
        )
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    if output::is_json() {
        output::success_data(result);
    } else {
        println!("origin: {origin}");
        println!("username: {username}");
        if let Some(ref ns) = args.namespace {
            let trimmed = ns.trim();
            if !trimmed.is_empty() {
                println!("namespace: {trimmed}");
            }
        }
        if let Some(ref label) = args.label {
            let trimmed = label.trim();
            if !trimmed.is_empty() {
                println!("label: {trimmed}");
            }
        }
        if let Some(id) = result.get("id").and_then(|v| v.as_str()) {
            println!("id: {id}");
        }
        println!("Credential created successfully.");
    }

    Ok(())
}

async fn run_update(args: UpdateArgs) -> anyhow::Result<()> {
    let origin = args
        .origin
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow::anyhow!("Missing required flag: --origin"))?;

    let (mode, base_url, auth_info) = api::resolve_with_auth();
    let client = SteelClient::new()?;

    let mut body = json!({"origin": origin});
    let obj = body.as_object_mut().unwrap();

    let has_value =
        args.username.is_some() || args.password.is_some() || args.totp_secret.is_some();
    if has_value {
        let mut value = json!({});
        if let Some(ref u) = args.username {
            let trimmed = u.trim();
            if !trimmed.is_empty() {
                value["username"] = json!(trimmed);
            }
        }
        if let Some(ref p) = args.password {
            let trimmed = p.trim();
            if !trimmed.is_empty() {
                value["password"] = json!(trimmed);
            }
        }
        if let Some(ref t) = args.totp_secret {
            let trimmed = t.trim();
            if !trimmed.is_empty() {
                value["totpSecret"] = json!(trimmed);
            }
        }
        obj.insert("value".into(), value);
    }
    if let Some(ref ns) = args.namespace {
        let trimmed = ns.trim();
        if !trimmed.is_empty() {
            obj.insert("namespace".into(), json!(trimmed));
        }
    }
    if let Some(ref label) = args.label {
        let trimmed = label.trim();
        if !trimmed.is_empty() {
            obj.insert("label".into(), json!(trimmed));
        }
    }

    let result = client
        .request(
            &base_url,
            mode,
            reqwest::Method::PUT,
            "/credentials",
            Some(body),
            &auth_info,
        )
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    if output::is_json() {
        output::success_data(result);
    } else {
        println!("origin: {origin}");
        if let Some(ref ns) = args.namespace {
            let trimmed = ns.trim();
            if !trimmed.is_empty() {
                println!("namespace: {trimmed}");
            }
        }
        if let Some(id) = result.get("id").and_then(|v| v.as_str()) {
            println!("id: {id}");
        }
        println!("Credential updated successfully.");
    }

    Ok(())
}

async fn run_delete(args: DeleteArgs) -> anyhow::Result<()> {
    let origin = args
        .origin
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow::anyhow!("Missing required flag: --origin"))?;

    let (mode, base_url, auth_info) = api::resolve_with_auth();
    let client = SteelClient::new()?;

    let mut body = json!({"origin": origin});
    if let Some(ref ns) = args.namespace {
        let trimmed = ns.trim();
        if !trimmed.is_empty() {
            body["namespace"] = json!(trimmed);
        }
    }

    client
        .request(
            &base_url,
            mode,
            reqwest::Method::DELETE,
            "/credentials",
            Some(body),
            &auth_info,
        )
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    if output::is_json() {
        let mut data = json!({"origin": origin});
        if let Some(ref ns) = args.namespace {
            let trimmed = ns.trim();
            if !trimmed.is_empty() {
                data["namespace"] = json!(trimmed);
            }
        }
        output::success_data(data);
    } else {
        println!("origin: {origin}");
        if let Some(ref ns) = args.namespace {
            let trimmed = ns.trim();
            if !trimmed.is_empty() {
                println!("namespace: {trimmed}");
            }
        }
        println!("Credential deleted successfully.");
    }

    Ok(())
}
