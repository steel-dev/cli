use std::time::Duration;

use clap::Parser;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinSet;

use crate::config;
use crate::config::auth;
use crate::config::settings::{Config, read_config_from, write_config_to};
use crate::status;

#[derive(Parser)]
pub struct Args {}

const LOGIN_TIMEOUT: Duration = Duration::from_secs(5 * 60);
const CALLBACK_READ_TIMEOUT: Duration = Duration::from_secs(10);
const API_KEY_TIMEOUT: Duration = Duration::from_secs(30);

pub async fn run(_args: Args) -> anyhow::Result<()> {
    // Check if already logged in
    let existing_auth = auth::resolve_auth();
    if existing_auth.api_key.is_some() {
        status!("You are already logged in.");
        return Ok(());
    }

    status!("Launching browser for authentication...");

    let (api_key, name) = login_flow().await?;

    save_api_key(&api_key, &name)?;

    crate::telemetry::track_event("login_completed", serde_json::Map::new());

    status!("Authentication successful! Your API key has been saved.");

    Ok(())
}

async fn login_flow() -> anyhow::Result<(String, String)> {
    let state = generate_random_hex(16);

    // Bind to random port on localhost
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();

    let auth_url = format!(
        "{}?cli_redirect=true&port={}&state={}",
        config::LOGIN_URL,
        port,
        state
    );

    eprintln!("Opening your browser for authentication...");
    eprintln!("If it does not open automatically, please click:");
    eprintln!("{auth_url}");

    if let Err(e) = open::that(&auth_url) {
        eprintln!("Warning: could not open browser automatically: {e}");
        eprintln!("Please open the URL above manually.");
    }

    let jwt = tokio::time::timeout(LOGIN_TIMEOUT, wait_for_callback(&listener, &state))
        .await
        .map_err(|_| anyhow::anyhow!("Login timed out. Please try again."))??;

    create_api_key_using_jwt(&jwt).await
}

async fn wait_for_callback(listener: &TcpListener, state: &str) -> anyhow::Result<String> {
    let mut connections = JoinSet::new();
    loop {
        tokio::select! {
            accepted = listener.accept() => {
                let (stream, _) = accepted?;
                let state = state.to_string();
                connections.spawn(async move { handle_callback(stream, &state).await });
            }
            Some(handled) = connections.join_next() => {
                if let Some(jwt) = handled?? {
                    return Ok(jwt);
                }
            }
        }
    }
}

async fn handle_callback(stream: TcpStream, state: &str) -> anyhow::Result<Option<String>> {
    let mut reader = BufReader::new(stream);
    let mut request_line = String::new();
    let read =
        tokio::time::timeout(CALLBACK_READ_TIMEOUT, reader.read_line(&mut request_line)).await;
    if !matches!(read, Ok(Ok(bytes)) if bytes > 0) {
        return Ok(None);
    }
    let mut stream = reader.into_inner();

    // Parse query params from GET /callback?jwt=...&state=...
    let query = parse_query_from_request(&request_line);
    let param = |key: &str| query.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone());

    let Some(received_state) = param("state") else {
        respond(&mut stream, "404 Not Found", "").await;
        return Ok(None);
    };
    if received_state != state {
        respond(
            &mut stream,
            "400 Bad Request",
            "Error: Invalid state parameter.",
        )
        .await;
        anyhow::bail!("Invalid state parameter. Possible CSRF attack.");
    }

    let Some(jwt) = param("jwt") else {
        respond(
            &mut stream,
            "400 Bad Request",
            "Error: JWT not found in callback.",
        )
        .await;
        anyhow::bail!("Callback did not include a JWT.");
    };

    let redirect = format!(
        "HTTP/1.1 302 Found\r\nLocation: {}\r\nConnection: close\r\n\r\n",
        config::SUCCESS_URL
    );
    let _ = stream.write_all(redirect.as_bytes()).await;
    let _ = stream.flush().await;

    Ok(Some(jwt))
}

async fn respond(stream: &mut TcpStream, status: &str, body: &str) {
    let response =
        format!("HTTP/1.1 {status}\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n{body}");
    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.flush().await;
}

async fn create_api_key_using_jwt(jwt: &str) -> anyhow::Result<(String, String)> {
    let client = reqwest::Client::builder()
        .timeout(API_KEY_TIMEOUT)
        .build()?;
    let response = client
        .post(config::API_KEYS_URL)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {jwt}"))
        .json(&serde_json::json!({"name": "CLI"}))
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Failed to get API key: {} {}",
            response.status().as_u16(),
            response.status().canonical_reason().unwrap_or("")
        );
    }

    let data: serde_json::Value = response.json().await?;
    let key = data
        .get("key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("No API key found in response"))?;
    let name = data
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("No name found in response"))?;

    Ok((key.to_string(), name.to_string()))
}

fn save_api_key(api_key: &str, name: &str) -> anyhow::Result<()> {
    let config_dir = config::config_dir();
    std::fs::create_dir_all(&config_dir)?;
    let config_path = config::config_path_in(&config_dir);

    let mut config = read_config_from(&config_path).unwrap_or_else(|_| Config::default());
    config.api_key = Some(api_key.to_string());
    config.name = Some(name.to_string());

    write_config_to(&config_path, &config)?;

    Ok(())
}

fn parse_query_from_request(request_line: &str) -> Vec<(String, String)> {
    // "GET /callback?jwt=xxx&state=yyy HTTP/1.1"
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return vec![];
    }

    let path = parts[1];
    let query_start = match path.find('?') {
        Some(i) => i + 1,
        None => return vec![],
    };

    let query_str = &path[query_start..];
    query_str
        .split('&')
        .filter_map(|pair| {
            let mut kv = pair.splitn(2, '=');
            let key = kv.next()?;
            let value = kv.next().unwrap_or("");
            Some((
                urlencoding::decode(key).unwrap_or_default().to_string(),
                urlencoding::decode(value).unwrap_or_default().to_string(),
            ))
        })
        .collect()
}

fn generate_random_hex(bytes: usize) -> String {
    let mut buf = vec![0u8; bytes];
    getrandom::fill(&mut buf).expect("failed to generate random bytes");
    buf.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- parse_query_from_request ---

    #[test]
    fn parse_query_normal_get() {
        let query = parse_query_from_request("GET /callback?code=abc&state=xyz HTTP/1.1\r\n");
        assert_eq!(query.len(), 2);
        assert_eq!(query[0], ("code".to_string(), "abc".to_string()));
        assert_eq!(query[1], ("state".to_string(), "xyz".to_string()));
    }

    #[test]
    fn parse_query_no_query_string() {
        let query = parse_query_from_request("GET /callback HTTP/1.1\r\n");
        assert!(query.is_empty());
    }

    #[test]
    fn parse_query_empty_input() {
        let query = parse_query_from_request("");
        assert!(query.is_empty());
    }

    #[test]
    fn parse_query_single_word() {
        let query = parse_query_from_request("GET");
        assert!(query.is_empty());
    }

    #[test]
    fn parse_query_url_encoded() {
        let query = parse_query_from_request("GET /cb?msg=hello%20world HTTP/1.1\r\n");
        assert_eq!(query.len(), 1);
        assert_eq!(query[0], ("msg".to_string(), "hello world".to_string()));
    }

    #[test]
    fn parse_query_empty_value() {
        let query = parse_query_from_request("GET /cb?key= HTTP/1.1\r\n");
        assert_eq!(query.len(), 1);
        assert_eq!(query[0], ("key".to_string(), "".to_string()));
    }

    #[test]
    fn parse_query_key_without_equals() {
        let query = parse_query_from_request("GET /cb?flag HTTP/1.1\r\n");
        assert_eq!(query.len(), 1);
        assert_eq!(query[0], ("flag".to_string(), "".to_string()));
    }

    #[test]
    fn parse_query_multiple_params() {
        let query = parse_query_from_request("GET /cb?a=1&b=2&c=3 HTTP/1.1\r\n");
        assert_eq!(query.len(), 3);
        assert_eq!(query[0].0, "a");
        assert_eq!(query[1].0, "b");
        assert_eq!(query[2].0, "c");
    }

    #[test]
    fn parse_query_value_with_equals() {
        let query = parse_query_from_request("GET /cb?q=a=b HTTP/1.1\r\n");
        assert_eq!(query.len(), 1);
        assert_eq!(query[0], ("q".to_string(), "a=b".to_string()));
    }

    // --- generate_random_hex ---

    async fn callback(
        request: &'static str,
        state: &'static str,
    ) -> (anyhow::Result<Option<String>>, String) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let client = tokio::spawn(async move {
            let mut stream = TcpStream::connect(addr).await.unwrap();
            stream.write_all(request.as_bytes()).await.unwrap();
            let mut reply = String::new();
            let _ = tokio::io::AsyncReadExt::read_to_string(&mut stream, &mut reply).await;
            reply
        });
        let (stream, _) = listener.accept().await.unwrap();
        let result = handle_callback(stream, state).await;
        (result, client.await.unwrap())
    }

    #[tokio::test]
    async fn callback_returns_the_jwt_and_redirects() {
        let (result, reply) =
            callback("GET /callback?jwt=abc&state=s1 HTTP/1.1\r\n\r\n", "s1").await;
        assert_eq!(result.unwrap().as_deref(), Some("abc"));
        assert!(reply.starts_with("HTTP/1.1 302"), "{reply}");
    }

    #[tokio::test]
    async fn unrelated_requests_are_ignored() {
        let (result, reply) = callback("GET /favicon.ico HTTP/1.1\r\n\r\n", "s1").await;
        assert!(result.unwrap().is_none());
        assert!(reply.starts_with("HTTP/1.1 404"), "{reply}");
    }

    #[tokio::test]
    async fn wrong_state_is_rejected() {
        let (result, reply) =
            callback("GET /callback?jwt=abc&state=evil HTTP/1.1\r\n\r\n", "s1").await;
        assert!(result.is_err());
        assert!(reply.starts_with("HTTP/1.1 400"), "{reply}");
    }

    #[tokio::test]
    async fn waits_past_idle_connections() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let idle = TcpStream::connect(addr).await.unwrap();
        let mut real = TcpStream::connect(addr).await.unwrap();
        real.write_all(b"GET /callback?jwt=abc&state=s1 HTTP/1.1\r\n\r\n")
            .await
            .unwrap();
        let jwt = tokio::time::timeout(Duration::from_secs(5), wait_for_callback(&listener, "s1"))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(jwt, "abc");
        drop(idle);
    }

    #[test]
    fn random_hex_length() {
        assert_eq!(generate_random_hex(16).len(), 32);
        assert_eq!(generate_random_hex(1).len(), 2);
    }

    #[test]
    fn random_hex_chars_only() {
        let hex = generate_random_hex(16);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn random_hex_uniqueness() {
        let a = generate_random_hex(16);
        let b = generate_random_hex(16);
        assert_ne!(a, b);
    }
}
