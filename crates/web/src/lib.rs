//! Browser runtime shared by the background worker and the UI pages.

pub mod client;
pub mod ext;

use hme_core::api::{ApiError, SETUP_URLS};
use hme_core::urls::is_icloud_service_url;
use hme_core::i18n::{Lang, T};
use hme_core::model::{ClientState, Options};
use hme_core::storage_keys;

pub async fn load_options() -> Options {
    ext::storage_get(storage_keys::OPTIONS).await.unwrap_or_default()
}

pub async fn save_options(options: &Options) -> Result<(), String> {
    ext::storage_set(storage_keys::OPTIONS, Some(options)).await
}

/// The stored session pointer, if it still points at Apple's servers.
pub async fn load_client_state() -> Option<ClientState> {
    ext::storage_get::<ClientState>(storage_keys::CLIENT_STATE)
        .await
        .filter(|s| SETUP_URLS.contains(&s.setup_url.as_str()) && is_icloud_service_url(&s.pms_url))
}

/// Persists only the service URLs: the account's name and Apple ID stay in
/// memory and are never written to disk.
pub async fn save_client_state(state: Option<&ClientState>) -> Result<(), String> {
    let stripped = state.map(|s| ClientState {
        account: None,
        ..s.clone()
    });
    ext::storage_set(storage_keys::CLIENT_STATE, stripped.as_ref()).await
}

pub async fn remember_setup_url(setup_url: &str) {
    if SETUP_URLS.contains(&setup_url) {
        let _ = ext::storage_set(storage_keys::LAST_SETUP_URL, Some(&setup_url.to_owned())).await;
    }
}

/// iCloud.com or iCloud.com.cn, for "Open iCloud" links.
pub async fn icloud_home() -> &'static str {
    match ext::storage_get::<String>(storage_keys::LAST_SETUP_URL).await {
        Some(u) if u.contains("icloud.com.cn") => "https://www.icloud.com.cn/",
        _ => "https://www.icloud.com/",
    }
}

pub fn lang_for(options: &Options) -> Lang {
    Lang::resolve(options.language, &ext::ui_language())
}

/// User-facing message for an API error.
pub fn describe_error(e: &ApiError, lang: Lang) -> String {
    match e {
        e if e.is_auth() => T::ErrSignedOut.get(lang).to_owned(),
        ApiError::NoHmeService => T::ErrNoHmeService.get(lang).to_owned(),
        ApiError::Network(_) => T::ErrNetwork.get(lang).to_owned(),
        ApiError::Rejected(m) => m.clone(),
        other => format!("{}: {other}", T::ErrGeneric.get(lang)),
    }
}
