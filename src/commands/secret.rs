use std::io::Read;

use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand};
use serde_json::Value;

use crate::api::client::SteelClient;
use crate::api::secrets::{CreateSecret, UpdateSecret};
use crate::util::{api, output};

#[derive(Subcommand)]
pub enum Command {
    /// List secrets
    List(ProjectArgs),

    /// Store a new secret
    Create(CreateArgs),

    /// Get one secret's metadata
    Get(IdArgs),

    /// Rename a secret or replace its value
    Update(UpdateArgs),

    /// Delete a secret
    Delete(IdArgs),
}

impl Command {
    pub const fn telemetry_name(&self) -> &'static str {
        match self {
            Self::List(_) => "list",
            Self::Create(_) => "create",
            Self::Get(_) => "get",
            Self::Update(_) => "update",
            Self::Delete(_) => "delete",
        }
    }
}

#[derive(Args)]
pub struct ProjectArgs {
    /// Project to use (defaults to the API key's project)
    #[arg(long = "project", value_name = "PROJECT_ID")]
    pub project_id: Option<String>,
}

#[derive(Parser)]
pub struct IdArgs {
    /// Secret ID
    pub secret_id: String,

    #[command(flatten)]
    pub project: ProjectArgs,
}

#[derive(Parser)]
pub struct CreateArgs {
    /// Secret name, also the environment variable name inside computers
    pub name: String,

    /// The secret value
    #[arg(long, value_name = "VALUE", conflicts_with = "value_stdin")]
    pub value: Option<String>,

    /// Read the secret value from stdin
    #[arg(long = "value-stdin")]
    pub value_stdin: bool,

    #[command(flatten)]
    pub project: ProjectArgs,
}

#[derive(Parser)]
pub struct UpdateArgs {
    /// Secret ID
    pub secret_id: String,

    /// New name
    #[arg(long)]
    pub name: Option<String>,

    /// New value
    #[arg(long, value_name = "VALUE", conflicts_with = "value_stdin")]
    pub value: Option<String>,

    /// Read the new value from stdin
    #[arg(long = "value-stdin")]
    pub value_stdin: bool,

    #[command(flatten)]
    pub project: ProjectArgs,
}

pub async fn run(command: Command) -> Result<()> {
    match command {
        Command::List(args) => run_list(args).await,
        Command::Create(args) => run_create(args).await,
        Command::Get(args) => run_get(args).await,
        Command::Update(args) => run_update(args).await,
        Command::Delete(args) => run_delete(args).await,
    }
}

pub fn trim_trailing_newline(text: &str) -> &str {
    let text = text.strip_suffix('\n').unwrap_or(text);
    text.strip_suffix('\r').unwrap_or(text)
}

pub fn read_value(value: Option<String>, from_stdin: bool) -> Result<Option<String>> {
    if from_stdin {
        let mut text = String::new();
        std::io::stdin()
            .read_to_string(&mut text)
            .context("Failed to read the value from stdin")?;
        let value = trim_trailing_newline(&text).to_string();
        if value.is_empty() {
            bail!("Nothing was read from stdin.");
        }
        return Ok(Some(value));
    }
    if value.as_deref() == Some("") {
        bail!("--value must not be empty.");
    }
    Ok(value)
}

pub fn name_of(secret: &Value) -> &str {
    secret["name"].as_str().unwrap_or("-")
}

pub fn id_of(secret: &Value) -> Result<String> {
    secret["id"]
        .as_str()
        .map(str::to_string)
        .context("the API returned a secret without an id")
}

async fn run_list(args: ProjectArgs) -> Result<()> {
    let (mode, base_url, auth) = api::resolve_with_auth();
    let client = SteelClient::new()?;
    let data = client
        .list_secrets(&base_url, mode, &auth, args.project_id.as_deref())
        .await?;
    if output::is_json() {
        output::success_data(data);
    } else {
        print_secrets(&data);
    }
    Ok(())
}

async fn run_create(args: CreateArgs) -> Result<()> {
    let Some(value) = read_value(args.value, args.value_stdin)? else {
        bail!("Give the value with --value or --value-stdin.");
    };
    let (mode, base_url, auth) = api::resolve_with_auth();
    let client = SteelClient::new()?;
    let request = CreateSecret {
        name: args.name,
        value,
        project_id: args.project.project_id,
    };
    let data = client
        .create_secret(&base_url, mode, &auth, &request)
        .await?;
    if output::is_json() {
        output::success_data(data);
    } else {
        println!("Created {} as {}.", name_of(&data), id_of(&data)?);
    }
    Ok(())
}

async fn run_get(args: IdArgs) -> Result<()> {
    let (mode, base_url, auth) = api::resolve_with_auth();
    let client = SteelClient::new()?;
    let data = client
        .get_secret(
            &base_url,
            mode,
            &auth,
            &args.secret_id,
            args.project.project_id.as_deref(),
        )
        .await?;
    output::success_data(data);
    Ok(())
}

async fn run_update(args: UpdateArgs) -> Result<()> {
    let request = UpdateSecret {
        name: args.name,
        value: read_value(args.value, args.value_stdin)?,
    };
    if request.is_empty() {
        bail!("Give something to change: --name, --value or --value-stdin.");
    }
    let (mode, base_url, auth) = api::resolve_with_auth();
    let client = SteelClient::new()?;
    let data = client
        .update_secret(
            &base_url,
            mode,
            &auth,
            &args.secret_id,
            args.project.project_id.as_deref(),
            &request,
        )
        .await?;
    if output::is_json() {
        output::success_data(data);
    } else {
        println!("Updated {}.", args.secret_id);
    }
    Ok(())
}

async fn run_delete(args: IdArgs) -> Result<()> {
    let (mode, base_url, auth) = api::resolve_with_auth();
    let client = SteelClient::new()?;
    let data = client
        .delete_secret(
            &base_url,
            mode,
            &auth,
            &args.secret_id,
            args.project.project_id.as_deref(),
        )
        .await?;
    if output::is_json() {
        output::success_data(data);
    } else {
        println!("Deleted {}.", args.secret_id);
    }
    Ok(())
}

fn print_secrets(data: &Value) {
    let secrets = data["secrets"].as_array().cloned().unwrap_or_default();
    if secrets.is_empty() {
        println!("No secrets.");
        return;
    }
    let rows: Vec<Vec<String>> = secrets
        .iter()
        .map(|secret| {
            vec![
                secret["id"].as_str().unwrap_or("").to_string(),
                name_of(secret).to_string(),
                secret["version"]
                    .as_u64()
                    .map_or_else(|| "-".to_string(), |version| version.to_string()),
                secret["updatedAt"].as_str().unwrap_or("").to_string(),
            ]
        })
        .collect();
    output::print_table(&["ID", "NAME", "VERSION", "UPDATED"], &rows);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trailing_newlines_are_dropped_once() {
        assert_eq!(trim_trailing_newline("abc\n"), "abc");
        assert_eq!(trim_trailing_newline("abc\r\n"), "abc");
        assert_eq!(trim_trailing_newline("abc\n\n"), "abc\n");
        assert_eq!(trim_trailing_newline("abc"), "abc");
    }

    #[test]
    fn values_from_flags_pass_through_and_empty_ones_are_refused() {
        assert_eq!(read_value(None, false).unwrap(), None);
        assert_eq!(
            read_value(Some("x".into()), false).unwrap().as_deref(),
            Some("x")
        );
        assert!(read_value(Some(String::new()), false).is_err());
    }
}
