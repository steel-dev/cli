use std::time::Instant;

use clap::Parser;
use serde_json::{Value, json};

use crate::api::client::{ApiError, SteelClient};
use crate::browser::daemon::process;
use crate::config::auth::Auth;
use crate::config::settings::ApiMode;
use crate::util::output;

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

pub async fn run(args: Args) -> anyhow::Result<()> {
    let (mode, base_url, auth) = crate::util::api::resolve_with_auth();

    let mut checks = if args.preflight {
        check_auth_and_api(mode, &base_url, &auth).await
    } else {
        let (auth_api, version) =
            tokio::join!(check_auth_and_api(mode, &base_url, &auth), check_version(),);
        let sessions = check_sessions();
        let mut all = auth_api;
        all.extend(sessions);
        all.extend(version);
        all
    };

    // Sort by category order
    const ORDER: &[&str] = &["auth", "api", "sessions", "version"];
    checks.sort_by_key(|c| ORDER.iter().position(|&o| o == c.category).unwrap_or(99));

    let has_fail = checks.iter().any(|c| c.status == "fail");
    let has_degraded = checks.iter().any(|c| c.status == "degraded");
    let overall = if has_fail {
        "fail"
    } else if has_degraded {
        "degraded"
    } else {
        "pass"
    };

    if output::is_json() {
        let data = json!({
            "overall": overall,
            "checks": checks.iter().map(|c| {
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
        });
        output::success_data(data);
    } else {
        print_text(&checks);
    }

    if has_fail {
        std::process::exit(1);
    }

    Ok(())
}

async fn check_auth_and_api(mode: ApiMode, base_url: &str, auth: &Auth) -> Vec<Check> {
    let mut checks = Vec::new();
    let has_key = auth.api_key.is_some();
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
            name: format!("API key configured (source: {source})"),
            status: "pass",
            detail: json!({"source": source, "key": masked}),
            fix: None,
            transient: false,
        });
    }

    let Ok(client) = SteelClient::new() else {
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

    let start = Instant::now();
    let result = client
        .request(
            base_url,
            mode,
            reqwest::Method::GET,
            "/sessions",
            None,
            auth,
        )
        .await;
    let latency_ms = start.elapsed().as_millis();

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
                    name: "API key valid".into(),
                    status: "pass",
                    detail: json!(null),
                    fix: None,
                    transient: false,
                });
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
                name: "API key invalid".into(),
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

fn check_sessions() -> Vec<Check> {
    let names = process::list_daemon_names();
    let mut active = Vec::new();
    let mut cleaned = Vec::new();

    for name in &names {
        if process::is_daemon_alive(name) {
            active.push(name.clone());
        } else {
            process::cleanup_stale(name);
            cleaned.push(name.clone());
        }
    }

    let mut checks = Vec::new();

    if !cleaned.is_empty() {
        checks.push(Check {
            category: "sessions",
            name: format!(
                "{} stale daemon(s) cleaned up ({})",
                cleaned.len(),
                cleaned.join(", ")
            ),
            status: "pass",
            detail: json!({"cleaned": cleaned}),
            fix: None,
            transient: false,
        });
    }

    checks.push(Check {
        category: "sessions",
        name: format!("{} active session(s)", active.len()),
        status: "pass",
        detail: json!({"active": active, "count": active.len()}),
        fix: None,
        transient: false,
    });

    checks
}

async fn check_version() -> Vec<Check> {
    let current = env!("CARGO_PKG_VERSION");
    let client = reqwest::Client::new();

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
                return if is_version_newer(latest, current) {
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
        "api" => "API",
        "sessions" => "Sessions",
        "version" => "Version",
        _ => s,
    }
}

fn is_version_newer(latest: &str, current: &str) -> bool {
    let parse = |s: &str| -> Vec<u64> {
        s.split('.')
            .map(|p| p.parse::<u64>().unwrap_or(0))
            .collect()
    };
    let l = parse(latest);
    let c = parse(current);
    for i in 0..l.len().max(c.len()) {
        let lv = l.get(i).copied().unwrap_or(0);
        let cv = c.get(i).copied().unwrap_or(0);
        if lv > cv {
            return true;
        }
        if lv < cv {
            return false;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_newer_patch() {
        assert!(is_version_newer("0.3.1", "0.3.0"));
    }

    #[test]
    fn version_newer_minor() {
        assert!(is_version_newer("0.4.0", "0.3.9"));
    }

    #[test]
    fn version_same() {
        assert!(!is_version_newer("0.3.0", "0.3.0"));
    }

    #[test]
    fn version_older() {
        assert!(!is_version_newer("0.2.0", "0.3.0"));
    }

    #[test]
    fn version_multi_digit() {
        assert!(is_version_newer("0.10.0", "0.9.1"));
    }

    #[test]
    fn format_categories() {
        assert_eq!(format_category("auth"), "Auth");
        assert_eq!(format_category("api"), "API");
        assert_eq!(format_category("sessions"), "Sessions");
        assert_eq!(format_category("version"), "Version");
    }

    #[test]
    fn check_sessions_empty() {
        // When no daemons exist, should return a single pass check
        let checks = check_sessions();
        assert!(checks.iter().all(|c| c.status == "pass"));
        assert!(checks.iter().any(|c| c.name.contains("0 active")));
    }
}
