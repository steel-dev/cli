use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileData {
    pub profile_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chrome_profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser: Option<String>,
}

/// Validate a profile name. Returns `Some(error_message)` if invalid.
pub fn validate_profile_name(name: &str) -> Option<String> {
    if name.is_empty() {
        return Some("Profile name cannot be empty.".to_string());
    }

    let first = name.chars().next().unwrap();
    if !first.is_ascii_alphanumeric() {
        return Some("Profile name must start with a letter or number.".to_string());
    }

    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Some(
            "Profile name can only contain letters, numbers, hyphens, and underscores.".to_string(),
        );
    }

    None
}

fn profiles_dir() -> PathBuf {
    crate::config::config_dir().join("profiles")
}

pub fn read_profile(name: &str) -> Result<Option<ProfileData>> {
    read_profile_in(name, &profiles_dir())
}

pub fn write_profile(
    name: &str,
    profile_id: &str,
    chrome_profile: Option<&str>,
    browser: Option<&str>,
) -> Result<()> {
    write_profile_in(name, profile_id, chrome_profile, browser, &profiles_dir())
}

pub fn list_profiles() -> Result<Vec<ProfileEntry>> {
    list_profiles_in(&profiles_dir())
}

pub fn delete_profile(name: &str) -> Result<bool> {
    delete_profile_in(name, &profiles_dir())
}

// --- internal helpers that accept an explicit directory ---

fn read_profile_in(name: &str, dir: &Path) -> Result<Option<ProfileData>> {
    let path = dir.join(format!("{name}.json"));
    if !path.exists() {
        return Ok(None);
    }
    let contents = std::fs::read_to_string(&path)?;
    let data: ProfileData = serde_json::from_str(&contents)?;
    Ok(Some(data))
}

fn write_profile_in(
    name: &str,
    profile_id: &str,
    chrome_profile: Option<&str>,
    browser: Option<&str>,
    dir: &Path,
) -> Result<()> {
    std::fs::create_dir_all(dir)?;

    let data = ProfileData {
        profile_id: profile_id.to_string(),
        chrome_profile: chrome_profile.map(|s| s.to_string()),
        browser: browser.map(|s| s.to_string()),
    };

    let contents = serde_json::to_string_pretty(&data)?;
    std::fs::write(dir.join(format!("{name}.json")), contents)?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct ProfileEntry {
    pub name: String,
    pub profile_id: String,
}

fn list_profiles_in(dir: &Path) -> Result<Vec<ProfileEntry>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        if name.is_empty() {
            continue;
        }

        // Skip corrupt files silently
        let Ok(contents) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(data) = serde_json::from_str::<ProfileData>(&contents) else {
            continue;
        };

        entries.push(ProfileEntry {
            name,
            profile_id: data.profile_id,
        });
    }

    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}

fn delete_profile_in(name: &str, dir: &Path) -> Result<bool> {
    let path = dir.join(format!("{name}.json"));
    if !path.exists() {
        return Ok(false);
    }
    std::fs::remove_file(&path)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- validate_profile_name ---

    #[test]
    fn valid_names() {
        assert!(validate_profile_name("myprof").is_none());
        assert!(validate_profile_name("my-prof").is_none());
        assert!(validate_profile_name("my_prof").is_none());
        assert!(validate_profile_name("a").is_none());
        assert!(validate_profile_name("1abc").is_none());
        assert!(validate_profile_name("A-b_C-3").is_none());
    }

    #[test]
    fn empty_name() {
        assert!(validate_profile_name("").is_some());
    }

    #[test]
    fn invalid_start() {
        assert!(validate_profile_name("-foo").is_some());
        assert!(validate_profile_name("_foo").is_some());
        assert!(validate_profile_name(".foo").is_some());
    }

    #[test]
    fn invalid_chars() {
        assert!(validate_profile_name("foo bar").is_some());
        assert!(validate_profile_name("foo.bar").is_some());
        assert!(validate_profile_name("foo/bar").is_some());
    }

    // --- file-based operations ---

    fn tmp_profiles_dir() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path().join("profiles");
        (tmp, dir)
    }

    #[test]
    fn write_read_delete_roundtrip() {
        let (_tmp, dir) = tmp_profiles_dir();

        write_profile_in(
            "test-prof",
            "prof_123",
            Some("Default"),
            Some("chrome"),
            &dir,
        )
        .unwrap();

        let data = read_profile_in("test-prof", &dir).unwrap().unwrap();
        assert_eq!(data.profile_id, "prof_123");
        assert_eq!(data.chrome_profile.as_deref(), Some("Default"));

        let entries = list_profiles_in(&dir).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "test-prof");
        assert_eq!(entries[0].profile_id, "prof_123");

        assert!(delete_profile_in("test-prof", &dir).unwrap());
        assert!(!delete_profile_in("test-prof", &dir).unwrap());
        assert!(read_profile_in("test-prof", &dir).unwrap().is_none());
    }

    #[test]
    fn read_nonexistent() {
        let (_tmp, dir) = tmp_profiles_dir();
        assert!(read_profile_in("nope", &dir).unwrap().is_none());
    }

    #[test]
    fn list_skips_corrupt() {
        let (_tmp, dir) = tmp_profiles_dir();
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("good.json"), r#"{"profileId":"p1"}"#).unwrap();
        std::fs::write(dir.join("bad.json"), "not json").unwrap();
        std::fs::write(dir.join("skip.txt"), "not a profile").unwrap();

        let entries = list_profiles_in(&dir).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "good");
    }

    #[test]
    fn write_without_chrome_profile() {
        let (_tmp, dir) = tmp_profiles_dir();

        write_profile_in("minimal", "prof_456", None, None, &dir).unwrap();
        let data = read_profile_in("minimal", &dir).unwrap().unwrap();
        assert_eq!(data.profile_id, "prof_456");
        assert!(data.chrome_profile.is_none());

        // Verify chromeProfile is omitted from JSON
        let json = std::fs::read_to_string(dir.join("minimal.json")).unwrap();
        assert!(!json.contains("chromeProfile"));
    }

    #[test]
    fn list_empty_dir() {
        let (_tmp, dir) = tmp_profiles_dir();
        let entries = list_profiles_in(&dir).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn profiles_dir_is_under_config_dir() {
        let dir = profiles_dir();
        let config = crate::config::config_dir();
        assert_eq!(dir, config.join("profiles"));
    }

    #[test]
    fn list_sorted_by_name() {
        let (_tmp, dir) = tmp_profiles_dir();
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("zz.json"), r#"{"profileId":"p3"}"#).unwrap();
        std::fs::write(dir.join("aa.json"), r#"{"profileId":"p1"}"#).unwrap();
        std::fs::write(dir.join("mm.json"), r#"{"profileId":"p2"}"#).unwrap();

        let entries = list_profiles_in(&dir).unwrap();
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["aa", "mm", "zz"]);
    }
}
