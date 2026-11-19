//! Toolbar popup: loading → signed out | generate ⇄ manage.

mod generate;
mod manage;

use crate::icons::{Icon, I};
use crate::widgets::{rich, templ, Toasts};
use crate::{ctx, tr};
use hme_core::api::ApiError;
use hme_core::i18n::T;
use hme_core::model::{ClientState, HmeEmail};
use hme_core::protocol::{Reply, Request, TabMessage};
use hme_core::urls;
use hme_web::client::Hme;
use hme_web::ext;
use leptos::prelude::*;
use leptos::task::spawn_local;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Loading,
    SignedOut,
    Generate,
    Manage,
}

/// State shared by the popup's screens.
#[derive(Clone, Copy)]
pub struct PopupCtx {
    pub screen: RwSignal<Screen>,
    pub session: RwSignal<Option<ClientState>>,
    /// Host of the active tab, e.g. `github.com`.
    pub site: RwSignal<Option<String>>,
    pub tab_id: RwSignal<Option<i32>>,
    /// Number of addresses, for the badge on "My addresses".
    pub count: RwSignal<Option<usize>>,
    pub forward_to: RwSignal<Option<String>>,
    /// Message shown on the signed-out screen (offline, no iCloud+ …).
    pub notice: RwSignal<Option<String>>,
    /// Addresses for the Manage screen (kept while switching screens).
    pub list: RwSignal<ListState>,
    /// `anonymousId` of the address open in the details view.
    pub selected: RwSignal<Option<String>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ListState {
    Loading,
    Failed(String),
    Ready(Vec<HmeEmail>),
}

impl ListState {
    pub fn items(&self) -> &[HmeEmail] {
        match self {
            ListState::Ready(v) => v,
            _ => &[],
        }
    }
}

pub fn pctx() -> PopupCtx {
    expect_context()
}

impl PopupCtx {
    pub fn client(&self) -> Option<Hme> {
        self.session.get_untracked().map(Hme::new)
    }

    /// Asks the background to re-check the iCloud session and routes to the
    /// right screen.
    pub fn sync(self) {
        self.screen.set(Screen::Loading);
        spawn_local(async move {
            let req = Request::SyncAuth {
                deauth_on_failure: true,
            };
            let reply: Reply = ext::send_message(&req).await.unwrap_or_else(Reply::error);
            match reply.client_state.filter(|_| reply.authenticated == Some(true)) {
                Some(state) => {
                    self.notice.set(None);
                    self.session.set(Some(state.clone()));
                    self.screen.set(Screen::Generate);
                    self.refresh_summary(state);
                }
                None => {
                    self.notice.set(reply.error);
                    self.session.set(None);
                    self.screen.set(Screen::SignedOut);
                }
            }
        });
    }

    /// Loads the forwarding address, the count and the list itself.
    fn refresh_summary(self, state: ClientState) {
        spawn_local(async move {
            match Hme::new(state).list().await {
                Ok(list) => {
                    self.count.set(Some(list.hme_emails.len()));
                    self.forward_to.set(Some(list.selected_forward_to));
                    self.list.set(ListState::Ready(list.hme_emails));
                }
                Err(e) => {
                    let lang = ctx().lang_now();
                    self.list.set(ListState::Failed(hme_web::describe_error(&e, lang)));
                }
            }
        });
    }

    /// Reloads the address list (Manage screen, retry button).
    pub fn reload_list(self) {
        if let Some(state) = self.session.get_untracked() {
            if !matches!(self.list.get_untracked(), ListState::Ready(_)) {
                self.list.set(ListState::Loading);
            }
            self.refresh_summary(state);
        }
    }

    /// Applies a local change to one address in the list.
    pub fn update_item(self, id: &str, f: impl FnOnce(&mut HmeEmail)) {
        self.list.update(|l| {
            if let ListState::Ready(items) = l {
                if let Some(item) = items.iter_mut().find(|e| e.anonymous_id == id) {
                    f(item);
                }
            }
        });
    }

    pub fn selected_item(self) -> Option<HmeEmail> {
        let id = self.selected.get()?;
        self.list.with(|l| l.items().iter().find(|e| e.anonymous_id == id).cloned())
    }

    /// Error text for an API failure; an expired session sends the user
    /// back through the session check.
    pub fn error_text(self, e: &ApiError) -> String {
        if e.is_auth() {
            self.sync();
        }
        hme_web::describe_error(e, ctx().lang_now())
    }

    pub fn sign_out(self) {
        self.screen.set(Screen::Loading);
        spawn_local(async move {
            let _: Result<Reply, _> = ext::send_message(&Request::SignOut).await;
            self.session.set(None);
            self.count.set(None);
            self.forward_to.set(None);
            self.list.set(ListState::Loading);
            self.selected.set(None);
            self.screen.set(Screen::SignedOut);
        });
    }

    /// Copies `text` and confirms with a toast.
    pub fn copy(self, text: String) {
        let c = ctx();
        spawn_local(async move {
            match ext::copy_text(&text).await {
                Ok(()) => c.toast(crate::ToastKind::Success, T::Copied.get(c.lang_now())),
                Err(e) => c.toast(crate::ToastKind::Error, e),
            }
        });
    }

    /// Puts `hme` into the email field of the active tab.
    pub fn fill_on_page(self, hme: String) {
        let c = ctx();
        let tab = self.tab_id.get_untracked();
        spawn_local(async move {
            let lang = c.lang_now();
            let filled = match tab {
                Some(id) => {
                    let reply = ext::send_to_tab(id, &TabMessage::Autofill { hme }, None).await;
                    js_sys::Reflect::get(&reply, &"filled".into())
                        .ok()
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0)
                        > 0.0
                }
                None => false,
            };
            if filled {
                c.toast(crate::ToastKind::Success, T::FilledOnPage.get(lang));
            } else {
                c.toast(crate::ToastKind::Error, T::NothingToFill.get(lang));
            }
        });
    }
}

#[component]
pub fn Popup() -> impl IntoView {
    let p = PopupCtx {
        screen: RwSignal::new(Screen::Loading),
        session: RwSignal::new(None),
        site: RwSignal::new(None),
        tab_id: RwSignal::new(None),
        count: RwSignal::new(None),
        forward_to: RwSignal::new(None),
        notice: RwSignal::new(None),
        list: RwSignal::new(ListState::Loading),
        selected: RwSignal::new(None),
    };
    provide_context(p);

    spawn_local(async move {
        if let Some(tab) = ext::active_tab().await {
            p.tab_id.set(tab.id);
            let host = tab
                .url
                .as_deref()
                .and_then(urls::hostname)
                .filter(|h| !h.is_empty());
            p.site.set(host.map(|h| urls::label_for_host(&h)));
        }
        p.sync();
    });

    let busy = move || p.screen.get() == Screen::Loading;
    view! {
        <div class="v-popup" aria-busy=move || busy().to_string()>
            <Header/>
            {move || match p.screen.get() {
                Screen::Loading => view! { <Loading/> }.into_any(),
                Screen::SignedOut => view! { <SignedOut/> }.into_any(),
                Screen::Generate => view! { <generate::Generate/> }.into_any(),
                Screen::Manage => view! { <manage::Manage/> }.into_any(),
            }}
            <Toasts/>
        </div>
    }
}

#[component]
fn Header() -> impl IntoView {
    let p = pctx();
    let c = ctx();
    let settings = move || {
        view! {
            <button
                class="v-icon-btn"
                type="button"
                aria-label=tr(T::Settings)
                title=tr(T::Settings)
                on:click=move |_| spawn_local(ext::open_options_page())
            >
                <Icon icon=I::Settings/>
            </button>
        }
    };
    let detail = move || {
        (p.screen.get() == Screen::Manage)
            .then(|| p.selected_item())
            .flatten()
    };
    move || match detail() {
        Some(item) => {
            let created = crate::widgets::format_date(item.create_timestamp, crate::widgets::DateStyle::Long);
            view! {
                <header class="v-popup__header v-popup__header--nav">
                    <button
                        class="v-icon-btn"
                        type="button"
                        aria-label=tr(T::BackToList)
                        title=tr(T::BackToList)
                        on:click=move |_| p.selected.set(None)
                    >
                        <Icon icon=I::ChevronLeft/>
                    </button>
                    <div class="v-brand">
                        <div class="v-brand__text">
                            <span class="v-brand__title">{item.label.clone()}</span>
                            <span class="v-brand__sub">{move || hme_core::i18n::fill(c.t(T::CreatedOn), &[&created])}</span>
                        </div>
                    </div>
                    {settings()}
                </header>
            }
            .into_any()
        }
        None => {
            let show_site = move || p.screen.get() == Screen::Generate && p.site.get().is_some();
            let managing = move || p.screen.get() == Screen::Manage;
            view! {
                <header class="v-popup__header">
                    <div class="v-brand">
                        <span class="v-logo"><Icon icon=I::Logo/></span>
                        <div class="v-brand__text">
                            <span class="v-brand__title">
                                {move || if managing() { c.t(T::ManageAddresses) } else { c.t(T::AppName) }}
                            </span>
                            <span class="v-brand__sub">
                                {move || if show_site() {
                                    templ(c.t(T::GenerateFor), p.site.get().unwrap_or_default()).into_any()
                                } else if let Some(f) = managing().then(|| p.forward_to.get()).flatten().filter(|f| !f.is_empty()) {
                                    templ(c.t(T::ForwardsTo), f).into_any()
                                } else {
                                    c.t(T::AppTagline).into_any()
                                }}
                            </span>
                        </div>
                    </div>
                    {settings()}
                </header>
            }
            .into_any()
        }
    }
}

#[component]
fn Loading() -> impl IntoView {
    view! {
        <main class="v-popup__body">
            <div class="v-loading" role="status" aria-live="polite">
                <span class="v-logo v-logo--xl v-logo--pulse"><Icon icon=I::Logo/></span>
                <p class="v-loading__title"><span class="v-spinner v-spinner--sm"></span>{tr(T::CheckingSession)}</p>
            </div>
        </main>
    }
}

#[component]
fn SignedOut() -> impl IntoView {
    let p = pctx();
    let c = ctx();
    let firefox = ext::is_firefox();
    let tip = move |icon: I, class: &'static str, key: T| {
        view! {
            <li class=format!("v-tip {class}")>
                <span class="v-tip__icon"><Icon icon=icon/></span>
                <span>{move || rich(c.t(key))}</span>
            </li>
        }
    };
    let icloud_url = move || {
        if p.session.get().as_ref().is_some_and(ClientState::is_china) {
            "https://www.icloud.com.cn/"
        } else {
            "https://www.icloud.com/"
        }
    };
    view! {
        <main class="v-popup__body v-view">
            <div class="v-intro">
                <h1 class="v-intro__title">{tr(T::SignInTitle)}</h1>
                <p class="v-intro__text">{tr(T::SignInSubtitle)}</p>
            </div>
            {move || p.notice.get().map(|n| view! {
                <div class="v-alert v-alert--warning" role="alert">
                    <Icon icon=I::Info/>
                    <div class="v-alert__body">{n}</div>
                </div>
            })}
            <ul class="v-tips">
                {tip(I::Lock, "", T::TipTrust)}
                {tip(I::Clock, "", T::TipKeepSignedIn)}
                {tip(I::Key, "v-tip--danger", T::TipNoPasskey)}
                {tip(I::Shield, "", T::TipAdp)}
                {firefox.then(|| view! {
                    <li class="v-tip v-tip--warn">
                        <span class="v-tip__icon"><Icon icon=I::Box/></span>
                        <span>
                            <span class="v-badge">"Firefox"</span>" "
                            {move || rich(c.t(T::TipFirefoxContainers))}
                        </span>
                    </li>
                })}
            </ul>
        </main>
        <footer class="v-popup__footer v-popup__footer--stack">
            <a class="v-btn v-btn--primary v-btn--lg v-btn--block" href=icloud_url target="_blank" rel="noreferrer">
                {tr(T::OpenICloud)}
                <Icon icon=I::External/>
            </a>
            <button class="v-btn v-btn--ghost v-btn--sm v-btn--block" type="button" on:click=move |_| p.sync()>
                <Icon icon=I::Refresh class="v-icon--sm"/>
                {tr(T::CheckAgain)}
            </button>
        </footer>
    }
}

/// "My addresses" (with count) and "Sign out", shared by both main screens.
#[component]
pub fn FooterNav(primary: Screen) -> impl IntoView {
    let p = pctx();
    let (icon, label, class) = match primary {
        Screen::Manage => (I::List, T::ManageAddresses, "v-btn v-btn--ghost v-btn--sm"),
        _ => (I::Plus, T::NewAddress, "v-btn v-btn--secondary v-btn--sm"),
    };
    view! {
        <footer class="v-popup__footer">
            <button
                class=class
                type="button"
                on:click=move |_| {
                    p.selected.set(None);
                    p.screen.set(primary);
                }
            >
                <Icon icon=icon class="v-icon--sm"/>
                {tr(label)}
                {move || (primary == Screen::Manage)
                    .then(|| p.count.get())
                    .flatten()
                    .map(|n| view! { <span class="v-badge">{n}</span> })}
            </button>
            <button class="v-btn v-btn--ghost v-btn--sm" type="button" on:click=move |_| p.sign_out()>
                <Icon icon=I::LogOut class="v-icon--sm"/>
                {tr(T::SignOut)}
            </button>
        </footer>
    }
}
