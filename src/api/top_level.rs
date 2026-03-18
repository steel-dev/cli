//! Top-level API: scrape, screenshot, pdf.
//! Ported from: cli/source/utils/topLevelTools.ts

use serde_json::{Value, json};

use crate::api::client::{ApiError, SteelClient};
use crate::config::auth::Auth;
use crate::config::settings::ApiMode;

/// Supported scrape output formats.
pub const SCRAPE_FORMATS: &[&str] = &["html", "readability", "cleaned_html", "markdown"];

/// Validate a comma-separated format string.
pub fn parse_scrape_formats(input: &str) -> Result<Vec<String>, String> {
    let formats: Vec<String> = input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if formats.is_empty() {
        return Err("Missing value for --format. Example: --format html,markdown".into());
    }

    let invalid: Vec<&String> = formats
        .iter()
        .filter(|f| !SCRAPE_FORMATS.contains(&f.as_str()))
        .collect();

    if !invalid.is_empty() {
        let invalid_str: Vec<&str> = invalid.iter().map(|s| s.as_str()).collect();
        return Err(format!(
            "Invalid scrape format(s): {}. Supported: {}",
            invalid_str.join(", "),
            SCRAPE_FORMATS.join(", ")
        ));
    }

    Ok(formats)
}

/// Extract the best text content from a scrape response.
/// Tries preferred formats first, then falls back.
/// Matches TS `getScrapeOutputText()`.
pub fn get_scrape_output_text(data: &Value, preferred: &[String]) -> Option<String> {
    let content = data.get("content")?;

    let mut keys: Vec<&str> = Vec::new();
    for f in preferred {
        if !keys.contains(&f.as_str()) {
            keys.push(f.as_str());
        }
    }
    for &fallback in &["markdown", "cleaned_html", "html", "readability"] {
        if !keys.contains(&fallback) {
            keys.push(fallback);
        }
    }

    for key in keys {
        match content.get(key) {
            Some(Value::String(s)) if !s.trim().is_empty() => return Some(s.clone()),
            Some(v) if key == "readability" && v.is_object() => {
                return Some(serde_json::to_string_pretty(v).unwrap_or_default());
            }
            _ => continue,
        }
    }

    None
}

/// Extract a hosted URL from a screenshot/pdf response.
pub fn get_hosted_url(data: &Value) -> Option<String> {
    data.get("url")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
}

impl SteelClient {
    pub async fn scrape(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
        url: &str,
        formats: &[String],
        delay: Option<u64>,
        pdf: bool,
        screenshot: bool,
        use_proxy: bool,
        region: Option<&str>,
    ) -> Result<Value, ApiError> {
        let mut body = json!({
            "url": url,
            "format": formats,
        });
        let obj = body.as_object_mut().unwrap();

        if let Some(d) = delay {
            obj.insert("delay".into(), json!(d));
        }
        if pdf {
            obj.insert("pdf".into(), json!(true));
        }
        if screenshot {
            obj.insert("screenshot".into(), json!(true));
        }
        if use_proxy {
            obj.insert("useProxy".into(), json!(true));
        }
        if let Some(r) = region {
            obj.insert("region".into(), json!(r));
        }

        self.request(
            base_url,
            mode,
            reqwest::Method::POST,
            "/scrape",
            Some(body),
            auth,
        )
        .await
    }

    pub async fn screenshot(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
        url: &str,
        delay: Option<u64>,
        full_page: bool,
        use_proxy: bool,
        region: Option<&str>,
    ) -> Result<Value, ApiError> {
        let mut body = json!({"url": url});
        let obj = body.as_object_mut().unwrap();

        if let Some(d) = delay {
            obj.insert("delay".into(), json!(d));
        }
        if full_page {
            obj.insert("fullPage".into(), json!(true));
        }
        if use_proxy {
            obj.insert("useProxy".into(), json!(true));
        }
        if let Some(r) = region {
            obj.insert("region".into(), json!(r));
        }

        self.request(
            base_url,
            mode,
            reqwest::Method::POST,
            "/screenshot",
            Some(body),
            auth,
        )
        .await
    }

    pub async fn pdf(
        &self,
        base_url: &str,
        mode: ApiMode,
        auth: &Auth,
        url: &str,
        delay: Option<u64>,
        use_proxy: bool,
        region: Option<&str>,
    ) -> Result<Value, ApiError> {
        let mut body = json!({"url": url});
        let obj = body.as_object_mut().unwrap();

        if let Some(d) = delay {
            obj.insert("delay".into(), json!(d));
        }
        if use_proxy {
            obj.insert("useProxy".into(), json!(true));
        }
        if let Some(r) = region {
            obj.insert("region".into(), json!(r));
        }

        self.request(
            base_url,
            mode,
            reqwest::Method::POST,
            "/pdf",
            Some(body),
            auth,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- parse_scrape_formats ---

    #[test]
    fn parse_formats_valid() {
        let result = parse_scrape_formats("html,markdown").unwrap();
        assert_eq!(result, vec!["html", "markdown"]);
    }

    #[test]
    fn parse_formats_single() {
        let result = parse_scrape_formats("markdown").unwrap();
        assert_eq!(result, vec!["markdown"]);
    }

    #[test]
    fn parse_formats_with_spaces() {
        let result = parse_scrape_formats(" html , cleaned_html ").unwrap();
        assert_eq!(result, vec!["html", "cleaned_html"]);
    }

    #[test]
    fn parse_formats_invalid() {
        let err = parse_scrape_formats("html,xml").unwrap_err();
        assert!(err.contains("xml"));
    }

    #[test]
    fn parse_formats_empty() {
        let err = parse_scrape_formats("").unwrap_err();
        assert!(err.contains("Missing value"));
    }

    // --- get_scrape_output_text ---

    #[test]
    fn output_text_preferred_format() {
        let data = json!({
            "content": {
                "html": "<h1>Hello</h1>",
                "markdown": "# Hello"
            }
        });

        let text = get_scrape_output_text(&data, &["html".into()]).unwrap();
        assert_eq!(text, "<h1>Hello</h1>");
    }

    #[test]
    fn output_text_falls_back_to_markdown() {
        let data = json!({
            "content": {
                "markdown": "# Hello"
            }
        });

        let text = get_scrape_output_text(&data, &[]).unwrap();
        assert_eq!(text, "# Hello");
    }

    #[test]
    fn output_text_readability_object() {
        let data = json!({
            "content": {
                "readability": {"title": "Hello", "content": "world"}
            }
        });

        let text = get_scrape_output_text(&data, &["readability".into()]).unwrap();
        assert!(text.contains("Hello"));
    }

    #[test]
    fn output_text_empty_content_returns_none() {
        let data = json!({"content": {"markdown": "  "}});
        assert!(get_scrape_output_text(&data, &[]).is_none());
    }

    #[test]
    fn output_text_no_content_returns_none() {
        let data = json!({"url": "http://example.com"});
        assert!(get_scrape_output_text(&data, &[]).is_none());
    }

    // --- get_hosted_url ---

    #[test]
    fn hosted_url_present() {
        let data = json!({"url": "https://cdn.steel.dev/abc.png"});
        assert_eq!(
            get_hosted_url(&data).as_deref(),
            Some("https://cdn.steel.dev/abc.png")
        );
    }

    #[test]
    fn hosted_url_empty() {
        let data = json!({"url": " "});
        assert!(get_hosted_url(&data).is_none());
    }

    #[test]
    fn hosted_url_missing() {
        let data = json!({"status": "ok"});
        assert!(get_hosted_url(&data).is_none());
    }

    // --- Integration tests with wiremock ---

    use crate::config::auth::AuthSource;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_auth() -> Auth {
        Auth {
            api_key: Some("test-key".into()),
            source: AuthSource::Env,
        }
    }

    #[tokio::test]
    async fn scrape_success() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/scrape"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "content": {"markdown": "# Hello"}
            })))
            .mount(&server)
            .await;

        let client = SteelClient::new().unwrap();
        let data = client
            .scrape(
                &server.uri(),
                ApiMode::Local,
                &test_auth(),
                "https://example.com",
                &["markdown".into()],
                None,
                false,
                false,
                false,
                None,
            )
            .await
            .unwrap();

        let text = get_scrape_output_text(&data, &["markdown".into()]).unwrap();
        assert_eq!(text, "# Hello");
    }

    #[tokio::test]
    async fn screenshot_success() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/screenshot"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"url": "https://cdn.steel.dev/shot.png"})),
            )
            .mount(&server)
            .await;

        let client = SteelClient::new().unwrap();
        let data = client
            .screenshot(
                &server.uri(),
                ApiMode::Local,
                &test_auth(),
                "https://example.com",
                None,
                false,
                false,
                None,
            )
            .await
            .unwrap();

        assert_eq!(
            get_hosted_url(&data).as_deref(),
            Some("https://cdn.steel.dev/shot.png")
        );
    }

    #[tokio::test]
    async fn pdf_success() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/pdf"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"url": "https://cdn.steel.dev/doc.pdf"})),
            )
            .mount(&server)
            .await;

        let client = SteelClient::new().unwrap();
        let data = client
            .pdf(
                &server.uri(),
                ApiMode::Local,
                &test_auth(),
                "https://example.com",
                None,
                false,
                None,
            )
            .await
            .unwrap();

        assert_eq!(
            get_hosted_url(&data).as_deref(),
            Some("https://cdn.steel.dev/doc.pdf")
        );
    }
}
