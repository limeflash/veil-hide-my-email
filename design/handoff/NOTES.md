# Veil — design handoff notes

Veil ("Hide My Email for iCloud") is drawn with the **Astryx** design system
(github.com/meta/astryx, MIT) and an original Astryx theme named **Postmark**.
The theme was called Moonveil before the unslop pass described in section 3.

## 1. How Astryx is used

- Astryx ships React 19 components styled with StyleX. Its pre-built `astryx.css` is atomic
  classes that only make sense together with the JS that assigns them. It does emit stable
  names (`astryx-button`, `data-astryx-*`), but those are theming *hooks* with no rules behind
  them, so there is nothing to reuse from plain HTML.
- So the Leptos UI uses two hand-written files:
  - `tokens.css` — Astryx's own custom-property **names** (from
    `packages/core/src/theme/tokens.stylex.ts`) with Postmark **values**.
  - `components.css` — each needed Astryx component restated in plain CSS from its source
    (anatomy, sizes, states), with `v-*` class names. Every section header names the Astryx
    component it mirrors.
- Nothing from Astryx is loaded at runtime. No fonts, CDNs, Tailwind or JS are needed for styling.

## 2. Theme "Postmark"

- Neutrals: one graphite ramp. Light: page `#FAFAFA`, cards `#FFFFFF`, text `#171717` /
  `#525252`. Dark: page `#141415`, cards `#19191B`, text `#EDEDED` / `#A1A1A6`.
- Accent `--color-accent` is ink, the way Astryx's own Neutral theme does it: `#171717` with
  white text in light, `#EDEDED` with `#141415` text in dark. It fills the primary button, the
  checked radio and switch, and the focus ring.
- The one coloured object is the **address label** (`v-address-card`): manila paper `#E9D8B4`
  with ink `#221C12`, the same in both modes because it is an object, not chrome. The logo
  tile, the extension icon and the in-page button's ready state use the same paper and ink.
- There is no stamp or other decoration on the label; the label itself is the one bright
  element. An inactive address loses the paper and becomes a plain card.
- Logo: a filled mark, the envelope's flap and body as two solid ink shapes with a V-shaped
  gap between them, on a flat paper tile. Same mark in `icon.svg`, `.v-logo` and the in-page button.
- Status colours are toned and only carry meaning: success `#15803D` / `#4CC38A`, error
  `#DC2626` / `#E5484D`, warning `#E9AF08` / `#F2BB55`.
- Radii: inner 6, element 8, container 12. Dark depth comes from surface steps and hairlines;
  light shadows are small and neutral.
- Type: Astryx default scale (base 14, ratio 1.2) on **system font stacks**; no font file is
  bundled. Weights are 400 and 500 only. Mono is used for addresses, dates and counts.
- Motion: `fast 150ms / medium 320ms / slow 800ms`, one curve `cubic-bezier(0.22, 1, 0.36, 1)`.
- Focus ring: 2px `--color-accent`, offset 2px.
- Theme-local tokens (Astryx convention `--astryx-theme-postmark-*`): `label-paper`,
  `label-ink`, `label-ink-secondary`, `label-hairline`, `label-fill`. The earlier
  `--astryx-theme-moonveil-*` tokens (`brand-gradient`, `aura`, `accent-glow`) are gone.
- App constants that are not Astryx tokens are `--veil-*` (popup size, page width, z-indexes).

### Switching theme

```
<html data-theme="light">   light
<html data-theme="dark">    dark
<html>                      follows prefers-color-scheme
```

The options page's System / Light / Dark control only sets or removes that attribute.
`[data-theme]` also works on a subtree. The dark values exist twice in `tokens.css`
(media query and attribute); keep the two blocks identical.

## 3. Unslop pass

Method: **unslop by Damir Sokolovsky (CC BY 4.0)**, github.com/heydamir/unslop,
https://heydamir.design/unslop. Its audit list, direction format and pass order were followed;
its scripts were not run.

### Audit of the Moonveil version

| Sign | Where it lived |
| --- | --- |
| Violet gradient tile with a white sheen and a coloured glow | `.v-logo` in every header, loading screen, welcome hero, `icon.svg` |
| Radial colour wash behind content | `.v-popup::before` and `.v-page::before` ("aura") |
| Iris/purple accent with no brand reason, on many elements per screen | primary button, address card ring and eyebrow, focus ring, links, switch, radio, step numbers, "Сейчас" badge |
| Purple-tinted neutrals | lavender page `#F3F3FA`, ink-violet dark surfaces and borders |
| Gradient wash and coloured ring on a card | `.v-address-card` (accent), `--success` (green) |
| Coloured glow | in-page button ready state, success flash ring |
| Icons with their own colours, on coloured tiles | `.v-tip__icon` purple / yellow / red |
| Mixed icon sizes and a heavy stroke | 14 / 16 / 20 / 32px at stroke 2 |
| Negative letter-spacing on small text | `.v-address` (15px), `.v-mono`, `.v-list-item__sub`, in-page tooltip address |
| Weight 600 used for ordinary hierarchy | field labels, titles, selected segment, banner titles |
| Pills for counts and meta | footer count badge, "Сейчас" and "Firefox" badges |
| Breathing glow on something that is not live data | `.v-logo--pulse` halo |
| Animated layout properties | switch thumb animated width / height / top / left |
| Three equal cards side by side | welcome steps |
| Everything stacked and centred, logo tile on top, no product visual | welcome hero |
| Headline that fits any product | «Veil установлен» |
| Trailing periods on one-sentence strings; wordy placeholder | tips, empty states, step texts; «Поиск по метке, адресу, заметке» |
| Off-grid values | 3px padding, 92px key column, 84px min-height, 15/22px address line |

Verdict: heavy on colour, logo and the welcome hero; light on spacing and structure. The full
pass order was run, within the fixed contract (Astryx token names, `v-*` classes, markup).

### Direction

- **Product.** Veil gives each website its own forwarding address from iCloud Hide My Email, so the real inbox stays private.
- **Character.** Discreet, exact, postal.
- **World.** A mail counter: envelopes, manila labels, ink, postmarks.
- **Moments.** One per screen: the address itself, shown as a manila mailing label.
- **Signature.** The manila label itself, with its dashed tear line. A postmark was tried and removed after user feedback: it read as a glitch over the address.

### What changed, pass by pass

1. **Spacing.** Off-grid values moved to the 4px scale (4px skeleton padding, 96px key column,
   80px note minimum, 16/24px address line). Tips rows 12px apart with one icon column.
2. **Color.** Graphite ramp replaces the violet-tinted one. Accent is ink. Aura washes, logo
   gradient, sheen, glows and the card gradients are removed. Status colours toned down.
   Inside the label the Astryx colour tokens are re-pointed at ink-on-paper, so nested
   IconButton, Badge and Skeleton need no special cases.
3. **Icons.** One size (16px) and one stroke (1.5); `.v-icon--sm` is now an alias of the default
   and `.v-icon--xl` is 24px for empty states only. Tip icons lose their tiles and take the
   secondary text colour.
4. **Copy.** See section 11.
5. **Type.** Weights 400 and 500 only. No negative tracking below 35px. Counts, dates and the
   label eyebrow are quiet mono text.
6. **Layout.** Welcome hero is an asymmetric split: brand row, headline and lead on the left,
   an address label on the right. Steps are one card with three numbered rows.
7. **Details.** Radii 6 / 8 / 12. Footer count and label status lose their pills. Logo is a flat
   paper tile with a filled ink mark.
8. **Visuals.** No generated imagery; an app popup does not need it. The label is built in CSS.
   `icon.svg` is redrawn flat and filled in the same paper and ink (plain paths, no filters or masks).
9. **Motion.** One easing curve. Logo halo removed (`.v-logo--pulse` is now a no-op). Switch
   thumb animates `transform` only. In-page success flash and its glow removed; enter scale is
   0.9 → 1.

### Class and markup changes from this pass

Token names and existing `v-*` classes are unchanged. Differences:

| What | Change |
| --- | --- |
| Theme-local tokens | `--astryx-theme-moonveil-*` → `--astryx-theme-postmark-label-*` (see section 2) |
| `v-address-card` | Same markup. Now a paper label, with no pseudo-element decoration. |
| Logo glyph | The SVG inside `.v-logo` and `.veil-icon--logo` is now two filled paths (see `icons.md`, "Logo (filled)"). Filled by `.v-logo .v-icon` / `.v-icon--filled` and by `.veil-icon--logo`. Glyph sizes: 20px in the 28px tile, 28px in `--lg`, 44px in `--xl`. |
| `v-address-card__eyebrow` | Mono, sentence case, ink-secondary; no accent colour. |
| `v-logo--pulse` | Kept, does nothing. |
| `v-tip--warn`, `v-tip--danger` | Kept, no visual difference; the bold words carry the meaning. |
| Firefox tags, "Сейчас" | Use plain `v-badge` instead of `v-badge--warning` / `v-badge--purple`. |
| `v-card--purple`, `v-chip--purple` | Removed. |
| `v-alert--accent` | Now a neutral notice (`--color-neutral` fill). Its action is `v-btn--secondary`. |
| Welcome hero | New: `v-hero__text`, `v-hero__visual`; the hero holds a `v-brand` row instead of `v-logo--xl`. |
| Welcome steps | `<ol class="v-card v-steps">` with `<li class="v-step">` rows; new `v-step__body`; the action is a direct child of the row. |
| `.v-popup::before`, `.v-page::before` | Removed. |
| Primary button hover | A surface mix instead of the overlay tint (the tint is invisible on ink). |

The earlier fix stays: a filled button keeps its own on-fill text colour wherever it is nested
(`.v-btn.v-btn--primary` and siblings).

## 4. Files

| File | What it is |
| --- | --- |
| `tokens.css` | Postmark tokens, light and dark |
| `components.css` | All `v-*` components |
| `popup.html` | 17 popup states (01–07 generate, 08–17 manage and details) |
| `options.html` | Options page plus status badge and Forward-to variants (B1–B7) |
| `welcome.html` | Welcome / quick start page (C1) |
| `inpage.html`, `inpage.css` | In-page button demo and its self-contained styles (D1–D2) |
| `components.html` | Icon sizes, theme swatches, component sampler (E1–E5) |
| `icon.svg` | Extension icon; rasterise to 16 / 32 / 48 / 128 |
| `icons.md` | The filled logo and the 33 line icons used, as inline SVG |
| `preview.css` | Preview chrome only (`.h-*`); do not port |

- Each state is a `<section class="h-state" data-state="…">`. Port the `.v-*` / `.veil-*` tree inside it.
- The HTML files are generated from the design-canvas artboards by `../build-handoff.py`, so
  they always match the published design. Edit the artboards in `../canvas/project/`, then rerun the script.
- `../old-custom/` holds the abandoned pre-Astryx design. It is not part of the handoff and can be deleted.
- Boolean attributes are written `checked="checked"`, `disabled="disabled"`, `open="open"`
  because the canvas runtime needs a value. Plain boolean attributes are fine in Leptos.
- The boards repeat `<main>` and `<h1>` once per state; a real screen has one of each.

## 5. Popup

- Fixed **380 × 560 px** (`--veil-popup-width/height`): header 56px, scrolling body (16px
  padding, 12px gaps), footer.
- Header: logo, title, subtitle (current host on Generate, forward-to address on Manage),
  then the settings IconButton. Drill-in views add `v-popup__header--nav` and start with a back IconButton.
- Footer: Generate has "Мои адреса" (with a count) and "Выйти". Manage has "Новый адрес"
  and "Выйти". Signed-out uses `v-popup__footer--stack` with the CTA and "check again".
- Navigation: Generate ⇄ Manage through the footer; Manage → Details is a drill-in. Add
  `v-view` to the body when a screen mounts and `v-view v-view--back` when returning.
- Generate: the form is `v-form v-form--fill` and the note field is `v-field v-field--grow`,
  so the note absorbs spare height and the CTA sits just above the footer.
- Search highlights fuzzy matches with `<mark class="v-mark">`.
- Delete is offered only on inactive addresses. It opens a BottomSheet confirm (`v-scrim` +
  `v-sheet`, `role="alertdialog"`); trap focus in it and close on Esc or scrim click.
- Toasts live in `v-toast-region` (absolute, above the footer). One at a time, auto-dismiss
  after about 2.5s (4s with an action); add `is-leaving`, then remove after `--duration-exit`.
  Use `role="status"` for success and `role="alert"` for errors.
- A busy button keeps its idle label and a `<span class="v-spinner"></span>` first child;
  `aria-busy="true"` hides the label and centres the spinner.

## 6. Component map

| Astryx component | Classes | Notes |
| --- | --- | --- |
| Button | `v-btn` `--primary` `--secondary` `--ghost` `--danger` · `--sm` 28 `--lg` 36 (default md 32) `--block` | Pressed scales 0.98 |
| IconButton | `v-icon-btn` `--secondary` `--primary` `--sm` `--lg` | Always give it `aria-label`. `--outline` is a deprecated alias of `--secondary`. |
| Spinner | `v-spinner` `--sm` `--lg` `--xl` `--secondary` | Ring: `--color-track` plus an accent arc |
| Skeleton | `v-skeleton` `--line` `--avatar` `--block` | Opacity breathes 0.25 ↔ 0.925 while loading |
| Field, TextInput, TextArea | `v-field` `v-label` `v-label__hint` `v-field__hint` `v-input` `v-textarea` | Focus = accent border + 2px inset `--color-accent-muted` ring |
| FieldStatus | `v-field__error`, `v-field__status--warning/--success` | Pale status block with an icon |
| TextInput with start icon | `v-input-group` > icon, `v-input`, clear `v-icon-btn` | The group is the box |
| Card | `v-card` `--muted` `--elevated`, `v-card__body`, `v-card__footer` | 1px border, flat |
| List, Item | `v-list`, `v-list-item` (`__avatar` `__body` `__title` `__sub` `__meta`), `.is-active`, `.is-inactive` | Hover `--color-overlay-hover`, selected `--color-accent-muted` |
| MetadataList | `v-kv` `__row` `__key` `__val` in `v-panel` | Details view |
| Badge | `v-badge` + `--info/--success/--warning/--error` (strong) or `--green/--yellow/--red/--blue/--gray/--purple` (pale), `v-badge__dot` | Dots in `--green` / `--yellow` take the status colour |
| Token | `v-chip` | Static tag |
| SegmentedControl | `v-segmented` `--fill` `--sm`, `v-segmented__item` (label > radio + span), `v-segmented__count` | Manage filter |
| Switch | `v-toggle` on `<input type="checkbox" role="switch">`, row `v-setting` | 40×24, thumb 16 → 20 by scale |
| RadioList | `v-radio-list`, `v-radio` > input + `v-radio__mark` + `v-radio__label` | Bare rows, 20px indicator |
| Banner | `v-alert` `--info/--warning/--error/--success/--accent`, `__body` `__title` `__action` | Pale fill, no border |
| Toast | `v-toast-region` (`--page`), `v-toast` `--success/--error`, `__content` `__action`, `.is-leaving` | Inverted surface |
| Tooltip | `v-tooltip` | Inverted surface |
| BottomSheet + AlertDialog | `v-scrim`, `v-sheet` `__handle` `__title` `__text` `__actions` | Attached to the popup's bottom edge |
| EmptyState | `v-empty` `--compact` `--error` `--accent`, `__icon` `__title` `__text` `__action` | Plain 24px icon |
| Section, Collapsible | `v-section…`, `v-faq__item` (`<details>`) | Options and welcome pages |

Veil-specific patterns built from those parts: `v-popup`, `v-brand`, `v-logo`,
`v-address-card` / `v-address`, `v-loading`, `v-intro`, `v-tips` / `v-tip`, `v-hero`,
`v-steps` / `v-step`, `v-page`.

## 7. Motion

| What | Spec |
| --- | --- |
| Hover / pressed, colour, border, ring | `--duration-fast` 150ms |
| Button press | `scale(0.98)`, 150ms |
| List row hover | `--duration-fast-min` 110ms |
| Switch thumb, segmented selection, radio dot | 150ms, transform / colour only |
| Screen enter (`v-view`) | fade + 8px slide from the right (`--back`: from the left), `--duration-medium` 320ms |
| Toast in / out | fade + 12px rise, 200ms / fade + 12px drop, `--duration-exit` 150ms |
| Sheet in / out | slide from the bottom edge, 320ms / 150ms; scrim fades in 150ms |
| Success tick (`v-check-draw`) | stroke draws in 320ms after an 80ms delay |
| Skeleton, label while generating | opacity pulse, `--duration-medium-max` 425ms, linear, alternate |
| Spinner, regenerate icon | 800ms linear rotation |

All of it uses one curve, `cubic-bezier(0.22, 1, 0.36, 1)`. With
`prefers-reduced-motion: reduce`, transitions go to 0s, enter animations and the
press scale are removed, spinners become an opacity pulse, and skeletons hold still.

## 8. In-page button (Shadow DOM)

- `inpage.css` is self-contained (its own `--veil-*` copies of Postmark values) and is the only stylesheet in the shadow root.
- Markup contract is unchanged: `.veil-root` > `.veil-btn[data-state]` (holding the three
  `.veil-icon--logo/--check/--alert` glyphs and `.veil-spinner`) + `.veil-tip`.
- `data-state`: `loading | ready | reserving | success | error`. All glyphs stay in the DOM; CSS shows the right one.
- `ready` and `reserving` are the paper tile with the filled ink mark; `loading` and `error` are a neutral chip; `success` is the status green with a check.
- Scheme follows the page's `prefers-color-scheme`. Set `data-scheme="light|dark"` on
  `.veil-root` when the page's real background disagrees with the OS setting.
- Size 28px, round. The host is positioned by the content script at the input's right edge,
  vertically centred, 6–8px inset.
- Tooltip opens on hover or keyboard focus after 250ms, or immediately with `.is-open`.
  `.veil-tip--below` flips it under the field; `.veil-tip--error` is the error variant.
- After `success`, remove the button about 900ms later.

## 9. Accessibility

- Text and icon colours are chosen for ≥ 4.5:1 on the surfaces they are used on, in both modes, including ink on the label paper.
- Status is never colour alone: every badge, banner, toast and field status has a word or an icon.
- Real controls throughout: `<button>`, `<a href>`, `<input>` + `<label for>`, `<details>`;
  switches are `role="switch"` checkboxes; segmented controls and radio lists are radio groups.
- Live regions: address label `aria-live="polite"`, banners `role="alert"`, loading `role="status"`.

## 10. Open items

- `options.html`: the "Исходный код" link is `href="#source"`; set the new repository URL.
- The credits name Dimitrios Dedoussis without a link; add the original repository URL if wanted.
- English strings were not drawn. Check the long ones ("Fill on page", "Inactive") in the
  half-width buttons and the segmented filter when the locale is wired.
- The welcome hero has no generated image. If one is wanted later, the label on the right is the slot.
- Checked: every handoff page was rendered in a browser in dark and light after the pass, with
  the theme forced through `data-theme`. Not checked: the prefers-color-scheme fallback, and
  the canvas boards themselves (same markup and CSS, but the canvas was not opened).

## 11. Copy changes

Mirror these in `crates/core/src/i18n.rs`. All strings are Russian; no English UI string was drawn.

| Where | Old | New |
| --- | --- | --- |
| Welcome hero headline | «Veil установлен» | «Свой адрес для каждого сайта» |
| Welcome hero lead | «Случайные адреса Hide My Email для iCloud — в один клик, прямо в браузере. Осталось три шага.» | «Veil установлен. Он создаёт адреса Hide My Email для iCloud прямо в браузере. Осталось три шага.» |
| Welcome step 2 text | «…нажмите на булавку рядом с Veil.» | same, no trailing period |
| Welcome step 3 text | «…или через правый клик по полю.» | same, no trailing period |
| Popup loading, second line (`v-loading__text`) | «Обычно это занимает пару секунд» | removed; the title «Проверяем сессию iCloud» stands alone |
| Signed-out tips (all five) | each ended with a period | no trailing period |
| Manage search placeholder | «Поиск по метке, адресу, заметке» | «Поиск» |
| Manage empty state text | «Создайте первый — он появится здесь, и вы сможете отключить его в любой момент.» | «Создайте первый — он появится здесь» |
| Options, Forward-to signed out | «Адреса для пересылки появятся после входа на iCloud.com.» | same, no trailing period |
| Busy buttons | «Сохраняем…» | keep the idle label («Использовать адрес», «Сохранить»); the spinner replaces it visually |

Rule applied: one-sentence helper texts, labels and captions end without a period; paragraphs
of two or more sentences keep theirs. Nothing new was claimed: the headline restates what the
product already does.
