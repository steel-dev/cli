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
    let env: Vec<(&str, &str)> = api_url
        .map(|url| ("STEEL_API_URL", url))
        .into_iter()
        .collect();
    steel_with(home, &env, args).await
}

async fn steel_with(home: &Path, env: &[(&str, &str)], args: &[&str]) -> Output {
    let mut cmd = tokio::process::Command::new(env!("CARGO_BIN_EXE_steel"));
    cmd.current_dir(home)
        .env("HOME", home)
        .env("STEEL_CONFIG_DIR", home)
        .env("STEEL_TELEMETRY_DISABLED", "1")
        .env_remove("STEEL_API_KEY")
        .env_remove("STEEL_API_URL")
        .env_remove("STEEL_AUTH_ISSUER")
        .env_remove("STEEL_AUTH_AUDIENCE")
        .envs(env.iter().copied());
    cmd.args(["--no-update-check", "--json"])
        .args(args)
        .output()
        .await
        .unwrap()
}

fn write_config(home: &Path, config: &serde_json::Value) {
    std::fs::write(home.join("config.json"), config.to_string()).unwrap();
}

/// A runtime that accepts session creation and release with any credentials.
async fn runtime() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/sessions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id":ID,"status":"live"})))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("/v1/sessions/{ID}/release")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success": true})))
        .mount(&server)
        .await;
    server
}

fn credentials(request: &wiremock::Request) -> (Option<String>, Option<String>) {
    let get = |name: &str| {
        request
            .headers
            .get(name)
            .map(|v| v.to_str().unwrap().to_string())
    };
    (get("authorization"), get("steel-api-key"))
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

#[tokio::test]
async fn explicit_local_runtime_does_not_receive_a_cloud_device_login() {
    let cloud = MockServer::start().await;
    let local = runtime().await;
    let home = tempfile::tempdir().unwrap();
    write_device_state(home.path(), &format!("{}/v1", cloud.uri()));
    write_config(
        home.path(),
        &json!({"browser": {"apiUrl": format!("{}/v1", local.uri())}}),
    );

    let start = steel(
        home.path(),
        None,
        &["--local", "browser", "start", "--session", "l"],
    )
    .await;
    assert!(start.status.success(), "{}", shown(&start));
    // The saved session keeps targeting the local runtime without --local.
    let stop = steel(home.path(), None, &["browser", "stop", "--session", "l"]).await;
    assert!(stop.status.success(), "{}", shown(&stop));

    let sent = local.received_requests().await.unwrap();
    assert_eq!(sent.len(), 2);
    assert!(sent.iter().all(|r| credentials(r) == (None, None)));
    assert!(cloud.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn explicit_local_runtime_gets_the_saved_api_key_as_before_device_login() {
    let cloud = MockServer::start().await;
    let local = runtime().await;
    let home = tempfile::tempdir().unwrap();
    write_device_state(home.path(), &format!("{}/v1", cloud.uri()));
    write_config(
        home.path(),
        &json!({"apiKey": "sk-config", "browser": {"apiUrl": format!("{}/v1", local.uri())}}),
    );

    let start = steel(home.path(), None, &["--local", "browser", "start"]).await;

    assert!(start.status.success(), "{}", shown(&start));
    let sent = local.received_requests().await.unwrap();
    assert_eq!(credentials(&sent[0]), (None, Some("sk-config".into())));
    assert!(cloud.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn a_device_login_issued_for_the_local_runtime_keeps_its_bearer() {
    let local = runtime().await;
    let home = tempfile::tempdir().unwrap();
    let local_url = format!("{}/v1", local.uri());
    write_device_state(home.path(), &local_url);
    write_config(home.path(), &json!({"browser": {"apiUrl": local_url}}));

    let start = steel(home.path(), None, &["--local", "browser", "start"]).await;

    assert!(start.status.success(), "{}", shown(&start));
    let sent = local.received_requests().await.unwrap();
    assert_eq!(credentials(&sent[0]), (Some(BEARER.into()), None));
}

#[tokio::test]
async fn an_explicit_api_url_override_still_refuses_a_mismatched_device_login() {
    let issued = MockServer::start().await;
    let other = runtime().await;
    let home = tempfile::tempdir().unwrap();
    write_device_state(home.path(), &format!("{}/v1", issued.uri()));
    let other_url = format!("{}/v1", other.uri());
    // A URL other than the configured local runtime is an arbitrary override, even
    // with --local (which --api-url overrides).
    write_config(
        home.path(),
        &json!({"browser": {"apiUrl": "http://127.0.0.1:9/v1"}}),
    );

    for args in [
        vec!["--api-url", other_url.as_str(), "browser", "start"],
        vec![
            "--local",
            "--api-url",
            other_url.as_str(),
            "browser",
            "start",
        ],
    ] {
        let output = steel(home.path(), None, &args).await;
        assert!(!output.status.success(), "{}", shown(&output));
    }
    let output = steel(home.path(), Some(&other_url), &["browser", "start"]).await;
    assert!(!output.status.success(), "{}", shown(&output));

    assert!(issued.received_requests().await.unwrap().is_empty());
    assert!(other.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn credential_precedence_is_env_key_then_device_login_then_config_key() {
    let api = runtime().await;
    let home = tempfile::tempdir().unwrap();
    let api_url = format!("{}/v1", api.uri());
    write_device_state(home.path(), &api_url);
    write_config(home.path(), &json!({"apiKey": "sk-config"}));

    let device = steel(home.path(), None, &["browser", "start", "--session", "d"]).await;
    assert!(device.status.success(), "{}", shown(&device));
    let env = steel_with(
        home.path(),
        &[("STEEL_API_KEY", "sk-env"), ("STEEL_API_URL", &api_url)],
        &["browser", "start", "--session", "e"],
    )
    .await;
    assert!(env.status.success(), "{}", shown(&env));

    let sent = api.received_requests().await.unwrap();
    assert_eq!(credentials(&sent[0]), (Some(BEARER.into()), None));
    assert_eq!(credentials(&sent[1]), (None, Some("sk-env".into())));
}

#[tokio::test]
async fn login_with_a_saved_api_key_does_not_start_device_authorization() {
    let issuer = MockServer::start().await;
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), &json!({"apiKey": "sk-config"}));

    let login = steel_with(
        home.path(),
        &[("STEEL_AUTH_ISSUER", &issuer.uri())],
        &["login", "--no-wait", "--no-browser"],
    )
    .await;

    assert!(login.status.success(), "{}", shown(&login));
    let out: serde_json::Value = serde_json::from_slice(&login.stdout).unwrap();
    assert_eq!(
        out["data"],
        json!({"status": "authenticated", "method": "api_key"})
    );
    assert!(issuer.received_requests().await.unwrap().is_empty());
    assert!(!home.path().join("auth.json").exists());
}

#[tokio::test]
async fn displayed_connect_urls_show_only_allowlisted_values_in_json_and_text() {
    let secrets = [
        "u-secret",
        "p-secret",
        "tok-secret",
        "sig-secret",
        "amz-secret",
        "auth-secret",
        "frag-secret",
    ];
    let given = "wss://u-secret:p-secret@connect.steel.dev/?sessionId=s1&token=tok-secret&sig=sig-secret&X-Amz-Signature=amz-secret&authorization=auth-secret#frag-secret";
    let api = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/sessions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"id":ID,"status":"live","websocketUrl":given})),
        )
        .mount(&api)
        .await;
    let home = tempfile::tempdir().unwrap();
    write_device_state(home.path(), &format!("{}/v1", api.uri()));

    let json_out = steel(home.path(), None, &["browser", "start", "--session", "j"]).await;
    assert!(json_out.status.success(), "{}", shown(&json_out));
    let out: serde_json::Value = serde_json::from_slice(&json_out.stdout).unwrap();
    let shown_url = out["data"]["connectUrl"].as_str().unwrap();
    assert!(shown_url.contains("sessionId=s1"), "{shown_url}");

    let text_out = tokio::process::Command::new(env!("CARGO_BIN_EXE_steel"))
        .current_dir(home.path())
        .env("HOME", home.path())
        .env("STEEL_CONFIG_DIR", home.path())
        .env("STEEL_TELEMETRY_DISABLED", "1")
        .env_remove("STEEL_API_KEY")
        .env_remove("STEEL_API_URL")
        .args(["--no-update-check", "browser", "start", "--session", "t"])
        .output()
        .await
        .unwrap();
    assert!(text_out.status.success(), "{}", shown(&text_out));
    assert!(
        shown(&text_out).contains("sessionId=s1"),
        "{}",
        shown(&text_out)
    );

    for output in [&json_out, &text_out] {
        for secret in secrets {
            assert!(
                !shown(output).contains(secret),
                "{secret}: {}",
                shown(output)
            );
        }
    }
}

/// Device state the CLI must refuse to read: it may not fall back to a saved key.
fn write_unsafe_device_state(home: &Path, kind: &str, api_base_url: &str) {
    use std::os::unix::fs::PermissionsExt;
    let file = home.join("auth.json");
    match kind {
        "insecure" => {
            write_device_state(home, api_base_url);
            std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();
        }
        "malformed" => {
            std::fs::write(&file, b"{not json").unwrap();
            std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).unwrap();
        }
        "symlink" => {
            let real = tempfile::tempdir().unwrap().keep();
            write_device_state(&real, api_base_url);
            std::os::unix::fs::symlink(real.join("auth.json"), &file).unwrap();
        }
        other => panic!("unknown state {other}"),
    }
}

#[tokio::test]
async fn unsafe_device_state_never_falls_back_to_a_saved_key_for_the_local_runtime() {
    for kind in ["insecure", "malformed", "symlink"] {
        let cloud = MockServer::start().await;
        let local = runtime().await;
        let local_url = format!("{}/v1", local.uri());
        let home = tempfile::tempdir().unwrap();
        write_unsafe_device_state(home.path(), kind, &format!("{}/v1", cloud.uri()));
        write_config(
            home.path(),
            &json!({"apiKey": "sk-config", "browser": {"apiUrl": local_url}}),
        );

        let attempts = [
            steel(home.path(), None, &["--local", "browser", "start"]).await,
            steel(
                home.path(),
                None,
                &["--api-url", &local_url, "browser", "start"],
            )
            .await,
            steel(home.path(), Some(&local_url), &["browser", "start"]).await,
        ];

        for output in &attempts {
            assert!(!output.status.success(), "{kind}: {}", shown(output));
            for secret in [
                "sk-config",
                "synthetic-access-token",
                "synthetic-refresh-token",
            ] {
                assert!(!shown(output).contains(secret), "{kind}: {}", shown(output));
            }
        }
        assert!(
            local.received_requests().await.unwrap().is_empty(),
            "{kind}"
        );
        assert!(
            cloud.received_requests().await.unwrap().is_empty(),
            "{kind}"
        );
    }
}

#[tokio::test]
async fn an_exact_url_override_for_the_local_runtime_follows_the_local_policy() {
    let cloud = MockServer::start().await;
    let local = runtime().await;
    let local_url = format!("{}/v1", local.uri());
    let home = tempfile::tempdir().unwrap();
    write_device_state(home.path(), &format!("{}/v1", cloud.uri()));
    write_config(
        home.path(),
        &json!({"apiKey": "sk-config", "browser": {"apiUrl": local_url}}),
    );

    let flag = steel(
        home.path(),
        None,
        &[
            "--api-url",
            &local_url,
            "browser",
            "start",
            "--session",
            "a",
        ],
    )
    .await;
    assert!(flag.status.success(), "{}", shown(&flag));
    let env = steel(
        home.path(),
        Some(&local_url),
        &["browser", "start", "--session", "b"],
    )
    .await;
    assert!(env.status.success(), "{}", shown(&env));

    let sent = local.received_requests().await.unwrap();
    assert_eq!(sent.len(), 2);
    assert!(
        sent.iter()
            .all(|r| credentials(r) == (None, Some("sk-config".into())))
    );
    assert!(cloud.received_requests().await.unwrap().is_empty());
}
