use std::io::{BufRead, BufReader, Write as IoWrite};

use clap::Parser;
use tokio::net::TcpListener;

use crate::config;
use crate::config::auth;
use crate::status;
use crate::config::settings::{Config, read_config_from, write_config_to};

#[derive(Parser)]
pub struct Args {}

const LOGIN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5 * 60);

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

    status!("Opening your browser for authentication...");
    status!("If it does not open automatically, please click:");
    status!("{auth_url}");

    if let Err(e) = open::that(&auth_url) {
        status!("Warning: could not open browser automatically: {e}");
        status!("Please open the URL above manually.");
    }

    // Wait for the callback with timeout
    let result = tokio::time::timeout(LOGIN_TIMEOUT, async {
        let (stream, _) = listener.accept().await?;
        stream.readable().await?;

        let std_stream = stream.into_std()?;
        let mut reader = BufReader::new(&std_stream);

        let mut request_line = String::new();
        reader.read_line(&mut request_line)?;

        // Parse query params from GET /callback?jwt=...&state=...
        let query = parse_query_from_request(&request_line);

        let received_state = query
            .iter()
            .find(|(k, _)| k == "state")
            .map(|(_, v)| v.as_str());

        if received_state != Some(&state) {
            send_response(&std_stream, 400, "Error: Invalid state parameter.")?;
            anyhow::bail!("Invalid state parameter. Possible CSRF attack.");
        }

        let jwt = query
            .iter()
            .find(|(k, _)| k == "jwt")
            .map(|(_, v)| v.clone());

        let Some(jwt) = jwt else {
            send_response(&std_stream, 400, "Error: JWT not found in callback.")?;
            anyhow::bail!("Callback did not include a JWT.");
        };

        // Redirect browser to success page
        let redirect = format!(
            "HTTP/1.1 302 Found\r\nLocation: {}\r\nConnection: close\r\n\r\n",
            config::SUCCESS_URL
        );
        (&std_stream).write_all(redirect.as_bytes())?;
        (&std_stream).flush()?;

        // Exchange JWT for API key
        create_api_key_using_jwt(&jwt).await
    })
    .await;

    match result {
        Ok(inner) => inner,
        Err(_) => anyhow::bail!("Login timed out. Please try again."),
    }
}

async fn create_api_key_using_jwt(jwt: &str) -> anyhow::Result<(String, String)> {
    let client = reqwest::Client::new();
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

fn send_response(stream: &std::net::TcpStream, status: u16, body: &str) -> anyhow::Result<()> {
    let response = format!(
        "HTTP/1.1 {status} Error\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n{body}"
    );
    let mut stream = stream;
    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(())
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
