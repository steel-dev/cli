//! Device grant/refresh transport contracts. No real tokens or user config.
use serde_json::{Value, json};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_string_contains, header, method, path},
};

fn command(home: &std::path::Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_steel"));
    command
        .env("HOME", home)
        .env("STEEL_CONFIG_DIR", home)
        .env("STEEL_TELEMETRY_DISABLED", "1")
        .env_remove("STEEL_API_KEY")
        .env_remove("STEEL_API_URL")
        .env_remove("STEEL_AUTH_ISSUER")
        .env_remove("STEEL_AUTH_AUDIENCE")
        .args(["--no-update-check", "--json"]);
    command
}

fn write_state(home: &std::path::Path, issuer: &str, api: &str, expires: u64) {
    let state = json!({"settings":{"issuer":issuer,"audience":"https://resource.example.test","api_base_url":api},
        "installation_id":"test-installation-123", "pending":null,
        "tokens":{"access_token":"old-access-fixture", "refresh_token":"old-refresh-fixture", "expires_at":expires}});
    let path = home.join("auth.json");
    std::fs::write(&path, serde_json::to_vec(&state).unwrap()).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
}

fn text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

async fn device(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/api/auth/device/code"))
        .and(body_string_contains("client_id=steel-cli"))
        .and(body_string_contains(
            "resource=https%3A%2F%2Fresource.example.test",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "device_code":"private-device-fixture", "user_code":"ABCD2345",
            "verification_uri":format!("{}/device",server.uri()),
            "verification_uri_complete":format!("{}/device?user_code=ABCD2345",server.uri()),
            "expires_in":600,"interval":1
        })))
        .expect(1)
        .mount(server)
        .await;
}

#[tokio::test]
async fn saved_pending_login_resumes_same_code_and_tokens_are_private() {
    let home = tempfile::tempdir().unwrap();
    let auth = MockServer::start().await;
    let api = MockServer::start().await;
    device(&auth).await;
    Mock::given(method("POST")).and(path("/api/auth/oauth2/token"))
        .and(body_string_contains("device_code=private-device-fixture"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"access_token":"access-fixture-secret","refresh_token":"refresh-fixture-secret","expires_in":600,"token_type":"Bearer"})))
        .expect(1).mount(&auth).await;
    let first = command(home.path())
        .env("STEEL_AUTH_ISSUER", auth.uri())
        .env("STEEL_AUTH_AUDIENCE", "https://resource.example.test")
        .env("STEEL_API_URL", api.uri())
        .args(["login", "--no-wait", "--no-browser"])
        .output()
        .unwrap();
    assert!(first.status.success(), "{}", text(&first));
    assert!(text(&first).contains("pending"));
    assert!(text(&first).contains("ABCD2345"));
    assert!(!text(&first).contains("private-device-fixture"));
    let again = command(home.path())
        .args(["login", "--resume", "--no-browser"])
        .output()
        .unwrap();
    assert!(again.status.success(), "{}", text(&again));
    assert!(!text(&again).contains("access-fixture-secret"));
    assert!(!text(&again).contains("refresh-fixture-secret"));
    let state: Value =
        serde_json::from_slice(&std::fs::read(home.path().join("auth.json")).unwrap()).unwrap();
    assert!(state["pending"].is_null());
    assert_eq!(state["tokens"]["refresh_token"], "refresh-fixture-secret");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(home.path().join("auth.json"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert_eq!(
            std::fs::metadata(home.path()).unwrap().permissions().mode() & 0o777,
            0o700
        );
    }
}

#[tokio::test]
async fn concurrent_product_commands_single_flight_rotating_refresh_and_use_bearer() {
    let home = tempfile::tempdir().unwrap();
    let auth = MockServer::start().await;
    let api = MockServer::start().await;
    write_state(home.path(), &auth.uri(), &api.uri(), 0);
    Mock::given(method("POST")).and(path("/api/auth/oauth2/token"))
        .and(body_string_contains("grant_type=refresh_token"))
        .and(body_string_contains("refresh_token=old-refresh-fixture"))
        .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_millis(150))
            .set_body_json(json!({"access_token":"new-access-fixture", "refresh_token":"rotated-refresh-fixture", "expires_in":600,"token_type":"Bearer"})))
        .expect(1).mount(&auth).await;
    Mock::given(method("GET"))
        .and(path("/sessions"))
        .and(header("Authorization", "Bearer new-access-fixture"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(2)
        .mount(&api)
        .await;
    let a = command(home.path())
        .args(["sessions", "list"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let b = command(home.path())
        .args(["sessions", "list"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    for output in [a.wait_with_output().unwrap(), b.wait_with_output().unwrap()] {
        assert!(output.status.success(), "{}", text(&output));
        assert!(!text(&output).contains("new-access-fixture"));
    }
    for request in api.received_requests().await.unwrap() {
        assert!(!request.headers.contains_key("steel-api-key"));
    }
    let state: Value =
        serde_json::from_slice(&std::fs::read(home.path().join("auth.json")).unwrap()).unwrap();
    assert_eq!(state["tokens"]["refresh_token"], "rotated-refresh-fixture");
    assert_eq!(state["refresh_in_flight"], false);
}

#[tokio::test]
async fn refresh_without_a_replacement_token_never_reuses_the_old_token() {
    let home = tempfile::tempdir().unwrap();
    let auth = MockServer::start().await;
    let api = MockServer::start().await;
    write_state(home.path(), &auth.uri(), &api.uri(), 0);
    Mock::given(method("POST"))
        .and(path("/api/auth/oauth2/token"))
        .and(body_string_contains("refresh_token=old-refresh-fixture"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({"access_token":"replacement-access-fixture", "expires_in":600,"token_type":"Bearer"}),
        ))
        .expect(1)
        .mount(&auth)
        .await;
    Mock::given(method("GET"))
        .and(path("/sessions"))
        .and(header("Authorization", "Bearer replacement-access-fixture"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&api)
        .await;

    let output = command(home.path())
        .args(["sessions", "list"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", text(&output));
    let state: Value =
        serde_json::from_slice(&std::fs::read(home.path().join("auth.json")).unwrap()).unwrap();
    assert!(state["tokens"]["refresh_token"].is_null());
}

#[tokio::test]
async fn context_change_never_sends_a_saved_token_to_other_transport() {
    let home = tempfile::tempdir().unwrap();
    let auth = MockServer::start().await;
    let intended = MockServer::start().await;
    let other = MockServer::start().await;
    write_state(home.path(), &auth.uri(), &intended.uri(), u64::MAX);
    let output = command(home.path())
        .env("STEEL_API_URL", other.uri())
        .args(["sessions", "list"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(text(&output).contains("Refusing to send tokens"));
    assert!(other.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn uncertain_refresh_is_not_replayed_after_process_interruption() {
    let home = tempfile::tempdir().unwrap();
    let auth = MockServer::start().await;
    let api = MockServer::start().await;
    write_state(home.path(), &auth.uri(), &api.uri(), 0);
    let path = home.path().join("auth.json");
    let mut state: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    state["refresh_in_flight"] = json!(true);
    std::fs::write(&path, serde_json::to_vec(&state).unwrap()).unwrap();
    let output = command(home.path())
        .args(["sessions", "list"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(text(&output).contains("avoid replaying"));
    assert!(auth.received_requests().await.unwrap().is_empty());
    assert!(api.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn logout_revokes_refresh_token_before_removing_local_state() {
    let home = tempfile::tempdir().unwrap();
    let auth = MockServer::start().await;
    write_state(
        home.path(),
        &auth.uri(),
        "https://product.example.test/v1",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 600,
    );
    Mock::given(method("POST"))
        .and(path("/api/auth/oauth2/revoke"))
        .and(body_string_contains("client_id=steel-cli"))
        .and(body_string_contains("token=old-refresh-fixture"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&auth)
        .await;
    let output = command(home.path()).arg("logout").output().unwrap();
    assert!(output.status.success(), "{}", text(&output));
    assert!(!home.path().join("auth.json").exists());
}

#[tokio::test]
async fn init_continues_pending_login_and_then_verifies_product_api() {
    let home = tempfile::tempdir().unwrap();
    let auth = MockServer::start().await;
    let api = MockServer::start().await;
    device(&auth).await;
    Mock::given(method("POST")).and(path("/api/auth/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"access_token":"init-access-fixture","refresh_token":"init-refresh-fixture","expires_in":600,"token_type":"Bearer"})))
        .expect(1).mount(&auth).await;
    for route in ["/sessions", "/projects"] {
        Mock::given(method("GET"))
            .and(path(route))
            .and(header("Authorization", "Bearer init-access-fixture"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .mount(&api)
            .await;
    }
    let first = command(home.path())
        .env("STEEL_AUTH_ISSUER", auth.uri())
        .env("STEEL_AUTH_AUDIENCE", "https://resource.example.test")
        .env("STEEL_API_URL", api.uri())
        .args(["init", "--agent", "--from", "opencode", "--no-skills"])
        .output()
        .unwrap();
    assert!(first.status.success(), "{}", text(&first));
    assert!(text(&first).contains("authorization"));
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let second = command(home.path())
        .args(["init", "--agent", "--no-skills"])
        .output()
        .unwrap();
    assert!(second.status.success(), "{}", text(&second));
    assert!(text(&second).contains("Credential accepted by product API"));
    assert!(text(&second).contains("unverified"));
    let config: Value =
        serde_json::from_slice(&std::fs::read(home.path().join("config.json")).unwrap()).unwrap();
    assert_eq!(config["onboarding"]["source"], "opencode");
}

#[tokio::test]
async fn slow_down_backoff_is_durable_and_denial_drops_only_pending_request() {
    let home = tempfile::tempdir().unwrap();
    let auth = MockServer::start().await;
    let api = MockServer::start().await;
    device(&auth).await;
    let start = command(home.path())
        .env("STEEL_AUTH_ISSUER", auth.uri())
        .env("STEEL_AUTH_AUDIENCE", "https://resource.example.test")
        .env("STEEL_API_URL", api.uri())
        .args(["login", "--no-wait", "--no-browser"])
        .output()
        .unwrap();
    assert!(start.status.success());
    let slow = Mock::given(method("POST"))
        .and(path("/api/auth/oauth2/token"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({"error":"slow_down"})))
        .expect(1)
        .mount_as_scoped(&auth)
        .await;
    let file = home.path().join("auth.json");
    let mut state: Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    state["pending"]["next_poll_at"] = json!(0);
    std::fs::write(&file, serde_json::to_vec(&state).unwrap()).unwrap();
    let result = command(home.path())
        .args(["login", "--resume", "--no-wait", "--no-browser"])
        .output()
        .unwrap();
    assert!(result.status.success());
    let mut state: Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    assert_eq!(state["pending"]["interval"], 6);
    assert!(state["pending"]["next_poll_at"].as_u64().unwrap() > 0);
    drop(slow);
    Mock::given(method("POST"))
        .and(path("/api/auth/oauth2/token"))
        .respond_with(ResponseTemplate::new(400).set_body_json(
            json!({"error":"access_denied", "error_description":"do-not-echo-response-body"}),
        ))
        .expect(1)
        .mount(&auth)
        .await;
    state["pending"]["next_poll_at"] = json!(0);
    std::fs::write(&file, serde_json::to_vec(&state).unwrap()).unwrap();
    let denied = command(home.path())
        .args(["login", "--resume", "--no-wait", "--no-browser"])
        .output()
        .unwrap();
    assert!(!denied.status.success());
    assert!(!text(&denied).contains("do-not-echo-response-body"));
    let state: Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    assert!(state["pending"].is_null());
    assert!(state["tokens"].is_null());
}

#[tokio::test]
async fn expired_pending_login_is_not_polled_or_silently_reissued_by_resume() {
    let home = tempfile::tempdir().unwrap();
    let auth = MockServer::start().await;
    let api = MockServer::start().await;
    device(&auth).await;
    let start = command(home.path())
        .env("STEEL_AUTH_ISSUER", auth.uri())
        .env("STEEL_AUTH_AUDIENCE", "https://resource.example.test")
        .env("STEEL_API_URL", api.uri())
        .args(["login", "--no-wait", "--no-browser"])
        .output()
        .unwrap();
    assert!(start.status.success());
    let file = home.path().join("auth.json");
    let mut state: Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    state["pending"]["expires_at"] = json!(0);
    std::fs::write(&file, serde_json::to_vec(&state).unwrap()).unwrap();
    let output = command(home.path())
        .args(["login", "--resume", "--no-browser"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(text(&output).contains("expired"));
    assert_eq!(auth.received_requests().await.unwrap().len(), 1);
}

#[cfg(unix)]
#[tokio::test]
async fn insecure_or_symlinked_state_fails_closed_without_key_fallback() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let home = tempfile::tempdir().unwrap();
    let api = MockServer::start().await;
    write_state(home.path(), &api.uri(), &api.uri(), u64::MAX);
    std::fs::write(
        home.path().join("config.json"),
        r#"{"apiKey":"unrelated-key-fixture"}"#,
    )
    .unwrap();
    let state = home.path().join("auth.json");
    std::fs::set_permissions(&state, std::fs::Permissions::from_mode(0o644)).unwrap();
    let output = command(home.path())
        .env("STEEL_API_URL", api.uri())
        .args(["sessions", "list"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(api.received_requests().await.unwrap().is_empty());
    let target = home.path().join("other.json");
    std::fs::rename(&state, &target).unwrap();
    symlink(&target, &state).unwrap();
    let output = command(home.path())
        .env("STEEL_API_URL", api.uri())
        .args(["sessions", "list"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(api.received_requests().await.unwrap().is_empty());
    let local = command(home.path())
        .args(["logout", "--local-only"])
        .output()
        .unwrap();
    assert!(local.status.success(), "{}", text(&local));
    assert!(target.exists());
    assert!(!state.exists());
}

#[cfg(unix)]
#[tokio::test]
async fn symlinked_auth_directory_fails_closed_before_device_request() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().unwrap();
    let real_config = root.path().join("real-config");
    std::fs::create_dir(&real_config).unwrap();
    let linked_config = root.path().join("linked-config");
    symlink(&real_config, &linked_config).unwrap();

    let output = command(&linked_config)
        .env("STEEL_AUTH_ISSUER", "http://127.0.0.1:1")
        .env("STEEL_AUTH_AUDIENCE", "http://127.0.0.1:2")
        .env("STEEL_API_URL", "http://127.0.0.1:3")
        .args(["login", "--no-wait", "--no-browser"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(text(&output).contains("symlinked auth directory"));
    assert!(std::fs::read_dir(real_config).unwrap().next().is_none());
}

#[tokio::test]
async fn killing_a_login_wait_keeps_the_same_request_resumable() {
    use std::io::BufRead;
    let home = tempfile::tempdir().unwrap();
    let auth = MockServer::start().await;
    let api = MockServer::start().await;
    device(&auth).await;
    Mock::given(method("POST")).and(path("/api/auth/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"access_token":"resume-access-fixture","refresh_token":"resume-refresh-fixture","expires_in":600,"token_type":"Bearer"})))
        .expect(1).mount(&auth).await;
    let mut child = command(home.path())
        .env("STEEL_AUTH_ISSUER", auth.uri())
        .env("STEEL_AUTH_AUDIENCE", "https://resource.example.test")
        .env("STEEL_API_URL", api.uri())
        .args(["login", "--no-browser"])
        .stderr(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let mut handoff = String::new();
    std::io::BufReader::new(child.stderr.take().unwrap())
        .read_line(&mut handoff)
        .unwrap();
    assert!(handoff.contains("Authorize Steel"));
    child.kill().unwrap();
    child.wait().unwrap();
    let resumed = command(home.path())
        .args(["login", "--resume", "--no-browser"])
        .output()
        .unwrap();
    assert!(resumed.status.success(), "{}", text(&resumed));
    assert!(text(&resumed).contains("authenticated"));
}

#[test]
fn product_audience_tokens_cannot_be_sent_to_an_override_gateway() {
    use steel_cli::config::device::validate_websocket_transport;
    assert!(
        validate_websocket_transport(
            "https://edge.example.test/v1",
            "wss://edge.example.test/v1/sessions/s1/logs"
        )
        .is_ok()
    );
    assert!(
        validate_websocket_transport(
            "https://edge.example.test/v1",
            "wss://another.example.test/v1/sessions/s1/logs"
        )
        .is_err()
    );
    assert!(
        validate_websocket_transport(
            "https://edge.example.test/v1",
            "ws://edge.example.test/v1/sessions/s1/logs"
        )
        .is_err()
    );
}
