//! Local-only setup contracts. Every endpoint and executable is synthetic.
use std::path::Path;
use std::process::Command;

use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

fn command(home: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_steel"));
    command
        .env("HOME", home)
        .env("STEEL_CONFIG_DIR", home.join("config"))
        .env("STEEL_TELEMETRY_DISABLED", "1")
        .env("STEEL_SKILLS_MANIFEST_URL", "http://127.0.0.1:1/manifest")
        .env_remove("STEEL_API_KEY")
        .env_remove("STEEL_ONBOARDING_FROM")
        .env_remove("XDG_CONFIG_HOME")
        .args(["--no-update-check", "--json"]);
    command
}

#[cfg(unix)]
fn program(home: &Path, name: &str, body: &str) {
    use std::os::unix::fs::PermissionsExt;
    let dir = home.join("bin");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

async fn api() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sessions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"sessions": []})))
        .mount(&server)
        .await;
    server
}

#[tokio::test]
async fn pending_then_resume_preserves_explicit_opencode_target() {
    let home = tempfile::tempdir().unwrap();
    let auth = MockServer::start().await;
    let verification = format!("{}/verify", auth.uri());
    Mock::given(method("POST"))
        .and(path("/api/auth/device/code"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "device_code": "synthetic-device-code",
            "user_code": "TEST-CODE",
            "verification_uri": verification,
            "verification_uri_complete": format!("{}/verify?code=TEST-CODE", auth.uri()),
            "expires_in": 600,
            "interval": 60
        })))
        .mount(&auth)
        .await;

    let first = command(home.path())
        .env("STEEL_AUTH_ISSUER", auth.uri())
        .env("STEEL_AUTH_AUDIENCE", auth.uri())
        .env("STEEL_API_URL", auth.uri())
        .args(["init", "--agent", "--no-skills", "--from", "opencode"])
        .output()
        .unwrap();
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stdout)
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&first.stdout).unwrap()["data"]["client"],
        "opencode"
    );

    let resumed = command(home.path())
        .env("STEEL_AUTH_ISSUER", auth.uri())
        .env("STEEL_AUTH_AUDIENCE", auth.uri())
        .env("STEEL_API_URL", auth.uri())
        .args(["init", "--agent", "--no-skills"])
        .output()
        .unwrap();
    assert!(resumed.status.success());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&resumed.stdout).unwrap()["data"]["client"],
        "opencode"
    );
    assert!(
        std::fs::read_to_string(home.path().join("config/config.json"))
            .unwrap()
            .contains("opencode")
    );
}

#[cfg(unix)]
#[tokio::test]
async fn installs_once_with_exact_noninteractive_opencode_arguments() {
    let home = tempfile::tempdir().unwrap();
    let api = api().await;
    for name in ["node", "git"] {
        program(home.path(), name, "exit 0");
    }
    program(
        home.path(),
        "npx",
        r#"
if [ "$1" = --version ]; then exit 0; fi
if [ -n "$STEEL_API_KEY" ]; then exit 9; fi
if [ -z "$HOME" ] || [ -z "$PATH" ]; then exit 8; fi
echo "$*" >> "$HOME/npx-args"
/bin/mkdir -p "$HOME/.agents/skills/steel-browser"
printf '%s\n' 'fixture skill' > "$HOME/.agents/skills/steel-browser/SKILL.md"
exit 0
"#,
    );
    let run = || {
        command(home.path())
            .env("PATH", home.path().join("bin"))
            .env("STEEL_API_KEY", "synthetic-key")
            .env("STEEL_API_URL", api.uri())
            .args(["init", "--agent", "--from", "opencode"])
            .output()
            .unwrap()
    };
    let first = run();
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stdout)
    );
    assert_eq!(
        String::from_utf8_lossy(&first.stdout)
            .lines()
            .filter(|line| !line.is_empty())
            .count(),
        1
    );
    let second = run();
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stdout)
    );
    let invocations = std::fs::read_to_string(home.path().join("npx-args")).unwrap();
    assert_eq!(invocations.lines().count(), 1);
    assert_eq!(
        invocations.trim(),
        "--yes skills add steel-dev/skills --skill steel-browser -a opencode -g -y"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn unreadable_install_and_installer_failure_do_not_claim_ready() {
    let api = api().await;
    for (body, expected) in [
        (
            "if [ \"$1\" = --version ]; then exit 0; fi\nexit 0",
            "not readable",
        ),
        (
            "if [ \"$1\" = --version ]; then exit 0; fi\nexit 7",
            "Setup incomplete at skill installation",
        ),
    ] {
        let home = tempfile::tempdir().unwrap();
        for name in ["node", "git"] {
            program(home.path(), name, "exit 0");
        }
        program(home.path(), "npx", body);
        let output = command(home.path())
            .env("PATH", home.path().join("bin"))
            .env("STEEL_API_KEY", "synthetic-key")
            .env("STEEL_API_URL", api.uri())
            .args(["init", "--agent", "--from", "opencode"])
            .output()
            .unwrap();
        assert!(!output.status.success());
        let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert!(result["error"].as_str().unwrap().contains(expected));
        assert_ne!(result["data"]["status"], "ready");
    }
}

#[tokio::test]
async fn no_skills_keeps_the_existing_setup_contract() {
    let home = tempfile::tempdir().unwrap();
    let api = api().await;
    let output = command(home.path())
        .env("STEEL_API_KEY", "synthetic-key")
        .env("STEEL_API_URL", api.uri())
        .args(["init", "--agent", "--from", "opencode", "--no-skills"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let lines = String::from_utf8_lossy(&output.stdout);
    assert_eq!(lines.lines().filter(|line| !line.is_empty()).count(), 1);
    let result: serde_json::Value = serde_json::from_str(&lines).unwrap();
    assert_eq!(result["data"]["skills"], "skipped");
}

#[cfg(unix)]
#[tokio::test]
async fn readable_skill_skips_prerequisites_and_installer() {
    let home = tempfile::tempdir().unwrap();
    let api = api().await;
    let skill = home.path().join(".agents/skills/steel-browser/SKILL.md");
    std::fs::create_dir_all(skill.parent().unwrap()).unwrap();
    std::fs::write(&skill, "fixture skill").unwrap();
    let empty_path = home.path().join("empty-path");
    std::fs::create_dir(&empty_path).unwrap();

    let output = command(home.path())
        .env("PATH", empty_path)
        .env("STEEL_API_KEY", "synthetic-key")
        .env("STEEL_API_URL", api.uri())
        .args(["init", "--agent", "--from", "opencode"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(!home.path().join("npx-args").exists());
}

#[cfg(unix)]
#[test]
fn missing_skill_blocks_at_prerequisites_before_auth() {
    let home = tempfile::tempdir().unwrap();
    let empty_path = home.path().join("empty-path");
    std::fs::create_dir(&empty_path).unwrap();
    let output = command(home.path())
        .env("PATH", empty_path)
        .args(["init", "--agent", "--from", "opencode"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        result["error"]
            .as_str()
            .unwrap()
            .contains("`node` is unavailable")
    );
    assert!(!home.path().join("config/auth.json").exists());
}

#[test]
fn from_requires_agent_and_does_not_write_config() {
    let home = tempfile::tempdir().unwrap();
    let output = command(home.path())
        .args(["init", "--from", "opencode", "--no-skills"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("--agent"));
    assert!(!home.path().join("config/config.json").exists());
}
