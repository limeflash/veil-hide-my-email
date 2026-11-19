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
}

/// Id of the right-click menu item.
pub const CONTEXT_MENU_ID: &str = "veil-generate";
