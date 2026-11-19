//! Welcome / quick-start page, opened after install.

use crate::icons::{Icon, I};
use crate::widgets::rich;
use crate::{ctx, tr};
use hme_core::i18n::T;
use hme_web::ext;
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn WelcomePage() -> impl IntoView {
    let c = ctx();
    let firefox = ext::is_firefox();
    let granted = RwSignal::new(true);
    if firefox {
        spawn_local(async move { granted.set(ext::has_host_access().await) });
    }
    let grant = move |_| {
        spawn_local(async move {
            let ok = ext::request_host_access().await;
            granted.set(ok);
            if ok {
                // Pick up an existing iCloud session right away.
                let _: Result<hme_core::protocol::Reply, _> =
                    ext::send_message(&hme_core::protocol::Request::SyncAuth { deauth_on_failure: false }).await;
            }
        })
    };

    let step = |n: u8, title: T, body: T, action: Option<AnyView>| {
        view! {
            <li class="v-card v-step">
                <span class="v-step__num">{n}</span>
                <h2 class="v-step__title">{tr(title)}</h2>
                <p class="v-step__text">{tr(body)}</p>
                {action.map(|a| view! { <div class="v-step__action">{a}</div> })}
            </li>
        }
    };

    let faq = move |q: T, a: T, open: bool| {
        view! {
            <details class="v-faq__item" open=open>
                <summary>{tr(q)}<Icon icon=I::ChevronRight/></summary>
                <div class="v-faq__body">{move || rich(c.t(a))}</div>
            </details>
        }
    };

    let options_url = ext::get_url("options.html");
    view! {
        <div class="v-page">
            <div class="v-page__container">
                <header class="v-hero">
                    <span class="v-logo v-logo--xl"><Icon icon=I::Logo/></span>
                    <h1 class="v-hero__title">{tr(T::WelcomeTitle)}</h1>
                    <p class="v-hero__lead">{tr(T::WelcomeLead)}</p>
                </header>
                <ol class="v-steps">
                    {step(1, T::Step1Title, T::Step1Body, Some(view! {
                        <a class="v-btn v-btn--primary v-btn--block" href="https://www.icloud.com/" target="_blank" rel="noreferrer">
                            {tr(T::OpenICloud)}<Icon icon=I::External/>
                        </a>
                    }.into_any()))}
                    {step(2, T::Step2Title, T::Step2Body, None)}
                    {step(3, T::Step3Title, T::Step3Body, None)}
                </ol>
                {firefox.then(|| view! {
                    <section class="v-card v-card--accent v-callout" aria-labelledby="wel-ff">
                        <div class="v-callout__body">
                            <h2 class="v-callout__title" id="wel-ff">
                                {tr(T::GrantTitle)}
                                <span class="v-badge v-badge--warning">{tr(T::FirefoxOnly)}</span>
                            </h2>
                            <p class="v-callout__text">{tr(T::GrantBody)}</p>
                        </div>
                        {move || if granted.get() {
                            view! { <span class="v-badge v-badge--success"><Icon icon=I::Check class="v-icon--sm"/>{tr(T::Granted)}</span> }.into_any()
                        } else {
                            view! { <button class="v-btn v-btn--primary" type="button" on:click=grant><Icon icon=I::Shield/>{tr(T::GrantButton)}</button> }.into_any()
                        }}
                    </section>
                })}
                <section class="v-section" aria-labelledby="wel-faq">
                    <div class="v-section__head">
                        <h2 class="v-section__title" id="wel-faq">{tr(T::FaqTitle)}</h2>
                    </div>
                    <div class="v-card v-faq">
                        {faq(T::FaqSignedInQ, T::FaqSignedInA, true)}
                        {faq(T::FaqKeepQ, T::FaqKeepA, true)}
                        {faq(T::FaqPasskeyQ, T::FaqPasskeyA, false)}
                        {faq(T::FaqAdpQ, T::FaqAdpA, false)}
                        {firefox.then(|| faq(T::FaqContainersQ, T::FaqContainersA, false))}
                    </div>
                </section>
                <p class="v-footnote">
                    {tr(T::Footnote)}" · "
                    <a class="v-link" href=options_url>{tr(T::Settings)}</a>
                </p>
            </div>
        </div>
    }
}
