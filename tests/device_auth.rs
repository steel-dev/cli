//! Offline protocol/credential tests; subprocesses never use normal user state.
use serde_json::{Value, json};
use std::process::{Command, Output};
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

fn state(home: &std::path::Path, issuer: &str, transport: &str, expires: u64) {
    let file = home.join("auth.json");
    std::fs::write(&file,serde_json::to_vec(&json!({"settings":{"issuer":issuer,"audience":"https://resource.example.test","api_base_url":transport},"installation_id":"synthetic-installation","pending":null,"tokens":{"access_token":"old-access-fixture","refresh_token":"old-refresh-fixture","expires_at":expires}})).unwrap()).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(file, std::fs::Permissions::from_mode(0o600)).unwrap();
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
    Mock::given(method("POST")).and(path("/api/auth/device/code"))
        .and(body_string_contains("client_id=steel-cli"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"device_code":"private-device-fixture","user_code":"ABCD2345","verification_uri":format!("{}/device",server.uri()),"verification_uri_complete":format!("{}/device?user_code=ABCD2345",server.uri()),"expires_in":600,"interval":1})))
        .expect(1).mount(server).await;
}

#[tokio::test]
async fn production_defaults_are_saved_and_independent_overrides_are_not_inferred() {
    let home = tempfile::tempdir().unwrap();
    let auth = MockServer::start().await;
    device(&auth).await;
    let result = command(home.path())
        .env("STEEL_AUTH_ISSUER", auth.uri())
        .args(["login", "--no-wait", "--no-browser"])
        .output()
        .unwrap();
    assert!(result.status.success(), "{}", text(&result));
    let saved: Value =
        serde_json::from_slice(&std::fs::read(home.path().join("auth.json")).unwrap()).unwrap();
    assert_eq!(saved["settings"]["issuer"], auth.uri());
    assert_eq!(saved["settings"]["audience"], "https://api.steel.dev");
    assert_eq!(
        saved["settings"]["api_base_url"],
        "https://api.steel.dev/v1"
    );
    let request = &auth.received_requests().await.unwrap()[0];
    assert!(
        String::from_utf8_lossy(&request.body).contains("resource=https%3A%2F%2Fapi.steel.dev")
    );
}

#[tokio::test]
async fn pending_login_resumes_same_request_and_persists_private_tokens() {
    use std::io::BufRead;
    let home = tempfile::tempdir().unwrap();
    let auth = MockServer::start().await;
    let api = MockServer::start().await;
    device(&auth).await;
    Mock::given(method("POST")).and(path("/api/auth/oauth2/token"))
        .and(body_string_contains("device_code=private-device-fixture"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"access_token":"new-access-fixture","refresh_token":"new-refresh-fixture","expires_in":600,"token_type":"Bearer"})))
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
    let mut line = String::new();
    std::io::BufReader::new(child.stderr.take().unwrap())
        .read_line(&mut line)
        .unwrap();
    assert!(line.contains("Authorize Steel"));
    let _ = child.kill();
    child.wait().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let result = command(home.path())
        .args(["login", "--resume", "--no-browser"])
        .output()
        .unwrap();
    assert!(result.status.success(), "{}", text(&result));
    assert!(text(&result).contains("authenticated"));
    assert!(!text(&result).contains("new-access-fixture"));
    assert!(!text(&result).contains("new-refresh-fixture"));
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
async fn concurrent_commands_rotate_once_and_use_bearer_without_an_api_key() {
    let home = tempfile::tempdir().unwrap();
    let auth = MockServer::start().await;
    let api = MockServer::start().await;
    state(home.path(), &auth.uri(), &api.uri(), 0);
    Mock::given(method("POST")).and(path("/api/auth/oauth2/token")).and(body_string_contains("refresh_token=old-refresh-fixture"))
        .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_millis(150)).set_body_json(json!({"access_token":"rotated-access-fixture","refresh_token":"rotated-refresh-fixture","expires_in":600,"token_type":"Bearer"})))
        .expect(1).mount(&auth).await;
    Mock::given(method("GET"))
        .and(path("/sessions"))
        .and(header("Authorization", "Bearer rotated-access-fixture"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(2)
        .mount(&api)
        .await;
    let start = || {
        command(home.path())
            .args(["sessions", "list"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap()
    };
    let a = start();
    let b = start();
    for result in [a.wait_with_output().unwrap(), b.wait_with_output().unwrap()] {
        assert!(result.status.success(), "{}", text(&result));
        assert!(!text(&result).contains("rotated-access-fixture"));
    }
    for request in api.received_requests().await.unwrap() {
        assert!(!request.headers.contains_key("steel-api-key"));
    }
    let saved: Value =
        serde_json::from_slice(&std::fs::read(home.path().join("auth.json")).unwrap()).unwrap();
    assert_eq!(saved["tokens"]["refresh_token"], "rotated-refresh-fixture");
    assert_eq!(saved["refresh_in_flight"], false);
}

#[tokio::test]
async fn changed_transport_and_uncertain_refresh_never_send_or_replay_a_token() {
    let home = tempfile::tempdir().unwrap();
    let auth = MockServer::start().await;
    let api = MockServer::start().await;
    let other = MockServer::start().await;
    state(home.path(), &auth.uri(), &api.uri(), u64::MAX);
    let result = command(home.path())
        .env("STEEL_API_URL", other.uri())
        .args(["sessions", "list"])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(other.received_requests().await.unwrap().is_empty());
    let file = home.path().join("auth.json");
    let mut data: Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    data["refresh_in_flight"] = json!(true);
    std::fs::write(&file, serde_json::to_vec(&data).unwrap()).unwrap();
    let result = command(home.path())
        .args(["sessions", "list"])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(text(&result).contains("refusing to replay"));
    assert!(auth.received_requests().await.unwrap().is_empty());
    assert!(api.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn logout_revokes_before_deleting_and_missing_refresh_does_not_reuse_old_value() {
    let home = tempfile::tempdir().unwrap();
    let auth = MockServer::start().await;
    let api = MockServer::start().await;
    state(home.path(), &auth.uri(), &api.uri(), 0);
    Mock::given(method("POST"))
        .and(path("/api/auth/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({"access_token":"access-without-refresh","expires_in":600,"token_type":"Bearer"}),
        ))
        .expect(1)
        .mount(&auth)
        .await;
    Mock::given(method("GET"))
        .and(path("/sessions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&api)
        .await;
    let result = command(home.path())
        .args(["sessions", "list"])
        .output()
        .unwrap();
    assert!(result.status.success());
    let saved: Value =
        serde_json::from_slice(&std::fs::read(home.path().join("auth.json")).unwrap()).unwrap();
    assert!(saved["tokens"]["refresh_token"].is_null());
    state(home.path(), &auth.uri(), &api.uri(), u64::MAX);
    Mock::given(method("POST"))
        .and(path("/api/auth/oauth2/revoke"))
        .and(body_string_contains("token=old-refresh-fixture"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&auth)
        .await;
    let result = command(home.path()).arg("logout").output().unwrap();
    assert!(result.status.success());
    assert!(!home.path().join("auth.json").exists());
}

#[cfg(unix)]
#[tokio::test]
async fn unsafe_or_symlinked_state_never_falls_back_to_saved_key() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let home = tempfile::tempdir().unwrap();
    let api = MockServer::start().await;
    state(home.path(), &api.uri(), &api.uri(), u64::MAX);
    std::fs::write(
        home.path().join("config.json"),
        r#"{"apiKey":"unrelated-key-fixture"}"#,
    )
    .unwrap();
    let file = home.path().join("auth.json");
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();
    let result = command(home.path())
        .env("STEEL_API_URL", api.uri())
        .args(["sessions", "list"])
        .output()
        .unwrap();
    assert!(!result.status.success());
    let target = home.path().join("other.json");
    std::fs::rename(&file, &target).unwrap();
    symlink(&target, &file).unwrap();
    let result = command(home.path())
        .env("STEEL_API_URL", api.uri())
        .args(["sessions", "list"])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(api.received_requests().await.unwrap().is_empty());
    assert!(
        command(home.path())
            .args(["logout", "--local-only"])
            .output()
            .unwrap()
            .status
            .success()
    );
    assert!(target.exists());
}

#[tokio::test]
async fn pending_backoff_and_expiry_survive_separate_commands() {
    let home = tempfile::tempdir().unwrap();
    let auth = MockServer::start().await;
    device(&auth).await;
    let start = command(home.path())
        .env("STEEL_AUTH_ISSUER", auth.uri())
        .args(["login", "--no-wait", "--no-browser"])
        .output()
        .unwrap();
    assert!(start.status.success());
    Mock::given(method("POST"))
        .and(path("/api/auth/oauth2/token"))
        .respond_with(ResponseTemplate::new(400).set_body_json(
            json!({"error":"slow_down","error_description":"never-echo-body-fixture"}),
        ))
        .expect(1)
        .mount(&auth)
        .await;
    let file = home.path().join("auth.json");
    let mut data: Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    data["pending"]["next_poll_at"] = json!(0);
    std::fs::write(&file, serde_json::to_vec(&data).unwrap()).unwrap();
    let result = command(home.path())
        .args(["login", "--resume", "--no-wait", "--no-browser"])
        .output()
        .unwrap();
    assert!(result.status.success());
    assert!(!text(&result).contains("never-echo-body-fixture"));
    let mut data: Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    assert_eq!(data["pending"]["interval"], 6);
    assert!(data["pending"]["next_poll_at"].as_u64().unwrap() > 0);
    data["pending"]["expires_at"] = json!(0);
    std::fs::write(&file, serde_json::to_vec(&data).unwrap()).unwrap();
    let expired = command(home.path())
        .args(["login", "--resume", "--no-browser"])
        .output()
        .unwrap();
    assert!(!expired.status.success());
    assert!(text(&expired).contains("expired"));
    assert_eq!(
        auth.received_requests().await.unwrap().len(),
        2,
        "Resume must not silently issue a new code"
    );
}

fn refreshless_state(home: &std::path::Path, issuer: &str, transport: &str) {
    state(home, issuer, transport, u64::MAX);
    let file = home.join("auth.json");
    let mut saved: Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    saved["tokens"]["refresh_token"] = Value::Null;
    std::fs::write(&file, serde_json::to_vec(&saved).unwrap()).unwrap();
}

#[tokio::test]
async fn logout_of_a_refreshless_login_refuses_until_local_only() {
    let home = tempfile::tempdir().unwrap();
    let auth = MockServer::start().await;
    refreshless_state(home.path(), &auth.uri(), &auth.uri());

    let result = command(home.path()).arg("logout").output().unwrap();

    assert!(!result.status.success(), "{}", text(&result));
    assert!(text(&result).contains("--local-only"), "{}", text(&result));
    assert!(!text(&result).contains("old-access-fixture"));
    assert!(home.path().join("auth.json").exists());
    assert!(auth.received_requests().await.unwrap().is_empty());

    let local = command(home.path())
        .args(["logout", "--local-only"])
        .output()
        .unwrap();
    assert!(local.status.success(), "{}", text(&local));
    assert!(!home.path().join("auth.json").exists());
}

#[tokio::test]
async fn failed_revocation_keeps_local_state() {
    let home = tempfile::tempdir().unwrap();
    let auth = MockServer::start().await;
    state(home.path(), &auth.uri(), &auth.uri(), u64::MAX);
    Mock::given(method("POST"))
        .and(path("/api/auth/oauth2/revoke"))
        .respond_with(ResponseTemplate::new(503))
        .expect(1)
        .mount(&auth)
        .await;

    let result = command(home.path()).arg("logout").output().unwrap();

    assert!(!result.status.success(), "{}", text(&result));
    assert!(
        text(&result).contains("local state retained"),
        "{}",
        text(&result)
    );
    assert!(home.path().join("auth.json").exists());
}

/// `origin` answers everything with a redirect to `target`, which must never be contacted.
async fn redirecting_issuer(status: u16) -> (MockServer, MockServer) {
    let origin = MockServer::start().await;
    let target = MockServer::start().await;
    Mock::given(wiremock::matchers::any())
        .respond_with(ResponseTemplate::new(status).insert_header(
            "location",
            format!("{}/api/auth/captured", target.uri()).as_str(),
        ))
        .mount(&origin)
        .await;
    (origin, target)
}

#[tokio::test]
async fn device_code_token_and_revoke_endpoints_never_follow_redirects() {
    for status in [302, 307, 308] {
        // Device authorization request.
        let home = tempfile::tempdir().unwrap();
        let (origin, target) = redirecting_issuer(status).await;
        let login = command(home.path())
            .env("STEEL_AUTH_ISSUER", origin.uri())
            .args(["login", "--no-wait", "--no-browser"])
            .output()
            .unwrap();
        assert!(!login.status.success(), "{}", text(&login));
        assert_eq!(origin.received_requests().await.unwrap().len(), 1);
        assert!(target.received_requests().await.unwrap().is_empty());

        // Token refresh: the refresh token must not reach the target.
        let home = tempfile::tempdir().unwrap();
        let (origin, target) = redirecting_issuer(status).await;
        let api = MockServer::start().await;
        state(home.path(), &origin.uri(), &api.uri(), 0);
        let refresh = command(home.path())
            .args(["sessions", "list"])
            .output()
            .unwrap();
        assert!(!refresh.status.success(), "{}", text(&refresh));
        assert!(target.received_requests().await.unwrap().is_empty());
        assert!(api.received_requests().await.unwrap().is_empty());

        // Revocation: state survives a redirected revoke.
        let home = tempfile::tempdir().unwrap();
        let (origin, target) = redirecting_issuer(status).await;
        state(home.path(), &origin.uri(), &origin.uri(), u64::MAX);
        let logout = command(home.path()).arg("logout").output().unwrap();
        assert!(!logout.status.success(), "{}", text(&logout));
        assert_eq!(origin.received_requests().await.unwrap().len(), 1);
        assert!(target.received_requests().await.unwrap().is_empty());
        assert!(home.path().join("auth.json").exists());
    }
}
