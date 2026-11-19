//! Browser runtime shared by the background worker and the UI pages.

pub mod client;
pub mod ext;

use hme_core::api::ApiError;
use hme_core::i18n::{Lang, T};
use hme_core::model::{ClientState, Options};
use hme_core::storage_keys;

pub async fn load_options() -> Options {
    ext::storage_get(storage_keys::OPTIONS).await.unwrap_or_default()
}

pub async fn save_options(options: &Options) -> Result<(), String> {
    ext::storage_set(storage_keys::OPTIONS, Some(options)).await
}

pub async fn load_client_state() -> Option<ClientState> {
    ext::storage_get(storage_keys::CLIENT_STATE).await
}

pub async fn save_client_state(state: Option<&ClientState>) -> Result<(), String> {
    ext::storage_set(storage_keys::CLIENT_STATE, state).await
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
