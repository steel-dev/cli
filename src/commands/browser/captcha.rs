use clap::{Parser, Subcommand};
use serde_json::{Value, json};

use crate::api::client::SteelClient;
use crate::browser::state::BrowserSession;
use crate::util::output;

#[derive(Subcommand)]
pub enum Command {
    /// Solve every detected CAPTCHA, or the one a flag picks
    Solve(SolveArgs),

    /// Check CAPTCHA status and list page and task ids
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
    /// Solve only the CAPTCHA on this page (ids from `captcha status`)
    #[arg(long)]
    pub page_id: Option<String>,

    /// Solve only the CAPTCHA on the page at this URL
    #[arg(long)]
    pub url: Option<String>,

    /// Solve only this CAPTCHA task (ids from `captcha status`)
    #[arg(long)]
    pub task_id: Option<String>,
}

#[derive(Parser)]
pub struct StatusArgs {
    /// Check only this page
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

fn client_for(session: Option<&str>) -> anyhow::Result<(SteelClient, BrowserSession)> {
    let target = super::existing_session(session)?;
    let client = SteelClient::new()?.at(&target.api_url, target.mode);
    Ok((client, target))
}

async fn run_solve(args: SolveArgs, session: Option<&str>) -> anyhow::Result<()> {
    let (client, target) = client_for(session)?;
    let (mode, session_id) = (target.mode, target.id);

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

    let mut data = json!({
        "sessionId": session_id,
        "mode": mode.to_string(),
        "success": success,
    });
    if let Some(msg) = message {
        data["message"] = json!(msg);
    }

    if !output::is_json() {
        println!("session_id: {session_id}");
        println!("mode: {mode}");
        println!("success: {success}");
        if let Some(msg) = message {
            println!("message: {msg}");
        }
    }

    if !success {
        let reason = message.map_or_else(String::new, |msg| format!(": {msg}"));
        return Err(output::fail(
            "captcha_not_solved",
            &format!("CAPTCHA solve failed{reason}"),
            Some(&data),
        ));
    }

    if output::is_json() {
        output::success_data(data);
    }
    Ok(())
}

async fn run_status(args: StatusArgs, session: Option<&str>) -> anyhow::Result<()> {
    let (client, target) = client_for(session)?;
    let session_id = target.id;
    let page_id = args
        .page_id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty());

    let timeout = args.timeout.unwrap_or(60_000);
    let interval = args.interval.unwrap_or(1_000);
    let started = std::time::Instant::now();
    let (status, types, pages) = loop {
        let pages = on_page(client.captcha_status(&session_id, page_id).await?, page_id);
        let (status, types) = normalize_captcha_status(&pages);
        if !args.wait || is_terminal_captcha_status(&status) {
            break (status, types, page_summaries(&pages));
        }
        if started.elapsed().as_millis() >= u128::from(timeout) {
            anyhow::bail!(
                "CAPTCHA status polling timed out after {timeout}ms. Last status: {status}"
            );
        }
        tokio::time::sleep(std::time::Duration::from_millis(interval)).await;
    };

    let mut data = json!({
        "status": status,
    });
    if !types.is_empty() {
        data["types"] = json!(types);
    }
    data["pages"] = json!(pages);

    if !output::is_json() {
        let type_suffix = if types.is_empty() {
            String::new()
        } else {
            format!(" {}", types.join(","))
        };
        println!("{status}{type_suffix}");
        for line in page_lines(&pages) {
            println!("{line}");
        }
    }

    if status != "solved" && status != "none" {
        return Err(output::fail(
            &format!("captcha_{status}"),
            &format!("CAPTCHA status: {status}"),
            Some(&data),
        ));
    }

    if output::is_json() {
        output::success_data(data);
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

fn on_page(pages: Vec<Value>, page_id: Option<&str>) -> Vec<Value> {
    match page_id {
        Some(id) => pages
            .into_iter()
            .filter(|page| page["pageId"] == id)
            .collect(),
        None => pages,
    }
}

fn page_summaries(pages: &[Value]) -> Vec<Value> {
    pages
        .iter()
        .map(|page| {
            let tasks: Vec<Value> = page["tasks"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|task| task.is_object())
                .map(|task| json!({"id": task["id"], "type": task["type"], "status": task["status"]}))
                .collect();
            json!({"pageId": page["pageId"], "url": page["url"], "tasks": tasks})
        })
        .collect()
}

fn page_lines(pages: &[Value]) -> Vec<String> {
    let text = |value: &Value| value.as_str().unwrap_or("?").to_string();
    let mut lines = Vec::new();
    for page in pages {
        let mut line = format!("page {}", text(&page["pageId"]));
        if let Some(url) = page["url"].as_str() {
            line.push(' ');
            line.push_str(url);
        }
        lines.push(line);
        for task in page["tasks"].as_array().into_iter().flatten() {
            lines.push(format!(
                "  task {} {} {}",
                text(&task["id"]),
                text(&task["type"]),
                text(&task["status"])
            ));
        }
    }
    lines
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

    fn two_pages() -> Vec<Value> {
        vec![
            json!({
                "pageId": "p-1",
                "url": "https://a.test/login",
                "isSolvingCaptcha": true,
                "tasks": [{"id": "k-1", "pageId": "p-1", "type": "turnstile", "status": "solving", "created": 1}]
            }),
            json!({
                "pageId": "p-2",
                "url": "https://b.test/",
                "isSolvingCaptcha": false,
                "tasks": [{"id": "k-2", "pageId": "p-2", "type": "recaptchaV2", "status": "solved"}]
            }),
        ]
    }

    #[test]
    fn status_lists_page_and_task_ids() {
        let pages = page_summaries(&two_pages());
        assert_eq!(
            pages,
            vec![
                json!({"pageId": "p-1", "url": "https://a.test/login", "tasks": [
                    {"id": "k-1", "type": "turnstile", "status": "solving"}
                ]}),
                json!({"pageId": "p-2", "url": "https://b.test/", "tasks": [
                    {"id": "k-2", "type": "recaptchaV2", "status": "solved"}
                ]}),
            ]
        );
        assert_eq!(
            page_lines(&pages),
            vec![
                "page p-1 https://a.test/login",
                "  task k-1 turnstile solving",
                "page p-2 https://b.test/",
                "  task k-2 recaptchaV2 solved",
            ]
        );
    }

    #[test]
    fn page_id_keeps_only_that_page() {
        let pages = on_page(two_pages(), Some("p-2"));
        assert_eq!(pages.len(), 1);
        assert_eq!(normalize_captcha_status(&pages).0, "solved");
        assert_eq!(normalize_captcha_status(&two_pages()).0, "solving");
        assert!(on_page(two_pages(), Some("p-9")).is_empty());
        assert_eq!(on_page(two_pages(), None).len(), 2);
    }

    #[test]
    fn page_lines_tolerate_missing_fields() {
        let pages = page_summaries(&[json!({"pageId": "p-1", "tasks": [{"status": "detected"}]})]);
        assert_eq!(page_lines(&pages), vec!["page p-1", "  task ? ? detected"]);
    }
}
