use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

// ─── Browser descriptors ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserId {
    Chrome,
    Edge,
    Brave,
    Arc,
    Opera,
    Vivaldi,
}

impl BrowserId {
    pub fn all() -> &'static [BrowserId] {
        &[
            BrowserId::Chrome,
            BrowserId::Edge,
            BrowserId::Brave,
            BrowserId::Arc,
            BrowserId::Opera,
            BrowserId::Vivaldi,
        ]
    }

    pub fn from_str(s: &str) -> Option<BrowserId> {
        match s.to_lowercase().as_str() {
            "chrome" => Some(BrowserId::Chrome),
            "edge" => Some(BrowserId::Edge),
            "brave" => Some(BrowserId::Brave),
            "arc" => Some(BrowserId::Arc),
            "opera" => Some(BrowserId::Opera),
            "vivaldi" => Some(BrowserId::Vivaldi),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            BrowserId::Chrome => "chrome",
            BrowserId::Edge => "edge",
            BrowserId::Brave => "brave",
            BrowserId::Arc => "arc",
            BrowserId::Opera => "opera",
            BrowserId::Vivaldi => "vivaldi",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            BrowserId::Chrome => "Google Chrome",
            BrowserId::Edge => "Microsoft Edge",
            BrowserId::Brave => "Brave Browser",
            BrowserId::Arc => "Arc",
            BrowserId::Opera => "Opera",
            BrowserId::Vivaldi => "Vivaldi",
        }
    }

    pub fn descriptor(self) -> BrowserDescriptor {
        match self {
            BrowserId::Chrome => BrowserDescriptor {
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
            BrowserId::Edge => BrowserDescriptor {
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
            BrowserId::Brave => BrowserDescriptor {
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
            BrowserId::Arc => BrowserDescriptor {
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
            BrowserId::Opera => BrowserDescriptor {
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
                    linux: Some("opera"),
                },
            },
            BrowserId::Vivaldi => BrowserDescriptor {
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
                    linux: Some("vivaldi"),
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

    fn process_names_for_current_os(&self) -> &[&str] {
        if cfg!(target_os = "macos") {
            self.process_names.darwin
        } else if cfg!(target_os = "windows") {
            self.process_names.win32
        } else {
            self.process_names.linux
        }
    }

    fn keychain_service_for_current_os(&self) -> Option<&str> {
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

const INCLUDE_ENTRIES: &[&str] = &[
    "Cookies",
    "Local Storage",
    "IndexedDB",
    "Preferences",
    "Bookmarks",
    "Favicons",
    "History",
    "Web Data",
];

const SKIP_NAMES: &[&str] = &["LOCK", "SingletonLock", "SingletonCookie", "SingletonSocket"];
const SKIP_EXTS: &[&str] = &[".log", ".pma"];

// ─── Profile discovery ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BrowserProfile {
    pub dir_name: String,
    pub display_name: String,
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
        let display_name = get_profile_display_name(&base, &dir_name);
        profiles.push(BrowserProfile {
            dir_name,
            display_name,
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

fn get_profile_display_name(base: &Path, dir_name: &str) -> String {
    let prefs_path = base.join(dir_name).join("Preferences");
    if let Ok(contents) = std::fs::read_to_string(&prefs_path) {
        if let Ok(prefs) = serde_json::from_str::<serde_json::Value>(&contents) {
            if let Some(name) = prefs
                .get("profile")
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
            {
                if !name.is_empty() && name != dir_name {
                    return name.to_string();
                }
            }
            if let Some(full_name) = prefs
                .get("account_info")
                .and_then(|a| a.as_array())
                .and_then(|a| a.first())
                .and_then(|a| a.get("full_name"))
                .and_then(|n| n.as_str())
            {
                if !full_name.is_empty() {
                    return full_name.to_string();
                }
            }
        }
    }
    dir_name.to_string()
}

pub fn is_browser_running(browser: BrowserId) -> bool {
    let desc = browser.descriptor();
    let names = desc.process_names_for_current_os();
    if names.is_empty() {
        return false;
    }

    if cfg!(target_os = "windows") {
        for name in names {
            let ok = std::process::Command::new("tasklist")
                .args(["/FI", &format!("IMAGENAME eq {name}"), "/NH"])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).contains(name))
                .unwrap_or(false);
            if ok {
                return true;
            }
        }
        false
    } else {
        // macOS and Linux: pgrep
        for name in names {
            let ok = std::process::Command::new("pgrep")
                .args(["-x", name])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if ok {
                return true;
            }
        }
        false
    }
}

// ─── Key providers ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
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
    let service = desc
        .keychain_service_for_current_os()
        .ok_or_else(|| anyhow::anyhow!("{} has no macOS Keychain service configured.", browser.display_name()))?;

    let output = std::process::Command::new("security")
        .args(["find-generic-password", "-w", "-s", service])
        .output()
        .context("Failed to run `security` command")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to retrieve {} password from Keychain. You may need to grant access.",
            browser.display_name()
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

fn get_linux_key_provider(browser: BrowserId) -> Result<KeyProvider> {
    let desc = browser.descriptor();
    let app_name = desc.keychain_service_for_current_os();

    let passphrase = if let Some(app) = app_name {
        std::process::Command::new("secret-tool")
            .args(["lookup", "application", app])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .filter(|s| !s.is_empty())
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
    let base_dir = desc
        .profile_base_dir()
        .ok_or_else(|| anyhow::anyhow!("{} is not supported on Windows.", browser.display_name()))?;

    let local_state_path = base_dir.join("Local State");
    let local_state_str = std::fs::read_to_string(&local_state_path)
        .with_context(|| format!("Could not read Local State at {}", local_state_path.display()))?;

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
        anyhow::bail!("PowerShell DPAPI decryption failed");
    }

    let result_b64 = String::from_utf8(output.stdout)?.trim().to_string();
    let key_bytes = base64::engine::general_purpose::STANDARD
        .decode(&result_b64)
        .context("Failed to decode DPAPI result")?;

    if key_bytes.len() != 32 {
        anyhow::bail!("DPAPI decrypted key is {} bytes, expected 32", key_bytes.len());
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
    if prefix != b"v10" && prefix != b"v20" {
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
            use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
            use aes_gcm::aead::Aead;

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
        if expected_hash.as_slice() == &plaintext[..32] {
            return Some(String::from_utf8_lossy(&plaintext[32..]).to_string());
        }
    }
    Some(String::from_utf8_lossy(plaintext).to_string())
}

fn encrypt_cookie(
    value: &str,
    key: &[u8; 16],
    host_key: &str,
    meta_version: i64,
) -> Vec<u8> {
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

fn reencrypt_cookies_db(
    original_path: &Path,
    provider: &KeyProvider,
) -> Result<(Vec<u8>, u64)> {
    let peanuts_key = derive_target_key();

    let tmp_path = std::env::temp_dir().join(format!(
        "steel-cookies-{}-{}.db",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    ));

    std::fs::copy(original_path, &tmp_path)?;

    let result = (|| -> Result<(Vec<u8>, u64)> {
        let conn = rusqlite::Connection::open(&tmp_path)?;

        let meta_version: i64 = conn
            .query_row(
                "SELECT value FROM meta WHERE key='version'",
                [],
                |row| row.get(0),
            )
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
        let tx = conn.unchecked_transaction()?;
        {
            let mut update =
                tx.prepare("UPDATE cookies SET encrypted_value = ? WHERE rowid = ?")?;

            for (rowid, host_key, encrypted_value) in &rows {
                let Some(plaintext) = decrypt_cookie(
                    encrypted_value,
                    &provider.source_key[..provider.source_key_len],
                    host_key,
                    meta_version,
                    provider.source_algorithm,
                ) else {
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
        Ok((buffer, converted))
    })();

    let _ = std::fs::remove_file(&tmp_path);
    result
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
        } else if path.is_file() {
            if let Ok(rel) = path.strip_prefix(base_dir) {
                if let Ok(data) = std::fs::read(&path) {
                    files.insert(rel.to_string_lossy().to_string(), data);
                }
            }
        }
    }
}

// ─── Package ─────────────────────────────────────────────────────────────────

pub struct PackageResult {
    pub zip_buffer: Vec<u8>,
    pub cookies_reencrypted: u64,
}

/// Package a browser profile for upload to Steel.
pub fn package_profile(
    browser: BrowserId,
    profile_dir_name: &str,
    on_progress: &dyn Fn(&str),
) -> Result<PackageResult> {
    let desc = browser.descriptor();
    let base_dir = desc
        .profile_base_dir()
        .ok_or_else(|| anyhow::anyhow!("{} is not supported on this platform.", browser.display_name()))?;

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

    for entry_name in INCLUDE_ENTRIES {
        let full_path = profile_dir.join(entry_name);
        if !full_path.exists() {
            continue;
        }

        let meta = std::fs::metadata(&full_path)?;

        if meta.is_file() {
            if *entry_name == "Cookies" {
                on_progress("Re-encrypting Cookies...");
                let (buffer, converted) = reencrypt_cookies_db(&full_path, &provider)?;
                zip_files.push((format!("Default/{entry_name}"), buffer));
                cookies_reencrypted = converted;
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
        let result = decrypt_cookie(&encrypted, &key2, ".test.com", 20, CryptoAlgorithm::Aes128Cbc);
        assert!(result.is_none());
    }

    #[test]
    fn decrypt_no_v10_prefix_returns_none() {
        let key = derive_key("k", 1);
        assert!(decrypt_cookie(b"xyz_data", &key, "host", 20, CryptoAlgorithm::Aes128Cbc).is_none());
    }

    #[test]
    fn decrypt_short_data_returns_none() {
        let key = derive_key("k", 1);
        assert!(decrypt_cookie(b"v1", &key, "host", 20, CryptoAlgorithm::Aes128Cbc).is_none());
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
}
