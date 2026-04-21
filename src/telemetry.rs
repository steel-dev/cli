use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::config;
use crate::config::settings::ApiMode;

const DEFAULT_POSTHOG_HOST: &str = "https://us.i.posthog.com";
const PROJECT_TOKEN: &str = "phc_yhxgGFVDmaCpccF38yspn4pAkVLFnHsGaWmYUyijuuRS";
const CAPTURE_PATH: &str = "/i/v0/e/";
const FLUSH_TIMEOUT: Duration = Duration::from_millis(150);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);

static GLOBAL: OnceLock<Mutex<GlobalTelemetry>> = OnceLock::new();

#[cfg(test)]
static TEST_OVERRIDE: OnceLock<Mutex<Option<TestTelemetryOverride>>> = OnceLock::new();

fn global() -> &'static Mutex<GlobalTelemetry> {
    GLOBAL.get_or_init(|| Mutex::new(GlobalTelemetry::default()))
}

#[derive(Default)]
struct GlobalTelemetry {
    client: Option<Arc<TelemetryClient>>,
    pending: Vec<tokio::task::JoinHandle<()>>,
}

#[cfg(test)]
#[derive(Clone)]
struct TestTelemetryOverride {
    config_dir: PathBuf,
    env: TelemetryEnv,
}

#[derive(Clone)]
struct TelemetryClient {
    http: reqwest::Client,
    capture_url: String,
    api_key: String,
    distinct_id: String,
}

struct TelemetryBootstrap {
    client: Arc<TelemetryClient>,
    install_created: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandContext {
    command_path: String,
}

#[derive(Debug, Default, Clone)]
struct TelemetryEnv {
    host: Option<String>,
    disabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TelemetryState {
    anonymous_install_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    first_seen_unix_ms: Option<u64>,
}

impl TelemetryEnv {
    fn from_env() -> Self {
        Self {
            host: std::env::var("STEEL_TELEMETRY_HOST").ok(),
            disabled: std::env::var("STEEL_TELEMETRY_DISABLED")
                .ok()
                .is_some_and(|value| matches!(value.trim(), "1" | "true" | "TRUE" | "True")),
        }
    }
}

impl TelemetryClient {
    fn from_parts(
        config_dir: &Path,
        env: &TelemetryEnv,
        config: Option<&crate::config::settings::Config>,
    ) -> Option<TelemetryBootstrap> {
        if telemetry_disabled(env, config) {
            return None;
        }

        let host = env
            .host
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(DEFAULT_POSTHOG_HOST)
            .trim_end_matches('/')
            .to_string();

        let http = reqwest::Client::builder()
            .user_agent(format!("steel-cli/{}", env!("CARGO_PKG_VERSION")))
            .timeout(REQUEST_TIMEOUT)
            .build()
            .ok()?;

        let identity = load_or_create_anonymous_install_id(config_dir);

        Some(TelemetryBootstrap {
            client: Arc::new(Self {
                http,
                capture_url: format!("{host}{CAPTURE_PATH}"),
                api_key: PROJECT_TOKEN.to_string(),
                distinct_id: identity.distinct_id,
            }),
            install_created: identity.is_new,
        })
    }
}

pub fn init_from_env() {
    #[cfg(test)]
    let bootstrap = {
        let config = crate::config::settings::read_config().ok();
        let override_state = TEST_OVERRIDE
            .get_or_init(|| Mutex::new(None))
            .lock()
            .clone();
        override_state
            .and_then(|state| {
                TelemetryClient::from_parts(&state.config_dir, &state.env, config.as_ref())
            })
            .or_else(|| {
                TelemetryClient::from_parts(
                    &config::config_dir(),
                    &TelemetryEnv::from_env(),
                    config.as_ref(),
                )
            })
    };

    #[cfg(not(test))]
    let bootstrap = {
        let config = crate::config::settings::read_config().ok();
        TelemetryClient::from_parts(
            &config::config_dir(),
            &TelemetryEnv::from_env(),
            config.as_ref(),
        )
    };

    let mut state = global().lock();
    state.client = bootstrap
        .as_ref()
        .map(|bootstrap| Arc::clone(&bootstrap.client));
    state.pending.clear();
    let install_created = bootstrap
        .as_ref()
        .is_some_and(|bootstrap| bootstrap.install_created);
    drop(state);

    if install_created {
        emit_first_run_notice();
        track_event("install_created", Map::new());
    }
}

fn emit_first_run_notice() {
    if crate::util::output::is_json() {
        return;
    }
    eprintln!();
    eprintln!("Steel CLI collects anonymous usage data to help improve the product.");
    eprintln!("No URLs, selectors, credentials, or command arguments are sent.");
    eprintln!(
        "Opt out: set STEEL_TELEMETRY_DISABLED=1, or add `\"telemetry\": {{\"disabled\": true}}` to ~/.config/steel/config.json"
    );
    eprintln!();
}

pub fn command_context(command_path: &str) -> CommandContext {
    CommandContext {
        command_path: command_path.to_string(),
    }
}

pub fn track_command_started(context: &CommandContext) {
    let mut properties = default_properties();
    properties.insert("command_path".into(), json!(context.command_path));
    track("command_started", properties);
}

pub fn track_command_completed(context: &CommandContext, duration: Duration) {
    let mut properties = default_properties();
    properties.insert("command_path".into(), json!(context.command_path));
    properties.insert("success".into(), json!(true));
    properties.insert("duration_ms".into(), json!(duration.as_millis() as u64));
    track("command_completed", properties);
}

pub fn track_command_failed(context: &CommandContext, duration: Duration, err: &anyhow::Error) {
    let mut properties = default_properties();
    properties.insert("command_path".into(), json!(context.command_path));
    properties.insert("success".into(), json!(false));
    properties.insert("duration_ms".into(), json!(duration.as_millis() as u64));
    properties.insert("error_class".into(), json!(error_class(err)));
    track("command_failed", properties);
}

pub fn track_event(event: &str, mut properties: Map<String, Value>) {
    let defaults = default_properties();
    for (key, value) in defaults {
        properties.entry(key).or_insert(value);
    }
    track(event, properties);
}

pub async fn flush_best_effort() {
    let handles = {
        let mut state = global().lock();
        state.pending.drain(..).collect::<Vec<_>>()
    };

    if handles.is_empty() {
        return;
    }

    let _ = tokio::time::timeout(FLUSH_TIMEOUT, async move {
        for handle in handles {
            let _ = handle.await;
        }
    })
    .await;
}

fn track(event: &str, properties: Map<String, Value>) {
    let client = {
        let state = global().lock();
        state.client.clone()
    };

    let Some(client) = client else {
        return;
    };

    let Ok(handle) = tokio::runtime::Handle::try_current() else {
        return;
    };

    let event = event.to_string();
    let join = handle.spawn(async move {
        let payload =
            build_capture_payload(&client.api_key, &client.distinct_id, &event, properties);
        let _ = client
            .http
            .post(&client.capture_url)
            .json(&payload)
            .send()
            .await;
    });

    let mut state = global().lock();
    state.pending.push(join);
}

fn build_capture_payload(
    api_key: &str,
    distinct_id: &str,
    event: &str,
    mut properties: Map<String, Value>,
) -> Value {
    properties.insert("$process_person_profile".into(), json!(false));
    properties.insert("$lib".into(), json!("steel-cli"));
    properties.insert("$lib_version".into(), json!(env!("CARGO_PKG_VERSION")));

    json!({
        "api_key": api_key,
        "event": event,
        "distinct_id": distinct_id,
        "properties": properties,
    })
}

fn default_properties() -> Map<String, Value> {
    let mut properties = Map::new();
    properties.insert("cli_version".into(), json!(env!("CARGO_PKG_VERSION")));
    properties.insert("os".into(), json!(std::env::consts::OS));
    properties.insert("arch".into(), json!(std::env::consts::ARCH));
    properties.insert("json".into(), json!(crate::util::output::is_json()));
    properties.insert("mode".into(), json!(telemetry_mode()));
    properties
}

fn telemetry_disabled(
    env: &TelemetryEnv,
    config: Option<&crate::config::settings::Config>,
) -> bool {
    env.disabled || config.is_some_and(crate::config::settings::Config::telemetry_disabled)
}

fn telemetry_mode() -> &'static str {
    let (mode, base_url) = crate::util::api::resolve();
    match mode {
        ApiMode::Cloud if base_url == config::DEFAULT_API_URL => "cloud",
        ApiMode::Local if base_url == config::DEFAULT_LOCAL_API_URL => "local",
        _ => "self_hosted",
    }
}

fn error_class(err: &anyhow::Error) -> &'static str {
    if let Some(api_err) = err.downcast_ref::<crate::api::client::ApiError>() {
        return match api_err {
            crate::api::client::ApiError::MissingAuth => "auth_required",
            crate::api::client::ApiError::Unreachable { .. } => "network_unreachable",
            crate::api::client::ApiError::RequestFailed { status, .. } => match *status {
                401 => "api_unauthorized",
                403 => "api_forbidden",
                404 | 410 => "api_not_found",
                429 => "api_rate_limited",
                s if s >= 500 => "api_server_error",
                _ => "api_request_failed",
            },
            crate::api::client::ApiError::Other(_) => "network_error",
        };
    }

    if err.chain().any(|source| source.is::<std::io::Error>()) {
        return "io_error";
    }

    "internal_error"
}

struct TelemetryIdentity {
    distinct_id: String,
    is_new: bool,
}

fn load_or_create_anonymous_install_id(config_dir: &Path) -> TelemetryIdentity {
    let path = telemetry_state_path(config_dir);

    if let Ok(contents) = std::fs::read_to_string(&path)
        && let Ok(state) = serde_json::from_str::<TelemetryState>(&contents)
        && !state.anonymous_install_id.trim().is_empty()
    {
        return TelemetryIdentity {
            distinct_id: state.anonymous_install_id,
            is_new: false,
        };
    }

    let anonymous_install_id = generate_uuid_like_id();
    let state = TelemetryState {
        anonymous_install_id: anonymous_install_id.clone(),
        first_seen_unix_ms: Some(now_unix_ms()),
    };
    let _ = write_telemetry_state(&path, &state);

    TelemetryIdentity {
        distinct_id: anonymous_install_id,
        is_new: true,
    }
}

fn telemetry_state_path(config_dir: &Path) -> PathBuf {
    config_dir.join("telemetry.json")
}

fn write_telemetry_state(path: &Path, state: &TelemetryState) -> anyhow::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    let payload = serde_json::to_vec_pretty(state)?;
    std::fs::write(&tmp, payload)?;
    std::fs::rename(tmp, path)?;
    Ok(())
}

fn generate_uuid_like_id() -> String {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).expect("failed to generate telemetry install id");
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15],
    )
}

fn now_unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
pub fn reset_for_test() {
    let mut state = global().lock();
    for handle in state.pending.drain(..) {
        handle.abort();
    }
    state.client = None;

    if let Some(lock) = TEST_OVERRIDE.get() {
        *lock.lock() = None;
    }
}

#[cfg(test)]
pub fn set_test_override(config_dir: &Path, host: &str) {
    let override_state = TestTelemetryOverride {
        config_dir: config_dir.to_path_buf(),
        env: TelemetryEnv {
            host: Some(host.to_string()),
            disabled: false,
        },
    };

    let lock = TEST_OVERRIDE.get_or_init(|| Mutex::new(None));
    *lock.lock() = Some(override_state);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn telemetry_enabled_by_default() {
        let dir = tempfile::TempDir::new().unwrap();
        let env = TelemetryEnv::default();

        let client = TelemetryClient::from_parts(dir.path(), &env, None);

        assert!(client.is_some());
    }

    #[test]
    fn telemetry_disabled_env_flag_turns_client_off() {
        let dir = tempfile::TempDir::new().unwrap();
        let env = TelemetryEnv {
            disabled: true,
            ..Default::default()
        };

        let client = TelemetryClient::from_parts(dir.path(), &env, None);

        assert!(client.is_none());
    }

    #[test]
    fn telemetry_uses_host_override() {
        let dir = tempfile::TempDir::new().unwrap();
        let env = TelemetryEnv {
            host: Some("http://127.0.0.1:9999/".into()),
            ..Default::default()
        };

        let client = TelemetryClient::from_parts(dir.path(), &env, None).unwrap();

        assert_eq!(client.client.capture_url, "http://127.0.0.1:9999/i/v0/e/");
    }

    #[test]
    fn telemetry_disabled_in_config_turns_client_off() {
        let dir = tempfile::TempDir::new().unwrap();
        let env = TelemetryEnv::default();
        let config = crate::config::settings::Config {
            telemetry: Some(crate::config::settings::TelemetryConfig {
                disabled: Some(true),
            }),
            ..Default::default()
        };

        let client = TelemetryClient::from_parts(dir.path(), &env, Some(&config));

        assert!(client.is_none());
    }

    #[test]
    fn anonymous_install_id_persists() {
        let dir = tempfile::TempDir::new().unwrap();

        let first = load_or_create_anonymous_install_id(dir.path());
        let second = load_or_create_anonymous_install_id(dir.path());

        assert_eq!(first.distinct_id, second.distinct_id);
        assert!(first.is_new);
        assert!(!second.is_new);
    }

    #[test]
    fn malformed_telemetry_state_regenerates_id() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = telemetry_state_path(dir.path());
        std::fs::write(&path, "{not-json").unwrap();

        let id = load_or_create_anonymous_install_id(dir.path());

        assert!(!id.distinct_id.is_empty());
        assert!(id.is_new);
        let contents = std::fs::read_to_string(path).unwrap();
        assert!(contents.contains("anonymousInstallId"));
    }

    #[test]
    fn unreadable_telemetry_state_falls_back_to_ephemeral_id() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = telemetry_state_path(dir.path());
        std::fs::create_dir_all(&path).unwrap();

        let id = load_or_create_anonymous_install_id(dir.path());

        assert!(!id.distinct_id.is_empty());
        assert!(id.is_new);
    }

    #[test]
    fn capture_payload_includes_distinct_id_and_properties() {
        let mut properties = Map::new();
        properties.insert("command_path".into(), json!("config"));

        let payload = build_capture_payload("phc_test", "anon-123", "command_started", properties);

        assert_eq!(payload["api_key"], "phc_test");
        assert_eq!(payload["event"], "command_started");
        assert_eq!(payload["distinct_id"], "anon-123");
        assert_eq!(payload["properties"]["command_path"], "config");
        assert_eq!(payload["properties"]["$process_person_profile"], false);
    }
}
