use std::time::{Duration, Instant};

use clap::Parser;
use serde_json::{Value, json};

use crate::api::client::{ApiError, SteelClient};
use crate::api::projects::{environment_label, parse_projects, resolve_current_project};
use crate::browser::state::{self, BrowserSession};
use crate::config::auth::Auth;
use crate::config::settings::ApiMode;
use crate::util::output;
use crate::util::version::is_newer;

const PROBE_TIMEOUT: Duration = Duration::from_secs(15);
const FAILURE_CODE: &str = "doctor_failed";

#[derive(Parser)]
pub struct Args {
    /// Only check auth and API connectivity
    #[arg(long)]
    pub preflight: bool,
}

struct Check {
    category: &'static str,
    name: String,
    status: &'static str,
    detail: Value,
    fix: Option<&'static str>,
    transient: bool,
}

pub struct Report {
    checks: Vec<Check>,
}

impl Report {
    pub fn failed(&self) -> bool {
        self.checks.iter().any(|c| c.status == "fail")
    }

    fn overall(&self) -> &'static str {
        if self.failed() {
            "fail"
        } else if self.checks.iter().any(|c| c.status == "degraded") {
            "degraded"
        } else {
            "pass"
        }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "overall": self.overall(),
            "checks": self.checks.iter().map(|c| {
                let mut obj = json!({
                    "category": c.category,
                    "name": c.name,
                    "status": c.status,
                    "transient": c.transient,
                });
                if !c.detail.is_null() && c.detail != json!({}) {
                    obj["detail"] = c.detail.clone();
                }
                if let Some(fix) = c.fix {
                    obj["fix"] = json!(fix);
                }
                obj
            }).collect::<Vec<_>>(),
        })
    }

    pub fn print_text(&self) {
        print_text(&self.checks);
    }

    pub fn emit(&self) -> anyhow::Result<()> {
        if output::is_json() {
            if self.failed() {
                return Err(output::fail(
                    FAILURE_CODE,
                    "Some checks failed.",
                    Some(&self.to_json()),
                ));
            }
            output::success_data(self.to_json());
            return Ok(());
        }
        self.print_text();
        if self.failed() {
            return Err(output::SilentExit::new(FAILURE_CODE).into());
        }
        Ok(())
    }
}

pub async fn run(args: Args) -> anyhow::Result<()> {
    report(args.preflight).await.emit()
}

pub async fn report(preflight: bool) -> Report {
    let (mode, base_url, auth) = crate::util::api::resolve_with_auth();

    let mut checks = if preflight {
        check_auth_and_api(mode, &base_url, &auth).await
    } else {
        let (auth_api, version) =
            tokio::join!(check_auth_and_api(mode, &base_url, &auth), check_version(),);
        let sessions = check_sessions(&state::list());
        let mut all = auth_api;
        all.extend(sessions);
        all.extend(version);
        all
    };

    // Sort by category order
    const ORDER: &[&str] = &["auth", "project", "api", "sessions", "version"];
    checks.sort_by_key(|c| ORDER.iter().position(|&o| o == c.category).unwrap_or(99));

    Report { checks }
}

async fn check_auth_and_api(mode: ApiMode, base_url: &str, auth: &Auth) -> Vec<Check> {
    let mut checks = Vec::new();
    let has_key = auth.has_credentials();
    let need_auth = mode == ApiMode::Cloud;

    if !has_key && need_auth {
        checks.push(Check {
            category: "auth",
            name: "API key not configured".into(),
            status: "fail",
            detail: json!({"source": "none"}),
            fix: Some("steel login"),
            transient: false,
        });
        return checks;
    }

    if has_key {
        let source = auth.source.to_string();
        let masked = auth.api_key.as_ref().map(|k| {
            if k.len() > 7 {
                format!("{}...", &k[..7])
            } else {
                k.clone()
            }
        });
        checks.push(Check {
            category: "auth",
            name: format!("Credential configured (source: {source})"),
            status: "pass",
            detail: json!({"source": source, "key": masked}),
            fix: None,
            transient: false,
        });
    }

    let Ok(client) = SteelClient::with(base_url.to_string(), mode, auth.clone()) else {
        checks.push(Check {
            category: "api",
            name: "Failed to initialize HTTP client".into(),
            status: "fail",
            detail: json!(null),
            fix: None,
            transient: false,
        });
        return checks;
    };

    let timed_sessions = async {
        let start = Instant::now();
        let result = client
            .request_within(reqwest::Method::GET, "/sessions", None, Some(PROBE_TIMEOUT))
            .await;
        (result, start.elapsed().as_millis())
    };
    // Project info is informational only: fetched alongside the auth probe,
    // silently skipped on any error so doctor never degrades because of it.
    let projects = async {
        if need_auth {
            tokio::time::timeout(PROBE_TIMEOUT, client.get_projects())
                .await
                .ok()
                .and_then(Result::ok)
        } else {
            None
        }
    };
    let ((result, latency_ms), projects_data) = tokio::join!(timed_sessions, projects);

    match result {
        Ok(_) => {
            checks.push(Check {
                category: "api",
                name: format!("{base_url} reachable ({latency_ms}ms)"),
                status: "pass",
                detail: json!({"url": base_url, "mode": mode.to_string(), "latency_ms": latency_ms}),
                fix: None,
                transient: false,
            });
            if has_key && need_auth {
                checks.push(Check {
                    category: "auth",
                    name: "Credential accepted by API".into(),
                    status: "pass",
                    detail: json!(null),
                    fix: None,
                    transient: false,
                });
                if let Some(check) = project_check(projects_data.as_ref()) {
                    checks.push(check);
                }
            }
        }
        Err(ApiError::Unreachable { .. } | ApiError::Other(_)) => {
            checks.push(Check {
                category: "api",
                name: format!("{base_url} unreachable"),
                status: "fail",
                detail: json!({"url": base_url, "mode": mode.to_string()}),
                fix: None,
                transient: true,
            });
        }
        Err(ApiError::RequestFailed { status: 401, .. }) => {
            checks.push(Check {
                category: "api",
                name: format!("{base_url} reachable ({latency_ms}ms)"),
                status: "pass",
                detail: json!({"url": base_url, "mode": mode.to_string(), "latency_ms": latency_ms}),
                fix: None,
                transient: false,
            });
            checks.push(Check {
                category: "auth",
                name: "Credential rejected by API".into(),
                status: "fail",
                detail: json!(null),
                fix: Some("steel login"),
                transient: false,
            });
        }
        Err(ApiError::RequestFailed {
            status, message, ..
        }) => {
            checks.push(Check {
                category: "api",
                name: format!("{base_url} error ({status}: {message})"),
                status: "fail",
                detail: json!({"url": base_url, "mode": mode.to_string(), "status": status}),
                fix: None,
                transient: true,
            });
        }
        Err(ApiError::DeviceAuth(error)) => {
            checks.push(Check {
                category: "auth",
                name: error.to_string(),
                status: "fail",
                detail: json!(null),
                fix: Some("steel login --resume"),
                transient: false,
            });
        }
        Err(ApiError::Redirected {
            status, location, ..
        }) => {
            checks.push(Check {
                category: "api",
                name: format!(
                    "{base_url} redirects ({status}) to {location}; redirects are refused"
                ),
                status: "fail",
                detail: json!({"url": base_url, "mode": mode.to_string(), "status": status}),
                fix: Some("Set STEEL_API_URL or --api-url to the final API address"),
                transient: false,
            });
        }
        Err(ApiError::MissingAuth) => {
            checks.push(Check {
                category: "auth",
                name: "API key not configured".into(),
                status: "fail",
                detail: json!(null),
                fix: Some("steel login"),
                transient: false,
            });
        }
    }

    checks
}

/// Build the project/environment check from a `/projects` response body.
/// Returns `None` when the response is absent or no single project resolves.
fn project_check(data: Option<&Value>) -> Option<Check> {
    let projects = parse_projects(data?);
    let current = resolve_current_project(&projects)?;
    Some(Check {
        category: "project",
        name: format!(
            "Project \"{}\" ({}) — {}",
            current.name,
            current.slug,
            environment_label(current.is_production)
        ),
        status: "pass",
        detail: json!({
            "project_id": current.id,
            "name": current.name,
            "slug": current.slug,
            "is_production": current.is_production,
        }),
        fix: None,
        transient: false,
    })
}

fn check_sessions(sessions: &[BrowserSession]) -> Vec<Check> {
    let names: Vec<&str> = sessions.iter().map(|s| s.name.as_str()).collect();
    vec![Check {
        category: "sessions",
        name: format!("{} local browser session(s)", names.len()),
        status: "pass",
        detail: json!({"active": names, "count": names.len()}),
        fix: None,
        transient: false,
    }]
}

async fn check_version() -> Vec<Check> {
    let current = env!("CARGO_PKG_VERSION");
    let Ok(client) = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
    else {
        return vec![];
    };

    let resp = client
        .get("https://api.github.com/repos/steel-dev/cli/releases/latest")
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", format!("steel-cli/{current}"))
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let data: Value = r.json().await.unwrap_or_default();
            if let Some(tag) = data.get("tag_name").and_then(|v| v.as_str()) {
                let latest = tag.strip_prefix('v').unwrap_or(tag);
                return if is_newer(latest, current) {
                    vec![Check {
                        category: "version",
                        name: format!("Update available: v{current} → v{latest}"),
                        status: "degraded",
                        detail: json!({"current": current, "latest": latest}),
                        fix: Some("steel update"),
                        transient: false,
                    }]
                } else {
                    vec![Check {
                        category: "version",
                        name: format!("v{current} (latest)"),
                        status: "pass",
                        detail: json!({"current": current, "latest": latest}),
                        fix: None,
                        transient: false,
                    }]
                };
            }
            vec![Check {
                category: "version",
                name: format!("v{current} (could not parse latest version)"),
                status: "degraded",
                detail: json!({"current": current}),
                fix: None,
                transient: true,
            }]
        }
        _ => vec![Check {
            category: "version",
            name: format!("v{current} (update check failed)"),
            status: "degraded",
            detail: json!({"current": current}),
            fix: None,
            transient: true,
        }],
    }
}

fn print_text(checks: &[Check]) {
    let use_color = !output::no_color();
    let mut current_category = "";

    for check in checks {
        if check.category != current_category {
            if !current_category.is_empty() {
                println!();
            }
            println!("  {}", format_category(check.category));
            current_category = check.category;
        }

        let symbol = match (check.status, use_color) {
            ("pass", true) => "\x1b[32m✓\x1b[0m",
            ("degraded", true) => "\x1b[33m!\x1b[0m",
            ("fail", true) => "\x1b[31m✗\x1b[0m",
            ("pass", false) => "✓",
            ("degraded", false) => "!",
            ("fail", false) => "✗",
            _ => "?",
        };

        println!("    {symbol} {}", check.name);
        if let Some(fix) = check.fix {
            if use_color {
                println!("      \x1b[2m→ {fix}\x1b[0m");
            } else {
                println!("      → {fix}");
            }
        }
    }
    println!();
}

fn format_category(s: &str) -> &str {
    match s {
        "auth" => "Auth",
        "project" => "Project",
        "api" => "API",
        "sessions" => "Sessions",
        "version" => "Version",
        _ => s,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn failing_report() -> Report {
        Report {
            checks: vec![Check {
                category: "auth",
                name: "API key not configured".into(),
                status: "fail",
                detail: json!(null),
                fix: Some("steel login"),
                transient: false,
            }],
        }
    }

    #[test]
    fn failed_report_is_detected() {
        let report = failing_report();
        assert!(report.failed());
        assert_eq!(report.to_json()["overall"], "fail");
        assert_eq!(report.to_json()["checks"][0]["fix"], "steel login");
    }

    #[test]
    fn failed_report_exits_with_the_doctor_error_code() {
        let err = failing_report().emit().unwrap_err();
        let silent = err.downcast_ref::<output::SilentExit>().unwrap();
        assert_eq!(silent.error_code, "doctor_failed");
    }

    #[test]
    fn format_categories() {
        assert_eq!(format_category("auth"), "Auth");
        assert_eq!(format_category("project"), "Project");
        assert_eq!(format_category("api"), "API");
        assert_eq!(format_category("sessions"), "Sessions");
        assert_eq!(format_category("version"), "Version");
    }

    #[test]
    fn project_check_resolves_single_project() {
        let data = json!({
            "projects": [{
                "id": "p1",
                "name": "Default project",
                "slug": "default",
                "isProduction": false,
                "isDefault": true
            }]
        });

        let check = project_check(Some(&data)).unwrap();
        assert_eq!(check.category, "project");
        assert_eq!(check.status, "pass");
        assert_eq!(
            check.name,
            "Project \"Default project\" (default) — development"
        );
        assert_eq!(check.detail["is_production"], json!(false));
        assert_eq!(check.detail["project_id"], json!("p1"));
    }

    #[test]
    fn project_check_production_label() {
        let data = json!({
            "projects": [{
                "id": "p2",
                "name": "Main",
                "slug": "main",
                "isProduction": true
            }]
        });

        let check = project_check(Some(&data)).unwrap();
        assert_eq!(check.name, "Project \"Main\" (main) — production");
    }

    #[test]
    fn project_check_none_without_data() {
        assert!(project_check(None).is_none());
    }

    #[test]
    fn project_check_none_when_ambiguous() {
        let data = json!({
            "projects": [
                {"id": "a", "name": "A", "slug": "a"},
                {"id": "b", "name": "B", "slug": "b"}
            ]
        });
        assert!(project_check(Some(&data)).is_none());
    }

    #[test]
    fn check_sessions_empty() {
        let checks = check_sessions(&[]);
        assert!(checks.iter().all(|c| c.status == "pass"));
        assert!(checks.iter().any(|c| c.name.contains("0 local")));
    }
}
