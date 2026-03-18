use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Result, bail};

pub fn socket_path(session_id: &str) -> PathBuf {
    crate::config::config_dir().join(format!("daemon-{session_id}.sock"))
}

pub fn pid_path(session_id: &str) -> PathBuf {
    crate::config::config_dir().join(format!("daemon-{session_id}.pid"))
}

fn log_path(session_id: &str) -> PathBuf {
    crate::config::config_dir().join(format!("daemon-{session_id}.log"))
}

pub fn cleanup_stale(session_id: &str) {
    let _ = std::fs::remove_file(socket_path(session_id));
    let _ = std::fs::remove_file(pid_path(session_id));
}

/// Spawn a daemon process for the given session. The CDP URL is passed via
/// environment variable to avoid leaking API keys in the process list.
pub fn spawn_daemon(session_id: &str, cdp_url: &str) -> Result<()> {
    let exe = std::env::current_exe()?;

    cleanup_stale(session_id);

    let config_dir = crate::config::config_dir();
    std::fs::create_dir_all(&config_dir)?;

    let log = std::fs::File::create(log_path(session_id))?;

    std::process::Command::new(exe)
        .args(["__daemon", "--session-id", session_id])
        .env("STEEL_DAEMON_CDP_URL", cdp_url)
        .stdin(std::process::Stdio::null())
        .stdout(log.try_clone()?)
        .stderr(log)
        .spawn()?;

    Ok(())
}

/// Wait until the daemon socket is connectable.
pub async fn wait_for_daemon(session_id: &str, timeout: Duration) -> Result<()> {
    let sock = socket_path(session_id);
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        if sock.exists() {
            if tokio::net::UnixStream::connect(&sock).await.is_ok() {
                return Ok(());
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    bail!(
        "Browser daemon failed to start within {}s",
        timeout.as_secs()
    );
}

/// Send a shutdown command to a running daemon and clean up files.
pub async fn stop_daemon(session_id: &str) -> Result<()> {
    use super::client::DaemonClient;
    use super::protocol::DaemonCommand;

    if let Ok(mut client) = DaemonClient::connect(session_id).await {
        let _ = client.send(DaemonCommand::Shutdown).await;
    }

    tokio::time::sleep(Duration::from_millis(200)).await;
    cleanup_stale(session_id);

    Ok(())
}

/// Kill a daemon process by reading its PID file, then clean up.
pub fn kill_daemon(session_id: &str) -> Result<()> {
    let pid_file = pid_path(session_id);
    if let Ok(contents) = std::fs::read_to_string(&pid_file) {
        if let Ok(pid) = contents.trim().parse::<u32>() {
            // Best-effort kill via command
            let _ = std::process::Command::new("kill")
                .arg(pid.to_string())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
        }
    }
    cleanup_stale(session_id);
    Ok(())
}
