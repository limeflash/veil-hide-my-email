//! "My addresses": search, filter, details, edit, (de)activate, delete.

use super::{pctx, FooterNav, ListState, Screen};
use crate::icons::{Icon, I};
use crate::widgets::{format_date, Address, DateStyle, ErrorAlert};
use crate::{ctx, tr, ToastKind};
use hme_core::i18n::{fill, T};
use hme_core::model::HmeEmail;
use hme_core::search;
use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Filter {
    All,
    Active,
    Inactive,
}

impl Filter {
    fn keeps(self, e: &HmeEmail) -> bool {
        match self {
            Filter::All => true,
            Filter::Active => e.is_active,
            Filter::Inactive => !e.is_active,
        }
    }
}

#[component]
pub fn Manage() -> impl IntoView {
    let p = pctx();
    p.reload_list();
    let back = RwSignal::new(false);
    // Remember the navigation direction for the slide animation.
    Effect::new(move |prev: Option<bool>| {
        let open = p.selected.get().is_some();
        if prev == Some(true) && !open {
            back.set(true);
        } else if open {
            back.set(false);
        }
        open
    });
    view! {
        {move || match p.selected.get() {
            Some(_) => view! { <Details/> }.into_any(),
            None => view! { <AddressList back=back/> }.into_any(),
        }}
        <FooterNav primary=Screen::Generate/>
    }
}

#[component]
fn AddressList(back: RwSignal<bool>) -> impl IntoView {
    let p = pctx();
    let c = ctx();
    let query = RwSignal::new(String::new());
    let filter = RwSignal::new(Filter::All);

    // Search first, then count per filter, then filter.
    let matched = Memo::new(move |_| {
        let q = query.get();
        p.list.with(|l| {
            let items = l.items();
            search::search(items, &q)
                .into_iter()
                .map(|i| items[i].clone())
                .collect::<Vec<_>>()
        })
    });
    let count = move |f: Filter| matched.with(|m| m.iter().filter(|e| f.keeps(e)).count());
    let visible = move || {
        let f = filter.get();
        matched.with(|m| m.iter().filter(|e| f.keeps(e)).cloned().collect::<Vec<_>>())
    };
    let loading = move || matches!(p.list.get(), ListState::Loading);

    let segment = move |f: Filter, key: T| {
        view! {
            <label class="v-segmented__item">
                <input
                    type="radio"
                    name="manage-filter"
                    prop:checked=move || filter.get() == f
                    disabled=loading
                    on:change=move |_| filter.set(f)
                />
                <span>
                    {tr(key)}
                    {move || (!loading()).then(|| view! { <span class="v-segmented__count">{count(f)}</span> })}
                </span>
            </label>
        }
    };

    let toolbar = move || {
        view! {
            <div class="v-toolbar">
                <div class="v-input-group">
                    <Icon icon=I::Search/>
                    <input
                        class="v-input"
                        type="search"
                        aria-label=tr(T::SearchAria)
                        placeholder=tr(T::SearchPlaceholder)
                        prop:value=move || query.get()
                        on:input=move |ev| query.set(event_target_value(&ev))
                        disabled=loading
                        autofocus
                    />
                    {move || (!query.get().is_empty()).then(|| view! {
                        <button
                            class="v-icon-btn v-icon-btn--sm"
                            type="button"
                            aria-label=tr(T::ClearSearch)
                            on:click=move |_| query.set(String::new())
                        >
                            <Icon icon=I::X class="v-icon--sm"/>
                        </button>
                    })}
                </div>
                <div class="v-segmented v-segmented--fill" role="radiogroup" aria-label=tr(T::FilterAria)>
                    {segment(Filter::All, T::FilterAll)}
                    {segment(Filter::Active, T::FilterActive)}
                    {segment(Filter::Inactive, T::FilterInactive)}
                </div>
            </div>
        }
    };

    let skeleton = || {
        let widths = [(38, 72), (52, 66), (30, 70), (46, 62), (34, 74), (42, 64)];
        view! {
            <ul class="v-list" aria-hidden="true">
                {widths.into_iter().map(|(a, b)| view! {
                    <li class="v-list-item v-list-item--skeleton">
                        <span class="v-skeleton v-skeleton--avatar"></span>
                        <span class="v-list-item__body">
                            <span class="v-skeleton" style=format!("width: {a}%")></span>
                            <span class="v-skeleton v-skeleton--line" style=format!("width: {b}%")></span>
                        </span>
                    </li>
                }).collect_view()}
            </ul>
        }
    };

    let row = move |e: HmeEmail| {
        let id = e.anonymous_id.clone();
        let avatar: String = e.label.chars().next().map(|ch| ch.to_uppercase().collect()).unwrap_or_else(|| "?".into());
        let label = e.label.clone();
        let date = format_date(e.create_timestamp, DateStyle::Short);
        let inactive = !e.is_active;
        view! {
            <li>
                <button
                    class="v-list-item"
                    class:is-inactive=inactive
                    type="button"
                    on:click=move |_| p.selected.set(Some(id.clone()))
                >
                    <span class="v-list-item__avatar" aria-hidden="true">{avatar}</span>
                    <span class="v-list-item__body">
                        <span class="v-list-item__title">
                            <span>
                                {move || search::highlight(&label, &query.get()).into_iter().map(|(seg, hit)| {
                                    if hit { view! { <mark class="v-mark">{seg}</mark> }.into_any() } else { seg.into_any() }
                                }).collect_view()}
                            </span>
                            {inactive.then(|| view! { <span class="v-badge">{tr(T::Inactive)}</span> })}
                        </span>
                        <span class="v-list-item__sub">{e.hme.clone()}</span>
                    </span>
                    <span class="v-list-item__meta">{date}</span>
                </button>
            </li>
        }
    };

    let body = move || match p.list.get() {
        ListState::Loading => skeleton().into_any(),
        ListState::Failed(msg) => view! {
            <div class="v-empty v-empty--error" role="alert">
                <span class="v-empty__icon"><Icon icon=I::Alert class="v-icon--xl"/></span>
                <h2 class="v-empty__title">{tr(T::LoadFailed)}</h2>
                <p class="v-empty__text">{msg}</p>
                <button class="v-btn v-btn--secondary v-empty__action" type="button" on:click=move |_| {
                    p.list.set(ListState::Loading);
                    p.reload_list();
                }>
                    <Icon icon=I::Refresh/>{tr(T::Retry)}
                </button>
            </div>
        }
        .into_any(),
        ListState::Ready(items) if items.is_empty() => view! {
            <div class="v-empty v-empty--accent">
                <span class="v-empty__icon"><Icon icon=I::Inbox class="v-icon--xl"/></span>
                <h2 class="v-empty__title">{tr(T::EmptyTitle)}</h2>
                <p class="v-empty__text">{tr(T::EmptyBody)}</p>
                <button class="v-btn v-btn--primary v-empty__action" type="button" on:click=move |_| p.screen.set(Screen::Generate)>
                    <Icon icon=I::Plus/>{tr(T::NewAddress)}
                </button>
            </div>
        }
        .into_any(),
        ListState::Ready(_) => {
            let items = visible();
            if items.is_empty() {
                let q = query.get();
                view! {
                    <div class="v-empty" role="status">
                        <span class="v-empty__icon"><Icon icon=I::Search class="v-icon--xl"/></span>
                        <h2 class="v-empty__title">{tr(T::NoResultsTitle)}</h2>
                        <p class="v-empty__text">{move || fill(c.t(T::NoResultsBody), &[&q])}</p>
                        <button class="v-btn v-btn--secondary v-empty__action" type="button" on:click=move |_| {
                            query.set(String::new());
                            filter.set(Filter::All);
                        }>
                            {tr(T::ResetSearch)}
                        </button>
                    </div>
                }
                .into_any()
            } else {
                view! { <ul class="v-list">{items.into_iter().map(row).collect_view()}</ul> }.into_any()
            }
        }
    };

    let has_items = move || !matches!(&p.list.get(), ListState::Ready(v) if v.is_empty()) && !matches!(p.list.get(), ListState::Failed(_));
    view! {
        <main class="v-popup__body v-view" class:v-view--back=move || back.get()>
            {move || has_items().then(toolbar)}
            {body}
        </main>
    }
}

#[component]
fn Details() -> impl IntoView {
    let p = pctx();
    let c = ctx();
    let editing = RwSignal::new(false);
    let confirm = RwSignal::new(false);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);

    let item = Memo::new(move |_| p.selected_item());

    // ----- actions -----
    let set_active = move |active: bool, with_undo: bool| {
        let (Some(client), Some(e)) = (p.client(), item.get_untracked()) else { return };
        let id = e.anonymous_id.clone();
        busy.set(true);
        error.set(None);
        spawn_local(async move {
            let lang = c.lang_now();
            match client.set_active(&id, active).await {
                Ok(()) => {
                    p.update_item(&id, |e| e.is_active = active);
                    if active {
                        c.toast(ToastKind::Success, T::Reactivated.get(lang));
                    } else if with_undo {
                        let undo_client = client.clone();
                        let undo_id = id.clone();
                        c.toast_with_action(ToastKind::Success, T::Deactivated.get(lang), T::Undo.get(lang), move || {
                            let (client, id) = (undo_client.clone(), undo_id.clone());
                            spawn_local(async move {
                                if client.set_active(&id, true).await.is_ok() {
                                    p.update_item(&id, |e| e.is_active = true);
                                }
                            });
                        });
                    }
                }
                Err(e) => error.set(Some(p.error_text(&e))),
            }
            busy.set(false);
        });
    };

    let delete = move || {
        let (Some(client), Some(e)) = (p.client(), item.get_untracked()) else { return };
        let id = e.anonymous_id.clone();
        busy.set(true);
        spawn_local(async move {
            let lang = c.lang_now();
            match client.delete(&id).await {
                Ok(()) => {
                    confirm.set(false);
                    p.selected.set(None);
                    p.list.update(|l| {
                        if let ListState::Ready(v) = l {
                            v.retain(|e| e.anonymous_id != id);
                        }
                    });
                    p.count.update(|n| *n = n.map(|n| n.saturating_sub(1)));
                    c.toast(ToastKind::Success, T::Deleted.get(lang));
                }
                Err(e) => {
                    confirm.set(false);
                    error.set(Some(p.error_text(&e)));
                }
            }
            busy.set(false);
        });
    };

    // ----- pieces -----
    let card = move |e: HmeEmail| {
        let hme = e.hme.clone();
        let copy_hme = e.hme.clone();
        let active = e.is_active;
        view! {
            <section class="v-address-card v-address-card--neutral" class:is-inactive=!active>
                <div class="v-address-card__eyebrow">
                    {if active {
                        view! { <span class="v-badge v-badge--green"><span class="v-badge__dot"></span>{tr(T::Active)}</span> }.into_any()
                    } else {
                        view! { <span class="v-badge"><span class="v-badge__dot"></span>{tr(T::Inactive)}</span> }.into_any()
                    }}
                </div>
                <div class="v-address-card__row">
                    <Address value=Signal::derive(move || hme.clone())/>
                    {(!active).then(|| view! {
                        <button class="v-icon-btn" type="button" aria-label=tr(T::CopyAddress) title=tr(T::CopyAddress)
                            on:click=move |_| p.copy(copy_hme.clone())>
                            <Icon icon=I::Copy/>
                        </button>
                    })}
                </div>
                {(!active).then(|| view! {
                    <div class="v-address-card__meta">
                        <Icon icon=I::Ban class="v-icon--sm"/>
                        <span>{tr(T::NotForwarding)}</span>
                    </div>
                })}
            </section>
        }
    };

    let kv = move |e: HmeEmail, with_label_and_note: bool| {
        let created = format_date(e.create_timestamp, DateStyle::LongWithTime);
        let forward = if e.forward_to_email.is_empty() {
            p.forward_to.get_untracked().unwrap_or_default()
        } else {
            e.forward_to_email.clone()
        };
        let note = e.note.clone();
        view! {
            <dl class="v-kv v-panel">
                {with_label_and_note.then(|| view! {
                    <div class="v-kv__row"><dt class="v-kv__key">{tr(T::Label)}</dt><dd class="v-kv__val">{e.label.clone()}</dd></div>
                })}
                <div class="v-kv__row"><dt class="v-kv__key">{tr(T::ForwardTo)}</dt><dd class="v-kv__val">{forward}</dd></div>
                <div class="v-kv__row"><dt class="v-kv__key">{tr(T::Created)}</dt><dd class="v-kv__val">{created}</dd></div>
                {with_label_and_note.then(|| if note.trim().is_empty() {
                    view! { <div class="v-kv__row"><dt class="v-kv__key">{tr(T::Note)}</dt><dd class="v-kv__val v-muted">{tr(T::NoNote)}</dd></div> }.into_any()
                } else {
                    view! { <div class="v-kv__row"><dt class="v-kv__key">{tr(T::Note)}</dt><dd class="v-kv__val">{note.clone()}</dd></div> }.into_any()
                })}
            </dl>
        }
    };

    let actions = move |e: HmeEmail| {
        if e.is_active {
            let (copy_hme, fill_hme) = (e.hme.clone(), e.hme.clone());
            view! {
                <div class="v-actions">
                    <button class="v-btn v-btn--primary" type="button" on:click=move |_| p.copy(copy_hme.clone())>
                        <Icon icon=I::Copy/>{tr(T::Copy)}
                    </button>
                    <button class="v-btn v-btn--secondary" type="button" on:click=move |_| p.fill_on_page(fill_hme.clone())>
                        <Icon icon=I::Fill/>{tr(T::FillOnPage)}
                    </button>
                </div>
            }
            .into_any()
        } else {
            view! {
                <div class="v-actions">
                    <button class="v-btn v-btn--secondary" type="button" disabled=move || busy.get()
                        aria-busy=move || busy.get().to_string() on:click=move |_| set_active(true, false)>
                        <Icon icon=I::Refresh/>{tr(T::Reactivate)}
                    </button>
                    <button class="v-btn v-btn--danger" type="button" disabled=move || busy.get() on:click=move |_| {
                        // A pending "Undo" toast would sit on top of the sheet.
                        c.dismiss_toasts();
                        confirm.set(true);
                    }>
                        <Icon icon=I::Trash/>{tr(T::Delete)}
                    </button>
                </div>
            }
            .into_any()
        }
    };

    let view_mode = move |e: HmeEmail| {
        let active = e.is_active;
        view! {
            {card(e.clone())}
            {actions(e.clone())}
            {move || error.get().map(|m| view! { <ErrorAlert title=c.t(T::ErrGeneric) message=m/> })}
            <h2 class="v-section-title">
                {tr(T::DetailsTitle)}
                <button class="v-btn v-btn--ghost v-btn--sm" type="button" on:click=move |_| editing.set(true)>
                    <Icon icon=I::Edit class="v-icon--sm"/>{tr(T::Edit)}
                </button>
            </h2>
            {kv(e, true)}
            {active.then(|| view! {
                <button class="v-btn v-btn--secondary v-btn--block" type="button" disabled=move || busy.get()
                    aria-busy=move || busy.get().to_string() on:click=move |_| set_active(false, true)>
                    <Icon icon=I::Ban/>{tr(T::Deactivate)}
                </button>
            })}
        }
    };

    let edit_mode = move |e: HmeEmail| {
        let label = RwSignal::new(e.label.clone());
        let note = RwSignal::new(e.note.clone());
        let invalid = RwSignal::new(false);
        let saving = RwSignal::new(false);
        let id = e.anonymous_id.clone();
        let on_submit = move |ev: SubmitEvent| {
            ev.prevent_default();
            let Some(client) = p.client() else { return };
            let (l, n, id) = (label.get_untracked().trim().to_owned(), note.get_untracked(), id.clone());
            if l.is_empty() {
                invalid.set(true);
                return;
            }
            saving.set(true);
            error.set(None);
            spawn_local(async move {
                let lang = c.lang_now();
                match client.update_metadata(&id, &l, &n).await {
                    Ok(()) => {
                        p.update_item(&id, |e| {
                            e.label = l.clone();
                            e.note = n.clone();
                        });
                        editing.set(false);
                        c.toast(ToastKind::Success, T::Saved.get(lang));
                    }
                    Err(e) => error.set(Some(p.error_text(&e))),
                }
                saving.set(false);
            });
        };
        view! {
            {card(e.clone())}
            <h2 class="v-section-title">{tr(T::EditingTitle)}</h2>
            {move || error.get().map(|m| view! { <ErrorAlert title=c.t(T::ErrGeneric) message=m/> })}
            <form class="v-panel v-panel--form" on:submit=on_submit novalidate>
                <div class="v-field">
                    <label class="v-label" for="edit-label">{tr(T::Label)}</label>
                    <input class="v-input" id="edit-label" type="text" autocomplete="off"
                        prop:value=move || label.get()
                        on:input=move |ev| { label.set(event_target_value(&ev)); invalid.set(false); }
                        aria-invalid=move || invalid.get().to_string()
                        disabled=move || saving.get() autofocus/>
                    {move || invalid.get().then(|| view! { <span class="v-field__error">{tr(T::LabelRequired)}</span> })}
                </div>
                <div class="v-field">
                    <label class="v-label" for="edit-note">{tr(T::Note)}" "<span class="v-label__hint">{tr(T::NoteOptional)}</span></label>
                    <textarea class="v-textarea" id="edit-note" rows="3"
                        prop:value=move || note.get()
                        on:input=move |ev| note.set(event_target_value(&ev))
                        disabled=move || saving.get()></textarea>
                </div>
                <div class="v-actions--end">
                    <button class="v-btn v-btn--ghost" type="button" on:click=move |_| { editing.set(false); error.set(None); }>
                        {tr(T::Cancel)}
                    </button>
                    <button class="v-btn v-btn--primary" type="submit" aria-busy=move || saving.get().to_string()>
                        <span class="v-spinner"></span>
                        {tr(T::Save)}
                    </button>
                </div>
            </form>
            {kv(e, false)}
        }
    };

    let sheet = move || {
        confirm.get().then(|| {
            let hme = item.get_untracked().map(|e| e.hme).unwrap_or_default();
            let template = c.t(T::DeleteConfirmBody);
            let (before, after) = template.split_once("{}").unwrap_or((template, ""));
            view! {
                <div class="v-scrim" on:click=move |_| if !busy.get_untracked() { confirm.set(false) }></div>
                <div class="v-sheet" role="alertdialog" aria-modal="true" aria-labelledby="del-title" aria-describedby="del-text">
                    <span class="v-sheet__handle" aria-hidden="true"></span>
                    <h2 class="v-sheet__title" id="del-title">{tr(T::DeleteConfirmTitle)}</h2>
                    <p class="v-sheet__text" id="del-text">{before}<span class="v-mono">{hme}</span>{after}</p>
                    <div class="v-sheet__actions">
                        <button class="v-btn v-btn--secondary" type="button" disabled=move || busy.get() on:click=move |_| confirm.set(false)>
                            {tr(T::Cancel)}
                        </button>
                        <button class="v-btn v-btn--danger" type="button" aria-busy=move || busy.get().to_string()
                            disabled=move || busy.get() on:click=move |_| delete()>
                            <span class="v-spinner"></span>
                            {tr(T::Delete)}
                        </button>
                    </div>
                </div>
            }
        })
    };

    view! {
        <main class="v-popup__body v-view">
            {move || match item.get() {
                Some(e) if editing.get() => edit_mode(e).into_any(),
                Some(e) => view_mode(e).into_any(),
                None => ().into_any(),
            }}
        </main>
        {sheet}
    }
}
