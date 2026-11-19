//! Platform-independent core of the Veil browser extension.
//!
//! Nothing in this crate touches the browser: the iCloud API is described as
//! data ([`api`]), messages are plain serde types ([`protocol`]), and search
//! and i18n are pure functions. The WASM crates (`hme-web`, `hme-background`,
//! `hme-ui`) do the I/O.

pub mod api;
pub mod i18n;
pub mod model;
pub mod protocol;
pub mod search;
pub mod urls;

/// Keys in `browser.storage.local`.
pub mod storage_keys {
    pub const OPTIONS: &str = "options";
    pub const CLIENT_STATE: &str = "clientState";
    /// Setup URL of the last successful sign-in; kept after sign-out so the
    /// "Open iCloud" links point at the right region (.com / .com.cn).
    pub const LAST_SETUP_URL: &str = "lastSetupUrl";
    /// `storage.session` (memory-only) cache of the address list, so the
    /// popup can render it before iCloud answers.
    pub const LIST_CACHE: &str = "listCache";
    /// `storage.session` cache of the proposed-but-unreserved address, so
    /// reopening the popup shows the same proposal instantly.
    pub const CANDIDATE_CACHE: &str = "candidate";
}

/// Id of the right-click menu item.
pub const CONTEXT_MENU_ID: &str = "veil-generate";
