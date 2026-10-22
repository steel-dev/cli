//! Token-only device credential -> product API -> native CDP -> explicit release.
//! Exercises real binary/wire format without any live service or product secret.
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use std::{
    process::Command,
    sync::{Arc, Mutex},
};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::{
    Message,
    handshake::server::{Request, Response},
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{header, method, path},
};

struct Fixture {
    home: tempfile::TempDir,
    api: MockServer,
    cdp: tokio::task::JoinHandle<()>,
}
impl Fixture {
    fn command(&self) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_steel"));
        cmd.env("HOME", self.home.path())
            .env("STEEL_CONFIG_DIR", self.home.path())
            .env("STEEL_TELEMETRY_DISABLED", "1")
            .env_remove("STEEL_API_KEY")
            .env_remove("STEEL_API_URL")
            .env_remove("STEEL_AUTH_ISSUER")
            .env_remove("STEEL_AUTH_AUDIENCE")
            .args(["--no-update-check", "--json"]);
        cmd
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = self
            .command()
            .args(["browser", "stop", "--session", "test"])
            .output();
        self.cdp.abort();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn device_auth_uses_bearer_and_normalizes_root_query_before_native_cdp() {
    use std::os::unix::fs::PermissionsExt;
    let api = MockServer::start().await;
    let home = tempfile::tempdir().unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!(
        "ws://{}?sessionId=test-session&token=synthetic-connect-token",
        listener.local_addr().unwrap()
    );
    let seen = Arc::new(Mutex::new(Vec::new()));
    let observations = Arc::clone(&seen);
    let cdp = tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let observations = Arc::clone(&observations);
            tokio::spawn(async move {
                let Ok(mut socket) = tokio_tungstenite::accept_hdr_async(
                    stream,
                    move |request: &Request, response: Response| {
                        observations
                            .lock()
                            .unwrap()
                            .push(request.uri().path_and_query().unwrap().as_str().to_string());
                        Ok(response)
                    },
                )
                .await
                else {
                    return;
                };
                while let Some(Ok(Message::Text(text))) = socket.next().await {
                    let request: Value = serde_json::from_str(&text).unwrap();
                    let result = match request["method"].as_str().unwrap_or("") {
                        "Target.getTargets" => {
                            json!({"targetInfos":[{"targetId":"page","type":"page","title":"Synthetic page","url":"about:blank","attached":false}]})
                        }
                        "Target.attachToTarget" => json!({"sessionId":"synthetic-cdp-session"}),
                        _ => json!({}),
                    };
                    if socket
                        .send(Message::Text(
                            json!({"id":request["id"],"result":result})
                                .to_string()
                                .into(),
                        ))
                        .await
                        .is_err()
                    {
                        break;
                    }
                    if request["method"] == "Browser.close" {
                        break;
                    }
                }
            });
        }
    });
    Mock::given(method("POST"))
        .and(path("/sessions"))
        .and(header("authorization", "Bearer synthetic-access-token"))
        .respond_with(ResponseTemplate::new(201).set_body_json(
            json!({"id":"test-session","status":"live","timeout":300000,"websocketUrl":endpoint}),
        ))
        .expect(1)
        .mount(&api)
        .await;
    Mock::given(method("POST"))
        .and(path("/sessions/test-session/release"))
        .and(header("authorization", "Bearer synthetic-access-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status":"released"})))
        .expect(1)
        .mount(&api)
        .await;
    let auth = json!({"settings":{"issuer":api.uri(),"audience":"https://resource.example.test","api_base_url":api.uri()},"installation_id":"synthetic-installation","pending":null,"tokens":{"access_token":"synthetic-access-token","refresh_token":"synthetic-refresh-token","expires_at":u64::MAX}});
    let file = home.path().join("auth.json");
    std::fs::write(&file, serde_json::to_vec(&auth).unwrap()).unwrap();
    std::fs::set_permissions(file, std::fs::Permissions::from_mode(0o600)).unwrap();
    let fixture = Fixture { home, api, cdp };
    let start = fixture
        .command()
        .args(["browser", "start", "--session", "test"])
        .output()
        .unwrap();
    assert!(
        start.status.success(),
        "{}",
        String::from_utf8_lossy(&start.stdout)
    );
    assert_eq!(
        *seen.lock().unwrap(),
        ["/?sessionId=test-session&token=synthetic-connect-token"]
    );
    assert!(!String::from_utf8_lossy(&start.stdout).contains("synthetic-connect-token"));
    assert!(!String::from_utf8_lossy(&start.stdout).contains("synthetic-access-token"));
    let stop = fixture
        .command()
        .args(["browser", "stop", "--session", "test"])
        .output()
        .unwrap();
    assert!(stop.status.success());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    loop {
        let requests = fixture.api.received_requests().await.unwrap();
        if requests.iter().any(|r| r.url.path().ends_with("/release")) {
            assert!(
                requests
                    .iter()
                    .all(|r| !r.headers.contains_key("steel-api-key"))
            );
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Explicit stop did not release session"
        );
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
}
