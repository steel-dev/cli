use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

// ─── Browser descriptors ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BrowserId {
    Chrome,
    Edge,
    Brave,
    Arc,
    Opera,
    Vivaldi,
}

impl BrowserId {
    pub const fn all() -> &'static [Self] {
        &[
            Self::Chrome,
            Self::Edge,
            Self::Brave,
            Self::Arc,
            Self::Opera,
            Self::Vivaldi,
        ]
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "chrome" => Some(Self::Chrome),
            "edge" => Some(Self::Edge),
            "brave" => Some(Self::Brave),
            "arc" => Some(Self::Arc),
            "opera" => Some(Self::Opera),
            "vivaldi" => Some(Self::Vivaldi),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Chrome => "chrome",
            Self::Edge => "edge",
            Self::Brave => "brave",
            Self::Arc => "arc",
            Self::Opera => "opera",
            Self::Vivaldi => "vivaldi",
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Chrome => "Google Chrome",
            Self::Edge => "Microsoft Edge",
            Self::Brave => "Brave Browser",
            Self::Arc => "Arc",
            Self::Opera => "Opera",
            Self::Vivaldi => "Vivaldi",
        }
    }

    pub const fn descriptor(self) -> BrowserDescriptor {
        match self {
            Self::Chrome => BrowserDescriptor {
                id: self,
                profile_base_dirs: PlatformStr {
                    darwin: Some("Google/Chrome"),
                    win32: Some("Google/Chrome/User Data"),
                    linux: Some("google-chrome"),
                },
                process_names: PlatformStrs {
                    darwin: &["Google Chrome"],
                    win32: &["chrome.exe"],
                    linux: &["chrome", "google-chrome"],
                },
                keychain_service: PlatformStr {
                    darwin: Some("Chrome Safe Storage"),
                    win32: None,
                    linux: Some("chrome"),
                },
            },
            Self::Edge => BrowserDescriptor {
                id: self,
                profile_base_dirs: PlatformStr {
                    darwin: Some("Microsoft Edge"),
                    win32: Some("Microsoft/Edge/User Data"),
                    linux: Some("microsoft-edge"),
                },
                process_names: PlatformStrs {
                    darwin: &["Microsoft Edge"],
                    win32: &["msedge.exe"],
                    linux: &["msedge", "microsoft-edge"],
                },
                keychain_service: PlatformStr {
                    darwin: Some("Microsoft Edge Safe Storage"),
                    win32: None,
                    linux: Some("chromium"),
                },
            },
            Self::Brave => BrowserDescriptor {
                id: self,
                profile_base_dirs: PlatformStr {
                    darwin: Some("BraveSoftware/Brave-Browser"),
                    win32: Some("BraveSoftware/Brave-Browser/User Data"),
                    linux: Some("BraveSoftware/Brave-Browser"),
                },
                process_names: PlatformStrs {
                    darwin: &["Brave Browser"],
                    win32: &["brave.exe"],
                    linux: &["brave", "brave-browser"],
                },
                keychain_service: PlatformStr {
                    darwin: Some("Brave Safe Storage"),
                    win32: None,
                    linux: Some("brave"),
                },
            },
            Self::Arc => BrowserDescriptor {
                id: self,
                profile_base_dirs: PlatformStr {
                    darwin: Some("Arc/User Data"),
                    win32: None,
                    linux: None,
                },
                process_names: PlatformStrs {
                    darwin: &["Arc"],
                    win32: &[],
                    linux: &[],
                },
                keychain_service: PlatformStr {
                    darwin: Some("Arc Safe Storage"),
                    win32: None,
                    linux: None,
                },
            },
            Self::Opera => BrowserDescriptor {
                id: self,
                profile_base_dirs: PlatformStr {
                    darwin: Some("com.operasoftware.Opera"),
                    win32: Some("Opera Software/Opera Stable"),
                    linux: Some("opera"),
                },
                process_names: PlatformStrs {
                    darwin: &["Opera"],
                    win32: &["opera.exe"],
                    linux: &["opera"],
                },
                keychain_service: PlatformStr {
                    darwin: Some("Opera Safe Storage"),
                    win32: None,
                    // Opera on Linux uses Chromium's default keyring name
                    linux: Some("chromium"),
                },
            },
            Self::Vivaldi => BrowserDescriptor {
                id: self,
                profile_base_dirs: PlatformStr {
                    darwin: Some("Vivaldi"),
                    win32: Some("Vivaldi/User Data"),
                    linux: Some("vivaldi"),
                },
                process_names: PlatformStrs {
                    darwin: &["Vivaldi"],
                    win32: &["vivaldi.exe"],
                    linux: &["vivaldi"],
                },
                keychain_service: PlatformStr {
                    darwin: Some("Vivaldi Safe Storage"),
                    win32: None,
                    // Vivaldi on Linux uses Chrome's keyring name
                    linux: Some("chrome"),
                },
            },
        }
    }
}

pub struct PlatformStr {
    pub darwin: Option<&'static str>,
    pub win32: Option<&'static str>,
    pub linux: Option<&'static str>,
}

pub struct PlatformStrs {
    pub darwin: &'static [&'static str],
    pub win32: &'static [&'static str],
    pub linux: &'static [&'static str],
}

pub struct BrowserDescriptor {
    pub id: BrowserId,
    pub profile_base_dirs: PlatformStr,
    pub process_names: PlatformStrs,
    pub keychain_service: PlatformStr,
}

impl BrowserDescriptor {
    /// Resolve the profile base directory for the current OS.
    pub fn profile_base_dir(&self) -> Option<PathBuf> {
        let home = dirs::home_dir()?;

        if cfg!(target_os = "macos") {
            let rel = self.profile_base_dirs.darwin?;
            Some(home.join("Library/Application Support").join(rel))
        } else if cfg!(target_os = "windows") {
            let rel = self.profile_base_dirs.win32?;
            let local_appdata = std::env::var("LOCALAPPDATA")
                .unwrap_or_else(|_| home.join("AppData/Local").to_string_lossy().to_string());
            Some(PathBuf::from(local_appdata).join(rel))
        } else {
            // Linux
            let rel = self.profile_base_dirs.linux?;
            Some(home.join(".config").join(rel))
        }
    }

    const fn process_names_for_current_os(&self) -> &[&str] {
        if cfg!(target_os = "macos") {
            self.process_names.darwin
        } else if cfg!(target_os = "windows") {
            self.process_names.win32
        } else {
            self.process_names.linux
        }
    }

    const fn keychain_service_for_current_os(&self) -> Option<&str> {
        if cfg!(target_os = "macos") {
            self.keychain_service.darwin
        } else if cfg!(target_os = "windows") {
            self.keychain_service.win32
        } else {
            self.keychain_service.linux
        }
    }
}

// ─── Constants ───────────────────────────────────────────────────────────────

const IV: [u8; 16] = [0x20; 16]; // 16 space bytes

const DEFAULT_ENTRIES: &[&str] = &["Cookies", "Local Storage"];

const FULL_ENTRIES: &[&str] = &[
    "Cookies",
    "Local Storage",
    "IndexedDB",
    "Bookmarks",
    "Favicons",
    "History",
    "Web Data",
];

const SKIP_NAMES: &[&str] = &[
    "LOCK",
    "SingletonLock",
    "SingletonCookie",
    "SingletonSocket",
    "Current Session",
    "Current Tabs",
    "Last Session",
    "Last Tabs",
    "Preferences",
    "Secure Preferences",
];
const SKIP_EXTS: &[&str] = &[".log", ".pma"];

// ─── Profile discovery ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BrowserProfile {
    pub dir_name: String,
    pub display_name: String,
    pub email: Option<String>,
    pub browser: BrowserId,
}

pub fn find_browser_profiles(browser: BrowserId) -> Vec<BrowserProfile> {
    let desc = browser.descriptor();
    let Some(base) = desc.profile_base_dir() else {
        return Vec::new();
    };
    if !base.exists() {
        return Vec::new();
    }

    let Ok(entries) = std::fs::read_dir(&base) else {
        return Vec::new();
    };

    let mut profiles = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if !path.join("Cookies").exists() {
            continue;
        }
        let dir_name = entry.file_name().to_string_lossy().to_string();
        let (display_name, email) = get_profile_metadata(&base, &dir_name);
        profiles.push(BrowserProfile {
            dir_name,
            display_name,
            email,
            browser,
        });
    }
    profiles.sort_by(|a, b| a.dir_name.cmp(&b.dir_name));
    profiles
}

pub fn detect_installed_browsers() -> Vec<BrowserId> {
    BrowserId::all()
        .iter()
        .copied()
        .filter(|b| !find_browser_profiles(*b).is_empty())
        .collect()
}

fn get_profile_metadata(base: &Path, dir_name: &str) -> (String, Option<String>) {
    let prefs_path = base.join(dir_name).join("Preferences");
    if let Ok(contents) = std::fs::read_to_string(&prefs_path)
        && let Ok(prefs) = serde_json::from_str::<serde_json::Value>(&contents)
    {
        let account = prefs
            .get("account_info")
            .and_then(|a| a.as_array())
            .and_then(|a| a.first());

        let email = account
            .and_then(|a| a.get("email"))
            .and_then(|e| e.as_str())
            .filter(|e| !e.is_empty())
            .map(|e| e.to_string());

        let get_str = |obj: Option<&serde_json::Value>, key: &str| {
            obj.and_then(|o| o.get(key))
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
        };

        // Prefer given_name (what Chrome shows), then full_name, then profile.name
        let name = get_str(account, "given_name")
            .or_else(|| get_str(account, "full_name"))
            .or_else(|| get_str(prefs.get("profile"), "name").filter(|n| n != dir_name))
            .unwrap_or_else(|| dir_name.to_string());

        return (name, email);
    }
    (dir_name.to_string(), None)
}

pub fn is_browser_running(browser: BrowserId) -> bool {
    let desc = browser.descriptor();
    let names = desc.process_names_for_current_os();
    if names.is_empty() {
        return false;
    }

    if cfg!(target_os = "windows") {
        names.iter().any(|name| {
            std::process::Command::new("tasklist")
                .args(["/FI", &format!("IMAGENAME eq {name}"), "/NH"])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).contains(name))
                .unwrap_or(false)
        })
    } else {
        // macOS and Linux: pgrep
        names.iter().any(|name| {
            std::process::Command::new("pgrep")
                .args(["-x", name])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        })
    }
}

// ─── Key providers ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CryptoAlgorithm {
    Aes128Cbc,
    Aes256Gcm,
}

struct KeyProvider {
    source_key: [u8; 32], // up to 32 bytes (16 for CBC, 32 for GCM)
    source_key_len: usize,
    source_algorithm: CryptoAlgorithm,
}

fn get_key_provider(browser: BrowserId) -> Result<KeyProvider> {
    if cfg!(target_os = "macos") {
        get_macos_key_provider(browser)
    } else if cfg!(target_os = "windows") {
        get_windows_key_provider(browser)
    } else {
        get_linux_key_provider(browser)
    }
}

fn get_macos_key_provider(browser: BrowserId) -> Result<KeyProvider> {
    let desc = browser.descriptor();
    let service = desc.keychain_service_for_current_os().ok_or_else(|| {
        anyhow::anyhow!(
            "{} has no macOS Keychain service configured.",
            browser.display_name()
        )
    })?;

    // Chromium stores the keychain entry with account = browser name (e.g. "Chrome")
    // and service = "Chrome Safe Storage". Specifying both avoids ambiguity.
    let account = service.strip_suffix(" Safe Storage").unwrap_or(service);

    let output = std::process::Command::new("security")
        .args(["find-generic-password", "-w", "-a", account, "-s", service])
        .output()
        .context("Failed to run `security` command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "Failed to retrieve {} password from Keychain: {}",
            browser.display_name(),
            stderr.trim()
        );
    }

    let passphrase = String::from_utf8(output.stdout)?.trim().to_string();
    let key_16 = derive_key(&passphrase, 1003);
    let mut source_key = [0u8; 32];
    source_key[..16].copy_from_slice(&key_16);

    Ok(KeyProvider {
        source_key,
        source_key_len: 16,
        source_algorithm: CryptoAlgorithm::Aes128Cbc,
    })
}

/// KWallet keyring name for a browser, matching Chromium's convention.
/// Ref: https://chromium.googlesource.com/chromium/src/+/refs/heads/main/components/os_crypt/sync/key_storage_kwallet.cc
fn kwallet_keyring_name(browser: BrowserId) -> &'static str {
    match browser {
        BrowserId::Chrome => "Chrome",
        BrowserId::Brave => "Brave",
        BrowserId::Edge | BrowserId::Opera => "Chromium",
        BrowserId::Vivaldi => "Chrome",
        BrowserId::Arc => "Arc",
    }
}

fn try_kwallet_password(browser: BrowserId) -> Option<String> {
    let name = kwallet_keyring_name(browser);
    let output = std::process::Command::new("kwallet-query")
        .args([
            "--read-password",
            &format!("{name} Safe Storage"),
            "--folder",
            &format!("{name} Keys"),
            "kdewallet",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let pw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if pw.is_empty() || pw.to_lowercase().starts_with("failed to read") {
        return None;
    }
    Some(pw)
}

fn get_linux_key_provider(browser: BrowserId) -> Result<KeyProvider> {
    let desc = browser.descriptor();
    let app_name = desc.keychain_service_for_current_os();

    // Try GNOME/freedesktop secret service first, then KWallet, then "peanuts" fallback.
    let passphrase = if let Some(app) = app_name {
        std::process::Command::new("secret-tool")
            .args(["lookup", "application", app])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| try_kwallet_password(browser))
            .unwrap_or_else(|| "peanuts".to_string())
    } else {
        "peanuts".to_string()
    };

    let key_16 = derive_key(&passphrase, 1);
    let mut source_key = [0u8; 32];
    source_key[..16].copy_from_slice(&key_16);

    Ok(KeyProvider {
        source_key,
        source_key_len: 16,
        source_algorithm: CryptoAlgorithm::Aes128Cbc,
    })
}

fn get_windows_key_provider(browser: BrowserId) -> Result<KeyProvider> {
    let desc = browser.descriptor();
    let base_dir = desc.profile_base_dir().ok_or_else(|| {
        anyhow::anyhow!("{} is not supported on Windows.", browser.display_name())
    })?;

    let local_state_path = base_dir.join("Local State");
    let local_state_str = std::fs::read_to_string(&local_state_path).with_context(|| {
        format!(
            "Could not read Local State at {}",
            local_state_path.display()
        )
    })?;

    let local_state: serde_json::Value = serde_json::from_str(&local_state_str)?;
    let encrypted_key_b64 = local_state
        .get("os_crypt")
        .and_then(|o| o.get("encrypted_key"))
        .and_then(|k| k.as_str())
        .ok_or_else(|| anyhow::anyhow!("Local State missing os_crypt.encrypted_key"))?;

    use base64::Engine;
    let encrypted_key_raw = base64::engine::general_purpose::STANDARD
        .decode(encrypted_key_b64)
        .context("Failed to base64-decode encrypted key")?;

    // Strip "DPAPI" prefix (5 bytes)
    if encrypted_key_raw.len() < 5 || &encrypted_key_raw[..5] != b"DPAPI" {
        anyhow::bail!("Unexpected encrypted key prefix (expected DPAPI)");
    }

    let dpapi_blob = base64::engine::general_purpose::STANDARD.encode(&encrypted_key_raw[5..]);

    // Use PowerShell to call DPAPI
    let ps_script = format!(
        "Add-Type -AssemblyName System.Security; \
         $blob = [Convert]::FromBase64String('{}'); \
         $plain = [Security.Cryptography.ProtectedData]::Unprotect($blob, $null, [Security.Cryptography.DataProtectionScope]::CurrentUser); \
         [Convert]::ToBase64String($plain)",
        dpapi_blob
    );

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_script])
        .output()
        .context("Failed to run PowerShell for DPAPI decryption")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("PowerShell DPAPI decryption failed: {}", stderr.trim());
    }

    let result_b64 = String::from_utf8(output.stdout)?.trim().to_string();
    let key_bytes = base64::engine::general_purpose::STANDARD
        .decode(&result_b64)
        .context("Failed to decode DPAPI result")?;

    if key_bytes.len() != 32 {
        anyhow::bail!(
            "DPAPI decrypted key is {} bytes, expected 32",
            key_bytes.len()
        );
    }

    let mut source_key = [0u8; 32];
    source_key.copy_from_slice(&key_bytes);

    Ok(KeyProvider {
        source_key,
        source_key_len: 32,
        source_algorithm: CryptoAlgorithm::Aes256Gcm,
    })
}

// ─── Crypto ──────────────────────────────────────────────────────────────────

fn derive_key(passphrase: &str, iterations: u32) -> [u8; 16] {
    use hmac::Hmac;
    use sha1::Sha1;

    let mut key = [0u8; 16];
    pbkdf2::pbkdf2::<Hmac<Sha1>>(passphrase.as_bytes(), b"saltysalt", iterations, &mut key)
        .expect("HMAC can be initialized with any key length");
    key
}

fn derive_target_key() -> [u8; 16] {
    derive_key("peanuts", 1)
}

fn decrypt_cookie(
    encrypted_value: &[u8],
    key: &[u8],
    host_key: &str,
    meta_version: i64,
    algorithm: CryptoAlgorithm,
) -> Option<String> {
    if encrypted_value.len() < 3 {
        return None;
    }
    let prefix = &encrypted_value[..3];
    if prefix != b"v10" && prefix != b"v11" && prefix != b"v20" {
        return None;
    }

    let payload = &encrypted_value[3..];

    match algorithm {
        CryptoAlgorithm::Aes128Cbc => {
            use aes::Aes128;
            use cbc::cipher::{BlockDecryptMut, KeyIvInit, block_padding::Pkcs7};

            type Aes128CbcDec = cbc::Decryptor<Aes128>;

            let mut buf = payload.to_vec();
            let plaintext = Aes128CbcDec::new(key[..16].into(), &IV.into())
                .decrypt_padded_mut::<Pkcs7>(&mut buf)
                .ok()?;

            strip_domain_hash(plaintext, host_key, meta_version)
        }
        CryptoAlgorithm::Aes256Gcm => {
            use aes_gcm::aead::Aead;
            use aes_gcm::{Aes256Gcm, KeyInit, Nonce};

            const NONCE_LEN: usize = 12;
            const TAG_LEN: usize = 16;
            if payload.len() < NONCE_LEN + TAG_LEN {
                return None;
            }

            let nonce_bytes = &payload[..NONCE_LEN];
            let ciphertext_and_tag = &payload[NONCE_LEN..];

            let cipher = Aes256Gcm::new(key[..32].into());
            let nonce = Nonce::from_slice(nonce_bytes);
            let plaintext = cipher.decrypt(nonce, ciphertext_and_tag).ok()?;

            strip_domain_hash(&plaintext, host_key, meta_version)
        }
    }
}

fn strip_domain_hash(plaintext: &[u8], host_key: &str, meta_version: i64) -> Option<String> {
    if meta_version >= 24 && plaintext.len() >= 32 {
        use sha2::{Digest, Sha256};
        let expected_hash = Sha256::digest(host_key.as_bytes());
        if expected_hash.as_slice() != &plaintext[..32] {
            return None; // hash mismatch — decryption failure
        }
        return Some(String::from_utf8_lossy(&plaintext[32..]).to_string());
    }
    Some(String::from_utf8_lossy(plaintext).to_string())
}

fn encrypt_cookie(value: &str, key: &[u8; 16], host_key: &str, meta_version: i64) -> Vec<u8> {
    use aes::Aes128;
    use cbc::cipher::{BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};

    let mut plaintext = Vec::new();

    if meta_version >= 24 {
        use sha2::{Digest, Sha256};
        let domain_hash = Sha256::digest(host_key.as_bytes());
        plaintext.extend_from_slice(&domain_hash);
    }
    plaintext.extend_from_slice(value.as_bytes());

    type Aes128CbcEnc = cbc::Encryptor<Aes128>;

    let block_size = 16;
    let padded_len = ((plaintext.len() / block_size) + 1) * block_size;
    let mut buf = vec![0u8; padded_len];
    buf[..plaintext.len()].copy_from_slice(&plaintext);

    let encrypted = Aes128CbcEnc::new(key.into(), &IV.into())
        .encrypt_padded_mut::<Pkcs7>(&mut buf, plaintext.len())
        .expect("buffer is large enough for PKCS7 padding");

    let mut result = Vec::with_capacity(3 + encrypted.len());
    result.extend_from_slice(b"v10");
    result.extend_from_slice(encrypted);
    result
}

// ─── Cookie re-encryption ────────────────────────────────────────────────────

struct ReencryptResult {
    buffer: Vec<u8>,
    converted: u64,
    skipped: u64,
}

fn reencrypt_cookies_db(original_path: &Path, provider: &KeyProvider) -> Result<ReencryptResult> {
    let peanuts_key = derive_target_key();

    let tmp = tempfile::NamedTempFile::new().context("Failed to create temp file for cookies")?;
    let tmp_path = tmp.path().to_path_buf();
    std::fs::copy(original_path, &tmp_path)?;

    let conn = rusqlite::Connection::open(&tmp_path)?;

    let meta_version: i64 = conn
        .query_row("SELECT value FROM meta WHERE key='version'", [], |row| {
            row.get(0)
        })
        .unwrap_or(0);

    let mut stmt = conn.prepare(
        "SELECT rowid, host_key, encrypted_value FROM cookies WHERE length(encrypted_value) > 3",
    )?;

    let rows: Vec<(i64, String, Vec<u8>)> = stmt
        .query_map([], |row| {
            let rowid: i64 = row.get(0)?;
            let host_key: String = row.get(1)?;
            // Some browsers (e.g. Brave) may store encrypted_value as TEXT
            // instead of BLOB, so handle both types.
            let encrypted_value: Vec<u8> = match row.get_ref(2)? {
                rusqlite::types::ValueRef::Blob(b) => b.to_vec(),
                rusqlite::types::ValueRef::Text(t) => t.to_vec(),
                _ => return Ok((rowid, host_key, Vec::new())),
            };
            Ok((rowid, host_key, encrypted_value))
        })?
        .collect::<std::result::Result<_, _>>()?;

    let mut converted: u64 = 0;
    let mut skipped: u64 = 0;
    let tx = conn.unchecked_transaction()?;
    {
        let mut update = tx.prepare("UPDATE cookies SET encrypted_value = ? WHERE rowid = ?")?;

        // On Linux, Chromium may have encrypted some cookies with an empty password
        // due to a bug. Try the empty-password key as a fallback.
        // Ref: https://chromium.googlesource.com/chromium/src/+/bbd54702284caca1f92d656fdcadf2ccca6f4165
        let empty_key_fallback = if provider.source_algorithm == CryptoAlgorithm::Aes128Cbc {
            let k = derive_key("", 1);
            let mut buf = [0u8; 32];
            buf[..16].copy_from_slice(&k);
            Some((buf, 16usize))
        } else {
            None
        };

        for (rowid, host_key, encrypted_value) in &rows {
            // Old Chromium versions (macOS/Windows) stored cookies as plaintext
            // without a version prefix. Treat non-v10/v11/v20 data as plaintext.
            let is_known_prefix = encrypted_value.len() >= 3
                && matches!(&encrypted_value[..3], b"v10" | b"v11" | b"v20");

            let plaintext = if is_known_prefix {
                decrypt_cookie(
                    encrypted_value,
                    &provider.source_key[..provider.source_key_len],
                    host_key,
                    meta_version,
                    provider.source_algorithm,
                )
                .or_else(|| {
                    let (key, len) = empty_key_fallback.as_ref()?;
                    decrypt_cookie(
                        encrypted_value,
                        &key[..*len],
                        host_key,
                        meta_version,
                        provider.source_algorithm,
                    )
                })
            } else {
                String::from_utf8(encrypted_value.clone()).ok()
            };

            let Some(plaintext) = plaintext else {
                skipped += 1;
                continue;
            };

            // Always re-encrypt with AES-128-CBC (peanuts key) for Steel
            let reencrypted = encrypt_cookie(&plaintext, &peanuts_key, host_key, meta_version);
            update.execute(rusqlite::params![reencrypted, rowid])?;
            converted += 1;
        }
    }
    tx.commit()?;

    drop(stmt);
    conn.close().map_err(|(_, e)| e)?;

    let buffer = std::fs::read(&tmp_path)?;
    // tmp (NamedTempFile) is dropped here, auto-deleting the file
    Ok(ReencryptResult {
        buffer,
        converted,
        skipped,
    })
}

// ─── File collection ─────────────────────────────────────────────────────────

fn collect_files(dir_path: &Path, base_dir: &Path) -> HashMap<String, Vec<u8>> {
    let mut files = HashMap::new();
    collect_files_recursive(dir_path, base_dir, &mut files);
    files
}

fn collect_files_recursive(dir_path: &Path, base_dir: &Path, files: &mut HashMap<String, Vec<u8>>) {
    let Ok(entries) = std::fs::read_dir(dir_path) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if SKIP_NAMES.contains(&name_str.as_ref()) {
            continue;
        }
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let dot_ext = format!(".{ext}");
            if SKIP_EXTS.contains(&dot_ext.as_str()) {
                continue;
            }
        }

        if path.is_dir() {
            collect_files_recursive(&path, base_dir, files);
        } else if path.is_file()
            && let Ok(rel) = path.strip_prefix(base_dir)
            && let Ok(data) = std::fs::read(&path)
        {
            files.insert(rel.to_string_lossy().to_string(), data);
        }
    }
}

// ─── Minimal Preferences ─────────────────────────────────────────────────────

/// Generate a minimal Preferences file for cross-environment portability.
fn minimal_preferences() -> Vec<u8> {
    let prefs = serde_json::json!({
        "profile": {
            "exit_type": "Normal",
            "exited_cleanly": true
        }
    });
    serde_json::to_vec_pretty(&prefs).expect("static JSON serialization")
}

// ─── Package ─────────────────────────────────────────────────────────────────

pub struct PackageResult {
    pub zip_buffer: Vec<u8>,
    pub cookies_reencrypted: u64,
    pub cookies_skipped: u64,
}

/// Package a browser profile for upload to Steel.
pub fn package_profile(
    browser: BrowserId,
    profile_dir_name: &str,
    full: bool,
    on_progress: &dyn Fn(&str),
) -> Result<PackageResult> {
    let desc = browser.descriptor();
    let base_dir = desc.profile_base_dir().ok_or_else(|| {
        anyhow::anyhow!(
            "{} is not supported on this platform.",
            browser.display_name()
        )
    })?;

    let profile_dir = base_dir.join(profile_dir_name);

    if !profile_dir.join("Cookies").exists() {
        anyhow::bail!(
            "{} profile \"{}\" not found at {}",
            browser.display_name(),
            profile_dir_name,
            profile_dir.display()
        );
    }

    on_progress("Reading encryption key...");
    let provider = get_key_provider(browser)?;

    let mut zip_files: Vec<(String, Vec<u8>)> = Vec::new();
    let mut cookies_reencrypted: u64 = 0;
    let mut cookies_skipped: u64 = 0;

    let entries = if full { FULL_ENTRIES } else { DEFAULT_ENTRIES };

    for entry_name in entries {
        let full_path = profile_dir.join(entry_name);
        if !full_path.exists() {
            continue;
        }

        let meta = std::fs::metadata(&full_path)?;

        if meta.is_file() {
            if *entry_name == "Cookies" {
                on_progress("Re-encrypting Cookies...");
                let result = reencrypt_cookies_db(&full_path, &provider)?;
                zip_files.push((format!("Default/{entry_name}"), result.buffer));
                cookies_reencrypted = result.converted;
                cookies_skipped = result.skipped;
            } else {
                let data = std::fs::read(&full_path)?;
                zip_files.push((format!("Default/{entry_name}"), data));
            }
        } else if meta.is_dir() {
            on_progress(&format!("Collecting {entry_name}/..."));
            let dir_files = collect_files(&full_path, &full_path);
            for (rel_path, data) in dir_files {
                zip_files.push((format!("Default/{entry_name}/{rel_path}"), data));
            }
        }
    }

    zip_files.push(("Default/Preferences".into(), minimal_preferences()));

    let total_bytes: usize = zip_files.iter().map(|(_, d)| d.len()).sum();
    let total_mb = total_bytes as f64 / 1024.0 / 1024.0;
    if total_mb >= 100.0 {
        on_progress(&format!(
            "Zipping {:.0} MB (this may take a moment)...",
            total_mb
        ));
    } else {
        on_progress(&format!("Zipping {:.0} MB...", total_mb));
    }
    let mut cursor = std::io::Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut cursor);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        for (name, data) in &zip_files {
            zip.start_file(name, options)?;
            zip.write_all(data)?;
        }
        zip.finish()?;
    }

    Ok(PackageResult {
        zip_buffer: cursor.into_inner(),
        cookies_reencrypted,
        cookies_skipped,
    })
}

// ─── Steel API ───────────────────────────────────────────────────────────────

pub async fn upload_profile_to_steel(
    zip_buffer: Vec<u8>,
    api_key: &str,
    api_base: &str,
) -> Result<String> {
    let part = reqwest::multipart::Part::bytes(zip_buffer)
        .file_name("userDataDir.zip")
        .mime_str("application/zip")?;
    let form = reqwest::multipart::Form::new().part("userDataDir", part);

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{api_base}/profiles"))
        .header("Steel-Api-Key", api_key)
        .multipart(form)
        .send()
        .await?;

    let status = res.status();
    let body: serde_json::Value = res.json().await?;

    if !status.is_success() {
        let msg = body
            .get("message")
            .and_then(|m| m.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| body.to_string());
        anyhow::bail!("Profile upload failed ({status}): {msg}");
    }

    body.get("id")
        .and_then(|id| id.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Profile upload response missing id"))
}

pub async fn update_profile_on_steel(
    profile_id: &str,
    zip_buffer: Vec<u8>,
    api_key: &str,
    api_base: &str,
) -> Result<()> {
    let part = reqwest::multipart::Part::bytes(zip_buffer)
        .file_name("userDataDir.zip")
        .mime_str("application/zip")?;
    let form = reqwest::multipart::Form::new().part("userDataDir", part);

    let client = reqwest::Client::new();
    let res = client
        .patch(format!("{api_base}/profiles/{profile_id}"))
        .header("Steel-Api-Key", api_key)
        .multipart(form)
        .send()
        .await?;

    if !res.status().is_success() {
        let status = res.status();
        let body: serde_json::Value = res.json().await?;
        let msg = body
            .get("message")
            .and_then(|m| m.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| body.to_string());
        anyhow::bail!("Profile update failed ({status}): {msg}");
    }

    Ok(())
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_key_peanuts() {
        let key = derive_key("peanuts", 1);
        assert_eq!(key.len(), 16);
        assert_eq!(derive_key("peanuts", 1), key);
    }

    #[test]
    fn derive_key_different_iterations() {
        let k1 = derive_key("peanuts", 1);
        let k2 = derive_key("peanuts", 1003);
        assert_ne!(k1, k2);
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = derive_key("testkey", 1);
        let value = "hello world";
        let host_key = ".example.com";

        let encrypted = encrypt_cookie(value, &key, host_key, 20);
        assert_eq!(&encrypted[..3], b"v10");
        let decrypted = decrypt_cookie(&encrypted, &key, host_key, 20, CryptoAlgorithm::Aes128Cbc);
        assert_eq!(decrypted.as_deref(), Some(value));
    }

    #[test]
    fn encrypt_decrypt_roundtrip_v24() {
        let key = derive_key("testkey", 1);
        let value = "secret cookie";
        let host_key = ".google.com";

        let encrypted = encrypt_cookie(value, &key, host_key, 24);
        let decrypted = decrypt_cookie(&encrypted, &key, host_key, 24, CryptoAlgorithm::Aes128Cbc);
        assert_eq!(decrypted.as_deref(), Some(value));
    }

    #[test]
    fn decrypt_wrong_key_returns_none() {
        let key1 = derive_key("key1", 1);
        let key2 = derive_key("key2", 1);
        let encrypted = encrypt_cookie("data", &key1, ".test.com", 20);
        let result = decrypt_cookie(
            &encrypted,
            &key2,
            ".test.com",
            20,
            CryptoAlgorithm::Aes128Cbc,
        );
        assert!(result.is_none());
    }

    #[test]
    fn decrypt_no_v10_prefix_returns_none() {
        let key = derive_key("k", 1);
        assert!(
            decrypt_cookie(b"xyz_data", &key, "host", 20, CryptoAlgorithm::Aes128Cbc).is_none()
        );
    }

    #[test]
    fn decrypt_short_data_returns_none() {
        let key = derive_key("k", 1);
        assert!(decrypt_cookie(b"v1", &key, "host", 20, CryptoAlgorithm::Aes128Cbc).is_none());
    }

    // ── yt-dlp test vectors ──────────────────────────────────────────────────

    #[test]
    fn ytdlp_linux_v10_peanuts() {
        let key = derive_key("peanuts", 1);
        let encrypted_value: &[u8] = b"v10\xccW%\xcd\xe6\xe6\x9fM\x22\x20\xa7\xb0\xca\xe4\x07\xd6";
        let result =
            decrypt_cookie(encrypted_value, &key, "", 0, CryptoAlgorithm::Aes128Cbc).unwrap();
        assert_eq!(result, "USD");
    }

    #[test]
    fn ytdlp_linux_v11_empty_password() {
        // v11 = same AES-128-CBC, but keyring password (here: empty string fallback)
        let key = derive_key("", 1);
        let encrypted_value: &[u8] = b"v11#\x81\x10>`w\x8f)\xc0\xb2\xc1\r\xf4\x1al\xdd\x93\xfd\xf8\xf8N\xf2\xa9\x83\xf1\xe9o\x0elVQd";
        let result =
            decrypt_cookie(encrypted_value, &key, "", 0, CryptoAlgorithm::Aes128Cbc).unwrap();
        assert_eq!(result, "tz=Europe.London");
    }

    #[test]
    fn ytdlp_macos_v10() {
        let key = derive_key("6eIDUdtKAacvlHwBVwvg/Q==", 1003);
        let encrypted_value: &[u8] = b"v10\xb3\xbe\xad\xa1[\x9fC\xa1\x98\xe0\x9a\x01\xd9\xcf\xbfc";
        let result =
            decrypt_cookie(encrypted_value, &key, "", 0, CryptoAlgorithm::Aes128Cbc).unwrap();
        assert_eq!(result, "2021-06-01-22");
    }

    #[test]
    fn ytdlp_windows_v10_aes256gcm() {
        let key: &[u8] = b"\x59\xef\xad\xad\xee\x72\x70\xf0\x59\xe6\x9b\x12\xc2\x3c\x7a\x16\x5d\x0a\xbb\xb8\xcb\xd7\x9b\x41\xc3\x14\x65\x99\x7b\xd6\xf4\x26";
        let encrypted_value: &[u8] = b"v10\x54\xb8\xf3\xb8\x01\xa7\x54\x74\x63\x56\xfc\x88\xb8\xb8\xef\x05\xb5\xfd\x18\xc9\x30\x00\x39\xab\xb1\x89\x33\x85\x29\x87\xe1\xa9\x2d\xa3\xad\x3d";
        let result =
            decrypt_cookie(encrypted_value, key, "", 0, CryptoAlgorithm::Aes256Gcm).unwrap();
        assert_eq!(result, "32101439");
    }

    #[test]
    fn ytdlp_derive_key_peanuts() {
        // Cross-check: yt-dlp's LinuxChromeCookieDecryptor.derive_key(b'abc')
        // == b'7\xa1\xec\xd4m\xfcA\xc7\xb19Z\xd0\x19\xdcM\x17'
        let abc_key_linux = derive_key("abc", 1);
        assert_eq!(
            abc_key_linux,
            [
                0x37, 0xa1, 0xec, 0xd4, 0x6d, 0xfc, 0x41, 0xc7, 0xb1, 0x39, 0x5a, 0xd0, 0x19, 0xdc,
                0x4d, 0x17
            ]
        );
        // Cross-check: yt-dlp's MacChromeCookieDecryptor.derive_key(b'abc')
        // == b'Y\xe2\xc0\xd0P\xf6\xf4\xe1l\xc1\x8cQ\xcb|\xcdY'
        let abc_key_mac = derive_key("abc", 1003);
        assert_eq!(
            abc_key_mac,
            [
                0x59, 0xe2, 0xc0, 0xd0, 0x50, 0xf6, 0xf4, 0xe1, 0x6c, 0xc1, 0x8c, 0x51, 0xcb, 0x7c,
                0xcd, 0x59
            ]
        );
    }

    #[test]
    fn collect_files_skips_lock_and_log() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();

        std::fs::write(dir.join("good.txt"), "data").unwrap();
        std::fs::write(dir.join("LOCK"), "lock").unwrap();
        std::fs::write(dir.join("SingletonLock"), "lock").unwrap();
        std::fs::write(dir.join("something.log"), "log").unwrap();
        std::fs::write(dir.join("data.pma"), "pma").unwrap();

        let files = collect_files(dir, dir);
        assert_eq!(files.len(), 1);
        assert!(files.contains_key("good.txt"));
    }

    #[test]
    fn collect_files_recursive_subdir() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        let sub = dir.join("sub");
        std::fs::create_dir(&sub).unwrap();

        std::fs::write(dir.join("a.txt"), "a").unwrap();
        std::fs::write(sub.join("b.txt"), "b").unwrap();

        let files = collect_files(dir, dir);
        assert_eq!(files.len(), 2);
        assert!(files.contains_key("a.txt"));
        assert!(files.contains_key("sub/b.txt"));
    }

    #[test]
    fn browser_id_roundtrip() {
        for id in BrowserId::all() {
            assert_eq!(BrowserId::from_str(id.as_str()), Some(*id));
        }
    }

    #[test]
    fn browser_id_from_str_unknown() {
        assert!(BrowserId::from_str("firefox").is_none());
    }

    #[test]
    fn all_browsers_have_descriptors() {
        for id in BrowserId::all() {
            let desc = id.descriptor();
            assert_eq!(desc.id, *id);
        }
    }

    #[test]
    fn minimal_preferences_has_clean_exit() {
        let data = minimal_preferences();
        let prefs: serde_json::Value = serde_json::from_slice(&data).unwrap();

        assert_eq!(prefs["profile"]["exit_type"], "Normal");
        assert_eq!(prefs["profile"]["exited_cleanly"], true);

        // Should only contain profile key — no extensions, themes, etc.
        let obj = prefs.as_object().unwrap();
        assert_eq!(obj.len(), 1);
        assert!(obj.contains_key("profile"));
    }

    #[test]
    fn collect_files_skips_preferences() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();

        std::fs::write(dir.join("good.txt"), "data").unwrap();
        std::fs::write(dir.join("Preferences"), "{}").unwrap();
        std::fs::write(dir.join("Secure Preferences"), "{}").unwrap();

        let files = collect_files(dir, dir);
        assert_eq!(files.len(), 1);
        assert!(files.contains_key("good.txt"));
    }

    #[test]
    fn collect_files_skips_session_files() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();

        std::fs::write(dir.join("good.txt"), "data").unwrap();
        std::fs::write(dir.join("Current Session"), "s").unwrap();
        std::fs::write(dir.join("Current Tabs"), "t").unwrap();
        std::fs::write(dir.join("Last Session"), "s").unwrap();
        std::fs::write(dir.join("Last Tabs"), "t").unwrap();

        let files = collect_files(dir, dir);
        assert_eq!(files.len(), 1);
        assert!(files.contains_key("good.txt"));
    }
}
