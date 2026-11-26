use std::time::Duration;

use clap::{Parser, ValueEnum};
use reqwest::{StatusCode, Url};
use serde_json::json;

use crate::util::error::invalid_input;
use crate::util::output;

const DOCS_ORIGIN: &str = "https://steel.dev";
const SITEMAP: &str = "sitemap";
const FETCH_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Lang {
    Typescript,
    Python,
    Go,
    Rust,
}

impl Lang {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Typescript => "typescript",
            Self::Python => "python",
            Self::Go => "go",
            Self::Rust => "rust",
        }
    }
}

#[derive(Parser)]
#[command(after_help = "Examples:
  steel docs
  steel docs sessions/timeouts
  steel docs https://steel.dev/docs/control/cli --lang python")]
pub struct Args {
    /// The page's path, such as sessions/timeouts, or its address on steel.dev/docs. Leave it out for the sitemap, which lists every page
    pub page: Option<String>,

    /// Keep only the code in this language
    #[arg(long, value_enum)]
    pub lang: Option<Lang>,
}

fn strip_prefix_ignore_case<'a>(value: &'a str, prefix: &str) -> Option<&'a str> {
    let head = value.get(..prefix.len())?;
    head.eq_ignore_ascii_case(prefix)
        .then(|| &value[prefix.len()..])
}

fn strip_host(value: &str) -> &str {
    let rest = strip_prefix_ignore_case(value, "https://")
        .or_else(|| strip_prefix_ignore_case(value, "http://"))
        .unwrap_or(value);
    match strip_prefix_ignore_case(rest, "steel.dev") {
        Some(path) if path.is_empty() || path.starts_with(['/', '?', '#']) => path,
        _ => value,
    }
}

fn is_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
}

fn docs_page(page: &str) -> Option<String> {
    let path = strip_host(page.trim());
    let path = path.split(['?', '#']).next().unwrap_or_default();
    let path = path.trim_matches('/');
    let path = path.strip_suffix(".md").unwrap_or(path);
    let path = if path == "docs" {
        ""
    } else {
        path.strip_prefix("docs/").unwrap_or(path)
    };
    if path.is_empty() || path.split('/').all(is_segment) {
        Some(path.to_string())
    } else {
        None
    }
}

fn page_address(page: &str) -> String {
    if page.is_empty() {
        format!("{DOCS_ORIGIN}/docs")
    } else {
        format!("{DOCS_ORIGIN}/docs/{page}")
    }
}

fn markdown_url(origin: &str, page: &str, lang: Option<Lang>) -> anyhow::Result<Url> {
    let path = if page.is_empty() {
        "/docs.md".to_string()
    } else {
        format!("/docs/{page}.md")
    };
    let mut url = Url::parse(origin)?.join(&path)?;
    if let Some(lang) = lang {
        url.query_pairs_mut().append_pair("lang", lang.as_str());
    }
    Ok(url)
}

async fn fetch(origin: &str, page: &str, lang: Option<Lang>) -> anyhow::Result<Option<String>> {
    let client = reqwest::Client::builder()
        .user_agent(format!("steel-cli/{}", env!("CARGO_PKG_VERSION")))
        .timeout(FETCH_TIMEOUT)
        .build()?;
    let response = client
        .get(markdown_url(origin, page, lang)?)
        .header(reqwest::header::ACCEPT, "text/markdown")
        .send()
        .await?;
    let status = response.status();
    if status == StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !status.is_success() {
        anyhow::bail!("Could not read {}: HTTP {status}", page_address(page));
    }
    Ok(Some(response.text().await?))
}

pub async fn run(args: Args) -> anyhow::Result<()> {
    let requested = args.page.as_deref().unwrap_or(SITEMAP);
    let page = docs_page(requested).ok_or_else(|| {
        invalid_input(format!(
            "{requested} is not a Steel docs page. Pass a path such as sessions/timeouts, or run `steel docs` for the list of every page."
        ))
    })?;
    let address = page_address(&page);

    let Some(markdown) = fetch(DOCS_ORIGIN, &page, args.lang).await? else {
        return Err(output::fail(
            "docs_not_found",
            &format!(
                "There is no docs page at {address}. Run `steel docs` for the list of every page."
            ),
            None,
        ));
    };

    let text = if markdown.ends_with('\n') {
        markdown.clone()
    } else {
        format!("{markdown}\n")
    };
    output::success(
        json!({ "page": page, "url": address, "markdown": markdown }),
        &text,
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn docs_page_accepts_paths_and_addresses() {
        for (input, expected) in [
            ("sessions/timeouts", "sessions/timeouts"),
            ("/sessions/timeouts/", "sessions/timeouts"),
            ("sessions/timeouts.md", "sessions/timeouts"),
            ("docs/sessions/timeouts", "sessions/timeouts"),
            (
                "https://steel.dev/docs/sessions/timeouts",
                "sessions/timeouts",
            ),
            ("HTTPS://Steel.dev/docs/control/cli#install", "control/cli"),
            ("steel.dev/docs/control/cli.md?lang=python", "control/cli"),
            ("https://steel.dev/docs", ""),
            ("steel.dev", ""),
            ("  sitemap  ", "sitemap"),
        ] {
            assert_eq!(docs_page(input).as_deref(), Some(expected), "{input}");
        }
    }

    #[test]
    fn docs_page_rejects_other_pages() {
        for input in [
            "https://example.com/docs/x",
            "steel.dev.evil.com/docs",
            "Sessions/Timeouts",
            "sessions//timeouts",
            "../etc/passwd",
            "sessions/-timeouts",
            "sessions/time--outs",
            "sessions timeouts",
        ] {
            assert_eq!(docs_page(input), None, "{input}");
        }
    }

    #[test]
    fn markdown_url_points_at_the_markdown_page() {
        assert_eq!(
            markdown_url(DOCS_ORIGIN, "", None).unwrap().as_str(),
            "https://steel.dev/docs.md"
        );
        assert_eq!(
            markdown_url(DOCS_ORIGIN, "control/cli", Some(Lang::Python))
                .unwrap()
                .as_str(),
            "https://steel.dev/docs/control/cli.md?lang=python"
        );
        assert_eq!(
            page_address("control/cli"),
            "https://steel.dev/docs/control/cli"
        );
    }

    #[tokio::test]
    async fn fetch_returns_the_page_or_none_when_missing() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/docs/control/cli.md"))
            .and(query_param("lang", "go"))
            .and(header("accept", "text/markdown"))
            .respond_with(ResponseTemplate::new(200).set_body_string("# CLI\n"))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/docs/missing.md"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/docs/broken.md"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let origin = server.uri();
        assert_eq!(
            fetch(&origin, "control/cli", Some(Lang::Go)).await.unwrap(),
            Some("# CLI\n".to_string())
        );
        assert_eq!(fetch(&origin, "missing", None).await.unwrap(), None);
        let err = fetch(&origin, "broken", None).await.unwrap_err();
        assert!(err.to_string().contains("HTTP 500"), "{err}");
    }
}
