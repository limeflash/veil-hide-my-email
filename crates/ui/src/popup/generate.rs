//! Main screen: propose an address, label it, reserve it.

use super::{pctx, FooterNav, Screen};
use crate::icons::{Icon, I};
use crate::widgets::{templ, templ2, Address, ErrorAlert};
use crate::{ctx, tr};
use hme_core::i18n::T;
use hme_core::model::HmeEmail;
use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn Generate() -> impl IntoView {
    let p = pctx();
    let c = ctx();

    let address = RwSignal::new(None::<String>);
    let generating = RwSignal::new(false);
    let gen_error = RwSignal::new(None::<String>);

    let default_label = move || p.site.get_untracked().unwrap_or_default();
    let label = RwSignal::new(default_label());
    let note = RwSignal::new(String::new());
    let label_invalid = RwSignal::new(false);

    let reserving = RwSignal::new(false);
    let reserve_error = RwSignal::new(None::<String>);
    let reserved = RwSignal::new(None::<HmeEmail>);

    let regenerate = move || {
        let Some(client) = p.client() else { return };
        generating.set(true);
        gen_error.set(None);
        reserve_error.set(None);
        spawn_local(async move {
            match client.generate().await {
                Ok(a) => address.set(Some(a)),
                Err(e) => gen_error.set(Some(p.error_text(&e))),
            }
            generating.set(false);
        });
    };
    regenerate();

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let (Some(client), Some(hme)) = (p.client(), address.get_untracked()) else { return };
        let label_text = label.get_untracked().trim().to_owned();
        if label_text.is_empty() {
            label_invalid.set(true);
            return;
        }
        let note_text = note.get_untracked();
        reserving.set(true);
        reserve_error.set(None);
        spawn_local(async move {
            match client.reserve(&hme, &label_text, Some(note_text.as_str())).await {
                Ok(r) => {
                    p.list.update(|l| {
                        if let super::ListState::Ready(v) = l {
                            v.insert(0, r.clone());
                        }
                    });
                    reserved.set(Some(r));
                    p.count.update(|n| *n = n.map(|n| n + 1));
                }
                Err(e) => reserve_error.set(Some(p.error_text(&e))),
            }
            reserving.set(false);
        });
    };

    let start_over = move |_| {
        reserved.set(None);
        note.set(String::new());
        label.set(default_label());
        label_invalid.set(false);
        regenerate();
    };

    let address_text = Signal::derive(move || address.get().unwrap_or_default());
    let forward_meta = move || {
        p.forward_to.get().filter(|f| !f.is_empty()).map(|f| {
            view! {
                <div class="v-address-card__meta">
                    <Icon icon=I::Forward class="v-icon--sm"/>
                    <span>{templ(c.t(T::ForwardsTo), f)}</span>
                </div>
            }
        })
    };

    // ----- form (before reserving) -----
    let form = move || {
        view! {
            <section class="v-address-card" aria-live="polite" aria-busy=move || generating.get().to_string()>
                <div class="v-address-card__eyebrow">
                    <span>{move || if generating.get() { c.t(T::EyebrowPicking) } else { c.t(T::EyebrowNew) }}</span>
                </div>
                <div class="v-address-card__row">
                    {move || match (address.get(), generating.get()) {
                        (None, true) => view! { <span class="v-skeleton v-skeleton--line" style="width: 70%; height: 20px"></span> }.into_any(),
                        _ => view! { <Address value=address_text/> }.into_any(),
                    }}
                    <button
                        class="v-icon-btn v-icon-btn--secondary"
                        class:is-spinning=move || generating.get()
                        type="button"
                        aria-label=move || if generating.get() { c.t(T::Generating) } else { c.t(T::Regenerate) }
                        title=tr(T::Regenerate)
                        aria-busy=move || generating.get().to_string()
                        disabled=move || reserving.get()
                        on:click=move |_| if !generating.get_untracked() { regenerate() }
                    >
                        <Icon icon=I::Refresh/>
                    </button>
                </div>
                {forward_meta}
            </section>

            {move || gen_error.get().map(|m| view! { <ErrorAlert title=c.t(T::GenerateFailed) message=m/> })}
            {move || reserve_error.get().map(|m| view! { <ErrorAlert title=c.t(T::ReserveFailed) message=m/> })}

            <form class="v-form v-form--fill" on:submit=on_submit aria-busy=move || reserving.get().to_string() novalidate>
                <div class="v-field">
                    <label class="v-label" for="gen-label">{tr(T::Label)}</label>
                    <input
                        class="v-input"
                        id="gen-label"
                        type="text"
                        autocomplete="off"
                        placeholder=tr(T::LabelPlaceholder)
                        prop:value=move || label.get()
                        on:input=move |ev| {
                            label.set(event_target_value(&ev));
                            label_invalid.set(false);
                        }
                        aria-invalid=move || label_invalid.get().to_string()
                        aria-describedby="gen-label-msg"
                        disabled=move || reserving.get()
                    />
                    {move || label_invalid.get().then(|| view! {
                        <span class="v-field__error" id="gen-label-msg">{tr(T::LabelRequired)}</span>
                    })}
                </div>
                <div class="v-field v-field--grow">
                    <label class="v-label" for="gen-note">
                        {tr(T::Note)}" "<span class="v-label__hint">{tr(T::NoteOptional)}</span>
                    </label>
                    <textarea
                        class="v-textarea"
                        id="gen-note"
                        rows="2"
                        placeholder=tr(T::NotePlaceholder)
                        prop:value=move || note.get()
                        on:input=move |ev| note.set(event_target_value(&ev))
                        disabled=move || reserving.get()
                    ></textarea>
                </div>
                <button
                    class="v-btn v-btn--primary v-btn--lg v-btn--block"
                    type="submit"
                    aria-busy=move || reserving.get().to_string()
                    disabled=move || generating.get() || address.get().is_none()
                >
                    {move || if reserving.get() {
                        view! { <span class="v-spinner"></span>{c.t(T::Reserving)} }.into_any()
                    } else if reserve_error.get().is_some() {
                        c.t(T::TryAgain).into_any()
                    } else {
                        c.t(T::UseAddress).into_any()
                    }}
                </button>
            </form>
        }
    };

    // ----- success -----
    let success = move |r: HmeEmail| {
        let hme_text = r.hme.clone();
        let (copy_text, fill_text) = (r.hme.clone(), r.hme.clone());
        let fwd = if r.forward_to_email.is_empty() {
            p.forward_to.get_untracked().unwrap_or_default()
        } else {
            r.forward_to_email.clone()
        };
        let label_text = r.label.clone();
        view! {
            <section class="v-address-card v-address-card--success" aria-live="polite">
                <div class="v-address-card__eyebrow">
                    <span><Icon icon=I::CheckDraw class="v-icon--sm"/>{tr(T::Reserved)}</span>
                </div>
                <div class="v-address-card__row">
                    <Address value=Signal::derive(move || hme_text.clone())/>
                </div>
                <div class="v-address-card__meta">
                    <Icon icon=I::Forward class="v-icon--sm"/>
                    <span>{move || templ2(c.t(T::LabelAndForward), label_text.clone(), fwd.clone())}</span>
                </div>
            </section>
            <div class="v-actions">
                <button class="v-btn v-btn--primary v-btn--lg" type="button" on:click=move |_| p.copy(copy_text.clone())>
                    <Icon icon=I::Copy/>{tr(T::Copy)}
                </button>
                <button class="v-btn v-btn--secondary v-btn--lg" type="button" on:click=move |_| p.fill_on_page(fill_text.clone())>
                    <Icon icon=I::Fill/>{tr(T::FillOnPage)}
                </button>
            </div>
            <button class="v-btn v-btn--ghost v-btn--block" type="button" on:click=start_over>
                <Icon icon=I::Plus/>{tr(T::CreateAnother)}
            </button>
        }
    };

    view! {
        <main class="v-popup__body v-view">
            {move || match reserved.get() {
                Some(r) => success(r).into_any(),
                None => form().into_any(),
            }}
        </main>
        <FooterNav primary=Screen::Manage/>
    }
}
