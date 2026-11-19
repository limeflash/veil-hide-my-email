//! Small building blocks shared by the pages.

use crate::icons::{Icon, I};
use crate::{ctx, ToastKind};
use leptos::prelude::*;

/// Renders `**bold**` segments of a translated string as `<strong>`.
pub fn rich(text: &'static str) -> impl IntoView {
    text.split("**")
        .enumerate()
        .map(|(i, part)| {
            if i % 2 == 1 {
                view! { <strong>{part}</strong> }.into_any()
            } else {
                part.into_any()
            }
        })
        .collect_view()
}

/// Splits a `{}` template around a bold value: "Forwards to {}" → text + `<b>`.
pub fn templ(template: &'static str, value: String) -> impl IntoView {
    let (before, after) = template.split_once("{}").unwrap_or((template, ""));
    view! { {before}<b>{value}</b>{after} }
}

/// Two `{}` slots, both bold.
pub fn templ2(template: &'static str, a: String, b: String) -> impl IntoView {
    let mut parts = template.splitn(3, "{}");
    let (p0, p1, p2) = (
        parts.next().unwrap_or(""),
        parts.next().unwrap_or(""),
        parts.next().unwrap_or(""),
    );
    view! { {p0}<b>{a}</b>{p1}<b>{b}</b>{p2} }
}

/// `local<span class="v-address__domain">@icloud.com</span>`
#[component]
pub fn Address(#[prop(into)] value: Signal<String>) -> impl IntoView {
    let parts = move || {
        let v = value.get();
        match v.rfind('@') {
            Some(i) => (v[..i].to_owned(), v[i..].to_owned()),
            None => (v, String::new()),
        }
    };
    view! {
        <output class="v-address" title=move || value.get()>
            {move || parts().0}
            <span class="v-address__domain">{move || parts().1}</span>
        </output>
    }
}

#[component]
pub fn ErrorAlert(#[prop(into)] title: Signal<&'static str>, #[prop(into)] message: Signal<String>) -> impl IntoView {
    view! {
        <div class="v-alert v-alert--error" role="alert">
            <Icon icon=I::Alert/>
            <div class="v-alert__body">
                <span class="v-alert__title">{move || title.get()}</span>
                {move || message.get()}
            </div>
        </div>
    }
}

/// Toast stack; `page` uses the fixed full-page region.
#[component]
pub fn Toasts(#[prop(optional)] page: bool) -> impl IntoView {
    let c = ctx();
    let class = if page {
        "v-toast-region v-toast-region--page"
    } else {
        "v-toast-region"
    };
    view! {
        <div class=class role="status" aria-live="polite">
            <For each=move || c.toasts.get() key=|t| t.id let:toast>
                {
                    let (kind_class, icon) = match toast.kind {
                        ToastKind::Success => ("v-toast v-toast--success", I::Check),
                        ToastKind::Error => ("v-toast v-toast--error", I::Alert),
                        ToastKind::Info => ("v-toast", I::Info),
                    };
                    let leaving = toast.leaving;
                    let action = toast.action.map(|(label, cb)| view! {
                        <button
                            class="v-toast__action"
                            type="button"
                            on:click=move |_| {
                                leaving.set(true);
                                cb.run(());
                            }
                        >
                            {label}
                        </button>
                    });
                    view! {
                        <div class=kind_class class:is-leaving=move || leaving.get()>
                            <Icon icon=icon/>
                            {toast.text.clone()}
                            {action}
                        </div>
                    }
                }
            </For>
        </div>
    }
}

#[derive(Clone, Copy)]
pub enum DateStyle {
    /// "12 нояб."
    Short,
    /// "12 ноября 2026"
    Long,
    /// "12 ноября 2026, 14:32"
    LongWithTime,
}

/// Formats a millisecond timestamp in the UI language.
pub fn format_date(ms: f64, style: DateStyle) -> String {
    use js_sys::{Object, Reflect};
    use wasm_bindgen::JsValue;
    let date = js_sys::Date::new(&JsValue::from_f64(ms));
    let opts = Object::new();
    let set = |k: &str, v: &str| {
        let _ = Reflect::set(&opts, &k.into(), &v.into());
    };
    set("day", "numeric");
    match style {
        DateStyle::Short => set("month", "short"),
        DateStyle::Long | DateStyle::LongWithTime => {
            set("month", "long");
            set("year", "numeric");
        }
    }
    if matches!(style, DateStyle::LongWithTime) {
        set("hour", "2-digit");
        set("minute", "2-digit");
    }
    let lang = ctx().lang_now().tag();
    date.to_locale_string(lang, &opts).into()
}
