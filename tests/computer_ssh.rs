//! End-to-end tests for `steel computer ssh` against an in-process bridge.
//!
//! A WebSocket server plays the box gateway and an SSH server sits behind it,
//! so the real `steel` binary exercises the handshake headers, the WebSocket
//! transport, the SSH login, exec, shell and the exit status.

use std::process::{Command, Output};
use std::sync::{Arc, Mutex};

use russh::keys::PrivateKey;
use russh::keys::ssh_key::private::{Ed25519Keypair, KeypairData};
use russh::server::{self, Auth, ChannelOpenHandle, Msg, Session};
use russh::{Channel, ChannelId};
use steel_cli::commands::computer::wsio::WsIo;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tokio_tungstenite::tungstenite::http::HeaderValue;

const COMPUTER: &str = "cmp_0000123456789abcdefghjkmnpqrs";
const SUBPROTOCOL: &str = "steel-ssh-v1";

#[derive(Default, Debug, Clone)]
struct Observed {
    path: String,
    subprotocol: String,
    api_key: String,
    user: String,
    password: String,
    exec: Option<String>,
    shell: bool,
}

type Shared = Arc<Mutex<Observed>>;

struct Bridge {
    observed: Shared,
    exit_status: u32,
}

impl server::Handler for Bridge {
    type Error = russh::Error;

    async fn auth_password(&mut self, user: &str, password: &str) -> Result<Auth, Self::Error> {
        {
            let mut observed = self.observed.lock().unwrap();
            observed.user = user.to_string();
            observed.password = password.to_string();
        }
        Ok(Auth::Accept)
    }

    async fn channel_open_session(
        &mut self,
        _channel: Channel<Msg>,
        reply: ChannelOpenHandle,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        reply.accept().await;
        Ok(())
    }

    async fn exec_request(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let command = String::from_utf8_lossy(data).to_string();
        self.observed.lock().unwrap().exec = Some(command.clone());
        session.channel_success(channel)?;
        session.data(channel, format!("ran {command}\n").into_bytes())?;
        finish(session, channel, self.exit_status)
    }

    async fn shell_request(
        &mut self,
        channel: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        self.observed.lock().unwrap().shell = true;
        session.channel_success(channel)?;
        session.data(channel, b"shell ready\n".to_vec())?;
        finish(session, channel, self.exit_status)
    }
}

fn finish(session: &mut Session, channel: ChannelId, exit_status: u32) -> Result<(), russh::Error> {
    session.exit_status_request(channel, exit_status)?;
    session.eof(channel)?;
    session.close(channel)?;
    Ok(())
}

async fn start_bridge(exit_status: u32) -> (u16, Shared) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let observed: Shared = Arc::default();
    let shared = observed.clone();
    tokio::spawn(async move {
        let (tcp, _) = listener.accept().await.unwrap();
        let handshake = shared.clone();
        let callback =
            move |request: &Request, mut response: Response| -> Result<Response, ErrorResponse> {
                let header = |name: &str| {
                    request
                        .headers()
                        .get(name)
                        .and_then(|value| value.to_str().ok())
                        .unwrap_or("")
                        .to_string()
                };
                {
                    let mut observed = handshake.lock().unwrap();
                    observed.path = request.uri().path().to_string();
                    observed.subprotocol = header("Sec-WebSocket-Protocol");
                    observed.api_key = header("Steel-Api-Key");
                }
                response.headers_mut().insert(
                    "Sec-WebSocket-Protocol",
                    HeaderValue::from_static(SUBPROTOCOL),
                );
                Ok(response)
            };
        let ws = tokio_tungstenite::accept_hdr_async(tcp, callback)
            .await
            .unwrap();
        let host_key = KeypairData::Ed25519(Ed25519Keypair::from_seed(&[7u8; 32]));
        let config = Arc::new(server::Config {
            keys: vec![PrivateKey::try_from(host_key).unwrap()],
            ..Default::default()
        });
        let handler = Bridge {
            observed: shared,
            exit_status,
        };
        let session = server::run_stream(config, WsIo::new(ws), handler)
            .await
            .unwrap();
        let _ = session.await;
    });
    (port, observed)
}

async fn run_steel(port: u16, args: &[&str]) -> Output {
    let tmp = tempfile::tempdir().expect("temp dir");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_steel"));
    cmd.env("STEEL_CONFIG_DIR", tmp.path());
    cmd.env("STEEL_API_URL", format!("http://127.0.0.1:{port}/v1"));
    cmd.env("STEEL_API_KEY", "ste-test-key");
    cmd.env("STEEL_TELEMETRY_DISABLED", "1");
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

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}

#[tokio::test(flavor = "multi_thread")]
async fn ssh_runs_a_command_through_the_bridge_and_returns_its_exit_status() {
    let (port, observed) = start_bridge(7).await;

    let output = run_steel(
        port,
        &["computer", "ssh", COMPUTER, "--", "echo", "hi there"],
    )
    .await;

    assert_eq!(
        text(&output.stdout),
        "ran 'echo' 'hi there'\n",
        "stderr: {}",
        text(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(7));
    let observed = observed.lock().unwrap().clone();
    assert_eq!(observed.path, format!("/v1/computers/{COMPUTER}/ssh"));
    assert_eq!(observed.subprotocol, SUBPROTOCOL);
    assert_eq!(observed.api_key, "ste-test-key");
    assert_eq!(observed.user, "root");
    assert_eq!(observed.password, "");
    assert_eq!(observed.exec.as_deref(), Some("'echo' 'hi there'"));
    assert!(!observed.shell);
}

#[tokio::test(flavor = "multi_thread")]
async fn ssh_opens_a_shell_when_no_command_is_given() {
    let (port, observed) = start_bridge(0).await;

    let output = run_steel(port, &["computer", "ssh", COMPUTER]).await;

    assert_eq!(
        text(&output.stdout),
        "shell ready\n",
        "stderr: {}",
        text(&output.stderr)
    );
    assert!(output.status.success());
    let observed = observed.lock().unwrap().clone();
    assert!(observed.shell);
    assert!(observed.exec.is_none());
}
