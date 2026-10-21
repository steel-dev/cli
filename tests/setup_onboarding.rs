//! Isolated setup contracts. Fake prerequisites and API; no live credentials.
use std::process::Command;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

fn command(home: &std::path::Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_steel"));
    cmd.env("HOME", home)
        .env("STEEL_CONFIG_DIR", home.join("config"))
        .env("STEEL_TELEMETRY_DISABLED", "1")
        .env("STEEL_SKILLS_MANIFEST_URL", "http://127.0.0.1:1/manifest")
        .env_remove("STEEL_API_KEY")
        .env_remove("STEEL_ONBOARDING_FROM")
        .env_remove("STEEL_AUTH_ISSUER")
        .env_remove("STEEL_AUTH_AUDIENCE")
        .env_remove("CLAUDE_CONFIG_DIR")
        .env_remove("XDG_CONFIG_HOME")
        .args(["--no-update-check", "--json"]);
    cmd
}

#[cfg(unix)]
fn program(home: &std::path::Path, name: &str, body: &str) {
    use std::os::unix::fs::PermissionsExt;
    let dir = home.join("bin");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn missing_target_is_actionable_and_does_not_wait_for_auth() {
    let home = tempfile::tempdir().unwrap();
    let out = command(home.path())
        .args(["init", "--agent"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("--from"));
}

#[test]
fn missing_node_is_blocked_before_auth() {
    let home = tempfile::tempdir().unwrap();
    std::fs::create_dir(home.path().join("empty")).unwrap();
    let out = command(home.path())
        .env("PATH", home.path().join("empty"))
        .args(["init", "--agent", "--from", "opencode"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("`node` is unavailable"));
}

#[test]
fn device_auth_requires_explicit_configuration_and_preserves_setup_source() {
    let home = tempfile::tempdir().unwrap();
    let out = command(home.path())
        .args(["init", "--agent", "--no-skills", "--from", "pi"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("STEEL_AUTH_ISSUER"));
    assert!(!text.contains("cli_redirect"));
    let cfg = std::fs::read_to_string(home.path().join("config/config.json")).unwrap();
    assert!(cfg.contains("pi"));
    // A resumed invocation retains source even though auth has not completed.
    let again = command(home.path())
        .args(["init", "--agent", "--no-skills"])
        .output()
        .unwrap();
    assert!(!again.status.success());
    assert!(
        std::fs::read_to_string(home.path().join("config/config.json"))
            .unwrap()
            .contains("pi")
    );
}

async fn api() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sessions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/projects"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;
    server
}

#[tokio::test]
async fn configured_environment_does_not_claim_skill_loading_or_task_success() {
    let home = tempfile::tempdir().unwrap();
    let api = api().await;
    let out = command(home.path())
        .env("STEEL_API_KEY", "test-fixture-not-a-secret")
        .env("STEEL_API_URL", api.uri())
        .args(["init", "--agent", "--no-skills"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("incomplete"));
    assert!(text.contains("unverified"));
    assert!(!text.contains("Steel is ready"));
}

#[cfg(unix)]
#[tokio::test]
async fn skills_failure_is_not_swallowed_and_npx_is_noninteractive() {
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
echo "$*" > "$HOME/npx-args"
echo "$npm_config_engine_strict" > "$HOME/engine-strict"
exit 1
"#,
    );
    let out = command(home.path())
        .env("PATH", home.path().join("bin"))
        .env("STEEL_API_KEY", "test-fixture-not-a-secret")
        .env("STEEL_API_URL", api.uri())
        .args(["init", "--agent", "--from", "opencode"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("Setup incomplete at skill installation")
    );
    assert_eq!(
        std::fs::read_to_string(home.path().join("npx-args"))
            .unwrap()
            .trim(),
        "--yes skills add steel-dev/skills --skill steel-browser -a opencode -g -y"
    );
    assert_eq!(
        std::fs::read_to_string(home.path().join("engine-strict"))
            .unwrap()
            .trim(),
        "true"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn installer_success_without_readable_skill_is_incomplete() {
    let home = tempfile::tempdir().unwrap();
    let api = api().await;
    for name in ["node", "git", "npx"] {
        program(home.path(), name, "exit 0");
    }
    let out = command(home.path())
        .env("PATH", home.path().join("bin"))
        .env("STEEL_API_KEY", "test-fixture-not-a-secret")
        .env("STEEL_API_URL", api.uri())
        .args(["init", "--agent", "--from", "opencode"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("not readable"));
}

#[cfg(unix)]
#[tokio::test]
async fn all_client_targets_install_readable_skills_and_resume() {
    let api = api().await;
    for agent in ["claude-code", "codex", "cursor", "opencode", "pi"] {
        let home = tempfile::tempdir().unwrap();
        for name in ["node", "git"] {
            program(home.path(), name, "exit 0");
        }
        program(
            home.path(),
            "npx",
            r#"
if [ "$1" = --version ]; then exit 0; fi
case "$*" in
  *"-a claude-code"*) root="$HOME/.claude/skills" ;;
  *) root="$HOME/.agents/skills" ;;
esac
/bin/mkdir -p "$root/steel-browser"
printf '%s\n' '---' 'name: steel-browser' 'description: fixture skill' '---' > "$root/steel-browser/SKILL.md"
"#,
        );
        let mut cmd = command(home.path());
        cmd.env("PATH", home.path().join("bin"))
            .env("STEEL_API_KEY", "test-fixture-not-a-secret")
            .env("STEEL_API_URL", api.uri());
        let out = cmd
            .args(["init", "--agent", "--from", agent])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{agent}: {}",
            String::from_utf8_lossy(&out.stdout)
        );
        let mut cmd = command(home.path());
        let again = cmd
            .env("PATH", home.path().join("bin"))
            .env("STEEL_API_KEY", "test-fixture-not-a-secret")
            .env("STEEL_API_URL", api.uri())
            .args(["init", "--agent"])
            .output()
            .unwrap();
        assert!(again.status.success(), "{agent} resume failed");
        let text = String::from_utf8_lossy(&again.stdout);
        assert!(text.contains("unverified"));
        assert!(text.contains(agent));
        if agent == "pi" {
            assert!(text.contains("/reload"));
        }
    }
}
