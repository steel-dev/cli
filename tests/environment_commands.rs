//! End-to-end tests for `steel environment` against a fake API host.

use std::process::{Command, Output};

use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

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

fn environment() -> serde_json::Value {
    json!({
        "id": ENVIRONMENT,
        "name": "dev",
        "version": 1,
        "spec": { "template": "steel", "vcpu": 2, "env": { "A": "1" } },
        "secrets": { "TOKEN": SECRET, "OLD": SECRET },
        "networkSecrets": [],
        "createdAt": "2026-09-12T00:00:00Z",
        "updatedAt": "2026-09-12T01:00:00Z"
    })
}

#[tokio::test(flavor = "multi_thread")]
async fn environment_list_prints_a_table() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/environments"))
        .and(query_param("projectId", PROJECT))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "environments": [environment()] })),
        )
        .expect(1)
        .mount(&server)
        .await;

    let output = run_steel(&server, &["environment", "list", "--project", PROJECT]).await;

    assert!(output.status.success(), "{}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("TEMPLATE"));
    assert!(text.contains(ENVIRONMENT));
    assert!(text.contains("dev"));
    assert!(text.contains("steel"));
    assert!(text.contains('2'));
}

#[tokio::test(flavor = "multi_thread")]
async fn environment_create_sends_the_spec_and_secrets() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/environments"))
        .and(body_json(json!({
            "name": "dev",
            "projectId": PROJECT,
            "spec": { "template": "steel", "vcpu": 2, "autoPause": true, "env": { "A": "1" } },
            "secrets": { "TOKEN": { "secretId": SECRET }, "KEY": { "value": "abc" } }
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(environment()))
        .expect(1)
        .mount(&server)
        .await;

    let output = run_steel(
        &server,
        &[
            "environment",
            "create",
            "dev",
            "--template",
            "steel",
            "--vcpu",
            "2",
            "--auto-pause",
            "--env",
            "A=1",
            "--secret",
            &format!("TOKEN={SECRET}"),
            "--secret-value",
            "KEY=abc",
            "--project",
            PROJECT,
        ],
    )
    .await;

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stdout(&output).contains(&format!("Created dev as {ENVIRONMENT}.")));
}

#[tokio::test(flavor = "multi_thread")]
async fn environment_create_refuses_plain_values_for_secret() {
    let server = MockServer::start().await;

    let output = run_steel(
        &server,
        &[
            "environment",
            "create",
            "dev",
            "--secret",
            "TOKEN=sk-live-abc",
        ],
    )
    .await;

    assert!(!output.status.success());
    assert!(stderr(&output).contains("steel secret create"));
}

#[tokio::test(flavor = "multi_thread")]
async fn environment_update_merges_the_spec_and_unsets_secrets() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/v1/environments/{ENVIRONMENT}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(environment()))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("/v1/environments/{ENVIRONMENT}")))
        .and(body_json(json!({
            "name": "staging",
            "spec": { "template": "steel", "vcpu": 2, "memoryMib": 4096, "env": { "A": "1", "B": "2" } },
            "secrets": { "OLD": null }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(environment()))
        .expect(1)
        .mount(&server)
        .await;

    let output = run_steel(
        &server,
        &[
            "environment",
            "update",
            ENVIRONMENT,
            "--name",
            "staging",
            "--memory",
            "4096",
            "--env",
            "B=2",
            "--unset-secret",
            "OLD",
        ],
    )
    .await;

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stdout(&output).contains(&format!("Updated {ENVIRONMENT}.")));
}

#[tokio::test(flavor = "multi_thread")]
async fn environment_update_skips_the_fetch_without_spec_changes() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/v1/environments/{ENVIRONMENT}")))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("/v1/environments/{ENVIRONMENT}")))
        .and(body_json(
            json!({ "secrets": { "TOKEN": { "secretId": SECRET } } }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(environment()))
        .expect(1)
        .mount(&server)
        .await;

    let output = run_steel(
        &server,
        &[
            "environment",
            "update",
            ENVIRONMENT,
            "--secret",
            &format!("TOKEN={SECRET}"),
        ],
    )
    .await;

    assert!(output.status.success(), "{}", stderr(&output));
}

#[tokio::test(flavor = "multi_thread")]
async fn environment_update_needs_a_change() {
    let server = MockServer::start().await;

    let output = run_steel(&server, &["environment", "update", ENVIRONMENT]).await;

    assert!(!output.status.success());
    assert!(stderr(&output).contains("--unset-secret"));
}

#[tokio::test(flavor = "multi_thread")]
async fn environment_get_and_delete_use_the_id() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/v1/environments/{ENVIRONMENT}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(environment()))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("/v1/environments/{ENVIRONMENT}")))
        .and(query_param("projectId", PROJECT))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = run_steel(&server, &["--json", "environment", "get", ENVIRONMENT]).await;
    assert!(output.status.success(), "{}", stderr(&output));
    let parsed: serde_json::Value = serde_json::from_str(&stdout(&output)).expect("json");
    assert_eq!(parsed["data"]["name"], "dev");

    let output = run_steel(
        &server,
        &["environment", "delete", ENVIRONMENT, "--project", PROJECT],
    )
    .await;
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stdout(&output).contains(&format!("Deleted {ENVIRONMENT}.")));
}
