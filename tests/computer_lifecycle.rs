//! End-to-end tests for `steel computer stop|start|restart` against a fake API host.

use std::process::{Command, Output};

use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

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

const ID: &str = "cmp_00x1492gqevd1nvs7vya5py3d0m33";

async fn mount(server: &MockServer, verb: &str, status: &str) {
    Mock::given(method("POST"))
        .and(path(format!("/v1/computers/{ID}/{verb}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": ID,
            "status": status
        })))
        .expect(1)
        .mount(server)
        .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn stop_reports_the_settled_status() {
    let server = MockServer::start().await;
    mount(&server, "stop", "stopped").await;

    let output = run_steel(&server, &["computer", "stop", ID]).await;

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(text.contains(&format!("{ID} is stopped.")));
}

#[tokio::test(flavor = "multi_thread")]
async fn start_reports_the_settled_status() {
    let server = MockServer::start().await;
    mount(&server, "start", "running").await;

    let output = run_steel(&server, &["computer", "start", ID]).await;

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(text.contains(&format!("{ID} is running.")));
}

#[tokio::test(flavor = "multi_thread")]
async fn restart_calls_restart_and_not_start() {
    let server = MockServer::start().await;
    mount(&server, "restart", "running").await;
    Mock::given(method("POST"))
        .and(path(format!("/v1/computers/{ID}/start")))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&server)
        .await;

    let output = run_steel(&server, &["computer", "restart", ID]).await;

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(text.contains(&format!("{ID} is running.")));
}
