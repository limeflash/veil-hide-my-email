//! Data model shared by every part of the extension.
//!
//! Field names mirror the JSON returned by iCloud's `premiummailsettings`
//! web service, so the structs can be deserialized straight from responses
//! and stored as-is in `storage.local`.

use serde::{Deserialize, Serialize};

/// One Hide My Email address as returned by `/v2/hme/list` and `/v1/hme/reserve`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HmeEmail {
    pub anonymous_id: String,
    pub hme: String,
    #[serde(default)]
    pub origin: String,
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub forward_to_email: String,
    #[serde(default)]
    pub is_active: bool,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub note: String,
    /// Milliseconds since the Unix epoch.
    #[serde(default)]
    pub create_timestamp: f64,
    #[serde(default)]
    pub recipient_mail_id: String,
}

/// Result of `/v2/hme/list`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HmeList {
    #[serde(default)]
    pub hme_emails: Vec<HmeEmail>,
    #[serde(default)]
    pub selected_forward_to: String,
    #[serde(default)]
    pub forward_to_emails: Vec<String>,
}

impl HmeList {
    /// Newest first, the order every list in the UI uses.
    pub fn sort_newest_first(&mut self) {
        self.hme_emails
            .sort_by(|a, b| b.create_timestamp.total_cmp(&a.create_timestamp));
    }

    pub fn count_active(&self) -> usize {
        self.hme_emails.iter().filter(|e| e.is_active).count()
    }
}

/// Who is signed in, as reported by `/validate` (`dsInfo`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    #[serde(default)]
    pub full_name: String,
    #[serde(default)]
    pub apple_id: String,
}

/// Everything needed to talk to iCloud without re-validating first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientState {
    /// e.g. `https://setup.icloud.com/setup/ws/1`
    pub setup_url: String,
    /// Base URL of the `premiummailsettings` web service.
    pub pms_url: String,
    #[serde(default)]
    pub account: Option<Account>,
}

impl ClientState {
    pub fn is_china(&self) -> bool {
        self.setup_url.contains("icloud.com.cn")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Autofill {
    /// Show the in-field button when an email input gets focus.
    #[serde(default = "yes")]
    pub button: bool,
    /// Show the "generate" item in the right-click menu of editable fields.
    #[serde(default = "yes")]
    pub context_menu: bool,
}

fn yes() -> bool {
    true
}

impl Default for Autofill {
    fn default() -> Self {
        Self {
            button: true,
            context_menu: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

impl Theme {
    /// Value for `<html data-theme>`; `None` means "follow the OS".
    pub fn attr(self) -> Option<&'static str> {
        match self {
            Theme::System => None,
            Theme::Light => Some("light"),
            Theme::Dark => Some("dark"),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LanguagePref {
    #[default]
    Auto,
    Ru,
    En,
}

/// User settings, stored under [`crate::storage_keys::OPTIONS`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    #[serde(default)]
    pub autofill: Autofill,
    #[serde(default)]
    pub theme: Theme,
    #[serde(default)]
    pub language: LanguagePref,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hme_email_from_icloud_json() {
        let json = r#"{
            "origin": "ON_DEMAND",
            "anonymousId": "abc123",
            "domain": "",
            "forwardToEmail": "me@example.com",
            "hme": "quiet_plover.4k@icloud.com",
            "isActive": true,
            "label": "github.com",
            "note": "",
            "createTimestamp": 1700000000000,
            "recipientMailId": ""
        }"#;
        let e: HmeEmail = serde_json::from_str(json).unwrap();
        assert_eq!(e.anonymous_id, "abc123");
        assert!(e.is_active);
        assert_eq!(e.create_timestamp, 1_700_000_000_000.0);
    }

    #[test]
    fn options_tolerate_partial_and_legacy_data() {
        let o: Options = serde_json::from_str(r#"{"autofill":{"button":false}}"#).unwrap();
        assert!(!o.autofill.button);
        assert!(o.autofill.context_menu);
        assert_eq!(o.theme, Theme::System);
        let o: Options = serde_json::from_str("{}").unwrap();
        assert_eq!(o, Options::default());
    }

    #[test]
    fn list_sorts_newest_first() {
        let mk = |id: &str, ts: f64| HmeEmail {
            anonymous_id: id.into(),
            hme: format!("{id}@icloud.com"),
            origin: String::new(),
            domain: String::new(),
            forward_to_email: String::new(),
            is_active: true,
            label: id.into(),
            note: String::new(),
            create_timestamp: ts,
            recipient_mail_id: String::new(),
        };
        let mut l = HmeList {
            hme_emails: vec![mk("a", 1.0), mk("b", 3.0), mk("c", 2.0)],
            ..Default::default()
        };
        l.sort_newest_first();
        let ids: Vec<_> = l.hme_emails.iter().map(|e| e.anonymous_id.as_str()).collect();
        assert_eq!(ids, ["b", "c", "a"]);
    }
}
