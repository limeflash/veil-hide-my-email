//! Background service worker logic.
//!
//! `background.js` registers every listener synchronously (a requirement for
//! MV3 service workers) and forwards events here once the WASM module is
//! ready. Each `#[wasm_bindgen]` export below handles one browser event.

use futures::future::{FutureExt, LocalBoxFuture, Shared};
use hme_core::api::ApiError;
use hme_core::i18n::{Lang, T};
use hme_core::model::ClientState;
use hme_core::protocol::{Phase, Reply, Request, TabMessage};
use hme_core::urls;
use hme_core::CONTEXT_MENU_ID;
use hme_web::client::{self, Hme};
use hme_web::ext;
use std::cell::RefCell;
use std::future::Future;
use wasm_bindgen::prelude::*;

type SyncResult = Result<ClientState, ApiError>;

thread_local! {
    /// The in-flight session check, shared by every caller that asks while
    /// it runs (icloud.com fires several auth requests per page load).
    static SYNC: RefCell<Option<Shared<LocalBoxFuture<'static, SyncResult>>>> = const { RefCell::new(None) };
}

async fn lang() -> Lang {
    hme_web::lang_for(&hme_web::load_options().await)
}

#[derive(Clone, Copy, Default)]
struct SyncOpts {
    /// Show the "Veil is ready" notification on the first successful sign-in.
    notify: bool,
    /// Forget the stored session when iCloud says it's gone.
    deauth_on_failure: bool,
}

/// Re-reads the iCloud session from the browser's cookies.
async fn sync_auth(setup_url: Option<String>, opts: SyncOpts) -> SyncResult {
    let shared = SYNC.with(|s| {
        s.borrow_mut()
            .get_or_insert_with(|| run_sync(setup_url).boxed_local().shared())
            .clone()
    });
    let result = shared.await;

    match &result {
        Ok(state) => {
            hme_web::remember_setup_url(&state.setup_url).await;
            let previous = hme_web::load_client_state().await;
            let same = previous
                .as_ref()
                .is_some_and(|p| p.setup_url == state.setup_url && p.pms_url == state.pms_url);
            if !same {
                let _ = hme_web::save_client_state(Some(state)).await;
            }
            refresh_menu().await;
            if opts.notify && previous.is_none() {
                let l = lang().await;
                ext::notify(T::NotifyReadyTitle.get(l), T::NotifyReadyBody.get(l)).await;
            }
        }
        // Only a definite answer signs out; offline, 5xx or an odd response
        // keep the session we have.
        Err(e) if opts.deauth_on_failure && (e.is_auth() || *e == ApiError::NoHmeService) => deauth().await,
        Err(_) => {}
    }
    result
}

async fn run_sync(setup_url: Option<String>) -> SyncResult {
    let preferred = match setup_url {
        Some(u) => Some(u),
        None => hme_web::load_client_state().await.map(|s| s.setup_url),
    };
    let result = client::authenticate_with_cookies(preferred.as_deref()).await;
    // Done: the next caller starts a fresh check.
    SYNC.with(|s| s.borrow_mut().take());
    result
}

async fn deauth() {
    let _ = hme_web::save_client_state(None).await;
    ext::session_set::<()>(hme_core::storage_keys::LIST_CACHE, None).await;
    ext::session_set::<()>(hme_core::storage_keys::CANDIDATE_CACHE, None).await;
    refresh_menu().await;
}

/// Updates the right-click item to match the sign-in state and settings.
async fn refresh_menu() {
    let options = hme_web::load_options().await;
    let l = hme_web::lang_for(&options);
    let signed_in = hme_web::load_client_state().await.is_some();
    let title = if signed_in { T::MenuGenerate } else { T::MenuSignedOut };
    ext::menu_update(CONTEXT_MENU_ID, title.get(l), signed_in, options.autofill.context_menu).await;
}

/// (Re)creates the right-click item, then fills in the real state.
async fn setup_menu() {
    let options = hme_web::load_options().await;
    let l = hme_web::lang_for(&options);
    ext::menu_recreate(CONTEXT_MENU_ID, T::MenuLoading.get(l), false, options.autofill.context_menu).await;
    refresh_menu().await;
}

enum Failure {
    SignedOut,
    Api(ApiError),
}

impl Failure {
    fn reply(self, l: Lang) -> Reply {
        match self {
            Failure::SignedOut => Reply::signed_out(T::ErrSignedOut.get(l)),
            Failure::Api(e) if e.is_auth() => Reply::signed_out(T::ErrSignedOut.get(l)),
            Failure::Api(e) => Reply::error(hme_web::describe_error(&e, l)),
        }
    }
}

/// Runs `f` with the stored session; if iCloud rejects it, re-syncs from the
/// cookies once and retries.
async fn with_session<T, F, Fut>(f: F) -> Result<T, Failure>
where
    F: Fn(Hme) -> Fut,
    Fut: Future<Output = Result<T, ApiError>>,
{
    let deauth_opts = SyncOpts {
        notify: false,
        deauth_on_failure: true,
    };
    let state = match hme_web::load_client_state().await {
        Some(s) => s,
        None => sync_auth(None, deauth_opts).await.map_err(|_| Failure::SignedOut)?,
    };
    match f(Hme::new(state.clone())).await {
        Err(e) if e.is_auth() => {
            let fresh = sync_auth(Some(state.setup_url), deauth_opts)
                .await
                .map_err(|_| Failure::SignedOut)?;
            f(Hme::new(fresh)).await.map_err(Failure::Api)
        }
        other => other.map_err(Failure::Api),
    }
}

/// Who sent a message. Content scripts run inside web pages and are treated
/// as untrusted; extension pages (popup, options, welcome) are trusted.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Sender {
    ExtensionPage,
    ContentScript,
}

async fn handle(req: Request, sender: Sender, tab_url: Option<String>) -> Reply {
    let l = lang().await;
    if sender == Sender::ContentScript && !req.is_allowed_from_content_script() {
        return Reply::error("not allowed");
    }
    match req {
        Request::SyncAuth { deauth_on_failure } => {
            let opts = SyncOpts {
                notify: true,
                // Pages must not be able to drop the stored session.
                deauth_on_failure: deauth_on_failure && sender == Sender::ExtensionPage,
            };
            // From an icloud.com(.cn) page: check that region first.
            let region = tab_url
                .as_deref()
                .filter(|u| urls::hostname(u).is_some_and(|h| urls::is_icloud_host(&h)))
                .map(|u| urls::setup_url_for_page(u).to_owned());
            let reply = match sync_auth(region, opts).await {
                Ok(state) => Reply::auth(Some(state)),
                Err(ApiError::Network(_)) => Reply {
                    error: Some(T::ErrNetwork.get(l).to_owned()),
                    ..Reply::auth(hme_web::load_client_state().await)
                },
                Err(ApiError::NoHmeService) => Reply {
                    error: Some(T::ErrNoHmeService.get(l).to_owned()),
                    ..Reply::auth(None)
                },
                Err(_) => Reply::auth(None),
            };
            match sender {
                ExtensionPage => reply,
                // Content scripts learn only whether a session exists.
                Sender::ContentScript => Reply {
                    client_state: None,
                    error: None,
                    ..reply
                },
            }
        }
        Request::CreateForSite => {
            let Some(label) = tab_url
                .as_deref()
                .and_then(urls::hostname)
                .map(|h| urls::label_for_host(&h))
            else {
                return Reply::error(T::ErrGeneric.get(l));
            };
            match create_and_reserve(label).await {
                Ok(hme) => Reply::hme(hme),
                Err(f) => f.reply(l),
            }
        }
        Request::SignOut => {
            if let Some(state) = hme_web::load_client_state().await {
                client::sign_out(&state.setup_url).await;
            }
            deauth().await;
            Reply::ok()
        }
    }
}

use Sender::ExtensionPage;

/// Generates a fresh address and reserves it under `label`.
async fn create_and_reserve(label: String) -> Result<String, Failure> {
    with_session(|h| {
        let label = label.clone();
        async move {
            let hme = h.generate().await?;
            h.reserve(&hme, &label, None).await.map(|r| r.hme)
        }
    })
    .await
}

// ---------------------------------------------------------------------------
// Exports called from background.js

/// `runtime.onMessage`. `from_extension_page` is decided in background.js
/// from `sender.url`; `tab_url` is the sender tab's top-level URL.
#[wasm_bindgen]
pub async fn on_message(message: JsValue, from_extension_page: bool, tab_url: Option<String>) -> JsValue {
    let sender = if from_extension_page { Sender::ExtensionPage } else { Sender::ContentScript };
    let reply = match ext::from_js::<Request>(message) {
        Ok(req) => handle(req, sender, tab_url).await,
        Err(_) => Reply::error("bad message"),
    };
    ext::to_js(&reply)
}

/// Runs every time the worker / event page starts. No network here: the
/// worker restarts often, and install/startup/icloud.com events already
/// cover session checks.
#[wasm_bindgen]
pub async fn boot() {
    // Chromium persists context menus across restarts; Firefox doesn't.
    if ext::is_firefox() {
        setup_menu().await;
    }
}

/// `runtime.onInstalled`
#[wasm_bindgen]
pub async fn on_installed(reason: String) {
    setup_menu().await;
    if reason == "install" {
        ext::open_tab(&ext::get_url("welcome.html")).await;
    }
    if reason == "install" || reason == "update" {
        let _ = sync_auth(
            None,
            SyncOpts {
                notify: true,
                deauth_on_failure: true,
            },
        )
        .await;
    }
}

/// `runtime.onStartup`
#[wasm_bindgen]
pub async fn on_startup() {
    let _ = sync_auth(None, SyncOpts { notify: true, ..Default::default() }).await;
}

/// `storage.onChanged` for the options key.
#[wasm_bindgen]
pub async fn on_options_changed() {
    refresh_menu().await;
}

/// `contextMenus.onClicked`: generate, reserve and fill the focused field.
#[wasm_bindgen]
pub async fn on_menu_click(menu_item_id: String, page_url: Option<String>, tab_id: Option<i32>, frame_id: Option<i32>) {
    if menu_item_id != CONTEXT_MENU_ID {
        return;
    }
    let Some(tab_id) = tab_id else { return };
    let l = lang().await;
    let send = |phase: Phase, text: Option<String>, copy: bool| async move {
        ext::send_to_tab(tab_id, &TabMessage::ContextFill { phase, text, copy }, frame_id).await
    };
    // Only reserve if a content script is there to receive the address
    // (tabs opened before install/update have none).
    let ack = send(Phase::Loading, None, false).await;
    let acked = js_sys::Reflect::get(&ack, &"ok".into())
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if !acked {
        ext::notify(T::AppName.get(l), T::ErrReloadPage.get(l)).await;
        return;
    }

    let label = page_url
        .as_deref()
        .and_then(urls::hostname)
        .map(|h| urls::label_for_host(&h))
        .unwrap_or_default();
    match create_and_reserve(label).await {
        Ok(hme) => {
            send(Phase::Done, Some(hme), true).await;
        }
        Err(f) => {
            send(Phase::Error, f.reply(l).error, false).await;
        }
    }
}

/// `webRequest.onResponseStarted` for icloud.com's accountLogin / validate /
/// refreshWebAuth: the user just signed in (or icloud.com refreshed the
/// session), so pick it up without waiting for the popup.
#[wasm_bindgen]
pub async fn on_auth_response(url: String, status: u16, from_extension: bool) {
    if !(200..300).contains(&status) || from_extension {
        return;
    }
    let setup = urls::setup_url_from_auth_request(&url);
    let _ = sync_auth(setup, SyncOpts { notify: true, ..Default::default() }).await;
}

/// `webRequest.onResponseStarted` for `/logout`: the user probably signed
/// out on icloud.com. Re-check instead of trusting it, since any page can
/// make the browser request that URL.
#[wasm_bindgen]
pub async fn on_logout_response(status: u16) {
    if (200..300).contains(&status) {
        let _ = sync_auth(
            None,
            SyncOpts {
                notify: false,
                deauth_on_failure: true,
            },
        )
        .await;
    }
}

/// Localized strings for the in-page button, so the content script doesn't
/// need its own dictionary. Returned as `{ key: text }`.
#[wasm_bindgen]
pub async fn inpage_strings() -> JsValue {
    let l = lang().await;
    let pairs = [
        ("clickToFill", T::InpageClickToFill),
        ("generating", T::InpageGenerating),
        ("filled", T::InpageFilled),
        ("signedOut", T::ErrSignedOut),
        ("appName", T::AppName),
    ];
    let mut map: std::collections::BTreeMap<&str, &str> = pairs.iter().map(|(k, t)| (*k, t.get(l))).collect();
    map.insert("dir", if l.is_rtl() { "rtl" } else { "ltr" });
    ext::to_js(&map)
}
