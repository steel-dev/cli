use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::config::settings::ApiMode;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotInfo {
    pub id: String,
    #[serde(default)]
    pub interactive: bool,
    #[serde(default)]
    pub compact: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depth: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserSession {
    pub name: String,
    pub id: String,
    pub mode: ApiMode,
    pub api_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inactivity_timeout_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tab: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub events: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dialog: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<SnapshotInfo>,
}

impl BrowserSession {
    pub const fn new(name: String, id: String, mode: ApiMode, api_url: String) -> Self {
        Self {
            name,
            id,
            mode,
            api_url,
            live_url: None,
            profile_id: None,
            timeout_ms: None,
            inactivity_timeout_ms: None,
            created_at_ms: None,
            next: None,
            tab: None,
            events: None,
            dialog: None,
            snapshot: None,
        }
    }
}

pub fn validate_session_name(name: &str) -> Option<String> {
    let Some(first) = name.chars().next() else {
        return Some("Session name cannot be empty.".to_string());
    };
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

pub fn looks_like_session_id(value: &str) -> bool {
    let groups: Vec<&str> = value.split('-').collect();
    groups.len() == 5
        && groups
            .iter()
            .zip([8, 4, 4, 4, 12])
            .all(|(group, len)| group.len() == len && group.chars().all(|c| c.is_ascii_hexdigit()))
}

fn sessions_dir() -> PathBuf {
    crate::config::config_dir().join("browser-sessions")
}

pub fn load(name: &str) -> Result<Option<BrowserSession>> {
    load_in(name, &sessions_dir())
}

pub fn save(session: &BrowserSession) -> Result<()> {
    save_in(session, &sessions_dir())
}

pub fn remove(name: &str) -> Result<bool> {
    remove_in(name, &sessions_dir())
}

pub fn list() -> Vec<BrowserSession> {
    list_in(&sessions_dir())
}

fn load_in(name: &str, dir: &Path) -> Result<Option<BrowserSession>> {
    let path = dir.join(format!("{name}.json"));
    if !path.exists() {
        return Ok(None);
    }
    let contents = std::fs::read_to_string(&path)?;
    Ok(Some(serde_json::from_str(&contents)?))
}

fn save_in(session: &BrowserSession, dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    let contents = serde_json::to_string_pretty(session)?;
    let target = dir.join(format!("{}.json", session.name));
    let tmp = dir.join(format!(".{}.{}.tmp", session.name, std::process::id()));
    std::fs::write(&tmp, contents)?;
    std::fs::rename(&tmp, &target)?;
    Ok(())
}

fn remove_in(name: &str, dir: &Path) -> Result<bool> {
    let path = dir.join(format!("{name}.json"));
    if !path.exists() {
        return Ok(false);
    }
    std::fs::remove_file(&path)?;
    Ok(true)
}

fn list_in(dir: &Path) -> Vec<BrowserSession> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut sessions: Vec<BrowserSession> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("json"))
        .filter_map(|path| std::fs::read_to_string(path).ok())
        .filter_map(|contents| serde_json::from_str(&contents).ok())
        .collect();
    sessions.sort_by(|a, b| a.name.cmp(&b.name));
    sessions
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(name: &str) -> BrowserSession {
        BrowserSession::new(
            name.to_string(),
            "11111111-2222-3333-4444-555555555555".to_string(),
            ApiMode::Cloud,
            "https://api.steel.dev/v1".to_string(),
        )
    }

    #[test]
    fn save_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = session("work");
        s.next = Some("op_abc".into());
        s.tab = Some("t2".into());
        s.snapshot = Some(SnapshotInfo {
            id: "s-4".into(),
            interactive: true,
            compact: false,
            depth: Some(3),
        });
        save_in(&s, dir.path()).unwrap();
        assert_eq!(load_in("work", dir.path()).unwrap(), Some(s));
    }

    #[test]
    fn load_missing_is_none() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(load_in("nope", dir.path()).unwrap(), None);
    }

    #[test]
    fn save_leaves_no_temp_files() {
        let dir = tempfile::tempdir().unwrap();
        save_in(&session("a"), dir.path()).unwrap();
        let names: Vec<String> = std::fs::read_dir(dir.path())
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        assert_eq!(names, vec!["a.json"]);
    }

    #[test]
    fn list_sorts_and_skips_corrupt_files() {
        let dir = tempfile::tempdir().unwrap();
        save_in(&session("b"), dir.path()).unwrap();
        save_in(&session("a"), dir.path()).unwrap();
        std::fs::write(dir.path().join("broken.json"), "{").unwrap();
        let names: Vec<String> = list_in(dir.path()).into_iter().map(|s| s.name).collect();
        assert_eq!(names, vec!["a", "b"]);
    }

    #[test]
    fn list_missing_dir_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        assert!(list_in(&dir.path().join("missing")).is_empty());
    }

    #[test]
    fn remove_reports_whether_it_existed() {
        let dir = tempfile::tempdir().unwrap();
        save_in(&session("a"), dir.path()).unwrap();
        assert!(remove_in("a", dir.path()).unwrap());
        assert!(!remove_in("a", dir.path()).unwrap());
    }

    #[test]
    fn old_state_files_without_optional_fields_load() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("x.json"),
            r#"{"name":"x","id":"abc","mode":"cloud","apiUrl":"https://api.steel.dev/v1"}"#,
        )
        .unwrap();
        let loaded = load_in("x", dir.path()).unwrap().unwrap();
        assert_eq!(loaded.next, None);
        assert_eq!(loaded.snapshot, None);
    }

    #[test]
    fn validate_session_name_rules() {
        assert!(validate_session_name("my-task_1").is_none());
        assert!(validate_session_name("").is_some());
        assert!(validate_session_name("-x").is_some());
        assert!(validate_session_name("a/b").is_some());
        assert!(validate_session_name("../x").is_some());
    }

    #[test]
    fn session_ids_are_recognized() {
        assert!(looks_like_session_id(
            "4f7c2b1e-9a3d-4e5f-8b6a-1c2d3e4f5a6b"
        ));
        assert!(!looks_like_session_id("default"));
        assert!(!looks_like_session_id("4f7c2b1e-9a3d-4e5f-8b6a"));
        assert!(!looks_like_session_id(
            "4f7c2b1e-9a3d-4e5f-8b6a-1c2d3e4f5a6g"
        ));
    }
}
