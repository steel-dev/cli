//! End-to-end tests for the checkpoint commands against a fake API host.
//!
//! The real `steel` binary runs against a wiremock server, so the request
//! shapes, the waiting loops and the printed tables are exercised as a user
//! sees them.

use std::process::{Command, Output};

use serde_json::json;
use wiremock::matchers::{body_partial_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const COMPUTER: &str = "cmp_0000123456789abcdefghjkmnpqrs";
const CHECKPOINT: &str = "ckpt_0000123456789abcdefghjkmn";

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

fn checkpoint(status: &str) -> serde_json::Value {
    json!({
        "id": CHECKPOINT,
        "computerId": COMPUTER,
        "name": "ready",
        "status": status,
        "sizeBytes": 2_147_483_648u64,
        "statusChangedAt": "2026-09-11T00:00:00Z"
    })
}

#[tokio::test(flavor = "multi_thread")]
async fn checkpoint_create_waits_until_the_snapshot_is_ready() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("/v1/computers/{COMPUTER}/checkpoints")))
        .and(header("steel-api-key", "ste-test-key"))
        .and(body_partial_json(json!({ "name": "ready" })))
        .respond_with(ResponseTemplate::new(201).set_body_json(checkpoint("creating")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/v1/checkpoints/{CHECKPOINT}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(checkpoint("creating")))
        .up_to_n_times(1)
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/v1/checkpoints/{CHECKPOINT}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(checkpoint("ready")))
        .expect(1)
        .mount(&server)
        .await;

    let output = run_steel(
        &server,
        &[
            "computer",
            "checkpoint",
            COMPUTER,
            "--name",
            "ready",
            "--wait",
        ],
    )
    .await;

    assert!(output.status.success());
    assert!(stdout(&output).contains(&format!("{CHECKPOINT} is ready.")));
}

#[tokio::test(flavor = "multi_thread")]
async fn checkpoint_create_stops_when_the_snapshot_fails() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("/v1/computers/{COMPUTER}/checkpoints")))
        .respond_with(ResponseTemplate::new(201).set_body_json(checkpoint("creating")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/v1/checkpoints/{CHECKPOINT}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(checkpoint("failed")))
        .expect(1)
        .mount(&server)
        .await;

    let output = run_steel(&server, &["computer", "checkpoint", COMPUTER, "--wait"]).await;

    assert!(!output.status.success());
}

#[tokio::test(flavor = "multi_thread")]
async fn checkpoint_list_prints_a_table() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/checkpoints"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "checkpoints": [checkpoint("ready")] })),
        )
        .expect(1)
        .mount(&server)
        .await;

    let output = run_steel(&server, &["checkpoint", "list"]).await;

    assert!(output.status.success());
    let text = stdout(&output);
    assert!(text.contains("ID"));
    assert!(text.contains(CHECKPOINT));
    assert!(text.contains("ready"));
    assert!(text.contains("2.0G"));
}

#[tokio::test(flavor = "multi_thread")]
async fn checkpoint_restore_starts_a_computer_and_can_wait_for_it() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("/v1/checkpoints/{CHECKPOINT}/computers")))
        .and(body_partial_json(json!({ "timeoutSeconds": 600 })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": COMPUTER,
            "status": "creating"
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/v1/computers/{COMPUTER}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": COMPUTER,
            "status": "running"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let output = run_steel(
        &server,
        &[
            "checkpoint",
            "restore",
            CHECKPOINT,
            "--timeout",
            "600",
            "--wait",
            "--use",
        ],
    )
    .await;

    assert!(output.status.success());
    let text = stdout(&output);
    assert!(text.contains(&format!("{COMPUTER} is running.")));
    assert!(text.contains("default computer"));
}

#[tokio::test(flavor = "multi_thread")]
async fn checkpoint_delete_reports_the_id() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(format!("/v1/checkpoints/{CHECKPOINT}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(checkpoint("deleting")))
        .expect(1)
        .mount(&server)
        .await;

    let output = run_steel(&server, &["checkpoint", "delete", CHECKPOINT]).await;

    assert!(output.status.success());
    assert!(stdout(&output).contains(&format!("Deleted {CHECKPOINT}.")));
}
