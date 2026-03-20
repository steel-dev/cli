use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Result, bail};

use super::protocol::DaemonCreateParams;

/// Validate a session name. Returns `Some(error_message)` if invalid.
/// Same rules as profile names: alphanumeric, hyphens, underscores, starts with alnum.
pub fn validate_session_name(name: &str) -> Option<String> {
    if name.is_empty() {
        return Some("Session name cannot be empty.".to_string());
    }
    let first = name.chars().next().unwrap();
    if !first.is_ascii_alphanumeric() {
        return Some("Session name must start with a letter or number.".to_string());
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Some(
            "Session name can only contain letters, numbers, hyphens, and underscores.".to_string(),
        );
    }
    None
}

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
#[allow(unsafe_code)]
pub fn is_daemon_alive(session_name: &str) -> bool {
    let pid_file = pid_path(session_name);
    let Ok(contents) = std::fs::read_to_string(&pid_file) else {
        return false;
    };
    let Ok(pid) = contents.trim().parse::<i32>() else {
        return false;
    };
    // kill(pid, 0) checks process existence without sending a signal
    // SAFETY: kill(pid, 0) is a read-only probe with no side effects.
    unsafe { libc::kill(pid, 0) == 0 }
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
///
pub fn spawn_daemon(
    session_name: &str,
    params: &DaemonCreateParams,
) -> Result<std::process::Child> {
    let exe = std::env::current_exe()?;

    cleanup_stale(session_name);

    let config_dir = crate::config::config_dir();
    std::fs::create_dir_all(&config_dir)?;

    let log = std::fs::File::create(log_path(session_name))?;

    let params_json = serde_json::to_string(params)?;

    let child = std::process::Command::new(exe)
        .args(["__daemon", "--session-name", session_name])
        .env("STEEL_DAEMON_PARAMS", params_json)
        .stdin(std::process::Stdio::null())
        .stdout(log.try_clone()?)
        .stderr(log)
        .spawn()?;

    Ok(child)
}

/// Wait until the daemon socket is connectable.
///
pub async fn wait_for_daemon(
    session_name: &str,
    mut child: std::process::Child,
    timeout: Duration,
) -> Result<()> {
    let sock = socket_path(session_name);
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        match child.try_wait() {
            Ok(Some(status)) => {
                let log_file = log_path(session_name);
                let log_contents = std::fs::read_to_string(&log_file).unwrap_or_default();
                let error_detail = log_contents
                    .lines()
                    .rev()
                    .find(|l| !l.trim().is_empty())
                    .unwrap_or("unknown error (check daemon log)");
                bail!("Browser daemon exited ({status}): {error_detail}");
            }
            Ok(None) => {} // still running
            Err(_) => {}   // can't query status; fall through to socket poll
        }

        if sock.exists() && tokio::net::UnixStream::connect(&sock).await.is_ok() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    bail!(
        "Browser daemon failed to start within {}s. Check logs: {}",
        timeout.as_secs(),
        log_path(session_name).display(),
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
#[allow(unsafe_code)]
pub fn kill_daemon(session_name: &str) -> Result<()> {
    let pid_file = pid_path(session_name);
    if let Ok(contents) = std::fs::read_to_string(&pid_file)
        && let Ok(pid) = contents.trim().parse::<i32>()
    {
        // SAFETY: sending SIGTERM to a known daemon PID we own.
        unsafe { libc::kill(pid, libc::SIGTERM) };
    }
    cleanup_stale(session_name);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── wait_for_daemon early exit detection ─────────────────────────

    /// Serialize tests that mutate STEEL_CONFIG_DIR to avoid env var races.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[tokio::test]
    #[allow(clippy::await_holding_lock, unsafe_code)] // intentional: serialize env var mutations
    async fn wait_for_daemon_surfaces_log_on_early_exit() {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = tempfile::TempDir::new().unwrap();
        let session_name = "test-early-exit";

        // Override config dir so log_path/socket_path resolve into our temp dir.
        // SAFETY: access serialized via ENV_LOCK.
        unsafe { std::env::set_var("STEEL_CONFIG_DIR", tmp.path()) };

        // Pre-create the daemon log with a known error message.
        let log = log_path(session_name);
        std::fs::write(
            &log,
            "Error: Steel API request failed (401): Invalid API key\n",
        )
        .unwrap();

        // Spawn a process that exits immediately.
        let child = std::process::Command::new("sh")
            .args(["-c", "exit 1"])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();

        // Give the child a moment to actually exit.
        tokio::time::sleep(Duration::from_millis(100)).await;

        let start = std::time::Instant::now();
        let result = wait_for_daemon(session_name, child, Duration::from_secs(30)).await;

        unsafe { std::env::remove_var("STEEL_CONFIG_DIR") };

        assert!(result.is_err(), "should fail when child exits early");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Invalid API key"),
            "should contain the actual error from the log, got: {err_msg}"
        );
        // Should detect failure quickly, not wait for the full timeout.
        assert!(
            start.elapsed().as_secs() < 5,
            "should detect early exit quickly, took {:?}",
            start.elapsed()
        );
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock, unsafe_code)] // intentional: serialize env var mutations
    async fn wait_for_daemon_empty_log_still_reports_exit() {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = tempfile::TempDir::new().unwrap();
        let session_name = "test-empty-log";

        unsafe { std::env::set_var("STEEL_CONFIG_DIR", tmp.path()) };

        // Log file doesn't exist — should still report the exit.
        let child = std::process::Command::new("sh")
            .args(["-c", "exit 42"])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;

        let result = wait_for_daemon(session_name, child, Duration::from_secs(30)).await;

        unsafe { std::env::remove_var("STEEL_CONFIG_DIR") };

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        // Should mention the exit status and some fallback message.
        assert!(
            err_msg.contains("daemon") || err_msg.contains("exit"),
            "should mention daemon/exit, got: {err_msg}"
        );
    }

    // ── validate_session_name ────────────────────────────────────────

    #[test]
    fn valid_session_names() {
        assert!(validate_session_name("default").is_none());
        assert!(validate_session_name("my-session").is_none());
        assert!(validate_session_name("my_session").is_none());
        assert!(validate_session_name("a").is_none());
        assert!(validate_session_name("1abc").is_none());
    }

    #[test]
    fn empty_session_name() {
        assert!(validate_session_name("").is_some());
    }

    #[test]
    fn invalid_start_char() {
        assert!(validate_session_name("-foo").is_some());
        assert!(validate_session_name("_foo").is_some());
        assert!(validate_session_name(".foo").is_some());
    }

    #[test]
    fn path_traversal_rejected() {
        assert!(validate_session_name("../../../tmp/evil").is_some());
        assert!(validate_session_name("foo/bar").is_some());
        assert!(validate_session_name("foo.bar").is_some());
    }

    #[test]
    fn spaces_rejected() {
        assert!(validate_session_name("foo bar").is_some());
    }
}
