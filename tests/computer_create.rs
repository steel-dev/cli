//! End-to-end tests for `steel computer create` with secrets, an environment and a project.

use std::process::{Command, Output};

use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const COMPUTER: &str = "cmp_0000123456789abcdefghjkmnpqrs";
const ENVIRONMENT: &str = "9b2d1c5e-1b2a-4c3d-8e4f-5a6b7c8d9e0f";
const SECRET: &str = "2f1b6f2e-2c6f-4d9a-9c2a-8f4c1e0d7b31";
const PROJECT: &str = "0c1d2e3f-4a5b-4c6d-8e7f-8091a2b3c4d5";

async fn run_steel(server: &MockServer, args: &[&str]) -> Output {
    let tmp = tempfile::tempdir().expect("temp dir");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_steel"));
    cmd.env("STEEL_CONFIG_DIR", tmp.path());
    cmd.env("STEEL_API_URL", format!("{}/v1", server.uri()));
    cmd.env("STEEL_API_KEY", "ste-test-key");
    cmd.env("STEEL_TELEMETRY_DISABLED", "1");
    cmd.env("STEEL_FORCE_TTY", "1");
    cmd.env_remove("STEEL_COMPUTER_ID");
    cmd.arg("--no-update-check");
    cmd.args(args);
    tokio::task::spawn_blocking(move || {
        let output = cmd.output().expect("failed to execute steel binary");
        drop(tmp);
        output
    })
    .await
    .expect("steel process")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

#[tokio::test(flavor = "multi_thread")]
async fn create_sends_secrets_environment_and_project() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/computers"))
        .and(body_json(json!({
            "environmentId": ENVIRONMENT,
            "secrets": { "TOKEN": SECRET },
            "projectId": PROJECT,
            "env": { "A": "1" }
        })))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(json!({ "id": COMPUTER, "status": "starting" })),
        )
        .expect(1)
        .mount(&server)
        .await;

    let output = run_steel(
        &server,
        &[
            "computer",
            "create",
            "--environment",
            ENVIRONMENT,
            "--secret",
            &format!("TOKEN={SECRET}"),
            "--project",
            PROJECT,
            "--env",
            "A=1",
        ],
    )
    .await;

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stdout(&output).contains(&format!("{COMPUTER} is starting.")));
}

#[tokio::test(flavor = "multi_thread")]
async fn create_refuses_plain_values_for_secret() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/computers"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&server)
        .await;

    let output = run_steel(
        &server,
        &["computer", "create", "--secret", "TOKEN=sk-live-abc"],
    )
    .await;

    assert!(!output.status.success());
    assert!(stderr(&output).contains("steel secret create"));
}
