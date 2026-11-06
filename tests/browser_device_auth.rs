//! Token-only device credential -> product API -> commands API -> explicit release.
//! Exercises the real binary and wire format without any live service or product secret.
use serde_json::json;
use std::path::Path;
use std::process::Output;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{header, method, path},
};

const ID: &str = "4f7c2b1e-9a3d-4e5f-8b6a-1c2d3e4f5a6b";
const BEARER: &str = "Bearer synthetic-access-token";

fn write_device_state(home: &Path, api_base_url: &str) {
    let issuer = api_base_url.trim_end_matches("/v1");
    use std::os::unix::fs::PermissionsExt;
    let auth = json!({"settings":{"issuer":issuer,"audience":"https://resource.example.test","api_base_url":api_base_url},"installation_id":"synthetic-installation","pending":null,"tokens":{"access_token":"synthetic-access-token","refresh_token":"synthetic-refresh-token","expires_at":u64::MAX}});
    let file = home.join("auth.json");
    std::fs::write(&file, serde_json::to_vec(&auth).unwrap()).unwrap();
    std::fs::set_permissions(file, std::fs::Permissions::from_mode(0o600)).unwrap();
}

async fn steel(home: &Path, api_url: Option<&str>, args: &[&str]) -> Output {
    let mut cmd = tokio::process::Command::new(env!("CARGO_BIN_EXE_steel"));
    cmd.current_dir(home)
        .env("HOME", home)
        .env("STEEL_CONFIG_DIR", home)
        .env("STEEL_TELEMETRY_DISABLED", "1")
        .env_remove("STEEL_API_KEY")
        .env_remove("STEEL_API_URL")
        .env_remove("STEEL_AUTH_ISSUER")
        .env_remove("STEEL_AUTH_AUDIENCE");
    if let Some(url) = api_url {
        cmd.env("STEEL_API_URL", url);
    }
    cmd.args(["--no-update-check", "--json"])
        .args(args)
        .output()
        .await
        .unwrap()
}

fn shown(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[tokio::test]
async fn device_auth_uses_bearer_for_start_commands_and_release() {
    let api = MockServer::start().await;
    let websocket_url =
        format!("wss://connect.steel.dev?sessionId={ID}&token=synthetic-connect-token");
    Mock::given(method("POST"))
        .and(path("/v1/sessions"))
        .and(header("authorization", BEARER))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({"id":ID,"status":"live","timeout":300000,"websocketUrl":websocket_url}),
        ))
        .expect(1)
        .mount(&api)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("/v1/sessions/{ID}/commands")))
        .and(header("authorization", BEARER))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "next": "op_1",
            "epoch": 1,
            "page": {"tab": "t1", "url": "https://example.com/", "title": "Example"},
            "execution": {"id": "x-1", "state": "completed", "effect": "applied", "replayed": false, "ran": 1},
            "results": [{"ok": true, "effect": "applied"}],
            "events": {"cursor": "c-1", "items": []}
        })))
        .expect(1)
        .mount(&api)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("/v1/sessions/{ID}/release")))
        .and(header("authorization", BEARER))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success": true})))
        .expect(1)
        .mount(&api)
        .await;
    let home = tempfile::tempdir().unwrap();
    write_device_state(home.path(), &format!("{}/v1", api.uri()));

    let start = steel(
        home.path(),
        None,
        &["browser", "start", "--session", "test"],
    )
    .await;
    assert!(start.status.success(), "{}", shown(&start));
    assert!(!shown(&start).contains("synthetic-connect-token"));
    assert!(!shown(&start).contains("synthetic-access-token"));

    let click = steel(
        home.path(),
        None,
        &["browser", "click", "e5", "--session", "test"],
    )
    .await;
    assert!(click.status.success(), "{}", shown(&click));

    let stop = steel(home.path(), None, &["browser", "stop", "--session", "test"]).await;
    assert!(stop.status.success(), "{}", shown(&stop));

    let requests = api.received_requests().await.unwrap();
    assert_eq!(requests.len(), 3);
    assert!(
        requests
            .iter()
            .all(|r| !r.headers.contains_key("steel-api-key"))
    );
}

#[tokio::test]
async fn device_bearer_is_never_sent_to_a_redirect_target() {
    for status in [302, 307, 308] {
        let origin = MockServer::start().await;
        let target = MockServer::start().await;
        Mock::given(wiremock::matchers::any())
            .respond_with(
                ResponseTemplate::new(status)
                    .insert_header("location", format!("{}/v1/sessions", target.uri()).as_str()),
            )
            .mount(&origin)
            .await;
        let home = tempfile::tempdir().unwrap();
        write_device_state(home.path(), &format!("{}/v1", origin.uri()));

        let output = steel(home.path(), None, &["browser", "start"]).await;

        assert!(!output.status.success(), "{}", shown(&output));
        assert!(shown(&output).contains("redirect"), "{}", shown(&output));
        assert!(!shown(&output).contains("synthetic-access-token"));
        let sent = origin.received_requests().await.unwrap();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].headers["authorization"], BEARER);
        assert!(target.received_requests().await.unwrap().is_empty());
    }
}

#[tokio::test]
async fn device_auth_refuses_an_api_url_it_was_not_issued_for() {
    let issued = MockServer::start().await;
    let other = MockServer::start().await;
    let home = tempfile::tempdir().unwrap();
    write_device_state(home.path(), &format!("{}/v1", issued.uri()));

    let other_url = format!("{}/v1", other.uri());
    let output = steel(home.path(), Some(&other_url), &["browser", "start"]).await;
    assert!(!output.status.success(), "{}", shown(&output));
    assert!(issued.received_requests().await.unwrap().is_empty());
    assert!(other.received_requests().await.unwrap().is_empty());
    assert!(!shown(&output).contains("synthetic-access-token"));
}

#[tokio::test]
async fn config_reports_device_login_without_the_token() {
    let api = MockServer::start().await;
    let home = tempfile::tempdir().unwrap();
    write_device_state(home.path(), &format!("{}/v1", api.uri()));

    let output = steel(home.path(), None, &["config"]).await;

    assert!(output.status.success(), "{}", shown(&output));
    let out: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(out["data"]["loggedIn"], true);
    assert_eq!(out["data"]["source"], "device (Bearer)");
    assert!(!shown(&output).contains("synthetic-access-token"));
    assert!(!shown(&output).contains("synthetic-refresh-token"));
}
