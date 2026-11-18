use std::fs::OpenOptions;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::config;
use crate::util::version::{is_newer, same_major};

const CHILD_ARG: &str = "__auto-update";
const CHECK_INTERVAL_MS: u64 = 24 * 60 * 60 * 1000;
const LOCK_STALE_AFTER: Duration = Duration::from_secs(10 * 60);
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(300);
const VERIFY_TIMEOUT: Duration = Duration::from_secs(10);
const LATEST_RELEASE_URL: &str = "https://api.github.com/repos/steel-dev/cli/releases/latest";
const RELEASES_URL: &str = "https://github.com/steel-dev/cli/releases";
const CURRENT: &str = env!("CARGO_PKG_VERSION");

const TARGET: Option<&str> = if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
    Some("aarch64-apple-darwin")
} else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
    Some("x86_64-apple-darwin")
} else if cfg!(all(
    target_os = "linux",
    target_env = "gnu",
    target_arch = "aarch64"
)) {
    Some("aarch64-unknown-linux-gnu")
} else if cfg!(all(
    target_os = "linux",
    target_env = "gnu",
    target_arch = "x86_64"
)) {
    Some("x86_64-unknown-linux-gnu")
} else {
    None
};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct State {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_check_unix_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    updated: Option<Updated>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    available: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Updated {
    from: String,
    to: String,
}

pub async fn latest_version(client: &reqwest::Client) -> Option<String> {
    let resp = client
        .get(LATEST_RELEASE_URL)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", format!("steel-cli/{CURRENT}"))
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let data: serde_json::Value = resp.json().await.ok()?;
    let tag = data.get("tag_name")?.as_str()?;
    Some(tag.strip_prefix('v').unwrap_or(tag).to_string())
}

pub fn start(skip: bool) {
    if skip || disabled() {
        return;
    }
    let Some(exe) = managed_exe() else {
        return;
    };
    let path = state_path(&config::config_dir());
    let mut state = read_state(&path);
    let original = state.clone();

    let interactive = !crate::util::output::is_json() && std::io::stderr().is_terminal();
    for note in take_notes(&mut state, CURRENT, interactive) {
        eprintln!("note: {note}");
    }

    let now = now_unix_ms();
    let due = check_due(&state, now);
    if due {
        state.last_check_unix_ms = Some(now);
    }

    if state == original || write_state(&path, &state).is_err() {
        return;
    }
    if due {
        spawn(&exe);
    }
}

pub fn is_child() -> bool {
    std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == CHILD_ARG)
}

pub async fn run_child() {
    let dir = config::config_dir();
    let Some(_lock) = Lock::acquire(&dir.join("update.lock")) else {
        return;
    };
    let (Some(exe), Some(target)) = (managed_exe(), TARGET) else {
        return;
    };
    let Ok(client) = reqwest::Client::builder()
        .user_agent(format!("steel-cli/{CURRENT}"))
        .timeout(DOWNLOAD_TIMEOUT)
        .build()
    else {
        return;
    };
    let Some(latest) = latest_version(&client).await else {
        return;
    };
    if !is_newer(&latest, CURRENT) {
        return;
    }

    let path = state_path(&dir);
    if !same_major(&latest, CURRENT) {
        let mut state = read_state(&path);
        state.available = Some(latest);
        let _ = write_state(&path, &state);
        return;
    }

    let base_url = format!("{RELEASES_URL}/download");
    if install(&client, &base_url, target, &latest, &exe)
        .await
        .is_ok()
    {
        let mut state = read_state(&path);
        state.updated = Some(Updated {
            from: CURRENT.to_string(),
            to: latest,
        });
        let _ = write_state(&path, &state);
    }
}

fn disabled() -> bool {
    env_flag("STEEL_AUTO_UPDATE_DISABLED")
        || in_ci()
        || crate::config::settings::read_config().is_ok_and(|config| config.auto_update_disabled())
}

fn env_flag(name: &str) -> bool {
    std::env::var(name).is_ok_and(|value| matches!(value.trim(), "1" | "true" | "TRUE" | "True"))
}

fn in_ci() -> bool {
    std::env::var("CI").is_ok_and(|value| !matches!(value.trim(), "" | "0" | "false" | "FALSE"))
}

fn managed_exe() -> Option<PathBuf> {
    TARGET?;
    let exe = std::env::current_exe().ok()?.canonicalize().ok()?;
    let dir = exe.parent()?;
    install_dirs()
        .into_iter()
        .filter_map(|candidate| candidate.canonicalize().ok())
        .any(|candidate| candidate == dir)
        .then_some(exe)
}

fn install_dirs() -> Vec<PathBuf> {
    let home = dirs::home_dir();
    let config_home = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| home.as_ref().map(|home| home.join(".config")));
    let receipt_prefix = config_home
        .and_then(|dir| {
            std::fs::read_to_string(dir.join("steel-cli").join("steel-cli-receipt.json")).ok()
        })
        .and_then(|contents| serde_json::from_str::<serde_json::Value>(&contents).ok())
        .and_then(|receipt| receipt["install_prefix"].as_str().map(PathBuf::from));
    receipt_prefix
        .into_iter()
        .chain(home.map(|home| home.join(".steel").join("bin")))
        .collect()
}

fn take_notes(state: &mut State, current: &str, interactive: bool) -> Vec<String> {
    let mut notes = Vec::new();

    if let Some(updated) = state.updated.take()
        && updated.to == current
    {
        if interactive {
            notes.push(format!(
                "steel updated itself from v{} to v{}. See what changed: {RELEASES_URL}/tag/v{}",
                updated.from, updated.to, updated.to
            ));
        } else {
            state.updated = Some(updated);
        }
    }

    if let Some(latest) = state.available.take()
        && is_newer(&latest, current)
    {
        if interactive {
            notes.push(format!(
                "steel v{latest} is available. It is a major release, so it was not installed automatically. Run `steel update` to install it."
            ));
        } else {
            state.available = Some(latest);
        }
    }

    notes
}

fn check_due(state: &State, now: u64) -> bool {
    state.last_check_unix_ms.is_none_or(|last| {
        now.checked_sub(last)
            .is_none_or(|elapsed| elapsed >= CHECK_INTERVAL_MS)
    })
}

fn spawn(exe: &Path) {
    use std::process::{Command, Stdio};

    let mut command = Command::new(exe);
    command
        .arg(CHILD_ARG)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let _ = command.spawn();
}

async fn install(
    client: &reqwest::Client,
    base_url: &str,
    target: &str,
    version: &str,
    exe: &Path,
) -> anyhow::Result<()> {
    let name = format!("steel-cli-{target}.tar.gz");
    let url = format!("{base_url}/v{version}/{name}");
    let archive = fetch(client, &url).await?;
    let checksum = fetch(client, &format!("{url}.sha256")).await?;
    verify_checksum(&archive, &checksum)?;

    let dir = exe.parent().context("install directory not found")?;
    let work = tempfile::Builder::new()
        .prefix(".steel-update-")
        .tempdir_in(dir)?;
    let archive_path = work.path().join(&name);
    std::fs::write(&archive_path, &archive)?;
    let status = tokio::process::Command::new("tar")
        .arg("-xzf")
        .arg(&archive_path)
        .arg("-C")
        .arg(work.path())
        .stdin(std::process::Stdio::null())
        .status()
        .await?;
    if !status.success() {
        bail!("could not extract {name}");
    }

    let staged = work
        .path()
        .join(format!("steel-cli-{target}"))
        .join("steel");
    check_version(&staged, version).await?;
    std::fs::rename(&staged, exe)?;
    Ok(())
}

async fn fetch(client: &reqwest::Client, url: &str) -> anyhow::Result<Vec<u8>> {
    let resp = client.get(url).send().await?.error_for_status()?;
    Ok(resp.bytes().await?.to_vec())
}

fn verify_checksum(data: &[u8], checksum_file: &[u8]) -> anyhow::Result<()> {
    let expected = std::str::from_utf8(checksum_file)?
        .split_whitespace()
        .next()
        .context("checksum file is empty")?;
    if !expected.eq_ignore_ascii_case(&sha256_hex(data)) {
        bail!("checksum mismatch");
    }
    Ok(())
}

fn sha256_hex(data: &[u8]) -> String {
    Sha256::digest(data)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

async fn check_version(binary: &Path, version: &str) -> anyhow::Result<()> {
    let output = tokio::time::timeout(
        VERIFY_TIMEOUT,
        tokio::process::Command::new(binary)
            .arg("--version")
            .stdin(std::process::Stdio::null())
            .kill_on_drop(true)
            .output(),
    )
    .await??;
    let printed = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() || printed.trim() != format!("steel {version}") {
        bail!("downloaded binary did not report v{version}");
    }
    Ok(())
}

struct Lock(PathBuf);

impl Lock {
    fn acquire(path: &Path) -> Option<Self> {
        std::fs::create_dir_all(path.parent()?).ok()?;
        for _ in 0..2 {
            match OpenOptions::new().write(true).create_new(true).open(path) {
                Ok(_) => return Some(Self(path.to_path_buf())),
                Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                    let stale = std::fs::metadata(path)
                        .and_then(|meta| meta.modified())
                        .ok()
                        .and_then(|modified| modified.elapsed().ok())
                        .is_some_and(|age| age >= LOCK_STALE_AFTER);
                    if !stale {
                        return None;
                    }
                    let _ = std::fs::remove_file(path);
                }
                Err(_) => return None,
            }
        }
        None
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn state_path(config_dir: &Path) -> PathBuf {
    config_dir.join("update.json")
}

fn read_state(path: &Path) -> State {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|contents| serde_json::from_str(&contents).ok())
        .unwrap_or_default()
}

fn write_state(path: &Path, state: &State) -> anyhow::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension(format!("json.{}.tmp", std::process::id()));
    std::fs::write(&tmp, serde_json::to_vec_pretty(state)?)?;
    std::fs::rename(tmp, path)?;
    Ok(())
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}

#[cfg(test)]
mod tests {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;

    fn updated(from: &str, to: &str) -> Option<Updated> {
        Some(Updated {
            from: from.into(),
            to: to.into(),
        })
    }

    #[test]
    fn first_run_is_due() {
        assert!(check_due(&State::default(), 1_000));
    }

    #[test]
    fn checks_once_a_day() {
        let state = State {
            last_check_unix_ms: Some(1_000),
            ..State::default()
        };
        assert!(!check_due(&state, 1_000 + CHECK_INTERVAL_MS - 1));
        assert!(check_due(&state, 1_000 + CHECK_INTERVAL_MS));
    }

    #[test]
    fn a_check_time_in_the_future_is_due() {
        let state = State {
            last_check_unix_ms: Some(5_000),
            ..State::default()
        };
        assert!(check_due(&state, 1_000));
    }

    #[test]
    fn update_note_shows_once_in_a_terminal() {
        let mut state = State {
            updated: updated("0.5.0", "0.6.0"),
            ..State::default()
        };
        let notes = take_notes(&mut state, "0.6.0", true);
        assert_eq!(notes.len(), 1);
        assert!(notes[0].contains("from v0.5.0 to v0.6.0"));
        assert_eq!(state.updated, None);
        assert!(take_notes(&mut state, "0.6.0", true).is_empty());
    }

    #[test]
    fn update_note_waits_for_a_terminal() {
        let mut state = State {
            updated: updated("0.5.0", "0.6.0"),
            ..State::default()
        };
        assert!(take_notes(&mut state, "0.6.0", false).is_empty());
        assert_eq!(state.updated, updated("0.5.0", "0.6.0"));
    }

    #[test]
    fn update_note_is_dropped_when_the_version_moved_on() {
        let mut state = State {
            updated: updated("0.5.0", "0.6.0"),
            ..State::default()
        };
        assert!(take_notes(&mut state, "0.7.0", true).is_empty());
        assert_eq!(state.updated, None);
    }

    #[test]
    fn major_release_note_shows_once() {
        let mut state = State {
            available: Some("1.0.0".into()),
            ..State::default()
        };
        assert!(take_notes(&mut state, "0.9.0", false).is_empty());
        assert_eq!(state.available.as_deref(), Some("1.0.0"));
        let notes = take_notes(&mut state, "0.9.0", true);
        assert_eq!(notes.len(), 1);
        assert!(notes[0].contains("steel update"));
        assert_eq!(state.available, None);
    }

    #[test]
    fn major_release_note_is_dropped_after_updating() {
        let mut state = State {
            available: Some("1.0.0".into()),
            ..State::default()
        };
        assert!(take_notes(&mut state, "1.0.0", false).is_empty());
        assert_eq!(state.available, None);
    }

    #[test]
    fn state_survives_a_round_trip() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = state_path(dir.path());
        let state = State {
            last_check_unix_ms: Some(42),
            updated: updated("0.5.0", "0.6.0"),
            available: Some("1.0.0".into()),
        };
        write_state(&path, &state).unwrap();
        assert_eq!(read_state(&path), state);
    }

    #[test]
    fn broken_state_reads_as_empty() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = state_path(dir.path());
        std::fs::write(&path, "not json").unwrap();
        assert_eq!(read_state(&path), State::default());
    }

    #[test]
    fn lock_is_exclusive_until_dropped() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("update.lock");
        let lock = Lock::acquire(&path).unwrap();
        assert!(Lock::acquire(&path).is_none());
        drop(lock);
        assert!(Lock::acquire(&path).is_some());
    }

    #[test]
    fn checksum_accepts_the_dist_format() {
        let data = b"hello";
        let file = format!("{} *steel-cli-test.tar.gz\n", sha256_hex(data));
        assert!(verify_checksum(data, file.as_bytes()).is_ok());
        assert!(verify_checksum(b"other", file.as_bytes()).is_err());
    }

    fn release_archive(version: &str) -> Vec<u8> {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::TempDir::new().unwrap();
        let bin_dir = dir.path().join("steel-cli-test-target");
        std::fs::create_dir(&bin_dir).unwrap();
        let bin = bin_dir.join("steel");
        std::fs::write(&bin, format!("#!/bin/sh\necho 'steel {version}'\n")).unwrap();
        std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        let archive = dir.path().join("release.tar.gz");
        let status = std::process::Command::new("tar")
            .arg("-czf")
            .arg(&archive)
            .arg("-C")
            .arg(dir.path())
            .arg("steel-cli-test-target")
            .status()
            .unwrap();
        assert!(status.success());
        std::fs::read(archive).unwrap()
    }

    fn checksum_of(archive: &[u8]) -> String {
        format!("{} *steel-cli-test-target.tar.gz", sha256_hex(archive))
    }

    async fn release_server(archive: Vec<u8>, checksum: String) -> MockServer {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v9.1.0/steel-cli-test-target.tar.gz"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(archive))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/v9.1.0/steel-cli-test-target.tar.gz.sha256"))
            .respond_with(ResponseTemplate::new(200).set_body_string(checksum))
            .mount(&server)
            .await;
        server
    }

    #[tokio::test]
    async fn install_replaces_the_binary() {
        let archive = release_archive("9.1.0");
        let checksum = checksum_of(&archive);
        let server = release_server(archive, checksum).await;
        let dir = tempfile::TempDir::new().unwrap();
        let exe = dir.path().join("steel");
        std::fs::write(&exe, "old").unwrap();

        install(
            &reqwest::Client::new(),
            &server.uri(),
            "test-target",
            "9.1.0",
            &exe,
        )
        .await
        .unwrap();

        let output = std::process::Command::new(&exe)
            .arg("--version")
            .output()
            .unwrap();
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            "steel 9.1.0"
        );
        let leftovers: Vec<_> = std::fs::read_dir(dir.path()).unwrap().collect();
        assert_eq!(leftovers.len(), 1);
    }

    #[tokio::test]
    async fn install_keeps_the_binary_on_a_bad_checksum() {
        let archive = release_archive("9.1.0");
        let server = release_server(archive, format!("{} *x", "0".repeat(64))).await;
        let dir = tempfile::TempDir::new().unwrap();
        let exe = dir.path().join("steel");
        std::fs::write(&exe, "old").unwrap();

        let result = install(
            &reqwest::Client::new(),
            &server.uri(),
            "test-target",
            "9.1.0",
            &exe,
        )
        .await;

        assert!(result.is_err());
        assert_eq!(std::fs::read_to_string(&exe).unwrap(), "old");
    }

    #[tokio::test]
    async fn install_keeps_the_binary_when_the_version_is_wrong() {
        let archive = release_archive("9.0.0");
        let checksum = checksum_of(&archive);
        let server = release_server(archive, checksum).await;
        let dir = tempfile::TempDir::new().unwrap();
        let exe = dir.path().join("steel");
        std::fs::write(&exe, "old").unwrap();

        let result = install(
            &reqwest::Client::new(),
            &server.uri(),
            "test-target",
            "9.1.0",
            &exe,
        )
        .await;

        assert!(result.is_err());
        assert_eq!(std::fs::read_to_string(&exe).unwrap(), "old");
        let leftovers: Vec<_> = std::fs::read_dir(dir.path()).unwrap().collect();
        assert_eq!(leftovers.len(), 1);
    }
}
