//! Messages exchanged between the extension's contexts.
//!
//! * [`Request`]: content script / popup → background (`runtime.sendMessage`),
//!   answered with a [`Reply`] through `sendResponse`. Content scripts may
//!   only send [`Request::is_allowed_from_content_script`] requests.
//! * [`TabMessage`]: background / popup → content script (`tabs.sendMessage`).
//!
//! The content script is plain JavaScript, so these shapes are the contract:
//! `{ "type": "generate" }`, `{ "type": "reserve", "hme": "…", "label": "…" }` …

use crate::model::ClientState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Request {
    /// Re-check the iCloud session from the browser's cookies.
    SyncAuth {
        #[serde(default)]
        deauth_on_failure: bool,
    },
    /// Generate and reserve an address for the page that sent the message.
    /// The background derives the label from the sender's tab URL, never
    /// from the message, so a page can't choose it.
    CreateForSite,
    /// Sign out of iCloud on this browser.
    SignOut,
}

impl Request {
    /// What a content script (running inside arbitrary web pages) may ask.
    pub fn is_allowed_from_content_script(&self) -> bool {
        matches!(self, Request::SyncAuth { .. } | Request::CreateForSite)
    }
}

/// Answer to a [`Request`]. Unused fields are omitted from the JSON.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reply {
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hme: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// `true` when the failure is "not signed in", so callers can show a CTA.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub signed_out: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authenticated: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_state: Option<ClientState>,
}

impl Reply {
    pub fn ok() -> Self {
        Self {
            ok: true,
            ..Default::default()
        }
    }

    pub fn hme(hme: String) -> Self {
        Self {
            ok: true,
            hme: Some(hme),
            ..Default::default()
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            ok: false,
            error: Some(message.into()),
            ..Default::default()
        }
    }

    pub fn signed_out(message: impl Into<String>) -> Self {
        Self {
            signed_out: true,
            ..Self::error(message)
        }
    }

    pub fn auth(client_state: Option<ClientState>) -> Self {
        Self {
            ok: true,
            authenticated: Some(client_state.is_some()),
            client_state,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    Loading,
    Done,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum TabMessage {
    /// From the popup: put this address into the email fields of the page.
    Autofill { hme: String },
    /// From the context menu: progress of generate + reserve for the
    /// currently focused field.
    ContextFill {
        phase: Phase,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        text: Option<String>,
        /// Also copy `text` to the clipboard when filling.
        #[serde(default)]
        copy: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn request_wire_format() {
        let r: Request = serde_json::from_value(json!({"type": "createForSite"})).unwrap();
        assert_eq!(r, Request::CreateForSite);
        assert!(r.is_allowed_from_content_script());
        let r: Request = serde_json::from_value(json!({"type": "signOut"})).unwrap();
        assert!(!r.is_allowed_from_content_script());
        // Removed requests are rejected rather than silently accepted.
        assert!(serde_json::from_value::<Request>(json!({"type": "reserve", "hme": "a", "label": "b"})).is_err());
        let r: Request =
            serde_json::from_value(json!({"type": "syncAuth", "deauthOnFailure": true})).unwrap();
        assert_eq!(
            r,
            Request::SyncAuth {
                deauth_on_failure: true
            }
        );
        let r: Request = serde_json::from_value(json!({"type": "syncAuth"})).unwrap();
        assert_eq!(
            r,
            Request::SyncAuth {
                deauth_on_failure: false
            }
        );
    }

    #[test]
    fn reply_omits_empty_fields() {
        assert_eq!(
            serde_json::to_value(Reply::hme("a@icloud.com".into())).unwrap(),
            json!({"ok": true, "hme": "a@icloud.com"})
        );
        assert_eq!(
            serde_json::to_value(Reply::signed_out("x")).unwrap(),
            json!({"ok": false, "error": "x", "signedOut": true})
        );
    }

    #[test]
    fn tab_message_wire_format() {
        assert_eq!(
            serde_json::to_value(TabMessage::ContextFill {
                phase: Phase::Done,
                text: Some("a@icloud.com".into()),
                copy: true
            })
            .unwrap(),
            json!({"type": "contextFill", "phase": "done", "text": "a@icloud.com", "copy": true})
        );
    }
}
