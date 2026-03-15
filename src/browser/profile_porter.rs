use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

// ─── Constants ───────────────────────────────────────────────────────────────

fn chrome_base_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Library/Application Support/Google/Chrome")
}

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

// ─── Chrome profile discovery ────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ChromeProfile {
    pub dir_name: String,
    pub display_name: String,
}

pub fn find_chrome_profiles() -> Vec<ChromeProfile> {
    let base = chrome_base_dir();
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
        let display_name = get_chrome_profile_display_name(&base, &dir_name);
        profiles.push(ChromeProfile {
            dir_name,
            display_name,
        });
    }
    profiles.sort_by(|a, b| a.dir_name.cmp(&b.dir_name));
    profiles
}

fn get_chrome_profile_display_name(base: &Path, dir_name: &str) -> String {
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

pub fn is_chrome_running() -> bool {
    std::process::Command::new("pgrep")
        .args(["-x", "Google Chrome"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

// ─── Keychain ────────────────────────────────────────────────────────────────

fn get_keychain_passphrase() -> Result<String> {
    let output = std::process::Command::new("security")
        .args([
            "find-generic-password",
            "-w",
            "-s",
            "Chrome Safe Storage",
        ])
        .output()
        .context("Failed to run `security` command")?;

    if !output.status.success() {
        anyhow::bail!("Failed to retrieve Chrome Safe Storage password from Keychain");
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
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

fn decrypt_cookie(
    encrypted_value: &[u8],
    key: &[u8; 16],
    host_key: &str,
    meta_version: i64,
) -> Option<String> {
    use aes::Aes128;
    use cbc::cipher::{BlockDecryptMut, KeyIvInit, block_padding::Pkcs7};

    if encrypted_value.len() < 3 {
        return None;
    }
    if &encrypted_value[..3] != b"v10" {
        return None;
    }

    type Aes128CbcDec = cbc::Decryptor<Aes128>;

    let mut buf = encrypted_value[3..].to_vec();
    let plaintext = Aes128CbcDec::new(key.into(), &IV.into())
        .decrypt_padded_mut::<Pkcs7>(&mut buf)
        .ok()?;

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

    // encrypt_padded_mut requires a buffer with enough space for padding
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
    macos_key: &[u8; 16],
    peanuts_key: &[u8; 16],
) -> Result<(Vec<u8>, u64)> {
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
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
            .collect::<std::result::Result<_, _>>()?;

        let mut converted: u64 = 0;
        let tx = conn.unchecked_transaction()?;
        {
            let mut update =
                tx.prepare("UPDATE cookies SET encrypted_value = ? WHERE rowid = ?")?;

            for (rowid, host_key, encrypted_value) in &rows {
                let Some(plaintext) =
                    decrypt_cookie(encrypted_value, macos_key, host_key, meta_version)
                else {
                    continue;
                };

                let reencrypted = encrypt_cookie(&plaintext, peanuts_key, host_key, meta_version);
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

    // Best-effort cleanup
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

pub fn package_chrome_profile(
    chrome_profile: &str,
    on_progress: &dyn Fn(&str),
) -> Result<PackageResult> {
    let profile_dir = chrome_base_dir().join(chrome_profile);

    if !profile_dir.join("Cookies").exists() {
        anyhow::bail!(
            "Chrome profile \"{}\" not found at {}",
            chrome_profile,
            profile_dir.display()
        );
    }

    on_progress("Reading Keychain...");
    let passphrase = get_keychain_passphrase()?;
    let macos_key = derive_key(&passphrase, 1003);
    let peanuts_key = derive_key("peanuts", 1);

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
                let (buffer, converted) =
                    reencrypt_cookies_db(&full_path, &macos_key, &peanuts_key)?;
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

    on_progress("Zipping...");
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
        // Deterministic — same input always produces same output
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

        // meta_version < 24: no domain hash
        let encrypted = encrypt_cookie(value, &key, host_key, 20);
        assert_eq!(&encrypted[..3], b"v10");
        let decrypted = decrypt_cookie(&encrypted, &key, host_key, 20);
        assert_eq!(decrypted.as_deref(), Some(value));
    }

    #[test]
    fn encrypt_decrypt_roundtrip_v24() {
        let key = derive_key("testkey", 1);
        let value = "secret cookie";
        let host_key = ".google.com";

        // meta_version >= 24: with domain hash
        let encrypted = encrypt_cookie(value, &key, host_key, 24);
        let decrypted = decrypt_cookie(&encrypted, &key, host_key, 24);
        assert_eq!(decrypted.as_deref(), Some(value));
    }

    #[test]
    fn decrypt_wrong_key_returns_none() {
        let key1 = derive_key("key1", 1);
        let key2 = derive_key("key2", 1);
        let encrypted = encrypt_cookie("data", &key1, ".test.com", 20);
        // Wrong key — PKCS7 unpadding will fail
        let result = decrypt_cookie(&encrypted, &key2, ".test.com", 20);
        assert!(result.is_none());
    }

    #[test]
    fn decrypt_no_v10_prefix_returns_none() {
        let key = derive_key("k", 1);
        assert!(decrypt_cookie(b"xyz_data", &key, "host", 20).is_none());
    }

    #[test]
    fn decrypt_short_data_returns_none() {
        let key = derive_key("k", 1);
        assert!(decrypt_cookie(b"v1", &key, "host", 20).is_none());
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
}
