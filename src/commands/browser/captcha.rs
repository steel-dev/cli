use clap::{Parser, Subcommand};
use serde_json::{Value, json};

use crate::api::client::SteelClient;
use crate::util::output;

#[derive(Subcommand)]
pub enum Command {
    /// Solve a CAPTCHA on the current page
    Solve(SolveArgs),

    /// Check CAPTCHA status
    Status(StatusArgs),
}

impl Command {
    pub const fn telemetry_name(&self) -> &'static str {
        match self {
            Self::Solve(_) => "solve",
            Self::Status(_) => "status",
        }
    }
}

#[derive(Parser)]
pub struct SolveArgs {
    /// Explicit session ID (overrides --session name lookup)
    #[arg(long)]
    pub session_id: Option<String>,

    /// Page ID
    #[arg(long)]
    pub page_id: Option<String>,

    /// Page URL for targeted CAPTCHA solving
    #[arg(long)]
    pub url: Option<String>,

    /// CAPTCHA task ID for targeted solving
    #[arg(long)]
    pub task_id: Option<String>,
}

#[derive(Parser)]
pub struct StatusArgs {
    /// Explicit session ID (overrides --session name lookup)
    #[arg(long)]
    pub session_id: Option<String>,

    /// Page ID
    #[arg(long)]
    pub page_id: Option<String>,

    /// Wait for terminal status
    #[arg(short, long)]
    pub wait: bool,

    /// Timeout in milliseconds for --wait mode
    #[arg(long)]
    pub timeout: Option<u64>,

    /// Poll interval in milliseconds for --wait mode
    #[arg(long)]
    pub interval: Option<u64>,
}

pub async fn run(command: Command, session: Option<&str>) -> anyhow::Result<()> {
    match command {
        Command::Solve(args) => run_solve(args, session).await,
        Command::Status(args) => run_status(args, session).await,
    }
}

fn resolve_session_id(
    explicit_session_id: Option<&str>,
    session: Option<&str>,
) -> anyhow::Result<String> {
    if let Some(id) = explicit_session_id {
        let trimmed = id.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }
    Ok(super::existing_session(session)?.id)
}

async fn run_solve(args: SolveArgs, session: Option<&str>) -> anyhow::Result<()> {
    let client = SteelClient::new()?;
    let mode = client.mode();

    let session_id = resolve_session_id(args.session_id.as_deref(), session)?;

    let raw = client
        .solve_captcha(
            &session_id,
            args.page_id.as_deref(),
            args.url.as_deref(),
            args.task_id.as_deref(),
        )
        .await?;
    let success = raw["success"].as_bool().unwrap_or(false);
    let message = raw["message"].as_str();

    if output::is_json() {
        let mut data = json!({
            "sessionId": session_id,
            "mode": mode.to_string(),
            "success": success,
        });
        if let Some(msg) = message {
            data["message"] = json!(msg);
        }
        output::success_data(data);
    } else {
        println!("session_id: {session_id}");
        println!("mode: {mode}");
        println!("success: {success}");
        if let Some(msg) = message {
            println!("message: {msg}");
        }
    }

    if !success {
        anyhow::bail!("CAPTCHA solve failed");
    }

    Ok(())
}

async fn run_status(args: StatusArgs, session: Option<&str>) -> anyhow::Result<()> {
    let client = SteelClient::new()?;

    let session_id = resolve_session_id(args.session_id.as_deref(), session)?;

    let timeout = args.timeout.unwrap_or(60_000);
    let interval = args.interval.unwrap_or(1_000);
    let started = std::time::Instant::now();
    let (status, types) = loop {
        let pages = client
            .captcha_status(&session_id, args.page_id.as_deref())
            .await?;
        let (status, types) = normalize_captcha_status(&pages);
        if !args.wait || is_terminal_captcha_status(&status) {
            break (status, types);
        }
        if started.elapsed().as_millis() >= u128::from(timeout) {
            anyhow::bail!(
                "CAPTCHA status polling timed out after {timeout}ms. Last status: {status}"
            );
        }
        tokio::time::sleep(std::time::Duration::from_millis(interval)).await;
    };

    if output::is_json() {
        let mut data = json!({
            "status": status,
        });
        if !types.is_empty() {
            data["types"] = json!(types);
        }
        output::success_data(data);
    } else {
        let type_suffix = if types.is_empty() {
            String::new()
        } else {
            format!(" {}", types.join(","))
        };
        println!("{status}{type_suffix}");
    }

    if status != "solved" && status != "none" {
        anyhow::bail!("CAPTCHA status: {}", status);
    }

    Ok(())
}

const KNOWN_CAPTCHA_TYPES: &[&str] = &["recaptchaV2", "recaptchaV3", "turnstile", "image_to_text"];

fn normalize_captcha_status(pages: &[Value]) -> (String, Vec<String>) {
    if pages.is_empty() {
        return ("none".to_string(), vec![]);
    }

    let mut all_tasks: Vec<&Value> = Vec::new();
    let mut any_solving = false;

    for page in pages {
        if page.get("isSolvingCaptcha").and_then(|v| v.as_bool()) == Some(true) {
            any_solving = true;
        }

        if let Some(tasks) = page.get("tasks").and_then(|v| v.as_array()) {
            for task in tasks {
                if task.is_object() {
                    all_tasks.push(task);
                }
            }
        }
    }

    if all_tasks.is_empty() {
        let status = if any_solving { "solving" } else { "none" };
        return (status.to_string(), vec![]);
    }

    let mut solving_types: Vec<String> = Vec::new();
    let mut failed_types: Vec<String> = Vec::new();
    let mut has_solving = false;
    let mut has_failed = false;
    let mut has_solved = false;

    for task in &all_tasks {
        let task_status = task.get("status").and_then(|v| v.as_str()).unwrap_or("");
        let task_type = task.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let is_known = KNOWN_CAPTCHA_TYPES.contains(&task_type);

        match task_status {
            "solving" | "detected" | "validating" => {
                has_solving = true;
                if is_known && !solving_types.contains(&task_type.to_string()) {
                    solving_types.push(task_type.to_string());
                }
            }
            "failed_to_solve" | "failed_to_detect" | "validation_failed" => {
                has_failed = true;
                if is_known && !failed_types.contains(&task_type.to_string()) {
                    failed_types.push(task_type.to_string());
                }
            }
            "solved" => {
                has_solved = true;
            }
            _ => {}
        }
    }

    if has_solving || any_solving {
        return ("solving".to_string(), solving_types);
    }

    if has_failed {
        return ("failed".to_string(), failed_types);
    }

    if has_solved {
        return ("solved".to_string(), vec![]);
    }

    ("none".to_string(), vec![])
}

fn is_terminal_captcha_status(status: &str) -> bool {
    matches!(status, "solved" | "failed" | "none")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_captcha_none() {
        let (status, types) = normalize_captcha_status(&[]);
        assert_eq!(status, "none");
        assert!(types.is_empty());
    }

    #[test]
    fn normalize_captcha_solving() {
        let pages = vec![json!({
            "isSolvingCaptcha": true,
            "tasks": [{"status": "solving", "type": "turnstile"}]
        })];
        let (status, types) = normalize_captcha_status(&pages);
        assert_eq!(status, "solving");
        assert_eq!(types, vec!["turnstile"]);
    }

    #[test]
    fn normalize_captcha_solved() {
        let pages = vec![json!({
            "tasks": [{"status": "solved", "type": "recaptchaV2"}]
        })];
        let (status, _types) = normalize_captcha_status(&pages);
        assert_eq!(status, "solved");
    }

    #[test]
    fn normalize_captcha_failed() {
        let pages = vec![json!({
            "tasks": [{"status": "failed_to_solve", "type": "turnstile"}]
        })];
        let (status, types) = normalize_captcha_status(&pages);
        assert_eq!(status, "failed");
        assert_eq!(types, vec!["turnstile"]);
    }
}
