//! iCloud client: executes the requests described by `hme_core::api` with
//! the global `fetch` (available both in pages and in the service worker).

use hme_core::api::{self, ApiError, ApiRequest, Pms};
use hme_core::model::{ClientState, HmeEmail, HmeList};
use hme_core::urls::candidate_setup_urls;
use js_sys::Promise;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestCredentials, RequestInit, Response};

#[wasm_bindgen]
extern "C" {
    /// `globalThis.fetch`, so this works in windows and workers alike.
    #[wasm_bindgen(js_name = fetch)]
    fn global_fetch(req: &Request) -> Promise;
}

async fn execute(req: &ApiRequest) -> Result<String, ApiError> {
    let init = RequestInit::new();
    init.set_method(req.method.as_str());
    // iCloud authenticates with its cookies; the declarativeNetRequest rules
    // in rules.json set Origin/Referer to https://www.icloud.com.
    init.set_credentials(RequestCredentials::Include);
    if let Some(body) = &req.body {
        init.set_body(&JsValue::from_str(body));
    }
    let request = Request::new_with_str_and_init(&req.url, &init)
        .map_err(|e| ApiError::Network(crate::ext::js_error(e)))?;
    let resp: Response = JsFuture::from(global_fetch(&request))
        .await
        .map_err(|e| ApiError::Network(crate::ext::js_error(e)))?
        .unchecked_into();
    if !resp.ok() {
        return Err(ApiError::Http(resp.status()));
    }
    let text = JsFuture::from(resp.text().map_err(|e| ApiError::Network(crate::ext::js_error(e)))?)
        .await
        .map_err(|e| ApiError::Network(crate::ext::js_error(e)))?;
    Ok(text.as_string().unwrap_or_default())
}

/// Validates the session behind `setup_url` and returns the state needed to
/// call Hide My Email.
pub async fn validate(setup_url: &str) -> Result<ClientState, ApiError> {
    let body = execute(&api::validate(setup_url)).await?;
    let info = api::parse_validate(&body)?;
    Ok(ClientState {
        setup_url: setup_url.to_owned(),
        pms_url: info.pms_url,
        account: info.account,
    })
}

/// Tries the preferred setup URL, then iCloud.com, then iCloud.com.cn.
pub async fn authenticate_with_cookies(preferred: Option<&str>) -> Result<ClientState, ApiError> {
    let mut last = ApiError::Http(401);
    for url in candidate_setup_urls(preferred) {
        match validate(&url).await {
            Ok(state) => return Ok(state),
            // A valid session without HME won't get better on the other region.
            Err(ApiError::NoHmeService) => return Err(ApiError::NoHmeService),
            Err(e) => last = e,
        }
    }
    Err(last)
}

pub async fn sign_out(setup_url: &str) {
    let _ = execute(&api::logout(setup_url, false)).await;
}

/// Hide My Email operations for one signed-in account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hme {
    pub state: ClientState,
}

impl Hme {
    pub fn new(state: ClientState) -> Self {
        Self { state }
    }

    fn pms(&self) -> Pms<'_> {
        Pms::new(&self.state.pms_url)
    }

    pub async fn list(&self) -> Result<HmeList, ApiError> {
        api::parse_list(&execute(&self.pms().list()).await?)
    }

    pub async fn generate(&self) -> Result<String, ApiError> {
        api::parse_generate(&execute(&self.pms().generate()).await?)
    }

    pub async fn reserve(&self, hme: &str, label: &str, note: Option<&str>) -> Result<HmeEmail, ApiError> {
        api::parse_reserve(&execute(&self.pms().reserve(hme, label, note)).await?)
    }

    pub async fn update_metadata(&self, id: &str, label: &str, note: &str) -> Result<(), ApiError> {
        let body = execute(&self.pms().update_metadata(id, label, note)).await?;
        api::parse_ack(&body, "update the address")
    }

    pub async fn set_active(&self, id: &str, active: bool) -> Result<(), ApiError> {
        let (req, what) = if active {
            (self.pms().reactivate(id), "reactivate the address")
        } else {
            (self.pms().deactivate(id), "deactivate the address")
        };
        api::parse_ack(&execute(&req).await?, what)
    }

    pub async fn delete(&self, id: &str) -> Result<(), ApiError> {
        api::parse_ack(&execute(&self.pms().delete(id)).await?, "delete the address")
    }

    pub async fn update_forward_to(&self, email: &str) -> Result<(), ApiError> {
        let body = execute(&self.pms().update_forward_to(email)).await?;
        api::parse_ack(&body, "change the forwarding address")
    }
}
