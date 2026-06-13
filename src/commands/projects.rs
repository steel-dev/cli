use clap::{Parser, Subcommand};
use dialoguer::{Input, Select};
use serde_json::json;

use crate::api::device_auth;
use crate::api::projects::Project;
use crate::config;
use crate::config::auth;
use crate::config::settings::{ApiMode, ProjectInfo, read_config_from, write_config_to};
use crate::status;
use crate::util::{api, output, style};

#[derive(Subcommand)]
pub enum Command {
    /// List projects in your organization
    List,

    /// Create a new project and make it active
    Create(CreateArgs),

    /// Switch the active project
    Select(SelectArgs),

    /// Show the currently active project
    Current,
}

impl Command {
    pub const fn telemetry_name(&self) -> &'static str {
        match self {
            Self::List => "list",
            Self::Create(_) => "create",
            Self::Select(_) => "select",
            Self::Current => "current",
        }
    }
}

#[derive(Parser)]
pub struct CreateArgs {
    /// Name for the new project
    pub name: Option<String>,
}

#[derive(Parser)]
pub struct SelectArgs {
    /// Project id or slug to make active
    pub project: Option<String>,
}

pub async fn run(command: Command) -> anyhow::Result<()> {
    match command {
        Command::List => run_list().await,
        Command::Create(args) => run_create(args).await,
        Command::Select(args) => run_select(args).await,
        Command::Current => run_current(),
    }
}

/// Resolve the account token or fail with a clear "log in first" message.
pub fn require_account_token() -> anyhow::Result<String> {
    auth::resolve_account_token()
        .ok_or_else(|| anyhow::anyhow!("You are not logged in. Run `steel login` first."))
}

async fn run_list() -> anyhow::Result<()> {
    let account_token = require_account_token()?;
    let (mode, base_url) = api::resolve();
    let projects = device_auth::list_projects(&base_url, mode, &account_token).await?;

    let active = read_config_from(&config::config_path())
        .ok()
        .and_then(|c| c.project)
        .map(|p| p.id);

    if output::is_json() {
        output::success_data(json!(
            projects
                .iter()
                .map(|p| json!({
                    "id": p.id,
                    "slug": p.slug,
                    "name": p.name,
                    "isDefault": p.is_default,
                    "active": active.as_deref() == Some(p.id.as_str()),
                }))
                .collect::<Vec<_>>()
        ));
    } else if projects.is_empty() {
        status!("No projects found.");
    } else {
        for p in &projects {
            let is_active = active.as_deref() == Some(p.id.as_str());
            let marker = if is_active {
                style::green("●")
            } else {
                " ".to_string()
            };
            let name = if is_active {
                style::bold(&p.name)
            } else {
                p.name.clone()
            };
            let slug = style::dim(&format!("[{}]", p.slug));
            let default_tag = if p.is_default {
                style::dim(" (default)")
            } else {
                String::new()
            };
            println!("{marker} {name} {slug}{default_tag}");
        }
    }

    Ok(())
}

async fn run_create(args: CreateArgs) -> anyhow::Result<()> {
    let account_token = require_account_token()?;
    let (mode, base_url) = api::resolve();

    let name = match args.name {
        Some(name) if !name.trim().is_empty() => name,
        _ => {
            if output::is_tty() && !output::is_json() {
                Input::with_theme(&*style::prompt_theme())
                    .with_prompt("Project name")
                    .default(default_project_name())
                    .interact_text()?
            } else {
                anyhow::bail!("Missing project name. Usage: steel projects create <name>");
            }
        }
    };

    let device_name = stored_device_name();
    let project = create_and_activate(&base_url, mode, &account_token, &name, &device_name).await?;

    if output::is_json() {
        output::success_data(json!({
            "id": project.id,
            "slug": project.slug,
            "name": project.name,
        }));
    } else {
        status!(
            "{} Created project {}.",
            style::tick(),
            style::bold(project.name.as_deref().unwrap_or(&project.id))
        );
        status!("{}", style::dim("Now your active project."));
    }

    Ok(())
}

async fn run_select(args: SelectArgs) -> anyhow::Result<()> {
    let account_token = require_account_token()?;
    let (mode, base_url) = api::resolve();
    let projects = device_auth::list_projects(&base_url, mode, &account_token).await?;

    if projects.is_empty() {
        anyhow::bail!("No projects to select. Create one with `steel projects create <name>`.");
    }

    let chosen = match args.project {
        Some(ref needle) => projects
            .iter()
            .find(|p| p.id == *needle || p.slug == *needle)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No project matching '{needle}'"))?,
        None => {
            if !output::is_tty() || output::is_json() {
                anyhow::bail!("Missing project. Usage: steel projects select <id|slug>");
            }
            let labels: Vec<String> = projects
                .iter()
                .map(|p| format!("{} [{}]", p.name, p.slug))
                .collect();
            let selection = Select::with_theme(&*style::prompt_theme())
                .with_prompt("Select active project")
                .items(&labels)
                .default(0)
                .interact()?;
            projects[selection].clone()
        }
    };

    let device_name = stored_device_name();
    let info = activate_project(&base_url, mode, &account_token, &chosen, &device_name).await?;

    if output::is_json() {
        output::success_data(json!({
            "id": info.id,
            "slug": info.slug,
            "name": info.name,
        }));
    } else {
        status!(
            "{} Active project is now {}.",
            style::tick(),
            style::bold(info.name.as_deref().unwrap_or(&info.id))
        );
    }

    Ok(())
}

fn run_current() -> anyhow::Result<()> {
    let cfg = read_config_from(&config::config_path()).unwrap_or_default();
    match cfg.project {
        Some(project) => {
            if output::is_json() {
                output::success_data(json!({
                    "id": project.id,
                    "slug": project.slug,
                    "name": project.name,
                }));
            } else {
                println!(
                    "{} {}",
                    style::bold(project.name.as_deref().unwrap_or(&project.id)),
                    style::dim(&format!("[{}]", project.slug.as_deref().unwrap_or("")))
                );
                println!(
                    "{}",
                    style::dim(&format!(
                        "environment: {}",
                        crate::api::projects::environment_label(project.is_production)
                    ))
                );
            }
        }
        None => {
            status!("No active project. Run `steel projects select` or `steel projects create`.");
        }
    }
    Ok(())
}

// --- Shared helpers used by login / init ---

/// Create a named project, mint a project API key, and make it the active project.
pub async fn create_and_activate(
    base_url: &str,
    mode: ApiMode,
    account_token: &str,
    name: &str,
    device_name: &str,
) -> anyhow::Result<ProjectInfo> {
    let project = device_auth::create_project(base_url, mode, account_token, name).await?;
    activate_project(base_url, mode, account_token, &project, device_name).await
}

/// Mint a project API key for `project` and persist it as the active project.
pub async fn activate_project(
    base_url: &str,
    mode: ApiMode,
    account_token: &str,
    project: &Project,
    device_name: &str,
) -> anyhow::Result<ProjectInfo> {
    let key = device_auth::create_project_api_key(
        base_url,
        mode,
        account_token,
        &project.id,
        &format!("CLI ({device_name})"),
    )
    .await?;

    let info = ProjectInfo {
        id: project.id.clone(),
        slug: Some(project.slug.clone()),
        name: Some(project.name.clone()),
        is_production: project.is_production,
    };
    save_active_project(&info, &key)?;
    Ok(info)
}

/// Ensure there is an active project + API key, selecting the default (or first)
/// project, or creating one if the organization somehow has none.
pub async fn ensure_active_project(
    base_url: &str,
    mode: ApiMode,
    account_token: &str,
    device_name: &str,
) -> anyhow::Result<ProjectInfo> {
    let cfg = read_config_from(&config::config_path()).unwrap_or_default();
    if let (Some(project), Some(_)) = (cfg.project.clone(), cfg.api_key.clone()) {
        return Ok(project);
    }

    let projects = device_auth::list_projects(base_url, mode, account_token).await?;
    let chosen = projects
        .iter()
        .find(|p| p.is_default)
        .or_else(|| projects.first())
        .cloned();

    let project = match chosen {
        Some(project) => project,
        None => {
            device_auth::create_project(base_url, mode, account_token, "Default project").await?
        }
    };

    activate_project(base_url, mode, account_token, &project, device_name).await
}

/// Interactive project selection for `steel init`.
pub async fn choose_project_interactive(
    base_url: &str,
    mode: ApiMode,
    account_token: &str,
    device_name: &str,
) -> anyhow::Result<ProjectInfo> {
    let projects = device_auth::list_projects(base_url, mode, account_token).await?;

    let Some(default_project) = projects
        .iter()
        .find(|p| p.is_default)
        .or_else(|| projects.first())
    else {
        let name = prompt_project_name(&default_project_name())?;
        return create_and_activate(base_url, mode, account_token, &name, device_name).await;
    };

    // Multiple projects: pick an existing one or create a new one.
    if projects.len() > 1 {
        let mut labels: Vec<String> = projects
            .iter()
            .map(|p| {
                let tag = if p.is_default { " (default)" } else { "" };
                format!("{} [{}]{tag}", p.name, p.slug)
            })
            .collect();
        let create_index = labels.len();
        labels.push("Create a new project".to_string());

        let default_index = projects.iter().position(|p| p.is_default).unwrap_or(0);
        let selection = Select::with_theme(&*style::prompt_theme())
            .with_prompt("Select a project")
            .items(&labels)
            .default(default_index)
            .interact()?;

        if selection == create_index {
            let name = prompt_project_name(&default_project_name())?;
            return create_and_activate(base_url, mode, account_token, &name, device_name).await;
        }

        return activate_project(
            base_url,
            mode,
            account_token,
            &projects[selection],
            device_name,
        )
        .await;
    }

    // Exactly one project (the default): reuse it unless the user renames.
    let default_name = default_project.name.clone();
    let name = prompt_project_name(&default_name)?;
    if name.trim().eq_ignore_ascii_case(default_name.trim()) {
        return activate_project(base_url, mode, account_token, default_project, device_name).await;
    }

    create_and_activate(base_url, mode, account_token, &name, device_name).await
}

fn prompt_project_name(default: &str) -> anyhow::Result<String> {
    Ok(Input::with_theme(&*style::prompt_theme())
        .with_prompt("Project name")
        .default(default.to_string())
        .interact_text()?)
}

fn save_active_project(project: &ProjectInfo, api_key: &str) -> anyhow::Result<()> {
    let path = config::config_path();
    let mut cfg = read_config_from(&path).unwrap_or_default();
    cfg.project = Some(project.clone());
    cfg.api_key = Some(api_key.to_string());
    write_config_to(&path, &cfg)?;
    Ok(())
}

fn stored_device_name() -> String {
    read_config_from(&config::config_path())
        .ok()
        .and_then(|c| c.name)
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(default_project_name)
}

/// Default project name derived from the current directory.
pub fn default_project_name() -> String {
    std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| "Default project".to_string())
}
