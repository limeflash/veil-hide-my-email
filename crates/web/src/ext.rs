//! Typed bindings to `js/ext.js`.

use js_sys::{Function, Promise};
use serde::de::DeserializeOwned;
use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen(module = "/js/ext.js")]
extern "C" {
    #[wasm_bindgen(js_name = isFirefox)]
    pub fn is_firefox() -> bool;
    #[wasm_bindgen(js_name = uiLanguage)]
    pub fn ui_language() -> String;
    #[wasm_bindgen(js_name = getURL)]
    pub fn get_url(path: &str) -> String;

    #[wasm_bindgen(js_name = storageGet)]
    fn storage_get_raw(key: &str) -> Promise;
    #[wasm_bindgen(js_name = storageSet)]
    fn storage_set_raw(key: &str, value: JsValue) -> Promise;
    #[wasm_bindgen(js_name = onStorageChanged)]
    pub fn on_storage_changed(cb: &Function);

    #[wasm_bindgen(js_name = sendMessage)]
    fn send_message_raw(message: JsValue) -> Promise;
    #[wasm_bindgen(js_name = sendToTab)]
    fn send_to_tab_raw(tab_id: i32, message: JsValue, frame_id: Option<i32>) -> Promise;

    #[wasm_bindgen(js_name = activeTab)]
    fn active_tab_raw() -> Promise;
    #[wasm_bindgen(js_name = openTab)]
    fn open_tab_raw(url: &str) -> Promise;
    #[wasm_bindgen(js_name = openOptionsPage)]
    fn open_options_page_raw() -> Promise;

    #[wasm_bindgen(js_name = menuRecreate)]
    fn menu_recreate_raw(id: &str, title: &str, enabled: bool, visible: bool) -> Promise;
    #[wasm_bindgen(js_name = menuUpdate)]
    fn menu_update_raw(id: &str, title: &str, enabled: bool, visible: bool) -> Promise;

    #[wasm_bindgen(js_name = notify)]
    fn notify_raw(title: &str, message: &str) -> Promise;
    #[wasm_bindgen(js_name = copyText)]
    fn copy_text_raw(text: &str) -> Promise;
    #[wasm_bindgen(js_name = hasHostAccess)]
    fn has_host_access_raw() -> Promise;
    #[wasm_bindgen(js_name = requestHostAccess)]
    fn request_host_access_raw() -> Promise;
}

/// Error from a WebExtension call, as a readable string.
pub fn js_error(e: JsValue) -> String {
    e.dyn_ref::<js_sys::Error>()
        .map(|e| String::from(e.message()))
        .or_else(|| e.as_string())
        .unwrap_or_else(|| format!("{e:?}"))
}

async fn wait(p: Promise) -> Result<JsValue, String> {
    JsFuture::from(p).await.map_err(js_error)
}

/// Serializes to plain JS objects (no `Map`s) so values survive
/// `storage.local` and `postMessage`.
pub fn to_js<T: Serialize + ?Sized>(value: &T) -> JsValue {
    value
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .unwrap_or(JsValue::NULL)
}

pub fn from_js<T: DeserializeOwned>(value: JsValue) -> Result<T, String> {
    serde_wasm_bindgen::from_value(value).map_err(|e| e.to_string())
}

pub async fn storage_get<T: DeserializeOwned>(key: &str) -> Option<T> {
    let v = wait(storage_get_raw(key)).await.ok()?;
    if v.is_null() || v.is_undefined() {
        return None;
    }
    from_js(v).ok()
}

pub async fn storage_set<T: Serialize>(key: &str, value: Option<&T>) -> Result<(), String> {
    let v = value.map(to_js).unwrap_or(JsValue::NULL);
    wait(storage_set_raw(key, v)).await.map(|_| ())
}

pub async fn send_message<Req: Serialize, Res: DeserializeOwned>(req: &Req) -> Result<Res, String> {
    let v = wait(send_message_raw(to_js(req))).await?;
    from_js(v)
}

pub async fn send_to_tab<M: Serialize>(tab_id: i32, msg: &M, frame_id: Option<i32>) -> JsValue {
    wait(send_to_tab_raw(tab_id, to_js(msg), frame_id))
        .await
        .unwrap_or(JsValue::NULL)
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct Tab {
    pub id: Option<i32>,
    pub url: Option<String>,
}

pub async fn active_tab() -> Option<Tab> {
    let v = wait(active_tab_raw()).await.ok()?;
    if v.is_null() {
        return None;
    }
    from_js(v).ok()
}

pub async fn open_tab(url: &str) {
    let _ = wait(open_tab_raw(url)).await;
}

pub async fn open_options_page() {
    let _ = wait(open_options_page_raw()).await;
}

pub async fn menu_recreate(id: &str, title: &str, enabled: bool, visible: bool) {
    let _ = wait(menu_recreate_raw(id, title, enabled, visible)).await;
}

pub async fn menu_update(id: &str, title: &str, enabled: bool, visible: bool) {
    let _ = wait(menu_update_raw(id, title, enabled, visible)).await;
}

pub async fn notify(title: &str, message: &str) {
    let _ = wait(notify_raw(title, message)).await;
}

pub async fn copy_text(text: &str) -> Result<(), String> {
    wait(copy_text_raw(text)).await.map(|_| ())
}

pub async fn has_host_access() -> bool {
    wait(has_host_access_raw())
        .await
        .map(|v| v.as_bool().unwrap_or(false))
        .unwrap_or(false)
}

pub async fn request_host_access() -> bool {
    wait(request_host_access_raw())
        .await
        .map(|v| v.as_bool().unwrap_or(false))
        .unwrap_or(false)
}
