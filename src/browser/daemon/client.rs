use std::time::Duration;

use anyhow::{Result, bail};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

use super::process;
use super::protocol::*;

pub struct DaemonClient {
    reader: BufReader<tokio::io::ReadHalf<UnixStream>>,
    writer: tokio::io::WriteHalf<UnixStream>,
    next_id: u64,
}

impl DaemonClient {
    pub async fn connect(session_name: &str) -> Result<Self> {
        let sock = process::socket_path(session_name);
        let stream = match UnixStream::connect(&sock).await {
            Ok(s) => s,
            Err(_) => {
                // Socket file may exist but the daemon process is dead — clean up
                process::cleanup_if_dead(session_name);
                return Err(anyhow::anyhow!(
                    "Cannot connect to browser daemon. Is a session running?"
                ));
            }
        };
        let (read_half, write_half) = tokio::io::split(stream);
        Ok(Self {
            reader: BufReader::new(read_half),
            writer: write_half,
            next_id: 1,
        })
    }

    pub async fn send(&mut self, command: DaemonCommand) -> Result<serde_json::Value> {
        let request = DaemonRequest {
            id: self.next_id,
            command,
        };
        self.next_id += 1;

        let mut json = serde_json::to_string(&request)?;
        json.push('\n');
        self.writer.write_all(json.as_bytes()).await?;
        self.writer.flush().await?;

        let mut line = String::new();
        let n = tokio::time::timeout(Duration::from_secs(120), self.reader.read_line(&mut line))
            .await
            .map_err(|_| anyhow::anyhow!("Daemon response timed out after 120s"))??;
        if n == 0 {
            bail!("Daemon disconnected unexpectedly");
        }

        let response: DaemonResponse = serde_json::from_str(&line)?;
        match response.result {
            DaemonResult::Ok { data } => Ok(data),
            DaemonResult::Error { message } => bail!("{message}"),
        }
    }
}
