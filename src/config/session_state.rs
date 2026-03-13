use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

const LOCK_TIMEOUT: Duration = Duration::from_millis(5000);
const LOCK_RETRY: Duration = Duration::from_millis(50);
const LOCK_STALE: Duration = Duration::from_millis(15_000);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionMode {
    Cloud,
    Local,
}

/// Persistent session state. Matches TS `BrowserSessionState`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionState {
    pub active_session_id: Option<String>,
    pub active_session_mode: Option<SessionMode>,
    pub active_session_name: Option<String>,
    pub named_sessions: NamedSessions,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NamedSessions {
    pub cloud: HashMap<String, String>,
    pub local: HashMap<String, String>,
}

impl NamedSessions {
    pub fn get(&self, mode: SessionMode) -> &HashMap<String, String> {
        match mode {
            SessionMode::Cloud => &self.cloud,
            SessionMode::Local => &self.local,
        }
    }

    pub fn get_mut(&mut self, mode: SessionMode) -> &mut HashMap<String, String> {
        match mode {
            SessionMode::Cloud => &mut self.cloud,
            SessionMode::Local => &mut self.local,
        }
    }
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            active_session_id: None,
            active_session_mode: None,
            active_session_name: None,
            named_sessions: NamedSessions::default(),
            updated_at: None,
        }
    }
}

impl SessionState {
    /// Set the active session. Matches TS `setActiveSessionState()`.
    pub fn set_active(
        &mut self,
        mode: SessionMode,
        session_id: &str,
        session_name: Option<&str>,
    ) {
        self.active_session_mode = Some(mode);
        self.active_session_id = Some(session_id.to_string());
        self.active_session_name = session_name.map(|s| s.to_string());
    }

    /// Clear the active session. Matches TS `clearActiveSessionState()`.
    pub fn clear_active(&mut self, mode: SessionMode, session_id: &str) {
        if self.active_session_mode == Some(mode)
            && self.active_session_id.as_deref() == Some(session_id)
        {
            self.active_session_mode = None;
            self.active_session_id = None;
            self.active_session_name = None;
        }

        // Remove from named sessions
        self.named_sessions
            .get_mut(mode)
            .retain(|_, v| v != session_id);
    }

    /// Find a candidate session ID. Matches TS `resolveCandidateSessionId()`.
    pub fn resolve_candidate(
        &self,
        mode: SessionMode,
        session_name: Option<&str>,
    ) -> Option<&str> {
        if let Some(name) = session_name {
            return self.named_sessions.get(mode).get(name).map(|s| s.as_str());
        }

        if self.active_session_mode == Some(mode) {
            return self.active_session_id.as_deref();
        }

        None
    }

    /// Find the name for a session ID. Matches TS `resolveNameFromState()`.
    pub fn resolve_name(&self, mode: SessionMode, session_id: &str) -> Option<&str> {
        for (name, id) in self.named_sessions.get(mode) {
            if id == session_id {
                return Some(name.as_str());
            }
        }

        if self.active_session_mode == Some(mode)
            && self.active_session_id.as_deref() == Some(session_id)
        {
            return self.active_session_name.as_deref();
        }

        None
    }
}

/// File paths for session state storage.
pub struct SessionStatePaths {
    pub state_path: PathBuf,
    pub lock_path: PathBuf,
    pub dir: PathBuf,
}

impl SessionStatePaths {
    pub fn new(config_dir: &Path) -> Self {
        let state_path = config_dir.join("browser-session-state.json");
        let lock_path = config_dir.join("browser-session-state.json.lock");
        Self {
            state_path,
            lock_path,
            dir: config_dir.to_path_buf(),
        }
    }

    pub fn default_paths() -> Self {
        Self::new(&crate::config::config_dir())
    }
}

/// Read session state from disk. Returns default if file doesn't exist or is invalid.
pub fn read_state(path: &Path) -> SessionState {
    let contents = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return SessionState::default(),
    };

    serde_json::from_str(&contents).unwrap_or_default()
}

/// Write session state to disk.
fn write_state(path: &Path, state: &mut SessionState) -> Result<()> {
    state.updated_at = Some(now_iso());
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let contents = serde_json::to_string_pretty(state)?;
    std::fs::write(path, contents)?;
    Ok(())
}

fn now_iso() -> String {
    // Simple ISO 8601 timestamp
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    // Format as basic ISO string (good enough for state tracking)
    format!("{secs}")
}

/// Acquire an exclusive file lock. Matches TS `acquireLock()`.
fn acquire_lock(lock_path: &Path) -> Result<()> {
    let started = std::time::Instant::now();

    loop {
        // Try exclusive create (O_CREAT | O_EXCL)
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(lock_path)
        {
            Ok(_) => return Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                // Check if lock is stale
                if let Ok(metadata) = std::fs::metadata(lock_path) {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(age) = SystemTime::now().duration_since(modified) {
                            if age > LOCK_STALE {
                                let _ = std::fs::remove_file(lock_path);
                                continue;
                            }
                        }
                    }
                }

                if started.elapsed() >= LOCK_TIMEOUT {
                    bail!("Timed out waiting for browser session state lock.");
                }

                std::thread::sleep(LOCK_RETRY);
            }
            Err(e) => {
                return Err(e).context("Failed to acquire session state lock");
            }
        }
    }
}

/// Release the file lock.
fn release_lock(lock_path: &Path) {
    let _ = std::fs::remove_file(lock_path);
}

/// Execute an operation under the session state lock.
/// Matches TS `withSessionStateLock()`.
///
/// If `write` is true (default), the state is written back after the operation.
pub fn with_lock<F, T>(paths: &SessionStatePaths, write: bool, operation: F) -> Result<T>
where
    F: FnOnce(&mut SessionState) -> T,
{
    std::fs::create_dir_all(&paths.dir)?;
    acquire_lock(&paths.lock_path)?;

    let result = (|| {
        let mut state = read_state(&paths.state_path);
        let result = operation(&mut state);
        if write {
            write_state(&paths.state_path, &mut state)?;
        }
        Ok(result)
    })();

    release_lock(&paths.lock_path);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn tmp_paths() -> (TempDir, SessionStatePaths) {
        let dir = TempDir::new().unwrap();
        let paths = SessionStatePaths::new(dir.path());
        (dir, paths)
    }

    // --- SessionState unit tests ---

    #[test]
    fn default_state() {
        let state = SessionState::default();
        assert!(state.active_session_id.is_none());
        assert!(state.active_session_mode.is_none());
        assert!(state.named_sessions.cloud.is_empty());
        assert!(state.named_sessions.local.is_empty());
    }

    #[test]
    fn set_and_resolve_active() {
        let mut state = SessionState::default();
        state.set_active(SessionMode::Cloud, "sess-1", Some("work"));

        assert_eq!(state.active_session_id.as_deref(), Some("sess-1"));
        assert_eq!(state.active_session_mode, Some(SessionMode::Cloud));
        assert_eq!(state.active_session_name.as_deref(), Some("work"));
    }

    #[test]
    fn clear_active_matching() {
        let mut state = SessionState::default();
        state.set_active(SessionMode::Cloud, "sess-1", Some("work"));
        state
            .named_sessions
            .cloud
            .insert("work".into(), "sess-1".into());

        state.clear_active(SessionMode::Cloud, "sess-1");

        assert!(state.active_session_id.is_none());
        assert!(state.named_sessions.cloud.is_empty());
    }

    #[test]
    fn clear_active_non_matching_mode_preserved() {
        let mut state = SessionState::default();
        state.set_active(SessionMode::Cloud, "sess-1", None);

        state.clear_active(SessionMode::Local, "sess-1");

        assert_eq!(state.active_session_id.as_deref(), Some("sess-1"));
    }

    #[test]
    fn resolve_candidate_by_name() {
        let mut state = SessionState::default();
        state
            .named_sessions
            .cloud
            .insert("work".into(), "sess-1".into());

        assert_eq!(
            state.resolve_candidate(SessionMode::Cloud, Some("work")),
            Some("sess-1")
        );
        assert_eq!(
            state.resolve_candidate(SessionMode::Cloud, Some("missing")),
            None
        );
    }

    #[test]
    fn resolve_candidate_by_active() {
        let mut state = SessionState::default();
        state.set_active(SessionMode::Local, "sess-2", None);

        assert_eq!(
            state.resolve_candidate(SessionMode::Local, None),
            Some("sess-2")
        );
        assert_eq!(state.resolve_candidate(SessionMode::Cloud, None), None);
    }

    #[test]
    fn resolve_name_from_named_sessions() {
        let mut state = SessionState::default();
        state
            .named_sessions
            .cloud
            .insert("work".into(), "sess-1".into());

        assert_eq!(
            state.resolve_name(SessionMode::Cloud, "sess-1"),
            Some("work")
        );
        assert_eq!(state.resolve_name(SessionMode::Cloud, "other"), None);
    }

    #[test]
    fn resolve_name_falls_back_to_active() {
        let mut state = SessionState::default();
        state.set_active(SessionMode::Cloud, "sess-1", Some("dev"));

        assert_eq!(
            state.resolve_name(SessionMode::Cloud, "sess-1"),
            Some("dev")
        );
    }

    // --- Serialization ---

    #[test]
    fn state_json_roundtrip() {
        let mut state = SessionState::default();
        state.set_active(SessionMode::Cloud, "abc-123", Some("main"));
        state
            .named_sessions
            .cloud
            .insert("main".into(), "abc-123".into());

        let json = serde_json::to_string_pretty(&state).unwrap();
        let parsed: SessionState = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.active_session_id.as_deref(), Some("abc-123"));
        assert_eq!(parsed.active_session_mode, Some(SessionMode::Cloud));
        assert_eq!(
            parsed.named_sessions.cloud.get("main").map(|s| s.as_str()),
            Some("abc-123")
        );
    }

    #[test]
    fn state_reads_ts_camel_case() {
        let json = r#"{
            "activeSessionId": "s1",
            "activeSessionMode": "local",
            "activeSessionName": "test",
            "namedSessions": {
                "cloud": {},
                "local": {"test": "s1"}
            },
            "updatedAt": "2025-01-01T00:00:00.000Z"
        }"#;

        let state: SessionState = serde_json::from_str(json).unwrap();
        assert_eq!(state.active_session_id.as_deref(), Some("s1"));
        assert_eq!(state.active_session_mode, Some(SessionMode::Local));
        assert_eq!(
            state.named_sessions.local.get("test").map(|s| s.as_str()),
            Some("s1")
        );
    }

    #[test]
    fn read_state_missing_file() {
        let state = read_state(Path::new("/nonexistent/state.json"));
        assert!(state.active_session_id.is_none());
    }

    #[test]
    fn read_state_invalid_json() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("state.json");
        std::fs::write(&path, "not json").unwrap();

        let state = read_state(&path);
        assert!(state.active_session_id.is_none());
    }

    // --- File lock ---

    #[test]
    fn with_lock_read_only() {
        let (_dir, paths) = tmp_paths();

        let result = with_lock(&paths, false, |state| {
            state
                .resolve_candidate(SessionMode::Cloud, None)
                .map(|s| s.to_string())
        })
        .unwrap();

        assert_eq!(result, None);
        assert!(!paths.state_path.exists());
    }

    #[test]
    fn with_lock_write() {
        let (_dir, paths) = tmp_paths();

        with_lock(&paths, true, |state| {
            state.set_active(SessionMode::Cloud, "sess-1", Some("work"));
            state
                .named_sessions
                .cloud
                .insert("work".into(), "sess-1".into());
        })
        .unwrap();

        let state = read_state(&paths.state_path);
        assert_eq!(state.active_session_id.as_deref(), Some("sess-1"));
        assert_eq!(
            state.named_sessions.cloud.get("work").map(|s| s.as_str()),
            Some("sess-1")
        );
    }

    #[test]
    fn lock_released_after_operation() {
        let (_dir, paths) = tmp_paths();

        with_lock(&paths, true, |state| {
            state.set_active(SessionMode::Local, "s1", None);
        })
        .unwrap();

        assert!(!paths.lock_path.exists());

        // Second lock should succeed
        with_lock(&paths, false, |_| {}).unwrap();
    }

    #[test]
    fn state_persists_across_calls() {
        let (_dir, paths) = tmp_paths();

        with_lock(&paths, true, |state| {
            state
                .named_sessions
                .local
                .insert("dev".into(), "sess-a".into());
        })
        .unwrap();

        let id = with_lock(&paths, false, |state| {
            state
                .resolve_candidate(SessionMode::Local, Some("dev"))
                .map(|s| s.to_string())
        })
        .unwrap();

        assert_eq!(id.as_deref(), Some("sess-a"));
    }
}
