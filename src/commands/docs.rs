use std::time::Duration;

use clap::{Parser, ValueEnum};
use reqwest::header::LOCATION;
use reqwest::redirect::Policy;
use reqwest::{Client, StatusCode, Url};
use serde_json::json;

use crate::util::error::invalid_input;
use crate::util::output;

const DOCS_ORIGIN: &str = "https://steel.dev";
const MOVED_HOST: &str = "docs.steel.dev";
const MOVED_ORIGIN: &str = "https://docs.steel.dev";
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

fn strip_scheme(value: &str) -> &str {
    strip_prefix_ignore_case(value, "https://")
        .or_else(|| strip_prefix_ignore_case(value, "http://"))
        .unwrap_or(value)
}

fn host_path<'a>(value: &'a str, host: &str) -> Option<&'a str> {
    let path = strip_prefix_ignore_case(value, host)?;
    (path.is_empty() || path.starts_with(['/', '?', '#'])).then_some(path)
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

fn clean_path(path: &str) -> Option<&str> {
    let path = path.split(['?', '#']).next().unwrap_or_default();
    let path = path.trim_matches('/');
    let path = path.strip_suffix(".md").unwrap_or(path);
    (path.is_empty() || path.split('/').all(is_segment)).then_some(path)
}

fn docs_path(path: &str) -> Option<String> {
    let path = clean_path(path)?;
    let path = if path == "docs" {
        ""
    } else {
        path.strip_prefix("docs/").unwrap_or(path)
    };
    Some(path.to_string())
}

#[derive(Debug, PartialEq, Eq)]
enum Target {
    Page(String),
    Moved(String),
}

fn target(input: &str) -> Option<Target> {
    let value = input.trim();
    let rest = strip_scheme(value);
    if let Some(path) = host_path(rest, MOVED_HOST) {
        return clean_path(path).map(|path| Target::Moved(path.to_string()));
    }
    let path = host_path(rest, "steel.dev")
        .or_else(|| host_path(rest, "www.steel.dev"))
        .unwrap_or(value);
    docs_path(path).map(Target::Page)
}

fn page_address(page: &str) -> String {
    match page {
        "" => format!("{DOCS_ORIGIN}/docs"),
        SITEMAP => format!("{DOCS_ORIGIN}/docs/{SITEMAP}.md"),
        _ => format!("{DOCS_ORIGIN}/docs/{page}"),
    }
}

fn http_client(redirect: Policy) -> anyhow::Result<Client> {
    Ok(Client::builder()
        .user_agent(format!("steel-cli/{}", env!("CARGO_PKG_VERSION")))
        .timeout(FETCH_TIMEOUT)
        .redirect(redirect)
        .build()?)
}

async fn resolve_moved(origin: &str, path: &str) -> anyhow::Result<Option<String>> {
    let url = Url::parse(origin)?.join(&format!("/{path}"))?;
    let response = http_client(Policy::none())?.get(url.clone()).send().await?;
    if !response.status().is_redirection() {
        return Ok(None);
    }
    let Some(location) = response
        .headers()
        .get(LOCATION)
        .and_then(|value| value.to_str().ok())
    else {
        return Ok(None);
    };
    let landed = url.join(location)?;
    let docs = landed.path() == "/docs" || landed.path().starts_with("/docs/");
    if landed.origin() != Url::parse(DOCS_ORIGIN)?.origin() || !docs {
        return Ok(None);
    }
    Ok(docs_path(landed.path()).filter(|page| path.is_empty() || !page.is_empty()))
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
    let response = http_client(Policy::default())?
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

fn not_found(message: &str) -> anyhow::Error {
    output::fail(
        "docs_not_found",
        &format!("{message} Run `steel docs` for the list of every page."),
        None,
    )
}

pub async fn run(args: Args) -> anyhow::Result<()> {
    let requested = args.page.as_deref().unwrap_or(SITEMAP);
    let page = match target(requested) {
        Some(Target::Page(page)) => page,
        Some(Target::Moved(path)) => {
            resolve_moved(MOVED_ORIGIN, &path).await?.ok_or_else(|| {
                not_found(&format!(
                    "{MOVED_ORIGIN}/{path} has moved, and steel.dev/docs has no page for it."
                ))
            })?
        }
        None => {
            return Err(invalid_input(format!(
                "{requested} is not a Steel docs page. Pass a path such as sessions/timeouts, or run `steel docs` for the list of every page."
            )));
        }
    };
    let address = page_address(&page);

    let Some(markdown) = fetch(DOCS_ORIGIN, &page, args.lang).await? else {
        return Err(not_found(&format!("There is no docs page at {address}.")));
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

    fn page(value: &str) -> Option<Target> {
        Some(Target::Page(value.to_string()))
    }

    #[test]
    fn target_accepts_paths_and_addresses() {
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
            ("https://www.steel.dev/docs/control/cli", "control/cli"),
            ("www.steel.dev/docs", ""),
            ("https://steel.dev/docs", ""),
            ("steel.dev", ""),
            ("  sitemap  ", "sitemap"),
        ] {
            assert_eq!(target(input), page(expected), "{input}");
        }
    }

    #[test]
    fn target_sends_old_docs_addresses_to_resolve() {
        for (input, expected) in [
            (
                "https://docs.steel.dev/overview/sessions-api/session-lifecycle",
                "overview/sessions-api/session-lifecycle",
            ),
            ("docs.steel.dev/control/cli.md#install", "control/cli"),
            ("https://docs.steel.dev/", ""),
        ] {
            assert_eq!(
                target(input),
                Some(Target::Moved(expected.to_string())),
                "{input}"
            );
        }
    }

    #[test]
    fn target_rejects_other_pages() {
        for input in [
            "https://example.com/docs/x",
            "steel.dev.evil.com/docs",
            "docs.steel.dev.evil.com/x",
            "docs.steel.dev/../x",
            "Sessions/Timeouts",
            "sessions//timeouts",
            "../etc/passwd",
            "sessions/-timeouts",
            "sessions/time--outs",
            "sessions timeouts",
        ] {
            assert_eq!(target(input), None, "{input}");
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
        assert_eq!(page_address(""), "https://steel.dev/docs");
        assert_eq!(page_address(SITEMAP), "https://steel.dev/docs/sitemap.md");
    }

    #[tokio::test]
    async fn resolve_moved_follows_one_redirect_to_steel_dev() {
        let server = MockServer::start().await;
        for (from, to) in [
            (
                "/overview/sessions-api/session-lifecycle",
                "https://steel.dev/docs/sessions/lifecycle",
            ),
            ("/", "https://steel.dev/docs"),
            ("/nope", "https://steel.dev/docs"),
            ("/pricing", "https://steel.dev/pricing"),
            ("/elsewhere", "https://example.com/docs/x"),
            ("/relative", "/docs/control/cli"),
        ] {
            Mock::given(method("GET"))
                .and(path(from))
                .respond_with(ResponseTemplate::new(308).insert_header("location", to))
                .mount(&server)
                .await;
        }
        Mock::given(method("GET"))
            .and(path("/stays"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let origin = server.uri();
        let resolve = |path: &'static str| resolve_moved(&origin, path);
        assert_eq!(
            resolve("overview/sessions-api/session-lifecycle")
                .await
                .unwrap(),
            Some("sessions/lifecycle".to_string())
        );
        assert_eq!(resolve("").await.unwrap(), Some(String::new()));
        assert_eq!(resolve("nope").await.unwrap(), None);
        assert_eq!(resolve("pricing").await.unwrap(), None);
        assert_eq!(resolve("elsewhere").await.unwrap(), None);
        assert_eq!(resolve("relative").await.unwrap(), None);
        assert_eq!(resolve("stays").await.unwrap(), None);
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
