//! First-party device authorization.
//!
//! Secrets live separately from config.json; a process lock serializes polls,
//! refresh rotation and logout. Never log response bodies or credential values.
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

const CLIENT: &str = "steel-cli";
const DEVICE_GRANT: &str = "urn:ietf:params:oauth:grant-type:device_code";

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Settings {
    pub issuer: String,
    pub audience: String,
    pub api_base_url: String,
}

impl Settings {
    pub fn resolve(saved: Option<&Self>) -> Result<Self> {
        let value = |name: &str, fallback: Option<&str>| -> Result<String> {
            std::env::var(name).ok().filter(|v| !v.trim().is_empty())
                .or_else(|| fallback.map(str::to_string))
                .with_context(|| format!("Set {name} for device login. Configure issuer, audience and product API transport separately."))
        };
        let settings = Self {
            issuer: value("STEEL_AUTH_ISSUER", saved.map(|s| s.issuer.as_str()))?
                .trim_end_matches('/')
                .to_string(),
            audience: value("STEEL_AUTH_AUDIENCE", saved.map(|s| s.audience.as_str()))?,
            api_base_url: value("STEEL_API_URL", saved.map(|s| s.api_base_url.as_str()))?
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
            let url = url::Url::parse(value).with_context(|| format!("Invalid auth {name} URL"))?;
            let local = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
            if !(url.scheme() == "https" || (local && url.scheme() == "http"))
                || !url.username().is_empty()
                || url.password().is_some()
                || url.fragment().is_some()
                || (name != "audience" && url.query().is_some())
            {
                bail!(
                    "Auth {name} must use HTTPS (HTTP only on loopback), without embedded credentials or fragments"
                );
            }
            if name == "issuer" && url.path() != "/" {
                bail!(
                    "Auth issuer must be an origin; do not use the product API URL or /api/auth path as issuer"
                );
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct State {
    pub settings: Settings,
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
struct Authorization {
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

pub fn state_path() -> PathBuf {
    super::config_dir().join("auth.json")
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn private_dir(path: &Path) -> Result<()> {
    if std::fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        bail!("Refusing to use a symlinked auth directory");
    }
    std::fs::create_dir_all(path)?;
    if std::fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        bail!("Refusing to use a symlinked auth directory");
    }
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
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
    }
    let file = options.open(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if file.metadata()?.permissions().mode() & 0o077 != 0 {
            bail!(
                "Auth state permissions are unsafe; set auth.json and auth.lock to mode 0600 before continuing"
            );
        }
    }
    Ok(file)
}

pub fn read_from(path: &Path) -> Result<Option<State>> {
    let mut file = match private_open(path, false) {
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
    // serde errors can include arbitrary input values. Never print them.
    Ok(Some(serde_json::from_str(&text).map_err(|_| {
        anyhow::anyhow!(
            "Invalid auth.json; restore it securely or log out before starting a new login"
        )
    })?))
}

fn save(path: &Path, state: &State) -> Result<()> {
    let parent = path.parent().context("Missing auth directory")?;
    private_dir(parent)?;
    if std::fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink()) {
        bail!("Refusing to replace symlinked auth state");
    }
    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    // NamedTempFile creates with mode 0600, before secret bytes are written.
    temp.write_all(&serde_json::to_vec(state)?)?;
    temp.as_file().sync_all()?;
    temp.persist(path)
        .map_err(|_| anyhow::anyhow!("Could not atomically save auth state"))?;
    File::open(parent)?.sync_all()?;
    Ok(())
}

async fn lock(path: &Path) -> Result<File> {
    let parent = path.parent().context("Missing auth directory")?;
    private_dir(parent)?;
    let lock_path = parent.join("auth.lock");
    tokio::task::spawn_blocking(move || {
        let file = private_open(&lock_path, true)?;
        let deadline = std::time::Instant::now() + Duration::from_secs(20);
        loop {
            match file.try_lock() {
                Ok(()) => return Ok(file),
                Err(std::fs::TryLockError::WouldBlock) => {
                    if std::time::Instant::now() >= deadline { bail!("Another Steel process is authorizing or refreshing. Retry this command; do not start a new login"); }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(std::fs::TryLockError::Error(error)) => return Err(error.into()),
            }
        }
    }).await?
}

fn http() -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("steel-cli")
        .build()?)
}

async fn token_request(
    settings: &Settings,
    fields: &[(&str, &str)],
) -> Result<std::result::Result<TokenResponse, String>> {
    let response = http()?
        .post(format!("{}/api/auth/oauth2/token", settings.issuer))
        .form(fields)
        .send()
        .await
        .context("Could not reach auth token endpoint. Pending login can resume; an uncertain refresh requires reauthorization to avoid token replay")?;
    let success = response.status().is_success();
    let body: serde_json::Value = response
        .json()
        .await
        .context("Auth service returned a non-JSON response")?;
    if !success {
        // Only a fixed known code reaches diagnostics, never error_description/body.
        let code = body
            .get("error")
            .and_then(|v| v.as_str())
            .filter(|code| {
                matches!(
                    *code,
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
        bail!("Auth service did not return a usable Bearer access token");
    }
    Ok(Ok(tokens))
}

fn store_tokens(state: &mut State, tokens: TokenResponse) {
    state.tokens = Some(Tokens {
        access_token: tokens.access_token,
        // A refresh response must supply the usable next refresh token. Retaining
        // the previous value could replay a consumed rotating token.
        refresh_token: tokens.refresh_token,
        expires_at: now().saturating_add(tokens.expires_in),
    });
    state.pending = None;
}

pub fn saved_settings() -> Result<Option<Settings>> {
    Ok(read_from(&state_path())?.map(|s| s.settings))
}

/// A product-audience JWT is not a gateway credential. WebSocket API routes
/// may use it only on the same transport origin, even with endpoint overrides.
pub fn validate_websocket_transport(api_base_url: &str, websocket_url: &str) -> Result<()> {
    let api = url::Url::parse(api_base_url)?;
    let mut websocket = url::Url::parse(websocket_url)?;
    let scheme = match websocket.scheme() {
        "wss" => "https",
        "ws" => "http",
        _ => bail!("Invalid product WebSocket scheme"),
    };
    websocket
        .set_scheme(scheme)
        .map_err(|_| anyhow::anyhow!("Invalid product WebSocket URL"))?;
    if api.origin() != websocket.origin()
        || !websocket.username().is_empty()
        || websocket.password().is_some()
    {
        bail!(
            "Refusing to send a product-audience token to a different WebSocket transport. Gateway access requires session connection credentials, not the product token"
        );
    }
    Ok(())
}

pub fn saved_access_token() -> Option<String> {
    match read_from(&state_path()) {
        Ok(state) => state.and_then(|s| s.tokens.map(|t| t.access_token)),
        // Never silently fall back to an unrelated saved key after corrupt or
        // insecure device storage. The request path reports the actual error.
        Err(_) => Some(String::new()),
    }
}

/// Bounded wait (< typical agent command timeout). A subsequent login/init
/// continues the persisted request and poll/backoff schedule, not a new grant.
pub async fn login(no_wait: bool, no_browser: bool, resume: bool) -> Result<bool> {
    let path = state_path();
    let _guard = lock(&path).await?;
    let existing = read_from(&path)?;
    let settings = Settings::resolve(existing.as_ref().map(|s| &s.settings))?;
    if existing.as_ref().is_some_and(|s| s.settings != settings) {
        bail!(
            "Auth issuer, audience or API transport changed. Log out before using a different auth context"
        );
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
        save(&path, &state)?;
        if resume {
            bail!("Device login expired. Run steel login to request a new code");
        }
    }
    let resuming = state.pending.is_some();
    if state.pending.is_none() {
        if resume {
            bail!("No pending login. Run steel login to request a code");
        }
        let response = http()?
            .post(format!("{}/api/auth/device/code", state.settings.issuer))
            .header("x-steel-device-installation", &state.installation_id)
            .header(
                "x-steel-device-hostname",
                std::env::var("HOSTNAME").unwrap_or_else(|_| "Steel CLI".into()),
            )
            .header("x-steel-device-version", env!("CARGO_PKG_VERSION"))
            .header("x-steel-device-platform", std::env::consts::OS)
            .form(&[
                ("client_id", CLIENT),
                ("scope", "openid offline_access steel"),
                ("resource", state.settings.audience.as_str()),
            ])
            .send()
            .await
            .context("Could not reach auth device endpoint")?;
        if !response.status().is_success() {
            bail!(
                "Device authorization request rejected (HTTP {}). Check issuer, audience and steel-cli registration",
                response.status().as_u16()
            );
        }
        let authz: Authorization = response
            .json()
            .await
            .map_err(|_| anyhow::anyhow!("Invalid device authorization response"))?;
        if authz.device_code.is_empty() || authz.user_code.is_empty() || authz.expires_in == 0 {
            bail!("Auth service returned an unusable device code");
        }
        // Never open an arbitrary URL returned by a misconfigured endpoint.
        for uri in [&authz.verification_uri, &authz.verification_uri_complete] {
            if url::Url::parse(uri)?.origin() != url::Url::parse(&state.settings.issuer)?.origin() {
                bail!("Device verification URL has a different origin than the issuer");
            }
        }
        state.pending = Some(Pending {
            device_code: authz.device_code,
            user_code: authz.user_code,
            verification_uri: authz.verification_uri,
            verification_uri_complete: authz.verification_uri_complete,
            expires_at: now().saturating_add(authz.expires_in),
            interval: authz.interval.max(1),
            next_poll_at: now().saturating_add(authz.interval.max(1)),
        });
        save(&path, &state)?; // Durable before handoff or browser launch.
    }
    let pending = state.pending.as_ref().expect("pending authorization");
    // Essential handoff must remain visible in JSON/non-TTY agent commands.
    eprintln!("Authorize Steel at {}", pending.verification_uri_complete);
    eprintln!(
        "Compare code {} in the browser. Or visit {} and enter that code.",
        pending.user_code, pending.verification_uri
    );
    eprintln!(
        "Login pending; resume with steel login --resume or rerun steel init --agent. The same request survives command interruption."
    );
    if !no_browser {
        let _ = open::that(&pending.verification_uri_complete);
    }
    let deadline = tokio::time::Instant::now()
        + Duration::from_secs(if no_wait { u64::from(resuming) } else { 45 });
    loop {
        let p = state.pending.as_mut().expect("pending authorization");
        if now() >= p.expires_at {
            state.pending = None;
            save(&path, &state)?;
            bail!("Device login expired. Run steel login again");
        }
        let delay = Duration::from_secs(p.next_poll_at.saturating_sub(now()));
        if delay >= deadline.saturating_duration_since(tokio::time::Instant::now()) {
            return Ok(false);
        }
        tokio::time::sleep(delay).await;
        // Save the next allowed poll before sending: killed/network-failed
        // commands cannot restart polling faster than the issuer's interval.
        p.next_poll_at = now().saturating_add(p.interval);
        save(&path, &state)?;
        let p = state.pending.as_ref().expect("pending authorization");
        match token_request(
            &state.settings,
            &[
                ("grant_type", DEVICE_GRANT),
                ("device_code", &p.device_code),
                ("client_id", CLIENT),
            ],
        )
        .await?
        {
            Ok(tokens) => {
                store_tokens(&mut state, tokens);
                save(&path, &state)?;
                return Ok(true);
            }
            Err(code) if code == "authorization_pending" => {}
            Err(code) if code == "slow_down" => {
                let p = state.pending.as_mut().expect("pending authorization");
                p.interval = p.interval.saturating_add(5);
                p.next_poll_at = now().saturating_add(p.interval);
                save(&path, &state)?;
            }
            Err(code) => {
                if matches!(
                    code.as_str(),
                    "access_denied" | "expired_token" | "invalid_grant"
                ) {
                    state.pending = None;
                    save(&path, &state)?;
                }
                bail!(
                    "Device login failed ({code}). Run steel login again; use an existing mapped product user/org. Signup provisioning is separate"
                );
            }
        }
    }
}

pub async fn access_token_for(api_base_url: &str) -> Result<String> {
    let path = state_path();
    let _guard = lock(&path).await?;
    let mut state = read_from(&path)?.context("Not logged in; run steel login")?;
    if state.refresh_in_flight {
        bail!(
            "A previous refresh was interrupted; its rotated token may have been consumed. To avoid replaying it, run steel logout --local-only then steel login"
        );
    }
    if Settings::resolve(Some(&state.settings))? != state.settings
        || api_base_url.trim_end_matches('/') != state.settings.api_base_url
    {
        bail!(
            "Auth context differs from saved login. Refusing to send tokens to another API transport; log out and sign in to the intended issuer/audience/transport"
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
        .context("Login expired without a refresh token; log out and run steel login")?;
    state.refresh_in_flight = true;
    save(&path, &state)?;
    // No retry of rotating refresh grants after ambiguous network failures.
    match token_request(
        &state.settings,
        &[
            ("grant_type", "refresh_token"),
            ("refresh_token", &refresh),
            ("client_id", CLIENT),
        ],
    )
    .await?
    {
        Ok(tokens) => {
            store_tokens(&mut state, tokens);
            state.refresh_in_flight = false;
            save(&path, &state)?;
            Ok(state
                .tokens
                .as_ref()
                .expect("saved tokens")
                .access_token
                .clone())
        }
        Err(code) => bail!(
            "Device token refresh failed ({code}); run steel logout --local-only, revoke the old device on the auth account page, then steel login. No API-key fallback was attempted"
        ),
    }
}

pub async fn logout(local_only: bool) -> Result<bool> {
    let path = state_path();
    let _guard = lock(&path).await?;
    if local_only {
        // Explicit recovery also works for corrupt/unsafe state without
        // reading its secrets or following a symlink to another file.
        return match std::fs::remove_file(&path) {
            Ok(()) => Ok(true),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error.into()),
        };
    }
    if let Some(state) = read_from(&path)? {
        state.settings.validate()?;
        if let Some(refresh) = state
            .tokens
            .as_ref()
            .and_then(|t| t.refresh_token.as_ref())
            .filter(|_| !local_only)
        {
            let response = http()?
                .post(format!("{}/api/auth/oauth2/revoke", state.settings.issuer))
                .form(&[
                    ("token", refresh.as_str()),
                    ("client_id", CLIENT),
                    ("token_type_hint", "refresh_token"),
                ])
                .send()
                .await
                .context(
                    "Could not revoke device login; local state retained so logout can be retried",
                )?;
            if !response.status().is_success() {
                bail!(
                    "Device revocation failed (HTTP {}); local state retained",
                    response.status().as_u16()
                );
            }
        }
        std::fs::remove_file(path)?;
        return Ok(true);
    }
    Ok(false)
}

fn random_id() -> String {
    let mut bytes = [0; 16];
    getrandom::fill(&mut bytes).expect("OS randomness");
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
