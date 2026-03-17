//! URL normalization with https fallback.
//! Ported from: cli/source/utils/topLevelTools.ts (resolveTopLevelToolUrl, normalizeUrlWithHttpsFallback)

use url::Url;

/// Check if the URL already has a protocol.
fn has_explicit_protocol(input: &str) -> bool {
    let lower = input.to_lowercase();
    lower.contains("://")
        || lower.starts_with("about:")
        || lower.starts_with("data:")
        || lower.starts_with("file:")
        || lower.starts_with("blob:")
        || lower.starts_with("javascript:")
}

/// Check if the input looks like a hostname without protocol.
fn looks_like_host(input: &str) -> bool {
    let host = input.split('/').next().unwrap_or("");
    let lower = host.to_lowercase();

    lower == "localhost"
        || lower.starts_with("localhost:")
        || (lower.starts_with('[') && lower.contains(']'))
        || is_ipv4_like(&lower)
        || host.contains('.')
}

fn is_ipv4_like(s: &str) -> bool {
    // Simple check: digits and dots, optionally with port
    let (host_part, _) = s.split_once(':').unwrap_or((s, ""));
    let parts: Vec<&str> = host_part.split('.').collect();
    parts.len() == 4 && parts.iter().all(|p| p.parse::<u8>().is_ok())
}

/// Normalize a URL by prepending https:// if missing.
/// Matches TS `normalizeUrlWithHttpsFallback()`.
pub fn normalize_url(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() || has_explicit_protocol(trimmed) || !looks_like_host(trimmed) {
        return trimmed.to_string();
    }

    let candidate = format!("https://{trimmed}");
    match Url::parse(&candidate) {
        Ok(_) => candidate,
        Err(_) => trimmed.to_string(),
    }
}

/// Resolve a URL from positional arg or --url flag.
/// Matches TS `resolveTopLevelToolUrl()`.
pub fn resolve_tool_url(url_arg: Option<&str>) -> anyhow::Result<String> {
    let candidate = url_arg
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Missing URL. Provide a target URL as the first argument."
            )
        })?;

    let normalized = normalize_url(candidate);

    // Validate it's a proper URL
    Url::parse(&normalized).map_err(|_| anyhow::anyhow!("Invalid URL: {candidate}"))?;

    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_adds_https() {
        assert_eq!(normalize_url("example.com"), "https://example.com");
    }

    #[test]
    fn normalize_preserves_https() {
        assert_eq!(normalize_url("https://example.com"), "https://example.com");
    }

    #[test]
    fn normalize_preserves_http() {
        assert_eq!(normalize_url("http://example.com"), "http://example.com");
    }

    #[test]
    fn normalize_localhost() {
        assert_eq!(normalize_url("localhost:3000"), "https://localhost:3000");
    }

    #[test]
    fn normalize_ip_address() {
        assert_eq!(
            normalize_url("192.168.1.1:8080"),
            "https://192.168.1.1:8080"
        );
    }

    #[test]
    fn normalize_with_path() {
        assert_eq!(
            normalize_url("example.com/path"),
            "https://example.com/path"
        );
    }

    #[test]
    fn normalize_empty_returns_empty() {
        assert_eq!(normalize_url(""), "");
    }

    #[test]
    fn normalize_trims_whitespace() {
        assert_eq!(normalize_url("  example.com  "), "https://example.com");
    }

    #[test]
    fn normalize_preserves_about() {
        assert_eq!(normalize_url("about:blank"), "about:blank");
    }

    #[test]
    fn normalize_no_dot_passthrough() {
        // "foobar" doesn't look like a host
        assert_eq!(normalize_url("foobar"), "foobar");
    }

    // --- resolve_tool_url ---

    #[test]
    fn resolve_from_arg() {
        let url = resolve_tool_url(Some("example.com")).unwrap();
        assert_eq!(url, "https://example.com");
    }

    #[test]
    fn resolve_missing_url_error() {
        let err = resolve_tool_url(None).unwrap_err();
        assert!(err.to_string().contains("Missing URL"));
    }

    #[test]
    fn resolve_empty_url_error() {
        let err = resolve_tool_url(Some("  ")).unwrap_err();
        assert!(err.to_string().contains("Missing URL"));
    }
}
