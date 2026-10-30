use std::path::{Path, PathBuf};
use std::process::Output;

use base64::Engine as _;
use serde_json::{Value, json};
use tempfile::TempDir;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const ID: &str = "4f7c2b1e-9a3d-4e5f-8b6a-1c2d3e4f5a6b";

struct Cli {
    config: TempDir,
    api: String,
}

impl Cli {
    fn new(server: &MockServer) -> Self {
        Self {
            config: tempfile::tempdir().unwrap(),
            api: format!("{}/v1", server.uri()),
        }
    }

    async fn run_in(&self, dir: &Path, args: &[&str]) -> Output {
        tokio::process::Command::new(env!("CARGO_BIN_EXE_steel"))
            .current_dir(dir)
            .env("STEEL_CONFIG_DIR", self.config.path())
            .env("STEEL_API_URL", &self.api)
            .env("STEEL_API_KEY", "sk-test")
            .env("STEEL_TELEMETRY_DISABLED", "1")
            .env_remove("STEEL_FORCE_TTY")
            .arg("--no-update-check")
            .args(args)
            .output()
            .await
            .unwrap()
    }

    async fn run(&self, args: &[&str]) -> Output {
        self.run_in(self.config.path(), args).await
    }

    fn state_path(&self, name: &str) -> PathBuf {
        self.config
            .path()
            .join("browser-sessions")
            .join(format!("{name}.json"))
    }

    fn state(&self, name: &str) -> Option<Value> {
        let text = std::fs::read_to_string(self.state_path(name)).ok()?;
        Some(serde_json::from_str(&text).unwrap())
    }

    fn write_state(&self, name: &str) {
        let dir = self.config.path().join("browser-sessions");
        std::fs::create_dir_all(&dir).unwrap();
        let state = json!({"name": name, "id": ID, "mode": "cloud", "apiUrl": self.api});
        std::fs::write(dir.join(format!("{name}.json")), state.to_string()).unwrap();
    }
}

fn json_out(output: &Output) -> Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(stdout.trim()).unwrap_or_else(|_| {
        panic!(
            "expected JSON, got stdout={stdout} stderr={}",
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn session_json() -> Value {
    json!({
        "id": ID,
        "status": "live",
        "createdAt": "2026-10-30T00:00:00.000Z",
        "timeout": 3_600_000,
        "inactivityTimeout": 600_000,
        "websocketUrl": format!("wss://connect.steel.dev?sessionId={ID}"),
        "sessionViewerUrl": format!("https://app.steel.dev/sessions/{ID}")
    })
}

fn completed(next: &str, results: Value) -> Value {
    json!({
        "next": next,
        "epoch": 1,
        "page": {"tab": "t1", "url": "https://example.com/", "title": "Example"},
        "execution": {"id": "x-1", "state": "completed", "effect": "applied", "replayed": false, "ran": 1},
        "results": results,
        "events": {"cursor": "c-4", "items": []}
    })
}

fn commands_path() -> String {
    format!("/v1/sessions/{ID}/commands")
}

async fn command_bodies(server: &MockServer) -> Vec<Value> {
    server
        .received_requests()
        .await
        .unwrap()
        .into_iter()
        .filter(|r| r.url.path() == commands_path())
        .map(|r| serde_json::from_slice(&r.body).unwrap())
        .collect()
}

#[tokio::test]
async fn start_act_and_stop_without_a_daemon() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/sessions"))
        .and(body_partial_json(json!({"inactivityTimeout": 600_000})))
        .respond_with(ResponseTemplate::new(200).set_body_json(session_json()))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(commands_path()))
        .and(body_partial_json(
            json!({"commands": [{"command": "click"}]}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(completed(
            "op_1",
            json!([{"ok": true, "effect": "applied"}]),
        )))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(commands_path()))
        .and(body_partial_json(json!({"commands": [{"command": "snapshot"}]})))
        .respond_with(ResponseTemplate::new(200).set_body_json(completed(
            "op_2",
            json!([{"ok": true, "effect": "none", "snapshotId": "s-1", "snapshot": "- button \"Save\" [ref=e5]"}]),
        )))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("/v1/sessions/{ID}/release")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success": true})))
        .expect(1)
        .mount(&server)
        .await;

    let cli = Cli::new(&server);

    let started = cli.run(&["browser", "start", "--session", "work"]).await;
    assert!(started.status.success(), "{started:?}");
    let out = json_out(&started);
    assert_eq!(out["data"]["id"], ID);
    assert_eq!(out["data"]["name"], "work");
    assert_eq!(cli.state("work").unwrap()["id"], ID);

    let clicked = cli
        .run(&["browser", "click", "e5", "--session", "work"])
        .await;
    assert!(clicked.status.success(), "{clicked:?}");
    let state = cli.state("work").unwrap();
    assert_eq!(state["next"], "op_1");
    assert_eq!(state["tab"], "t1");
    assert_eq!(state["events"], "c-4");

    let snap = cli
        .run(&["browser", "snapshot", "-i", "--session", "work"])
        .await;
    assert!(snap.status.success(), "{snap:?}");
    let out = json_out(&snap);
    assert_eq!(
        out["data"]["results"][0]["snapshot"],
        "- button \"Save\" [ref=e5]"
    );
    let state = cli.state("work").unwrap();
    assert_eq!(state["snapshot"]["id"], "s-1");
    assert_eq!(state["snapshot"]["interactive"], true);

    let bodies = command_bodies(&server).await;
    assert_eq!(
        bodies[0],
        json!({"commands": [{"command": "click", "target": "e5"}], "eventsAfter": "c-0"})
    );
    assert_eq!(
        bodies[1],
        json!({
            "after": "op_1",
            "commands": [{"command": "snapshot", "interactive": true, "tab": "t1"}],
            "eventsAfter": "c-4"
        })
    );
    let keys: Vec<_> = server
        .received_requests()
        .await
        .unwrap()
        .into_iter()
        .filter(|r| r.url.path() == commands_path())
        .map(|r| r.headers.get("idempotency-key").cloned())
        .collect();
    assert!(keys.iter().all(Option::is_some));
    assert_ne!(keys[0], keys[1]);

    let stopped = cli.run(&["browser", "stop", "--session", "work"]).await;
    assert!(stopped.status.success(), "{stopped:?}");
    assert!(cli.state("work").is_none());
}

#[tokio::test]
async fn diff_observes_against_the_last_snapshot() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(commands_path()))
        .respond_with(|_: &Request| {
            let mut body = completed("op_3", json!([{"ok": true, "effect": "applied"}]));
            body["observe"] = json!({"snapshotId": "s-8", "base": "s-7", "diff": "-  - button \"Save\"\n+  - button \"Saved\"", "full": false});
            ResponseTemplate::new(200).set_body_json(body)
        })
        .mount(&server)
        .await;
    let cli = Cli::new(&server);
    let dir = cli.config.path().join("browser-sessions");
    std::fs::create_dir_all(&dir).unwrap();
    let state = json!({
        "name": "s", "id": ID, "mode": "cloud", "apiUrl": cli.api,
        "snapshot": {"id": "s-7", "interactive": true, "compact": false}
    });
    std::fs::write(dir.join("s.json"), state.to_string()).unwrap();

    let out = cli
        .run(&["browser", "click", "e5", "--diff", "--session", "s"])
        .await;
    assert!(out.status.success(), "{out:?}");
    let body = &command_bodies(&server).await[0];
    assert_eq!(
        body["observe"],
        json!({"mode": "diff", "base": "s-7", "interactive": true})
    );
    let state = cli.state("s").unwrap();
    assert_eq!(state["snapshot"]["id"], "s-8");
    assert_eq!(state["snapshot"]["interactive"], true);
}

#[tokio::test]
async fn an_ended_session_clears_its_state() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(commands_path()))
        .respond_with(ResponseTemplate::new(410).set_body_json(json!({
            "error": {"code": "session_ended", "message": "Session has ended."}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let cli = Cli::new(&server);
    cli.write_state("gone");

    let out = cli
        .run(&["browser", "click", "e1", "--session", "gone"])
        .await;
    assert!(!out.status.success());
    let out = json_out(&out);
    assert_eq!(out["success"], false);
    assert_eq!(out["error_code"], "session_ended");
    assert!(
        out["error"]
            .as_str()
            .unwrap()
            .contains("steel browser start --session gone")
    );
    assert!(cli.state("gone").is_none());
}

#[tokio::test]
async fn a_failed_command_exits_non_zero_with_the_engine_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(commands_path()))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "next": "op_5",
            "epoch": 1,
            "page": {"tab": "t1", "url": "https://example.com/", "title": "Example"},
            "execution": {"id": "x-2", "state": "failed", "effect": "none", "replayed": false, "ran": 1},
            "results": [{"ok": false, "effect": "none", "error": {"code": "stale_ref", "message": "Ref e9 is not known. Take a new snapshot.", "reason": "unknown"}}]
        })))
        .mount(&server)
        .await;
    let cli = Cli::new(&server);
    cli.write_state("s");

    let out = cli.run(&["browser", "click", "e9", "--session", "s"]).await;
    assert_eq!(out.status.code(), Some(1));
    let out = json_out(&out);
    assert_eq!(out["success"], false);
    assert_eq!(out["error_code"], "stale_ref");
    assert_eq!(out["data"]["execution"]["id"], "x-2");
    assert_eq!(cli.state("s").unwrap()["next"], "op_5");
}

#[tokio::test]
async fn a_session_id_attaches_without_a_local_start() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/v1/sessions/{ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(session_json()))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(commands_path()))
        .respond_with(ResponseTemplate::new(200).set_body_json(completed(
            "op_1",
            json!([{"ok": true, "effect": "none", "value": "Example"}]),
        )))
        .mount(&server)
        .await;
    let cli = Cli::new(&server);

    let out = cli.run(&["browser", "get", "title", "--session", ID]).await;
    assert!(out.status.success(), "{out:?}");
    assert_eq!(json_out(&out)["data"]["results"][0]["value"], "Example");
    assert_eq!(cli.state(ID).unwrap()["id"], ID);
}

#[tokio::test]
async fn screenshots_are_written_to_the_output_path() {
    let server = MockServer::start().await;
    let data = base64::engine::general_purpose::STANDARD.encode(b"PNGDATA");
    Mock::given(method("POST"))
        .and(path(commands_path()))
        .and(body_partial_json(
            json!({"commands": [{"command": "screenshot", "format": "jpeg"}]}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(completed(
            "op_1",
            json!([{"ok": true, "effect": "none", "data": data, "format": "jpeg"}]),
        )))
        .expect(1)
        .mount(&server)
        .await;
    let cli = Cli::new(&server);
    cli.write_state("s");
    let work = tempfile::tempdir().unwrap();

    let out = cli
        .run_in(
            work.path(),
            &["browser", "screenshot", "-o", "shot.jpg", "--session", "s"],
        )
        .await;
    assert!(out.status.success(), "{out:?}");
    let saved = work.path().join("shot.jpg");
    assert_eq!(std::fs::read(&saved).unwrap(), b"PNGDATA");
    let result = &json_out(&out)["data"]["results"][0];
    assert!(result.get("data").is_none());
    assert!(result["path"].as_str().unwrap().ends_with("shot.jpg"));
}

#[tokio::test]
async fn the_default_session_starts_on_first_use() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/sessions"))
        .and(body_partial_json(json!({"inactivityTimeout": 600_000})))
        .respond_with(ResponseTemplate::new(200).set_body_json(session_json()))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(commands_path()))
        .respond_with(ResponseTemplate::new(200).set_body_json(completed(
            "op_1",
            json!([{"ok": true, "effect": "applied"}]),
        )))
        .expect(1)
        .mount(&server)
        .await;
    let cli = Cli::new(&server);

    let out = cli
        .run(&["browser", "navigate", "https://example.com"])
        .await;
    assert!(out.status.success(), "{out:?}");
    assert!(String::from_utf8_lossy(&out.stderr).contains("starting a new one"));
    assert_eq!(cli.state("default").unwrap()["next"], "op_1");
}

#[tokio::test]
async fn removed_commands_fail_without_calling_the_api() {
    let server = MockServer::start().await;
    let cli = Cli::new(&server);

    let out = cli.run(&["browser", "cookies"]).await;
    assert!(!out.status.success());
    assert!(
        json_out(&out)["error"]
            .as_str()
            .unwrap()
            .contains("removed in 0.5")
    );
    assert!(server.received_requests().await.unwrap().is_empty());
    assert!(cli.state("default").is_none());
}

#[tokio::test]
async fn batch_sends_one_request() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(commands_path()))
        .respond_with(ResponseTemplate::new(200).set_body_json(completed(
            "op_1",
            json!([
                {"ok": true, "effect": "applied"},
                {"ok": true, "effect": "applied"},
                {"ok": true, "effect": "none", "snapshotId": "s-2", "snapshot": "- textbox \"City\" [ref=e3]: Seoul"}
            ]),
        )))
        .expect(1)
        .mount(&server)
        .await;
    let cli = Cli::new(&server);
    cli.write_state("s");

    let out = cli
        .run(&[
            "browser",
            "batch",
            "fill e3 Seoul",
            "click e5",
            "snapshot -c",
            "--session",
            "s",
        ])
        .await;
    assert!(out.status.success(), "{out:?}");
    assert_eq!(
        command_bodies(&server).await[0]["commands"],
        json!([
            {"command": "fill", "target": "e3", "value": "Seoul"},
            {"command": "click", "target": "e5"},
            {"command": "snapshot", "compact": true}
        ])
    );
    assert_eq!(cli.state("s").unwrap()["snapshot"]["compact"], true);
}
