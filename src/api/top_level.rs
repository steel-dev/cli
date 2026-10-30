//! Top-level API: scrape, screenshot, pdf.
//! Ported from: cli/source/utils/topLevelTools.ts

use serde_json::{Value, json};

use crate::api::client::{ApiError, SteelClient};

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

pub struct Target<'a> {
    pub url: &'a str,
    pub delay: Option<u64>,
    pub use_proxy: bool,
    pub region: Option<&'a str>,
}

impl Target<'_> {
    fn body(&self) -> Value {
        let mut body = json!({"url": self.url});
        if let Some(delay) = self.delay {
            body["delay"] = json!(delay);
        }
        if self.use_proxy {
            body["useProxy"] = json!(true);
        }
        if let Some(region) = self.region {
            body["region"] = json!(region);
        }
        body
    }
}

impl SteelClient {
    pub async fn scrape(
        &self,
        target: &Target<'_>,
        formats: &[String],
        pdf: bool,
        screenshot: bool,
    ) -> Result<Value, ApiError> {
        let mut body = target.body();
        body["format"] = json!(formats);
        if pdf {
            body["pdf"] = json!(true);
        }
        if screenshot {
            body["screenshot"] = json!(true);
        }
        self.request(reqwest::Method::POST, "/scrape", Some(body))
            .await
    }

    pub async fn screenshot(
        &self,
        target: &Target<'_>,
        full_page: bool,
    ) -> Result<Value, ApiError> {
        let mut body = target.body();
        if full_page {
            body["fullPage"] = json!(true);
        }
        self.request(reqwest::Method::POST, "/screenshot", Some(body))
            .await
    }

    pub async fn pdf(&self, target: &Target<'_>) -> Result<Value, ApiError> {
        self.request(reqwest::Method::POST, "/pdf", Some(target.body()))
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

    use crate::config::auth::{Auth, AuthSource};
    use crate::config::settings::ApiMode;
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn client(server: &MockServer) -> SteelClient {
        let auth = Auth {
            api_key: Some("test-key".into()),
            source: AuthSource::Env,
        };
        SteelClient::with(server.uri(), ApiMode::Local, auth).unwrap()
    }

    const fn target(url: &str) -> Target<'_> {
        Target {
            url,
            delay: None,
            use_proxy: false,
            region: None,
        }
    }

    #[test]
    fn target_body_includes_only_set_options() {
        assert_eq!(
            target("https://example.com").body(),
            json!({"url": "https://example.com"})
        );
        let full = Target {
            url: "https://example.com",
            delay: Some(500),
            use_proxy: true,
            region: Some("lax"),
        };
        assert_eq!(
            full.body(),
            json!({"url": "https://example.com", "delay": 500, "useProxy": true, "region": "lax"})
        );
    }

    #[tokio::test]
    async fn scrape_success() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/scrape"))
            .and(body_json(
                json!({"url": "https://example.com", "format": ["markdown"]}),
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "content": {"markdown": "# Hello"}
            })))
            .mount(&server)
            .await;

        let data = client(&server)
            .scrape(
                &target("https://example.com"),
                &["markdown".into()],
                false,
                false,
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
            .and(body_json(
                json!({"url": "https://example.com", "fullPage": true}),
            ))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"url": "https://cdn.steel.dev/shot.png"})),
            )
            .mount(&server)
            .await;

        let data = client(&server)
            .screenshot(&target("https://example.com"), true)
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

        let data = client(&server)
            .pdf(&target("https://example.com"))
            .await
            .unwrap();

        assert_eq!(
            get_hosted_url(&data).as_deref(),
            Some("https://cdn.steel.dev/doc.pdf")
        );
    }
}
