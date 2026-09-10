//! End-to-end tests for `steel computer exec` against a fake API host.
//!
//! The real `steel` binary runs against a wiremock server that plays the
//! box gateway, so the request shape, the ndjson stream, the collect mode,
//! the wake retry and the exit codes are all exercised as a user sees them.

use std::process::{Command, Output};

use serde_json::json;
use wiremock::matchers::{body_partial_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const COMPUTER: &str = "cmp_0000123456789abcdefghjkmnpqrs";

fn exec_path() -> String {
    format!("/v1/computers/{COMPUTER}/exec")
}

async fn run_steel(server: &MockServer, args: &[&str], env: &[(&str, &str)]) -> Output {
    let tmp = tempfile::tempdir().expect("temp dir");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_steel"));
    cmd.env("STEEL_CONFIG_DIR", tmp.path());
    cmd.env("STEEL_API_URL", format!("{}/v1", server.uri()));
    cmd.env("STEEL_API_KEY", "ste-test-key");
    cmd.env("STEEL_TELEMETRY_DISABLED", "1");
    cmd.env("STEEL_FORCE_TTY", "1");
    cmd.env_remove("STEEL_COMPUTER_ID");
    for (key, value) in env {
        cmd.env(key, value);
    }
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

fn ndjson(events: &[serde_json::Value]) -> ResponseTemplate {
    let body = events
        .iter()
        .map(|event| event.to_string() + "\n")
        .collect::<String>();
    ResponseTemplate::new(200).set_body_raw(body, "application/x-ndjson")
}

#[tokio::test(flavor = "multi_thread")]
async fn exec_streams_output_and_returns_the_remote_exit_code() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(exec_path()))
        .and(header("steel-api-key", "ste-test-key"))
        .and(body_partial_json(json!({
            "argv": ["sh", "-c", "echo hi; exit 3"],
            "cwd": "/tmp",
            "env": { "A": "1" },
            "timeoutSeconds": 30,
            "stream": true
        })))
        .respond_with(ndjson(&[
            json!({"event": "start"}),
            json!({"event": "output", "data": "hi\n"}),
            json!({"event": "keepalive"}),
            json!({"event": "output", "data": "and more"}),
            json!({"event": "exit", "exitCode": 3, "timedOut": false}),
        ]))
        .expect(1)
        .mount(&server)
        .await;

    let output = run_steel(
        &server,
        &[
            "computer",
            "exec",
            COMPUTER,
            "--cwd",
            "/tmp",
            "--env",
            "A=1",
            "--timeout",
            "30",
            "--",
            "sh",
            "-c",
            "echo hi; exit 3",
        ],
        &[],
    )
    .await;

    assert_eq!(stdout(&output), "hi\nand more");
    assert_eq!(output.status.code(), Some(3));
    assert_eq!(stderr(&output), "");
}

#[tokio::test(flavor = "multi_thread")]
async fn exec_uses_the_default_computer_and_maps_timeouts_to_124() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(exec_path()))
        .and(body_partial_json(
            json!({ "command": "sleep 999", "stream": true }),
        ))
        .respond_with(ndjson(&[
            json!({"event": "start"}),
            json!({"event": "exit", "exitCode": -1, "timedOut": true}),
        ]))
        .expect(1)
        .mount(&server)
        .await;

    let output = run_steel(
        &server,
        &["computer", "exec", "-c", "sleep 999"],
        &[("STEEL_COMPUTER_ID", COMPUTER)],
    )
    .await;

    assert_eq!(stdout(&output), "");
    assert_eq!(output.status.code(), Some(124));
}

#[tokio::test(flavor = "multi_thread")]
async fn exec_in_json_mode_collects_one_result() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(exec_path()))
        .and(body_partial_json(
            json!({ "argv": ["id"], "stream": false }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"   {"output":"uid=0(root)\n","exitCode":0,"timedOut":false,"truncated":false}"#,
            "application/json",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let output = run_steel(
        &server,
        &["--json", "computer", "exec", COMPUTER, "--", "id"],
        &[],
    )
    .await;

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let parsed: serde_json::Value = serde_json::from_str(stdout(&output).trim()).unwrap();
    assert_eq!(parsed["success"], json!(true));
    assert_eq!(parsed["data"]["output"], json!("uid=0(root)\n"));
    assert_eq!(parsed["data"]["exitCode"], json!(0));
}

#[tokio::test(flavor = "multi_thread")]
async fn exec_retries_while_the_computer_wakes() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(exec_path()))
        .respond_with(
            ResponseTemplate::new(503)
                .insert_header("Retry-After", "1")
                .set_body_json(json!({"error": "computer is not available"})),
        )
        .up_to_n_times(1)
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(exec_path()))
        .respond_with(ndjson(&[
            json!({"event": "start"}),
            json!({"event": "output", "data": "awake\n"}),
            json!({"event": "exit", "exitCode": 0, "timedOut": false}),
        ]))
        .expect(1)
        .mount(&server)
        .await;

    let output = run_steel(&server, &["computer", "exec", COMPUTER, "--", "true"], &[]).await;

    assert_eq!(stdout(&output), "awake\n");
    assert!(output.status.success());
    assert!(
        stderr(&output).contains("not ready yet"),
        "stderr: {}",
        stderr(&output)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn exec_reports_an_unknown_computer() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(exec_path()))
        .respond_with(
            ResponseTemplate::new(404).set_body_json(json!({"error": "computer not found"})),
        )
        .mount(&server)
        .await;

    let output = run_steel(&server, &["computer", "exec", COMPUTER, "--", "true"], &[]).await;

    assert_eq!(output.status.code(), Some(5));
    assert!(
        stderr(&output).contains("computer not found"),
        "stderr: {}",
        stderr(&output)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn exec_fails_when_the_stream_ends_early() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(exec_path()))
        .respond_with(ndjson(&[
            json!({"event": "start"}),
            json!({"event": "output", "data": "partial"}),
        ]))
        .mount(&server)
        .await;

    let output = run_steel(&server, &["computer", "exec", COMPUTER, "--", "true"], &[]).await;

    assert_eq!(stdout(&output), "partial");
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("before the command finished"),
        "stderr: {}",
        stderr(&output)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn list_prints_a_table_and_create_remembers_the_default() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/computers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "computers": [{
                "id": COMPUTER, "status": "running", "region": "us-east", "template": "steel",
                "vcpu": 2, "memoryMib": 2048, "diskMib": 25600, "timeoutSeconds": 3600,
                "autoPause": false, "checkpointId": null, "statusChangedAt": "2026-09-10T00:00:00Z"
            }]
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/computers"))
        .and(body_partial_json(json!({ "template": "steel", "vcpu": 4 })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": COMPUTER, "status": "creating", "region": null, "template": "steel",
            "vcpu": 4, "memoryMib": 2048, "diskMib": 25600, "timeoutSeconds": 3600,
            "autoPause": false, "checkpointId": null, "statusChangedAt": "2026-09-10T00:00:00Z"
        })))
        .mount(&server)
        .await;

    let output = run_steel(&server, &["computer", "list"], &[]).await;
    let text = stdout(&output);
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(text.starts_with("ID"), "table header missing: {text}");
    assert!(
        text.contains(COMPUTER) && text.contains("running"),
        "{text}"
    );

    let output = run_steel(
        &server,
        &[
            "computer",
            "create",
            "--template",
            "steel",
            "--vcpu",
            "4",
            "--use",
        ],
        &[],
    )
    .await;
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(
        stdout(&output).contains("is creating"),
        "{}",
        stdout(&output)
    );
    assert!(
        stdout(&output).contains("default computer"),
        "{}",
        stdout(&output)
    );
}
