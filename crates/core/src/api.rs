//! iCloud endpoints, described as plain data.
//!
//! The core never performs I/O: it builds an [`ApiRequest`] and parses the
//! response body, while `hme-web` executes the request with `fetch`. That keeps
//! everything here testable with `cargo test`.

use crate::model::{Account, HmeEmail, HmeList};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::json;
use std::fmt;

pub const DEFAULT_SETUP_URL: &str = "https://setup.icloud.com/setup/ws/1";
pub const CN_SETUP_URL: &str = "https://setup.icloud.com.cn/setup/ws/1";
pub const SETUP_URLS: [&str; 2] = [DEFAULT_SETUP_URL, CN_SETUP_URL];

/// Note attached to addresses reserved without a user-written note.
pub const DEFAULT_NOTE: &str = "Generated through the Veil browser extension";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
}

impl Method {
    pub fn as_str(self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiRequest {
    pub method: Method,
    pub url: String,
    /// JSON body. Sent without an explicit content type, like icloud.com does.
    pub body: Option<String>,
}

impl ApiRequest {
    fn get(url: String) -> Self {
        Self {
            method: Method::Get,
            url,
            body: None,
        }
    }

    fn post(url: String, body: Option<serde_json::Value>) -> Self {
        Self {
            method: Method::Post,
            url,
            body: body.map(|b| b.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiError {
    /// The server answered with a non-2xx status.
    Http(u16),
    /// `fetch` itself failed (offline, blocked, CORS…).
    Network(String),
    /// The body was not the JSON we expected.
    Parse(String),
    /// `success: false`, with iCloud's own message when it sent one.
    Rejected(String),
    /// `/validate` succeeded but the account has no Hide My Email service.
    NoHmeService,
}

impl ApiError {
    /// 401/403/421 mean the iCloud session is gone.
    pub fn is_auth(&self) -> bool {
        matches!(self, ApiError::Http(401 | 403 | 421))
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Http(code) => write!(f, "iCloud responded with HTTP {code}"),
            ApiError::Network(m) => write!(f, "Network error: {m}"),
            ApiError::Parse(m) => write!(f, "Unexpected response from iCloud: {m}"),
            ApiError::Rejected(m) => f.write_str(m),
            ApiError::NoHmeService => {
                f.write_str("Hide My Email is not available for this iCloud account")
            }
        }
    }
}

impl std::error::Error for ApiError {}

// ---------------------------------------------------------------------------
// setup.icloud.com

pub fn validate(setup_url: &str) -> ApiRequest {
    ApiRequest::post(format!("{setup_url}/validate"), None)
}

pub fn logout(setup_url: &str, trust: bool) -> ApiRequest {
    ApiRequest::post(
        format!("{setup_url}/logout"),
        Some(json!({ "trustBrowsers": trust, "allBrowsers": trust })),
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidateInfo {
    pub pms_url: String,
    pub account: Option<Account>,
}

pub fn parse_validate(body: &str) -> Result<ValidateInfo, ApiError> {
    #[derive(Deserialize)]
    struct Service {
        url: String,
    }
    #[derive(Deserialize)]
    struct Webservices {
        premiummailsettings: Option<Service>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Validate {
        webservices: Option<Webservices>,
        ds_info: Option<Account>,
    }

    let v: Validate = parse_json(body)?;
    let pms_url = v
        .webservices
        .and_then(|w| w.premiummailsettings)
        .map(|s| s.url.trim_end_matches('/').to_owned())
        .filter(|u| !u.is_empty())
        .ok_or(ApiError::NoHmeService)?;
    Ok(ValidateInfo {
        pms_url,
        account: v.ds_info,
    })
}

// ---------------------------------------------------------------------------
// premiummailsettings

/// Builder for the `premiummailsettings` endpoints of one account.
#[derive(Debug, Clone, Copy)]
pub struct Pms<'a> {
    base: &'a str,
}

impl<'a> Pms<'a> {
    pub fn new(pms_url: &'a str) -> Self {
        Self { base: pms_url }
    }

    fn v1(&self, path: &str) -> String {
        format!("{}/v1/hme/{path}", self.base)
    }

    pub fn list(&self) -> ApiRequest {
        ApiRequest::get(format!("{}/v2/hme/list", self.base))
    }

    pub fn generate(&self) -> ApiRequest {
        ApiRequest::post(self.v1("generate"), None)
    }

    pub fn reserve(&self, hme: &str, label: &str, note: Option<&str>) -> ApiRequest {
        let note = note.filter(|n| !n.trim().is_empty()).unwrap_or(DEFAULT_NOTE);
        ApiRequest::post(
            self.v1("reserve"),
            Some(json!({ "hme": hme, "label": label, "note": note })),
        )
    }

    pub fn update_metadata(&self, anonymous_id: &str, label: &str, note: &str) -> ApiRequest {
        ApiRequest::post(
            self.v1("updateMetaData"),
            Some(json!({ "anonymousId": anonymous_id, "label": label, "note": note })),
        )
    }

    pub fn deactivate(&self, anonymous_id: &str) -> ApiRequest {
        ApiRequest::post(
            self.v1("deactivate"),
            Some(json!({ "anonymousId": anonymous_id })),
        )
    }

    pub fn reactivate(&self, anonymous_id: &str) -> ApiRequest {
        ApiRequest::post(
            self.v1("reactivate"),
            Some(json!({ "anonymousId": anonymous_id })),
        )
    }

    pub fn delete(&self, anonymous_id: &str) -> ApiRequest {
        ApiRequest::post(
            self.v1("delete"),
            Some(json!({ "anonymousId": anonymous_id })),
        )
    }

    pub fn update_forward_to(&self, email: &str) -> ApiRequest {
        ApiRequest::post(
            self.v1("updateForwardTo"),
            Some(json!({ "forwardToEmail": email })),
        )
    }
}

#[derive(Deserialize)]
struct Envelope<T> {
    #[serde(default)]
    success: bool,
    result: Option<T>,
    error: Option<ErrorBody>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ErrorBody {
    error_message: Option<String>,
}

fn parse_json<T: DeserializeOwned>(body: &str) -> Result<T, ApiError> {
    serde_json::from_str(body).map_err(|e| ApiError::Parse(e.to_string()))
}

fn unwrap_envelope<T: DeserializeOwned>(body: &str, what: &str) -> Result<Option<T>, ApiError> {
    let env: Envelope<T> = parse_json(body)?;
    if !env.success {
        let msg = env
            .error
            .and_then(|e| e.error_message)
            .filter(|m| !m.is_empty())
            .unwrap_or_else(|| format!("iCloud could not {what}"));
        return Err(ApiError::Rejected(msg));
    }
    Ok(env.result)
}

pub fn parse_list(body: &str) -> Result<HmeList, ApiError> {
    // `list` historically omits `success`, so only the result matters.
    #[derive(Deserialize)]
    struct ListEnvelope {
        result: Option<HmeList>,
    }
    let mut list = parse_json::<ListEnvelope>(body)?
        .result
        .ok_or_else(|| ApiError::Parse("missing result".into()))?;
    list.sort_newest_first();
    Ok(list)
}

pub fn parse_generate(body: &str) -> Result<String, ApiError> {
    #[derive(Deserialize)]
    struct R {
        hme: String,
    }
    unwrap_envelope::<R>(body, "generate an address")?
        .map(|r| r.hme)
        .ok_or_else(|| ApiError::Parse("missing hme".into()))
}

pub fn parse_reserve(body: &str) -> Result<HmeEmail, ApiError> {
    #[derive(Deserialize)]
    struct R {
        hme: HmeEmail,
    }
    unwrap_envelope::<R>(body, "reserve the address")?
        .map(|r| r.hme)
        .ok_or_else(|| ApiError::Parse("missing hme".into()))
}

/// For endpoints whose only output is `success`.
pub fn parse_ack(body: &str, what: &str) -> Result<(), ApiError> {
    unwrap_envelope::<serde_json::Value>(body, what).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_urls() {
        let pms = Pms::new("https://p68-maildomainws.icloud.com");
        assert_eq!(
            pms.list().url,
            "https://p68-maildomainws.icloud.com/v2/hme/list"
        );
        let r = pms.reserve("a@icloud.com", "github.com", Some("  "));
        assert_eq!(r.method, Method::Post);
        assert_eq!(r.url, "https://p68-maildomainws.icloud.com/v1/hme/reserve");
        let body: serde_json::Value = serde_json::from_str(r.body.as_deref().unwrap()).unwrap();
        assert_eq!(body["note"], DEFAULT_NOTE);
        assert_eq!(body["label"], "github.com");
        assert_eq!(
            validate(DEFAULT_SETUP_URL).url,
            "https://setup.icloud.com/setup/ws/1/validate"
        );
    }

    #[test]
    fn parses_validate() {
        let body = r#"{"dsInfo":{"fullName":"Jane Doe","appleId":"jane@example.com"},
            "webservices":{"premiummailsettings":{"url":"https://p68-maildomainws.icloud.com:443/","status":"active"}}}"#;
        let v = parse_validate(body).unwrap();
        assert_eq!(v.pms_url, "https://p68-maildomainws.icloud.com:443");
        assert_eq!(v.account.unwrap().full_name, "Jane Doe");
        assert_eq!(
            parse_validate(r#"{"webservices":{}}"#),
            Err(ApiError::NoHmeService)
        );
    }

    #[test]
    fn parses_envelopes() {
        assert_eq!(
            parse_generate(r#"{"success":true,"result":{"hme":"x@icloud.com"}}"#).unwrap(),
            "x@icloud.com"
        );
        assert_eq!(
            parse_generate(r#"{"success":false,"error":{"errorMessage":"Limit reached"}}"#),
            Err(ApiError::Rejected("Limit reached".into()))
        );
        assert!(matches!(
            parse_ack(r#"{"success":false}"#, "delete the address"),
            Err(ApiError::Rejected(m)) if m.contains("delete")
        ));
        assert!(parse_ack(r#"{"success":true}"#, "x").is_ok());
        assert!(matches!(parse_generate("<html>"), Err(ApiError::Parse(_))));
    }

    #[test]
    fn parses_list() {
        let body = r#"{"success":true,"result":{"forwardToEmails":["a@x.com","b@x.com"],
            "selectedForwardTo":"a@x.com","hmeEmails":[
              {"anonymousId":"1","hme":"one@icloud.com","createTimestamp":1},
              {"anonymousId":"2","hme":"two@icloud.com","createTimestamp":2,"isActive":true}]}}"#;
        let l = parse_list(body).unwrap();
        assert_eq!(l.hme_emails[0].anonymous_id, "2");
        assert_eq!(l.count_active(), 1);
        assert_eq!(l.forward_to_emails.len(), 2);
    }
}
