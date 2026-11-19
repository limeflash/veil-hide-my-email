//! Options page: forwarding address, autofill, appearance, about.

use crate::icons::{Icon, I};
use crate::widgets::{rich, Toasts};
use crate::{ctx, tr, ToastKind};
use hme_core::i18n::{Lang, T};
use hme_core::model::{ClientState, LanguagePref, Options, Theme};
use hme_core::protocol::{Reply, Request};
use hme_web::client::Hme;
use hme_web::ext;
use leptos::prelude::*;
use leptos::task::spawn_local;

const ORIGINAL_FORK_URL: &str = "https://github.com/imNebula/icloud-hide-my-email-browser-extension";
const SOURCE_URL: &str = "https://github.com/limeflash/veil-hide-my-email";

#[derive(Clone, PartialEq)]
enum Forward {
    Loading,
    SignedOut,
    Failed(String),
    Ready { emails: Vec<String>, current: String },
}

#[component]
pub fn OptionsPage() -> impl IntoView {
    let session = RwSignal::new(None::<ClientState>);
    let forward = RwSignal::new(Forward::Loading);
    let home = RwSignal::new("https://www.icloud.com/");
    provide_context(home);
    let c = ctx();

    spawn_local(async move {
        home.set(hme_web::icloud_home().await);
        let req = Request::SyncAuth {
            deauth_on_failure: false,
        };
        let reply: Reply = ext::send_message(&req).await.unwrap_or_else(Reply::error);
        let Some(state) = reply.client_state.filter(|_| reply.authenticated == Some(true)) else {
            forward.set(Forward::SignedOut);
            return;
        };
        session.set(Some(state.clone()));
        match Hme::new(state).list().await {
            Ok(list) => forward.set(Forward::Ready {
                emails: list.forward_to_emails,
                current: list.selected_forward_to,
            }),
            Err(e) => forward.set(Forward::Failed(hme_web::describe_error(&e, c.lang_now()))),
        }
    });

    view! {
        <div class="v-page">
            <div class="v-page__container">
                <PageHeader session=session forward=forward/>
                <ForwardSection session=session forward=forward/>
                <AutofillSection/>
                <AppearanceSection/>
                <AboutSection/>
            </div>
            <Toasts page=true/>
        </div>
    }
}

#[component]
fn PageHeader(session: RwSignal<Option<ClientState>>, forward: RwSignal<Forward>) -> impl IntoView {
    let status = move || {
        if forward.get() == Forward::Loading {
            return ().into_any();
        }
        match session.get() {
            Some(s) => {
                let region = if s.is_china() { T::RegionChina } else { T::RegionGlobal };
                let account = s.account.map(|a| a.apple_id).filter(|a| !a.is_empty());
                view! {
                    <span class="v-badge v-badge--green" title=account>
                        <span class="v-badge__dot"></span>
                        {tr(T::SignedIn)}" · "{tr(region)}
                    </span>
                }
                .into_any()
            }
            None => view! {
                <span class="v-badge v-badge--yellow"><span class="v-badge__dot"></span>{tr(T::SignedOut)}</span>
            }
            .into_any(),
        }
    };
    view! {
        <header class="v-page-header">
            <div class="v-brand v-brand--lg">
                <span class="v-logo v-logo--lg"><Icon icon=I::Logo/></span>
                <div class="v-brand__text">
                    <span class="v-brand__title">{tr(T::AppName)}</span>
                    <span class="v-brand__sub">{tr(T::AppTagline)}</span>
                </div>
            </div>
            {status}
        </header>
    }
}

#[component]
fn Section(id: &'static str, title: T, #[prop(optional)] desc: Option<T>, children: Children) -> impl IntoView {
    view! {
        <section class="v-section" aria-labelledby=id>
            <div class="v-section__head">
                <h2 class="v-section__title" id=id>{tr(title)}</h2>
                {desc.map(|d| view! { <p class="v-section__desc">{tr(d)}</p> })}
            </div>
            {children()}
        </section>
    }
}

#[component]
fn ForwardSection(session: RwSignal<Option<ClientState>>, forward: RwSignal<Forward>) -> impl IntoView {
    let c = ctx();
    let home = expect_context::<RwSignal<&'static str>>();
    let selected = RwSignal::new(String::new());
    let saving = RwSignal::new(false);
    let saved = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        if let Forward::Ready { current, .. } = forward.get() {
            selected.set(current);
        }
    });

    let current = move || match forward.get() {
        Forward::Ready { current, .. } => current,
        _ => String::new(),
    };
    let dirty = move || !selected.get().is_empty() && selected.get() != current();

    let save = move |_| {
        let Some(state) = session.get_untracked() else { return };
        let email = selected.get_untracked();
        saving.set(true);
        error.set(None);
        spawn_local(async move {
            let lang = c.lang_now();
            match Hme::new(state).update_forward_to(&email).await {
                Ok(()) => {
                    forward.update(|f| {
                        if let Forward::Ready { current, .. } = f {
                            *current = email.clone();
                        }
                    });
                    saved.set(true);
                    c.toast(ToastKind::Success, T::FwdSaved.get(lang));
                }
                Err(e) => error.set(Some(hme_web::describe_error(&e, lang))),
            }
            saving.set(false);
        });
    };

    let body = move || match forward.get() {
        Forward::Loading => view! {
            <div class="v-card" aria-busy="true">
                <div class="v-card__body">
                    <div class="v-radio-list" aria-hidden="true">
                        <span class="v-skeleton v-skeleton--block"></span>
                        <span class="v-skeleton v-skeleton--block"></span>
                    </div>
                </div>
                <div class="v-card__footer">
                    <span class="v-card__footer-status" role="status"><span class="v-spinner v-spinner--sm"></span>{tr(T::FwdLoading)}</span>
                    <button class="v-btn v-btn--primary" type="button" disabled>{tr(T::Save)}</button>
                </div>
            </div>
        }
        .into_any(),
        Forward::SignedOut => view! {
            <div class="v-card">
                <div class="v-empty v-empty--compact">
                    <span class="v-empty__icon"><Icon icon=I::Lock class="v-icon--xl"/></span>
                    <h3 class="v-empty__title">{tr(T::SignInTitle)}</h3>
                    <p class="v-empty__text">{tr(T::FwdSignedOut)}</p>
                    <a class="v-btn v-btn--primary v-empty__action" href=move || home.get() target="_blank" rel="noreferrer">
                        {tr(T::OpenICloud)}<Icon icon=I::External/>
                    </a>
                </div>
            </div>
        }
        .into_any(),
        Forward::Failed(msg) => view! {
            <div class="v-alert v-alert--error" role="alert">
                <Icon icon=I::Alert/>
                <div class="v-alert__body">{msg}</div>
            </div>
        }
        .into_any(),
        Forward::Ready { emails, current } => {
            let options = emails
                .into_iter()
                .map(|email| {
                    let is_current = email == current;
                    let (value, value2) = (email.clone(), email.clone());
                    view! {
                        <label class="v-radio" class:is-active=move || selected.get() == value>
                            <input
                                type="radio"
                                name="forward"
                                prop:checked=move || selected.get() == value2
                                disabled=move || saving.get()
                                on:change={
                                    let email = email.clone();
                                    move |_| {
                                        selected.set(email.clone());
                                        saved.set(false);
                                    }
                                }
                            />
                            <span class="v-radio__mark"></span>
                            <span class="v-radio__label">{email.clone()}</span>
                            {is_current.then(|| view! { <span class="v-badge">{tr(T::FwdCurrent)}</span> })}
                        </label>
                    }
                })
                .collect_view();
            view! {
                <div class="v-card">
                    <div class="v-card__body">
                        <div class="v-radio-list" role="radiogroup" aria-labelledby="opt-forward">{options}</div>
                    </div>
                    {move || error.get().map(|m| view! {
                        <div class="v-card__body">
                            <div class="v-alert v-alert--error" role="alert"><Icon icon=I::Alert/><div class="v-alert__body">{m}</div></div>
                        </div>
                    })}
                    <div class="v-card__footer">
                        {move || if saved.get() {
                            view! {
                                <span class="v-card__footer-status v-card__footer-status--success" role="status">
                                    <Icon icon=I::Check/>{tr(T::Saved)}
                                </span>
                            }.into_any()
                        } else {
                            view! { <span class="v-card__footer-status">{tr(T::FwdHint)}</span> }.into_any()
                        }}
                        <button
                            class="v-btn v-btn--primary"
                            type="button"
                            disabled=move || !dirty() || saving.get()
                            aria-busy=move || saving.get().to_string()
                            on:click=save
                        >
                            <span class="v-spinner"></span>
                            {tr(T::Save)}
                        </button>
                    </div>
                </div>
            }
            .into_any()
        }
    };

    view! {
        <Section id="opt-forward" title=T::FwdTitle desc=T::FwdDesc>
            {body}
        </Section>
    }
}

fn save_options(f: impl FnOnce(&mut Options)) {
    let c = ctx();
    let mut next = c.options.get_untracked();
    f(&mut next);
    c.options.set(next);
    spawn_local(async move {
        if let Err(e) = hme_web::save_options(&next).await {
            c.toast(ToastKind::Error, e);
        }
    });
}

#[component]
fn AutofillSection() -> impl IntoView {
    let c = ctx();
    let toggle = move |title: T, desc: T, get: fn(&Options) -> bool, set: fn(&mut Options, bool)| {
        view! {
            <label class="v-setting">
                <span class="v-setting__body">
                    <span class="v-setting__title">{tr(title)}</span>
                    <span class="v-setting__desc">{tr(desc)}</span>
                </span>
                <input
                    class="v-toggle"
                    type="checkbox"
                    role="switch"
                    prop:checked=move || get(&c.options.get())
                    on:change=move |ev| {
                        let on = event_target_checked(&ev);
                        save_options(|o| set(o, on));
                    }
                />
            </label>
        }
    };
    view! {
        <Section id="opt-autofill" title=T::AutofillTitle desc=T::AutofillDesc>
            <div class="v-card">
                {toggle(T::AutofillButton, T::AutofillButtonDesc, |o| o.autofill.button, |o, v| o.autofill.button = v)}
                {toggle(T::AutofillMenu, T::AutofillMenuDesc, |o| o.autofill.context_menu, |o, v| o.autofill.context_menu = v)}
            </div>
        </Section>
    }
}

/// A radio-based segmented control bound to one field of [`Options`].
fn segmented<V: Copy + PartialEq + Send + Sync + 'static>(
    name: &'static str,
    labelled_by: &'static str,
    items: Vec<(V, T, Option<I>)>,
    get: fn(&Options) -> V,
    set: fn(&mut Options, V),
) -> impl IntoView {
    let c = ctx();
    view! {
        <div class="v-segmented" role="radiogroup" aria-labelledby=labelled_by>
            {items.into_iter().map(|(value, label, icon)| view! {
                <label class="v-segmented__item">
                    <input
                        type="radio"
                        name=name
                        prop:checked=move || get(&c.options.get()) == value
                        on:change=move |_| save_options(|o| set(o, value))
                    />
                    <span>{icon.map(|i| view! { <Icon icon=i class="v-icon--sm"/> })}{tr(label)}</span>
                </label>
            }).collect_view()}
        </div>
    }
}

#[component]
fn AppearanceSection() -> impl IntoView {
    view! {
        <Section id="opt-look" title=T::AppearanceTitle>
            <div class="v-card">
                <div class="v-setting v-setting--wrap">
                    <span class="v-setting__body">
                        <span class="v-setting__title" id="opt-theme">{tr(T::Theme)}</span>
                        <span class="v-setting__desc">{tr(T::ThemeDesc)}</span>
                    </span>
                    {segmented(
                        "theme",
                        "opt-theme",
                        vec![
                            (Theme::System, T::ThemeSystem, Some(I::Monitor)),
                            (Theme::Light, T::ThemeLight, Some(I::Sun)),
                            (Theme::Dark, T::ThemeDark, Some(I::Moon)),
                        ],
                        |o| o.theme,
                        |o, v| o.theme = v,
                    )}
                </div>
                <div class="v-setting v-setting--wrap">
                    <span class="v-setting__body">
                        <span class="v-setting__title" id="opt-lang">{tr(T::Language)}</span>
                        <span class="v-setting__desc">{tr(T::LanguageDesc)}</span>
                    </span>
                    <LanguageSelect/>
                </div>
            </div>
        </Section>
    }
}

/// Auto + every language, each shown in its own script.
#[component]
fn LanguageSelect() -> impl IntoView {
    let c = ctx();
    let current = move || match c.options.get().language.lang() {
        None => "auto".to_string(),
        Some(l) => l.tag().to_string(),
    };
    view! {
        <select
            class="v-select"
            aria-labelledby="opt-lang"
            prop:value=current
            on:change=move |ev| {
                let v = event_target_value(&ev);
                let pref = Lang::ALL
                    .into_iter()
                    .find(|l| l.tag() == v)
                    .map(LanguagePref::from_lang)
                    .unwrap_or(LanguagePref::Auto);
                save_options(|o| o.language = pref);
            }
        >
            <option value="auto">{tr(T::LangAuto)}</option>
            {Lang::ALL
                .into_iter()
                .map(|l| view! { <option value=l.tag() lang=l.tag()>{l.native_name()}</option> })
                .collect_view()}
        </select>
    }
}

#[component]
fn AboutSection() -> impl IntoView {
    let c = ctx();
    let credits = move || {
        let template = c.t(T::Credits);
        let (before, after) = template.split_once("{}").unwrap_or((template, ""));
        view! {
            {before}
            <a class="v-link" href=ORIGINAL_FORK_URL target="_blank" rel="noreferrer">"imNebula"</a>
            {after}
        }
    };
    view! {
        <Section id="opt-about" title=T::AboutTitle>
            <div class="v-card">
                <div class="v-card__body">
                    <p class="v-prose">{move || rich(c.t(T::Disclaimer))}</p>
                    <p class="v-prose">{credits}</p>
                </div>
                <div class="v-card__footer">
                    <span class="v-card__footer-status"><span class="v-badge">"MIT"</span>{tr(T::OpenSource)}</span>
                    <a class="v-btn v-btn--secondary v-btn--sm" href=SOURCE_URL target="_blank" rel="noreferrer">
                        <Icon icon=I::Github class="v-icon--sm"/>{tr(T::SourceCode)}<Icon icon=I::External class="v-icon--sm"/>
                    </a>
                </div>
            </div>
        </Section>
    }
}
