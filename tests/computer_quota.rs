//! End-to-end test for `steel computer quota` against a fake API host.

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

#[tokio::test(flavor = "multi_thread")]
async fn quota_prints_usage_against_each_limit() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/computers/quota"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "computerCount": 2,
            "computerLimit": 5,
            "runningCount": 1,
            "runningLimit": 3,
            "checkpointCount": 0,
            "checkpointLimit": 2
        })))
        .expect(1)
        .mount(&server)
        .await;

    let output = run_steel(&server, &["computer", "quota"]).await;

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(text.contains("Computers    2/5"));
    assert!(text.contains("Running      1/3"));
    assert!(text.contains("Checkpoints  0/2"));
}
