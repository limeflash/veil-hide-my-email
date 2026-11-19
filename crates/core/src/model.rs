//! Data model shared by every part of the extension.
//!
//! Field names mirror the JSON returned by iCloud's `premiummailsettings`
//! web service, so the structs can be deserialized straight from responses
//! and stored as-is in `storage.local`.

use serde::{Deserialize, Deserializer, Serialize};

/// `#[serde(default)]` only covers missing fields; iCloud also sends `null`.
fn null_default<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Ok(Option::<T>::deserialize(d)?.unwrap_or_default())
}

/// One Hide My Email address as returned by `/v2/hme/list` and `/v1/hme/reserve`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HmeEmail {
    pub anonymous_id: String,
    pub hme: String,
    #[serde(default, deserialize_with = "null_default")]
    pub origin: String,
    #[serde(default, deserialize_with = "null_default")]
    pub domain: String,
    #[serde(default, deserialize_with = "null_default")]
    pub forward_to_email: String,
    #[serde(default, deserialize_with = "null_default")]
    pub is_active: bool,
    #[serde(default, deserialize_with = "null_default")]
    pub label: String,
    #[serde(default, deserialize_with = "null_default")]
    pub note: String,
    /// Milliseconds since the Unix epoch.
    #[serde(default, deserialize_with = "null_default")]
    pub create_timestamp: f64,
    #[serde(default, deserialize_with = "null_default")]
    pub recipient_mail_id: String,
}

/// Result of `/v2/hme/list`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HmeList {
    #[serde(default, deserialize_with = "null_default")]
    pub hme_emails: Vec<HmeEmail>,
    #[serde(default, deserialize_with = "null_default")]
    pub selected_forward_to: String,
    #[serde(default, deserialize_with = "null_default")]
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
    #[serde(default, deserialize_with = "null_default")]
    pub full_name: String,
    #[serde(default, deserialize_with = "null_default")]
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
    En,
    Ru,
    Es,
    De,
    Fr,
    Tr,
    Ar,
    Fa,
    Zh,
    Ja,
    Ko,
    Uk,
}

impl LanguagePref {
    /// The fixed language, or `None` for "follow the browser".
    pub fn lang(self) -> Option<crate::i18n::Lang> {
        use crate::i18n::Lang;
        Some(match self {
            LanguagePref::Auto => return None,
            LanguagePref::En => Lang::En,
            LanguagePref::Ru => Lang::Ru,
            LanguagePref::Es => Lang::Es,
            LanguagePref::De => Lang::De,
            LanguagePref::Fr => Lang::Fr,
            LanguagePref::Tr => Lang::Tr,
            LanguagePref::Ar => Lang::Ar,
            LanguagePref::Fa => Lang::Fa,
            LanguagePref::Zh => Lang::Zh,
            LanguagePref::Ja => Lang::Ja,
            LanguagePref::Ko => Lang::Ko,
            LanguagePref::Uk => Lang::Uk,
        })
    }

    pub fn from_lang(lang: crate::i18n::Lang) -> Self {
        use crate::i18n::Lang;
        match lang {
            Lang::En => LanguagePref::En,
            Lang::Ru => LanguagePref::Ru,
            Lang::Es => LanguagePref::Es,
            Lang::De => LanguagePref::De,
            Lang::Fr => LanguagePref::Fr,
            Lang::Tr => LanguagePref::Tr,
            Lang::Ar => LanguagePref::Ar,
            Lang::Fa => LanguagePref::Fa,
            Lang::Zh => LanguagePref::Zh,
            Lang::Ja => LanguagePref::Ja,
            Lang::Ko => LanguagePref::Ko,
            Lang::Uk => LanguagePref::Uk,
        }
    }
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
    fn nulls_are_tolerated() {
        let e: HmeEmail = serde_json::from_str(
            r#"{"anonymousId":"a","hme":"x@icloud.com","note":null,"label":null,"isActive":null,"createTimestamp":null}"#,
        )
        .unwrap();
        assert_eq!(e.note, "");
        assert!(!e.is_active);
        let a: Account = serde_json::from_str(r#"{"fullName":null}"#).unwrap();
        assert_eq!(a.full_name, "");
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
