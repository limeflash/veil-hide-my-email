//! Small URL helpers. Hand-rolled on purpose: pulling in the `url` crate
//! (with IDNA tables) would add ~100 KB to every WASM module for three
//! string operations.

use crate::api::{CN_SETUP_URL, DEFAULT_SETUP_URL, SETUP_URLS};

/// Host of an absolute `http(s)` URL, without port or credentials.
pub fn hostname(url: &str) -> Option<String> {
    let (scheme, rest) = url.split_once("://")?;
    if !matches!(scheme.to_ascii_lowercase().as_str(), "http" | "https") {
        return None;
    }
    let authority = rest.split(['/', '?', '#']).next()?;
    let host_port = authority.rsplit('@').next()?;
    let host = if let Some(stripped) = host_port.strip_prefix('[') {
        // IPv6 literal
        format!("[{}]", stripped.split(']').next()?)
    } else {
        host_port.split(':').next()?.to_owned()
    };
    (!host.is_empty()).then(|| host.to_ascii_lowercase())
}

/// `example.com` from `www.example.com`; used as the default label.
pub fn label_for_host(host: &str) -> String {
    host.strip_prefix("www.").unwrap_or(host).to_owned()
}

pub fn is_icloud_host(host: &str) -> bool {
    let h = host.to_ascii_lowercase();
    ["icloud.com", "icloud.com.cn"]
        .iter()
        .any(|d| h == *d || h.ends_with(&format!(".{d}")))
}

/// Setup URL matching the iCloud site a tab is on.
pub fn setup_url_for_page(url: &str) -> &'static str {
    match hostname(url) {
        Some(h) if h.ends_with("icloud.com.cn") => CN_SETUP_URL,
        _ => DEFAULT_SETUP_URL,
    }
}

/// Extracts `https://setup.icloud.com/setup/ws/1` from the URL of an
/// `accountLogin` / `validate` / `refreshWebAuth` request made by icloud.com.
pub fn setup_url_from_auth_request(url: &str) -> Option<String> {
    let without_query = url.split(['?', '#']).next()?;
    let lower = without_query.to_ascii_lowercase();
    let root = ["https://setup.icloud.com.cn/setup/ws/", "https://setup.icloud.com/setup/ws/"]
        .into_iter()
        .find(|p| lower.starts_with(p))?;
    let tail = &without_query[root.len()..];
    let (version, action) = tail.split_once('/')?;
    if version.is_empty() || !version.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let action = action.trim_end_matches('/').to_ascii_lowercase();
    matches!(action.as_str(), "accountlogin" | "validate" | "refreshwebauth")
        .then(|| format!("{root}{version}"))
}

/// Setup URLs to try, the preferred one first, without duplicates.
pub fn candidate_setup_urls(preferred: Option<&str>) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(3);
    for u in preferred.into_iter().chain(SETUP_URLS) {
        if !u.is_empty() && !out.iter().any(|o| o == u) {
            out.push(u.to_owned());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hostnames() {
        assert_eq!(hostname("https://www.GitHub.com/login?x=1").as_deref(), Some("www.github.com"));
        assert_eq!(hostname("http://user:pw@host.io:8080/a").as_deref(), Some("host.io"));
        assert_eq!(hostname("https://[::1]:3000/").as_deref(), Some("[::1]"));
        assert_eq!(hostname("chrome://extensions"), None);
        assert_eq!(hostname("about:blank"), None);
        assert_eq!(label_for_host("www.github.com"), "github.com");
    }

    #[test]
    fn icloud_hosts() {
        assert!(is_icloud_host("www.icloud.com"));
        assert!(is_icloud_host("icloud.com.cn"));
        assert!(!is_icloud_host("noticloud.com"));
        assert!(!is_icloud_host("icloud.com.evil.org"));
        assert_eq!(setup_url_for_page("https://www.icloud.com.cn/mail"), CN_SETUP_URL);
        assert_eq!(setup_url_for_page("https://www.icloud.com/"), DEFAULT_SETUP_URL);
    }

    #[test]
    fn auth_request_urls() {
        assert_eq!(
            setup_url_from_auth_request(
                "https://setup.icloud.com/setup/ws/1/accountLogin?clientBuildNumber=1"
            )
            .as_deref(),
            Some(DEFAULT_SETUP_URL)
        );
        assert_eq!(
            setup_url_from_auth_request("https://setup.icloud.com.cn/setup/ws/1/validate").as_deref(),
            Some(CN_SETUP_URL)
        );
        assert_eq!(setup_url_from_auth_request("https://setup.icloud.com/setup/ws/1/logout"), None);
        assert_eq!(setup_url_from_auth_request("https://evil.com/setup/ws/1/validate"), None);
    }

    #[test]
    fn candidates() {
        assert_eq!(candidate_setup_urls(None), vec![DEFAULT_SETUP_URL, CN_SETUP_URL]);
        assert_eq!(
            candidate_setup_urls(Some(CN_SETUP_URL)),
            vec![CN_SETUP_URL, DEFAULT_SETUP_URL]
        );
    }
}
