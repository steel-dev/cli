//! Private state and protocol for the first-party device grant.
//!
//! Keep auth separate from API keys/config.json. Serializing polling, refresh
//! and logout across CLI processes is required by rotating refresh tokens.
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

const CLIENT_ID: &str = "steel-cli";
const DEVICE_GRANT: &str = "urn:ietf:params:oauth:grant-type:device_code";

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Settings {
    pub issuer: String,
    pub audience: String,
    pub api_base_url: String,
}

impl Settings {
    pub fn resolve(saved: Option<&Self>) -> Result<Self> {
        let value = |name: &str, previous: Option<&str>, default: &str| {
            std::env::var(name)
                .ok()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| previous.unwrap_or(default).to_string())
        };
        let settings = Self {
            issuer: value(
                "STEEL_AUTH_ISSUER",
                saved.map(|s| s.issuer.as_str()),
                super::DEFAULT_AUTH_ISSUER,
            )
            .trim_end_matches('/')
            .to_string(),
            audience: value(
                "STEEL_AUTH_AUDIENCE",
                saved.map(|s| s.audience.as_str()),
                super::DEFAULT_AUTH_AUDIENCE,
            ),
            api_base_url: value(
                "STEEL_API_URL",
                saved.map(|s| s.api_base_url.as_str()),
                super::DEFAULT_API_URL,
            )
            .trim_end_matches('/')
            .to_string(),
        };
        settings.validate()?;
        Ok(settings)
    }

    fn validate(&self) -> Result<()> {
        for (name, value) in [
            ("issuer", &self.issuer),
            ("audience", &self.audience),
            ("API transport", &self.api_base_url),
        ] {
            let url = url::Url::parse(value).with_context(|| format!("Invalid {name} URL"))?;
            let local = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
            if !(url.scheme() == "https" || (local && url.scheme() == "http"))
                || !url.username().is_empty()
                || url.password().is_some()
                || url.fragment().is_some()
                || (name != "audience" && url.query().is_some())
            {
                bail!(
                    "{name} requires HTTPS (HTTP only on loopback), without credentials or fragments"
                );
            }
            if name == "issuer" && url.path() != "/" {
                bail!("Auth issuer must be an origin, not an API path");
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct State {
    settings: Settings,
    installation_id: String,
    pending: Option<Pending>,
    tokens: Option<Tokens>,
    #[serde(default)]
    refresh_in_flight: bool,
}

#[derive(Serialize, Deserialize)]
struct Pending {
    device_code: String,
    user_code: String,
    verification_uri: String,
    verification_uri_complete: String,
    expires_at: u64,
    interval: u64,
    next_poll_at: u64,
}

#[derive(Serialize, Deserialize)]
struct Tokens {
    access_token: String,
    refresh_token: Option<String>,
    expires_at: u64,
}

#[derive(Deserialize)]
struct DeviceCode {
    device_code: String,
    user_code: String,
    verification_uri: String,
    verification_uri_complete: String,
    expires_in: u64,
    interval: u64,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u64,
    token_type: String,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn state_path() -> std::path::PathBuf {
    super::config_dir().join("auth.json")
}

fn private_dir(path: &Path) -> Result<()> {
    if std::fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink()) {
        bail!("Refusing symlinked auth directory");
    }
    std::fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

fn private_open(path: &Path, create: bool) -> Result<File> {
    let mut options = OpenOptions::new();
    options.read(true).write(create).create(create);
    #[cfg(unix)]
    {
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
        let file = options.open(path)?;
        if file.metadata()?.permissions().mode() & 0o077 != 0 {
            bail!("Auth files require mode 0600");
        }
        Ok(file)
    }
    #[cfg(not(unix))]
    Ok(options.open(path)?)
}

fn read() -> Result<Option<State>> {
    let path = state_path();
    if let Some(parent) = path.parent()
        && std::fs::symlink_metadata(parent).is_ok_and(|m| m.file_type().is_symlink())
    {
        bail!("Refusing symlinked auth directory");
    }
    let mut file = match private_open(&path, false) {
        Ok(file) => file,
        Err(error)
            if error
                .downcast_ref::<std::io::Error>()
                .is_some_and(|e| e.kind() == std::io::ErrorKind::NotFound) =>
        {
            return Ok(None);
        }
        Err(error) => return Err(error.context("Cannot read private auth state")),
    };
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    // Never echo malformed input, which can contain credential values.
    serde_json::from_str(&text).map(Some).map_err(|_| {
        anyhow::anyhow!("Invalid auth.json; restore securely or run steel logout --local-only")
    })
}

fn save(state: &State) -> Result<()> {
    let path = state_path();
    let parent = path.parent().context("Missing auth directory")?;
    private_dir(parent)?;
    if std::fs::symlink_metadata(&path).is_ok_and(|m| m.file_type().is_symlink()) {
        bail!("Refusing symlinked auth state");
    }
    let mut temp = tempfile::NamedTempFile::new_in(parent)?; // 0600 before writing secrets.
    temp.write_all(&serde_json::to_vec(state)?)?;
    temp.as_file().sync_all()?;
    temp.persist(&path)
        .map_err(|_| anyhow::anyhow!("Could not atomically save auth state"))?;
    File::open(parent)?.sync_all()?;
    Ok(())
}

async fn lock() -> Result<File> {
    let dir = super::config_dir();
    private_dir(&dir)?;
    tokio::task::spawn_blocking(move || {
        let file = private_open(&dir.join("auth.lock"), true)?;
        let deadline = std::time::Instant::now() + Duration::from_secs(20);
        loop {
            match file.try_lock() {
                Ok(()) => return Ok(file),
                Err(std::fs::TryLockError::WouldBlock) => {
                    if std::time::Instant::now() >= deadline {
                        bail!(
                            "Another CLI process is authorizing or refreshing; retry this command"
                        );
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(std::fs::TryLockError::Error(error)) => return Err(error.into()),
            }
        }
    })
    .await?
}

fn http() -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("steel-cli")
        .build()?)
}

async fn exchange(
    settings: &Settings,
    fields: &[(&str, &str)],
) -> Result<std::result::Result<TokenResponse, String>> {
    let response = http()?.post(format!("{}/api/auth/oauth2/token", settings.issuer)).form(fields).send().await.context("Could not reach auth service; resume pending login or reauthorize an interrupted refresh")?;
    let success = response.status().is_success();
    let body: serde_json::Value = response
        .json()
        .await
        .context("Auth service returned non-JSON data")?;
    if !success {
        let code = body
            .get("error")
            .and_then(|v| v.as_str())
            .filter(|s| {
                matches!(
                    *s,
                    "authorization_pending"
                        | "slow_down"
                        | "access_denied"
                        | "expired_token"
                        | "invalid_grant"
                        | "invalid_client"
                        | "invalid_target"
                        | "invalid_scope"
                )
            })
            .unwrap_or("auth_service_error");
        return Ok(Err(code.to_string()));
    }
    let tokens: TokenResponse =
        serde_json::from_value(body).map_err(|_| anyhow::anyhow!("Invalid auth token response"))?;
    if tokens.access_token.is_empty()
        || tokens.expires_in == 0
        || !tokens.token_type.eq_ignore_ascii_case("bearer")
    {
        bail!("Auth service did not return a usable Bearer token");
    }
    Ok(Ok(tokens))
}

fn store_tokens(state: &mut State, response: TokenResponse) {
    state.tokens = Some(Tokens {
        access_token: response.access_token,
        refresh_token: response.refresh_token,
        expires_at: now().saturating_add(response.expires_in),
    });
    state.pending = None;
    state.refresh_in_flight = false;
}

pub fn saved_settings() -> Result<Option<Settings>> {
    Ok(read()?.map(|s| s.settings))
}

pub fn saved_access_token() -> Option<String> {
    match read() {
        Ok(state) => state.and_then(|s| s.tokens.map(|t| t.access_token)),
        Err(_) => Some(String::new()), // Request path reports error, never falls back to a saved key.
    }
}

/// Returns false while approval is pending. Repeated calls continue the same
/// request and polling schedule; --resume never replaces an expired request.
pub async fn login(no_wait: bool, no_browser: bool, resume: bool) -> Result<bool> {
    let _guard = lock().await?;
    let existing = read()?;
    let settings = Settings::resolve(existing.as_ref().map(|s| &s.settings))?;
    if existing.as_ref().is_some_and(|s| s.settings != settings) {
        bail!("Auth issuer, audience or API transport changed; log out before switching context");
    }
    let mut state = existing.unwrap_or_else(|| State {
        settings,
        installation_id: random_id(),
        pending: None,
        tokens: None,
        refresh_in_flight: false,
    });
    if state.tokens.is_some() {
        return Ok(true);
    }
    if state
        .pending
        .as_ref()
        .is_some_and(|p| p.expires_at <= now())
    {
        state.pending = None;
        save(&state)?;
        if resume {
            bail!("Device login expired; run steel login for a new code");
        }
    }
    let new_request = state.pending.is_none();
    if new_request {
        if resume {
            bail!("No pending login; run steel login");
        }
        let response = http()?
            .post(format!("{}/api/auth/device/code", state.settings.issuer))
            .header("x-steel-device-installation", &state.installation_id)
            .header("x-steel-device-version", env!("CARGO_PKG_VERSION"))
            .header("x-steel-device-platform", std::env::consts::OS)
            .form(&[
                ("client_id", CLIENT_ID),
                ("scope", "openid offline_access steel"),
                ("resource", state.settings.audience.as_str()),
            ])
            .send()
            .await
            .context("Could not reach device authorization endpoint")?;
        if !response.status().is_success() {
            bail!(
                "Device authorization rejected (HTTP {}); check issuer, audience and client registration",
                response.status().as_u16()
            );
        }
        let code: DeviceCode = response
            .json()
            .await
            .map_err(|_| anyhow::anyhow!("Invalid device authorization response"))?;
        if code.device_code.is_empty() || code.user_code.is_empty() || code.expires_in == 0 {
            bail!("Unusable device authorization response");
        }
        for uri in [&code.verification_uri, &code.verification_uri_complete] {
            let url = url::Url::parse(uri)?;
            if url.origin() != url::Url::parse(&state.settings.issuer)?.origin()
                || !url.username().is_empty()
                || url.password().is_some()
            {
                bail!("Verification URL does not belong to auth issuer");
            }
        }
        let interval = code.interval.max(1);
        state.pending = Some(Pending {
            device_code: code.device_code,
            user_code: code.user_code,
            verification_uri: code.verification_uri,
            verification_uri_complete: code.verification_uri_complete,
            expires_at: now().saturating_add(code.expires_in),
            interval,
            next_poll_at: now().saturating_add(interval),
        });
        save(&state)?;
    }
    let pending = state.pending.as_ref().expect("pending request");
    eprintln!("Authorize Steel at {}", pending.verification_uri_complete);
    eprintln!(
        "Compare code {} in the browser, or visit {} and enter it.",
        pending.user_code, pending.verification_uri
    );
    eprintln!(
        "After approval, run steel login --resume or repeat steel init. Pending login survives interruption."
    );
    if new_request && !no_browser {
        let _ = open::that(&pending.verification_uri_complete);
    }
    let deadline = tokio::time::Instant::now()
        + Duration::from_secs(if no_wait { u64::from(!new_request) } else { 45 });
    loop {
        let p = state.pending.as_mut().expect("pending request");
        if now() >= p.expires_at {
            state.pending = None;
            save(&state)?;
            bail!("Device login expired; run steel login again");
        }
        let delay = Duration::from_secs(p.next_poll_at.saturating_sub(now()));
        if delay >= deadline.saturating_duration_since(tokio::time::Instant::now()) {
            return Ok(false);
        }
        tokio::time::sleep(delay).await;
        p.next_poll_at = now().saturating_add(p.interval);
        save(&state)?;
        let p = state.pending.as_ref().expect("pending request");
        match exchange(
            &state.settings,
            &[
                ("grant_type", DEVICE_GRANT),
                ("device_code", &p.device_code),
                ("client_id", CLIENT_ID),
            ],
        )
        .await?
        {
            Ok(tokens) => {
                store_tokens(&mut state, tokens);
                save(&state)?;
                return Ok(true);
            }
            Err(code) if code == "authorization_pending" => {}
            Err(code) if code == "slow_down" => {
                let p = state.pending.as_mut().expect("pending request");
                p.interval = p.interval.saturating_add(5);
                p.next_poll_at = now().saturating_add(p.interval);
                save(&state)?;
            }
            Err(code) => {
                if matches!(
                    code.as_str(),
                    "access_denied" | "expired_token" | "invalid_grant"
                ) {
                    state.pending = None;
                    save(&state)?;
                }
                bail!("Device login failed ({code}); run steel login again");
            }
        }
    }
}

pub async fn access_token_for(api_base_url: &str) -> Result<String> {
    let _guard = lock().await?;
    let mut state = read()?.context("Not logged in; run steel login")?;
    if state.refresh_in_flight {
        bail!(
            "Previous refresh was interrupted; refusing to replay its token. Run steel logout --local-only, revoke the old device in your account, then log in again"
        );
    }
    if Settings::resolve(Some(&state.settings))? != state.settings
        || api_base_url.trim_end_matches('/') != state.settings.api_base_url
    {
        bail!(
            "Refusing to send tokens to a changed auth context or API transport; log out before switching"
        );
    }
    let tokens = state
        .tokens
        .as_ref()
        .context("Login pending; run steel login --resume")?;
    if tokens.expires_at > now().saturating_add(30) {
        return Ok(tokens.access_token.clone());
    }
    let refresh = tokens
        .refresh_token
        .clone()
        .context("No usable refresh token; log out and sign in again")?;
    state.refresh_in_flight = true;
    save(&state)?; // Durable before consuming a rotating token.
    match exchange(
        &state.settings,
        &[
            ("grant_type", "refresh_token"),
            ("refresh_token", &refresh),
            ("client_id", CLIENT_ID),
        ],
    )
    .await?
    {
        Ok(tokens) => {
            store_tokens(&mut state, tokens);
            save(&state)?;
            Ok(state.tokens.expect("saved tokens").access_token)
        }
        Err(code) => bail!(
            "Refresh failed ({code}); use steel logout --local-only, revoke the old device in your account, then log in again"
        ),
    }
}

pub async fn logout(local_only: bool) -> Result<bool> {
    let _guard = lock().await?;
    let path = state_path();
    if local_only {
        return match std::fs::remove_file(path) {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e.into()),
        };
    }
    let Some(state) = read()? else {
        return Ok(false);
    };
    state.settings.validate()?;
    if let Some(refresh) = state.tokens.and_then(|t| t.refresh_token) {
        let response = http()?
            .post(format!("{}/api/auth/oauth2/revoke", state.settings.issuer))
            .form(&[
                ("token", refresh.as_str()),
                ("client_id", CLIENT_ID),
                ("token_type_hint", "refresh_token"),
            ])
            .send()
            .await
            .context("Revocation unreachable; local state retained for retry")?;
        if !response.status().is_success() {
            bail!(
                "Revocation failed (HTTP {}); local state retained",
                response.status().as_u16()
            );
        }
    }
    std::fs::remove_file(path)?;
    Ok(true)
}

pub fn validate_websocket_transport(api_base: &str, endpoint: &str) -> Result<()> {
    let base = url::Url::parse(api_base)?;
    let mut target = url::Url::parse(endpoint)?;
    let scheme = match target.scheme() {
        "wss" => "https",
        "ws" => "http",
        _ => bail!("Invalid product WebSocket scheme"),
    };
    target
        .set_scheme(scheme)
        .map_err(|_| anyhow::anyhow!("Invalid WebSocket URL"))?;
    if target.origin() != base.origin()
        || !target.username().is_empty()
        || target.password().is_some()
    {
        bail!(
            "Refusing a product-audience token on a different WebSocket transport; use session-scoped gateway credentials"
        );
    }
    Ok(())
}

fn random_id() -> String {
    let mut bytes = [0; 16];
    getrandom::fill(&mut bytes).expect("OS randomness");
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
