use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Result, bail};

use super::protocol::DaemonCreateParams;

/// Scan the config directory for `daemon-*.sock` files and return the session names.
pub fn list_daemon_names() -> Vec<String> {
    let dir = crate::config::config_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return vec![];
    };
    let mut names = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if let Some(rest) = name.strip_prefix("daemon-")
            && let Some(session_name) = rest.strip_suffix(".sock")
            && !session_name.is_empty()
        {
            names.push(session_name.to_string());
        }
    }
    names
}

pub fn socket_path(session_name: &str) -> PathBuf {
    crate::config::config_dir().join(format!("daemon-{session_name}.sock"))
}

pub fn pid_path(session_name: &str) -> PathBuf {
    crate::config::config_dir().join(format!("daemon-{session_name}.pid"))
}

fn log_path(session_name: &str) -> PathBuf {
    crate::config::config_dir().join(format!("daemon-{session_name}.log"))
}

pub fn cleanup_stale(session_name: &str) {
    let _ = std::fs::remove_file(socket_path(session_name));
    let _ = std::fs::remove_file(pid_path(session_name));
}

/// Check if the daemon process for this session is still alive using its PID file.
/// Returns `false` if the PID file is missing, unreadable, or the process is not running.
pub fn is_daemon_alive(session_name: &str) -> bool {
    let pid_file = pid_path(session_name);
    let Ok(contents) = std::fs::read_to_string(&pid_file) else {
        return false;
    };
    let Ok(pid) = contents.trim().parse::<i32>() else {
        return false;
    };
    // kill -0 checks process existence without sending a signal
    std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// If a daemon's socket file exists but its process is dead, clean up stale files.
/// Returns `true` if stale files were removed.
pub fn cleanup_if_dead(session_name: &str) -> bool {
    if socket_path(session_name).exists() && !is_daemon_alive(session_name) {
        cleanup_stale(session_name);
        return true;
    }
    false
}

/// Spawn a daemon process for the given session. The create params are passed via
/// environment variable to avoid leaking API keys in the process list.
pub fn spawn_daemon(session_name: &str, params: &DaemonCreateParams) -> Result<()> {
    let exe = std::env::current_exe()?;

    cleanup_stale(session_name);

    let config_dir = crate::config::config_dir();
    std::fs::create_dir_all(&config_dir)?;

    let log = std::fs::File::create(log_path(session_name))?;

    let params_json = serde_json::to_string(params)?;

    std::process::Command::new(exe)
        .args(["__daemon", "--session-name", session_name])
        .env("STEEL_DAEMON_PARAMS", params_json)
        .stdin(std::process::Stdio::null())
        .stdout(log.try_clone()?)
        .stderr(log)
        .spawn()?;

    Ok(())
}

/// Wait until the daemon socket is connectable.
pub async fn wait_for_daemon(session_name: &str, timeout: Duration) -> Result<()> {
    let sock = socket_path(session_name);
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        if sock.exists() && tokio::net::UnixStream::connect(&sock).await.is_ok() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    bail!(
        "Browser daemon failed to start within {}s",
        timeout.as_secs()
    );
}

/// Send a shutdown command to a running daemon and clean up files.
pub async fn stop_daemon(session_name: &str) -> Result<()> {
    use super::client::DaemonClient;
    use super::protocol::DaemonCommand;

    if let Ok(mut client) = DaemonClient::connect(session_name).await {
        let _ = client.send(DaemonCommand::Shutdown).await;
    }

    tokio::time::sleep(Duration::from_millis(200)).await;
    cleanup_stale(session_name);

    Ok(())
}

/// Kill a daemon process by reading its PID file, then clean up.
pub fn kill_daemon(session_name: &str) -> Result<()> {
    let pid_file = pid_path(session_name);
    if let Ok(contents) = std::fs::read_to_string(&pid_file)
        && let Ok(pid) = contents.trim().parse::<u32>()
    {
        // Best-effort kill via command
        let _ = std::process::Command::new("kill")
            .arg(pid.to_string())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
    cleanup_stale(session_name);
    Ok(())
}
