use std::collections::BTreeMap;

use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand};
use serde_json::{Value, json};

use crate::api::client::SteelClient;
use crate::api::environments::{CreateEnvironment, EnvironmentSpec, UpdateEnvironment};
use crate::api::secrets::parse_secret_ids;
use crate::commands::secret::ProjectArgs;
use crate::util::pairs::parse_pairs;
use crate::util::{api, output};

#[derive(Subcommand)]
pub enum Command {
    /// List environments
    List(ProjectArgs),

    /// Create an environment
    Create(CreateArgs),

    /// Get one environment
    Get(IdArgs),

    /// Change an environment's name, spec or secrets
    Update(UpdateArgs),

    /// Delete an environment
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

#[derive(Parser)]
pub struct IdArgs {
    /// Environment ID
    pub environment_id: String,

    #[command(flatten)]
    pub project: ProjectArgs,
}

#[derive(Args)]
pub struct SpecArgs {
    /// Template name
    #[arg(long)]
    pub template: Option<String>,

    /// Number of vCPUs
    #[arg(long)]
    pub vcpu: Option<u32>,

    /// Memory in MiB
    #[arg(long = "memory", value_name = "MIB")]
    pub memory_mib: Option<u32>,

    /// Disk in MiB
    #[arg(long = "disk", value_name = "MIB")]
    pub disk_mib: Option<u32>,

    /// Stop computers after this many seconds of running time
    #[arg(long = "timeout", value_name = "SECONDS")]
    pub timeout_seconds: Option<u32>,

    /// Pause instead of stopping when the timeout is reached
    #[arg(long = "auto-pause")]
    pub auto_pause: bool,

    /// Environment variable for computers, repeatable
    #[arg(long = "env", value_name = "KEY=VALUE")]
    pub env: Vec<String>,
}

impl SpecArgs {
    pub fn to_spec(&self) -> Result<EnvironmentSpec> {
        Ok(EnvironmentSpec {
            template: self.template.clone(),
            vcpu: self.vcpu,
            memory_mib: self.memory_mib,
            disk_mib: self.disk_mib,
            timeout_seconds: self.timeout_seconds,
            auto_pause: self.auto_pause.then_some(true),
            env: parse_pairs("--env", "KEY=VALUE", &self.env)?,
        })
    }
}

#[derive(Args)]
pub struct SecretArgs {
    /// Attach a stored secret as an environment variable, repeatable
    #[arg(long = "secret", value_name = "NAME=SECRET_ID")]
    pub secrets: Vec<String>,

    /// Store a new secret and attach it, repeatable
    #[arg(long = "secret-value", value_name = "NAME=VALUE")]
    pub secret_values: Vec<String>,
}

#[derive(Parser)]
pub struct CreateArgs {
    /// Environment name
    pub name: String,

    #[command(flatten)]
    pub spec: SpecArgs,

    #[command(flatten)]
    pub secrets: SecretArgs,

    #[command(flatten)]
    pub project: ProjectArgs,
}

#[derive(Parser)]
pub struct UpdateArgs {
    /// Environment ID
    pub environment_id: String,

    /// New name
    #[arg(long)]
    pub name: Option<String>,

    #[command(flatten)]
    pub spec: SpecArgs,

    #[command(flatten)]
    pub secrets: SecretArgs,

    /// Detach a secret, repeatable
    #[arg(long = "unset-secret", value_name = "NAME")]
    pub unset_secrets: Vec<String>,

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

pub fn secret_declarations(
    ids: &[String],
    values: &[String],
    unset: &[String],
) -> Result<BTreeMap<String, Value>> {
    let mut declarations = BTreeMap::new();
    for (name, id) in parse_secret_ids(ids)? {
        declarations.insert(name, json!({ "secretId": id }));
    }
    for (name, value) in parse_pairs("--secret-value", "NAME=VALUE", values)? {
        if value.is_empty() {
            bail!("--secret-value {name} must not be empty.");
        }
        if declarations
            .insert(name.clone(), json!({ "value": value }))
            .is_some()
        {
            bail!("{name} was given more than once.");
        }
    }
    for name in unset {
        let name = name.trim();
        if name.is_empty() {
            bail!("--unset-secret wants a name.");
        }
        if declarations.insert(name.to_string(), Value::Null).is_some() {
            bail!("{name} was given more than once.");
        }
    }
    Ok(declarations)
}

pub fn name_of(environment: &Value) -> &str {
    environment["name"].as_str().unwrap_or("-")
}

pub fn id_of(environment: &Value) -> Result<String> {
    environment["id"]
        .as_str()
        .map(str::to_string)
        .context("the API returned an environment without an id")
}

async fn run_list(args: ProjectArgs) -> Result<()> {
    let (mode, base_url, auth) = api::resolve_with_auth();
    let client = SteelClient::new()?;
    let data = client
        .list_environments(&base_url, mode, &auth, args.project_id.as_deref())
        .await?;
    if output::is_json() {
        output::success_data(data);
    } else {
        print_environments(&data);
    }
    Ok(())
}

async fn run_create(args: CreateArgs) -> Result<()> {
    let spec = args.spec.to_spec()?;
    let request = CreateEnvironment {
        name: args.name,
        project_id: args.project.project_id,
        spec: (!spec.is_empty()).then(|| spec.body()),
        secrets: secret_declarations(&args.secrets.secrets, &args.secrets.secret_values, &[])?,
    };
    let (mode, base_url, auth) = api::resolve_with_auth();
    let client = SteelClient::new()?;
    let data = client
        .create_environment(&base_url, mode, &auth, &request)
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
        .get_environment(
            &base_url,
            mode,
            &auth,
            &args.environment_id,
            args.project.project_id.as_deref(),
        )
        .await?;
    output::success_data(data);
    Ok(())
}

async fn run_update(args: UpdateArgs) -> Result<()> {
    let project = args.project.project_id.as_deref();
    let spec_changes = args.spec.to_spec()?;
    let secrets = secret_declarations(
        &args.secrets.secrets,
        &args.secrets.secret_values,
        &args.unset_secrets,
    )?;
    if args.name.is_none() && spec_changes.is_empty() && secrets.is_empty() {
        bail!(
            "Give something to change: --name, a spec flag, --secret, --secret-value or --unset-secret."
        );
    }
    let (mode, base_url, auth) = api::resolve_with_auth();
    let client = SteelClient::new()?;
    let spec = if spec_changes.is_empty() {
        None
    } else {
        let current = client
            .get_environment(&base_url, mode, &auth, &args.environment_id, project)
            .await?;
        Some(spec_changes.apply_to(current["spec"].clone()))
    };
    let request = UpdateEnvironment {
        name: args.name,
        spec,
        secrets,
    };
    let data = client
        .update_environment(
            &base_url,
            mode,
            &auth,
            &args.environment_id,
            project,
            &request,
        )
        .await?;
    if output::is_json() {
        output::success_data(data);
    } else {
        println!("Updated {}.", args.environment_id);
    }
    Ok(())
}

async fn run_delete(args: IdArgs) -> Result<()> {
    let (mode, base_url, auth) = api::resolve_with_auth();
    let client = SteelClient::new()?;
    let data = client
        .delete_environment(
            &base_url,
            mode,
            &auth,
            &args.environment_id,
            args.project.project_id.as_deref(),
        )
        .await?;
    if output::is_json() {
        output::success_data(data);
    } else {
        println!("Deleted {}.", args.environment_id);
    }
    Ok(())
}

fn print_environments(data: &Value) {
    let environments = data["environments"].as_array().cloned().unwrap_or_default();
    if environments.is_empty() {
        println!("No environments.");
        return;
    }
    let rows: Vec<Vec<String>> = environments
        .iter()
        .map(|environment| {
            vec![
                environment["id"].as_str().unwrap_or("").to_string(),
                name_of(environment).to_string(),
                environment["spec"]["template"]
                    .as_str()
                    .unwrap_or("-")
                    .to_string(),
                environment["secrets"]
                    .as_object()
                    .map_or(0, serde_json::Map::len)
                    .to_string(),
                environment["updatedAt"].as_str().unwrap_or("").to_string(),
            ]
        })
        .collect();
    output::print_table(&["ID", "NAME", "TEMPLATE", "SECRETS", "UPDATED"], &rows);
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: &str = "2f1b6f2e-2c6f-4d9a-9c2a-8f4c1e0d7b31";

    #[test]
    fn declarations_cover_ids_values_and_unsets() {
        let declarations = secret_declarations(
            &[format!("TOKEN={ID}")],
            &["KEY=abc".into()],
            &["OLD".into()],
        )
        .unwrap();
        assert_eq!(declarations["TOKEN"], json!({ "secretId": ID }));
        assert_eq!(declarations["KEY"], json!({ "value": "abc" }));
        assert_eq!(declarations["OLD"], Value::Null);
    }

    #[test]
    fn declarations_refuse_conflicts_and_blanks() {
        assert!(secret_declarations(&["TOKEN=plain".into()], &[], &[]).is_err());
        assert!(secret_declarations(&[], &["KEY=".into()], &[]).is_err());
        assert!(secret_declarations(&[format!("A={ID}")], &["A=x".into()], &[]).is_err());
        assert!(secret_declarations(&[], &["A=x".into()], &["A".into()]).is_err());
        assert!(secret_declarations(&[], &[], &[" ".into()]).is_err());
    }
}
