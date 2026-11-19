//! Veil UI: popup, options and welcome pages, written with Leptos.
//!
//! Each HTML page loads `boot.js`, which calls [`mount`] with the page name.
//! Markup and class names follow the design handoff (`design/handoff`), and
//! the styles in `extension/styles` are used unchanged.

mod icons;
mod options;
mod popup;
mod welcome;
mod widgets;

use hme_core::i18n::{Lang, T};
use hme_core::model::Options;
use hme_core::storage_keys;
use hme_web::ext;
use leptos::prelude::*;
use std::time::Duration;
use wasm_bindgen::prelude::*;

/// Shared by every page: settings, language and toasts.
#[derive(Clone, Copy)]
pub struct Ctx {
    pub options: RwSignal<Options>,
    pub lang: Memo<Lang>,
    toasts: RwSignal<Vec<Toast>>,
    next_toast: StoredValue<u32>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Success,
    Error,
    Info,
}

#[derive(Clone)]
pub struct Toast {
    id: u32,
    kind: ToastKind,
    text: String,
    leaving: RwSignal<bool>,
    action: Option<(&'static str, Callback<()>)>,
}

impl Ctx {
    /// Tracked translation: re-renders when the language changes.
    pub fn t(&self, key: T) -> &'static str {
        key.get(self.lang.get())
    }

    pub fn lang_now(&self) -> Lang {
        self.lang.get_untracked()
    }

    pub fn toast(&self, kind: ToastKind, text: impl Into<String>) {
        self.push_toast(kind, text.into(), None);
    }

    /// Toast with a button, e.g. "Undo".
    pub fn toast_with_action(
        &self,
        kind: ToastKind,
        text: impl Into<String>,
        label: &'static str,
        action: impl Fn() + Send + Sync + 'static,
    ) {
        self.push_toast(kind, text.into(), Some((label, Callback::new(move |_: ()| action()))));
    }

    fn push_toast(&self, kind: ToastKind, text: String, action: Option<(&'static str, Callback<()>)>) {
        let has_action = action.is_some();
        let id = self.next_toast.get_value();
        self.next_toast.set_value(id + 1);
        let leaving = RwSignal::new(false);
        let toasts = self.toasts;
        toasts.update(|v| {
            // Keep the stack short in a 600px popup.
            if v.len() >= 2 {
                v.remove(0);
            }
            v.push(Toast {
                id,
                kind,
                text,
                leaving,
                action,
            })
        });
        set_timeout(
            move || {
                leaving.set(true);
                set_timeout(
                    move || toasts.update(|v| v.retain(|t| t.id != id)),
                    Duration::from_millis(220),
                );
            },
            Duration::from_millis(if kind == ToastKind::Error || has_action { 4500 } else { 2200 }),
        );
    }
}

impl Ctx {
    pub fn dismiss_toasts(&self) {
        self.toasts.set(Vec::new());
    }
}

pub fn ctx() -> Ctx {
    expect_context()
}

/// Reactive translation for use directly in `view!`.
pub fn tr(key: T) -> impl Fn() -> &'static str + Copy + Send + Sync + 'static {
    let c = ctx();
    move || c.t(key)
}

fn apply_document_settings(options: &Options, lang: Lang) {
    let Some(html) = document().document_element() else { return };
    let _ = match options.theme.attr() {
        Some(t) => html.set_attribute("data-theme", t),
        None => html.remove_attribute("data-theme"),
    };
    let _ = html.set_attribute("lang", lang.tag());
}

fn provide_ctx(initial: Options) -> Ctx {
    let options = RwSignal::new(initial);
    let browser_lang = ext::ui_language();
    let lang = Memo::new(move |_| Lang::resolve(options.get().language, &browser_lang));
    let c = Ctx {
        options,
        lang,
        toasts: RwSignal::new(Vec::new()),
        next_toast: StoredValue::new(0),
    };
    provide_context(c);

    // Settings changed in another page (e.g. the options tab) apply live.
    let on_change = Closure::<dyn Fn(String, JsValue)>::new(move |key: String, value: JsValue| {
        if key == storage_keys::OPTIONS {
            options.set(ext::from_js(value).unwrap_or_default());
        }
    });
    ext::on_storage_changed(on_change.as_ref().unchecked_ref());
    on_change.forget();

    Effect::new(move |_| {
        apply_document_settings(&options.get(), lang.get());
        document().set_title(&format!("{} — {}", T::AppName.get(lang.get()), T::AppTagline.get(lang.get())));
    });
    c
}

/// Entry point, called by `boot.js` with `popup`, `options` or `welcome`.
#[wasm_bindgen]
pub async fn mount(page: String) {
    console_error_panic_hook::set_once();
    // Load settings first so the first frame already has the right theme
    // and language.
    let options = hme_web::load_options().await;
    apply_document_settings(&options, Lang::resolve(options.language, &ext::ui_language()));
    leptos::mount::mount_to_body(move || {
        provide_ctx(options);
        match page.as_str() {
            "options" => view! { <options::OptionsPage/> }.into_any(),
            "welcome" => view! { <welcome::WelcomePage/> }.into_any(),
            _ => view! { <popup::Popup/> }.into_any(),
        }
    });
}
