//! End-to-end tests for `steel secret` against a fake API host.

use std::io::Write;
use std::process::{Command, Output, Stdio};

use serde_json::json;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const SECRET: &str = "2f1b6f2e-2c6f-4d9a-9c2a-8f4c1e0d7b31";
const PROJECT: &str = "0c1d2e3f-4a5b-4c6d-8e7f-8091a2b3c4d5";

fn command(server: &MockServer, dir: &std::path::Path, args: &[&str]) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_steel"));
    cmd.env("STEEL_CONFIG_DIR", dir);
    cmd.env("STEEL_API_URL", format!("{}/v1", server.uri()));
    cmd.env("STEEL_API_KEY", "ste-test-key");
    cmd.env("STEEL_TELEMETRY_DISABLED", "1");
    cmd.env("STEEL_FORCE_TTY", "1");
    cmd.arg("--no-update-check");
    cmd.args(args);
    cmd
}

async fn run_steel(server: &MockServer, args: &[&str]) -> Output {
    let tmp = tempfile::tempdir().expect("temp dir");
    let mut cmd = command(server, tmp.path(), args);
    tokio::task::spawn_blocking(move || {
        let output = cmd.output().expect("failed to execute steel binary");
        drop(tmp);
        output
    })
    .await
    .expect("steel process")
}

async fn run_steel_with_stdin(server: &MockServer, args: &[&str], stdin: &str) -> Output {
    let tmp = tempfile::tempdir().expect("temp dir");
    let mut cmd = command(server, tmp.path(), args);
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let input = stdin.to_string();
    tokio::task::spawn_blocking(move || {
        let mut child = cmd.spawn().expect("failed to execute steel binary");
        child
            .stdin
            .take()
            .expect("stdin")
            .write_all(input.as_bytes())
            .expect("write stdin");
        let output = child.wait_with_output().expect("steel process");
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

fn secret(name: &str, version: u64) -> serde_json::Value {
    json!({
        "id": SECRET,
        "name": name,
        "source": "steel",
        "version": version,
        "createdAt": "2026-09-12T00:00:00Z",
        "updatedAt": "2026-09-12T01:00:00Z"
    })
}

#[tokio::test(flavor = "multi_thread")]
async fn secret_list_prints_a_table() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/secrets"))
        .and(header("steel-api-key", "ste-test-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "secrets": [secret("TOKEN", 3)] })),
        )
        .expect(1)
        .mount(&server)
        .await;

    let output = run_steel(&server, &["secret", "list"]).await;

    assert!(output.status.success());
    let text = stdout(&output);
    assert!(text.contains("ID"));
    assert!(text.contains("VERSION"));
    assert!(text.contains(SECRET));
    assert!(text.contains("TOKEN"));
    assert!(text.contains('3'));
}

#[tokio::test(flavor = "multi_thread")]
async fn secret_list_scopes_to_a_project() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/secrets"))
        .and(query_param("projectId", PROJECT))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "secrets": [] })))
        .expect(1)
        .mount(&server)
        .await;

    let output = run_steel(&server, &["secret", "list", "--project", PROJECT]).await;

    assert!(output.status.success());
    assert!(stdout(&output).contains("No secrets."));
}

#[tokio::test(flavor = "multi_thread")]
async fn secret_create_sends_the_value_and_project() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/secrets"))
        .and(body_json(json!({
            "name": "TOKEN",
            "value": "sk-live-abc",
            "projectId": PROJECT
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(secret("TOKEN", 1)))
        .expect(1)
        .mount(&server)
        .await;

    let output = run_steel(
        &server,
        &[
            "secret",
            "create",
            "TOKEN",
            "--value",
            "sk-live-abc",
            "--project",
            PROJECT,
        ],
    )
    .await;

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stdout(&output).contains(&format!("Created TOKEN as {SECRET}.")));
}

#[tokio::test(flavor = "multi_thread")]
async fn secret_create_reads_the_value_from_stdin() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/secrets"))
        .and(body_json(json!({ "name": "TOKEN", "value": "from stdin" })))
        .respond_with(ResponseTemplate::new(201).set_body_json(secret("TOKEN", 1)))
        .expect(1)
        .mount(&server)
        .await;

    let output = run_steel_with_stdin(
        &server,
        &["secret", "create", "TOKEN", "--value-stdin"],
        "from stdin\n",
    )
    .await;

    assert!(output.status.success(), "{}", stderr(&output));
}

#[tokio::test(flavor = "multi_thread")]
async fn secret_create_needs_a_value() {
    let server = MockServer::start().await;

    let output = run_steel(&server, &["secret", "create", "TOKEN"]).await;

    assert!(!output.status.success());
    assert!(stderr(&output).contains("--value"));
}

#[tokio::test(flavor = "multi_thread")]
async fn secret_get_prints_the_metadata() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/v1/secrets/{SECRET}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(secret("TOKEN", 2)))
        .expect(1)
        .mount(&server)
        .await;

    let output = run_steel(&server, &["--json", "secret", "get", SECRET]).await;

    assert!(output.status.success());
    let parsed: serde_json::Value = serde_json::from_str(&stdout(&output)).expect("json");
    assert_eq!(parsed["data"]["id"], SECRET);
    assert_eq!(parsed["data"]["version"], 2);
}

#[tokio::test(flavor = "multi_thread")]
async fn secret_update_sends_only_the_given_fields() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path(format!("/v1/secrets/{SECRET}")))
        .and(query_param("projectId", PROJECT))
        .and(body_json(json!({ "value": "rotated" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(secret("TOKEN", 2)))
        .expect(1)
        .mount(&server)
        .await;

    let output = run_steel(
        &server,
        &[
            "secret",
            "update",
            SECRET,
            "--value",
            "rotated",
            "--project",
            PROJECT,
        ],
    )
    .await;

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stdout(&output).contains(&format!("Updated {SECRET}.")));
}

#[tokio::test(flavor = "multi_thread")]
async fn secret_update_needs_a_change() {
    let server = MockServer::start().await;

    let output = run_steel(&server, &["secret", "update", SECRET]).await;

    assert!(!output.status.success());
    assert!(stderr(&output).contains("--name"));
}

#[tokio::test(flavor = "multi_thread")]
async fn secret_delete_reports_the_id() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(format!("/v1/secrets/{SECRET}")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let output = run_steel(&server, &["secret", "delete", SECRET]).await;

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stdout(&output).contains(&format!("Deleted {SECRET}.")));
}
