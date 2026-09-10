use std::io::IsTerminal;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use clap::Parser;
use russh::client::{self, AuthResult, Handler};
use russh::keys::PublicKeyOrCertificate;
use russh::{Channel, ChannelMsg, Disconnect};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;

use super::resolve_computer_id;
use super::wsio::WsIo;
use crate::api::client::ApiError;
use crate::config::auth::Auth;
use crate::config::settings::ApiMode;
use crate::util::api;
use crate::util::output::SilentExit;

pub const SUBPROTOCOL: &str = "steel-ssh-v1";
const REMOTE_USER: &str = "root";

#[derive(Parser)]
pub struct Args {
    /// Computer ID (defaults to STEEL_COMPUTER_ID or `steel computer use`)
    pub computer_id: Option<String>,

    /// Run this command instead of opening a shell, given after `--`
    #[arg(last = true, value_name = "COMMAND")]
    pub command: Vec<String>,
}

struct Client;

impl Handler for Client {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _key: &PublicKeyOrCertificate,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

pub async fn run(args: Args) -> Result<()> {
    let id = resolve_computer_id(args.computer_id.as_deref())?;
    let (mode, base_url, auth) = api::resolve_with_auth();
    if mode == ApiMode::Cloud && auth.api_key.is_none() {
        return Err(ApiError::MissingAuth.into());
    }
    let url = ssh_url(&base_url, &id)?;
    let transport = connect(&url, &auth).await?;

    let config = Arc::new(client::Config {
        keepalive_interval: Some(Duration::from_secs(15)),
        ..Default::default()
    });
    let mut handle = client::connect_stream(config, transport, Client)
        .await
        .context("The SSH handshake with the computer failed")?;
    let mut auth_result = handle.authenticate_password(REMOTE_USER, "").await?;
    if !matches!(auth_result, AuthResult::Success) {
        auth_result = handle.authenticate_none(REMOTE_USER).await?;
    }
    if !matches!(auth_result, AuthResult::Success) {
        bail!("The computer refused the SSH login.");
    }

    let mut channel = handle.channel_open_session().await?;
    let code = if args.command.is_empty() {
        shell(&mut channel).await?
    } else {
        exec(&mut channel, &args.command).await?
    };
    let _ = handle.disconnect(Disconnect::ByApplication, "", "en").await;
    if code != 0 {
        return Err(SilentExit(code).into());
    }
    Ok(())
}

pub fn ssh_url(base_url: &str, id: &str) -> Result<String> {
    let mut url =
        url::Url::parse(base_url).with_context(|| format!("Invalid API URL: {base_url}"))?;
    let scheme = match url.scheme() {
        "https" | "wss" => "wss",
        "http" | "ws" => "ws",
        other => bail!("Unsupported API URL scheme for SSH: {other}"),
    };
    url.set_scheme(scheme)
        .map_err(|_| anyhow::anyhow!("Failed to set the WebSocket scheme"))?;
    let base_path = url.path().trim_end_matches('/').to_string();
    let base_path = if base_path.ends_with("/v1") {
        base_path
    } else {
        format!("{base_path}/v1")
    };
    url.set_path(&format!(
        "{base_path}/computers/{}/ssh",
        urlencoding::encode(id)
    ));
    url.set_query(None);
    Ok(url.to_string())
}

async fn connect(url: &str, auth: &Auth) -> Result<WsIo<MaybeTlsStream<TcpStream>>> {
    let mut request = url
        .into_client_request()
        .with_context(|| format!("Invalid SSH URL: {url}"))?;
    request.headers_mut().insert(
        "Sec-WebSocket-Protocol",
        HeaderValue::from_static(SUBPROTOCOL),
    );
    if let Some(key) = auth.api_key.as_deref().filter(|key| !key.trim().is_empty()) {
        request.headers_mut().insert(
            "Steel-Api-Key",
            HeaderValue::from_str(key.trim()).context("The API key is not a valid header value")?,
        );
    }
    let (stream, response) = connect_async(request)
        .await
        .context("Failed to open the SSH bridge to the computer")?;
    let accepted = response
        .headers()
        .get("Sec-WebSocket-Protocol")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case(SUBPROTOCOL));
    if !accepted {
        bail!("The gateway did not accept the {SUBPROTOCOL} subprotocol.");
    }
    Ok(WsIo::new(stream))
}

async fn shell(channel: &mut Channel<client::Msg>) -> Result<i32> {
    let interactive = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    if interactive {
        let (cols, rows) = terminal::size().unwrap_or((80, 24));
        let term = std::env::var("TERM").unwrap_or_else(|_| "xterm-256color".to_string());
        channel
            .request_pty(true, &term, cols, rows, 0, 0, &[])
            .await?;
    }
    channel.request_shell(true).await?;
    let _raw = if interactive {
        Some(terminal::RawMode::enable()?)
    } else {
        None
    };
    pump(channel, interactive).await
}

async fn exec(channel: &mut Channel<client::Msg>, command: &[String]) -> Result<i32> {
    channel.exec(true, shell_join(command)).await?;
    pump(channel, false).await
}

pub fn shell_join(argv: &[String]) -> String {
    argv.iter()
        .map(|arg| format!("'{}'", arg.replace('\'', "'\\''")))
        .collect::<Vec<_>>()
        .join(" ")
}

async fn pump(channel: &mut Channel<client::Msg>, forward_resize: bool) -> Result<i32> {
    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut stderr = tokio::io::stderr();
    let mut resize = terminal::resize_signal(forward_resize)?;
    let mut buffer = [0u8; 8192];
    let mut code = 0;
    let mut stdin_open = true;
    loop {
        tokio::select! {
            message = channel.wait() => match message {
                None | Some(ChannelMsg::Close) | Some(ChannelMsg::Eof) => break,
                Some(ChannelMsg::Data { data }) => {
                    stdout.write_all(&data).await?;
                    stdout.flush().await?;
                }
                Some(ChannelMsg::ExtendedData { data, .. }) => {
                    stderr.write_all(&data).await?;
                    stderr.flush().await?;
                }
                Some(ChannelMsg::ExitStatus { exit_status }) => code = exit_status as i32,
                Some(_) => {}
            },
            read = stdin.read(&mut buffer), if stdin_open => match read? {
                0 => {
                    stdin_open = false;
                    channel.eof().await?;
                }
                n => channel.data(&buffer[..n]).await?,
            },
            _ = terminal::resized(&mut resize) => {
                if let Some((cols, rows)) = terminal::size() {
                    channel.window_change(cols, rows, 0, 0).await?;
                }
            }
        }
    }
    Ok(code)
}

#[cfg(unix)]
mod terminal {
    use std::io;

    use rustix::termios::{self, OptionalActions, Termios};
    use tokio::signal::unix::{Signal, SignalKind, signal};

    pub struct RawMode {
        original: Termios,
    }

    impl RawMode {
        pub fn enable() -> io::Result<Self> {
            let stdin = io::stdin();
            let original = termios::tcgetattr(&stdin)?;
            let mut raw = original.clone();
            raw.make_raw();
            termios::tcsetattr(&stdin, OptionalActions::Now, &raw)?;
            Ok(Self { original })
        }
    }

    impl Drop for RawMode {
        fn drop(&mut self) {
            let _ = termios::tcsetattr(io::stdin(), OptionalActions::Now, &self.original);
        }
    }

    pub fn size() -> Option<(u32, u32)> {
        let size = termios::tcgetwinsize(io::stdout()).ok()?;
        (size.ws_col > 0).then(|| (u32::from(size.ws_col), u32::from(size.ws_row)))
    }

    pub fn resize_signal(enabled: bool) -> io::Result<Option<Signal>> {
        if enabled {
            Ok(Some(signal(SignalKind::window_change())?))
        } else {
            Ok(None)
        }
    }

    pub async fn resized(signal: &mut Option<Signal>) {
        match signal {
            Some(signal) => {
                signal.recv().await;
            }
            None => std::future::pending().await,
        }
    }
}

#[cfg(not(unix))]
mod terminal {
    use std::io;

    pub struct RawMode;

    impl RawMode {
        pub fn enable() -> io::Result<Self> {
            Ok(Self)
        }
    }

    pub fn size() -> Option<(u32, u32)> {
        None
    }

    pub fn resize_signal(_enabled: bool) -> io::Result<Option<()>> {
        Ok(None)
    }

    pub async fn resized(_signal: &mut Option<()>) {
        std::future::pending().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ssh_urls_follow_the_api_host() {
        assert_eq!(
            ssh_url("https://api.steel.dev/v1", "cmp_a").unwrap(),
            "wss://api.steel.dev/v1/computers/cmp_a/ssh"
        );
        assert_eq!(
            ssh_url("http://localhost:3000", "cmp_a").unwrap(),
            "ws://localhost:3000/v1/computers/cmp_a/ssh"
        );
        assert!(ssh_url("ftp://x", "cmp_a").is_err());
    }

    #[test]
    fn remote_commands_are_quoted_for_the_shell() {
        let argv = vec!["echo".to_string(), "it's here".to_string()];
        assert_eq!(shell_join(&argv), "'echo' 'it'\\''s here'");
    }
}
