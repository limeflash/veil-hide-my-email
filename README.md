<p align="center"><img src="extension/icon.svg" width="96" alt="Veil"></p>

<h1 align="center">Veil — Hide My Email for iCloud</h1>

<p align="center">
A browser extension for iCloud+ <b>Hide My Email</b>, written in <b>Rust</b> and compiled to WebAssembly.<br>
Chrome · Edge · Brave · Firefox (Manifest V3)
</p>

Veil creates random `@icloud.com` forwarding addresses right in the browser, so sites never see your real email. It is a ground-up Rust rewrite of [imNebula/icloud-hide-my-email-browser-extension](https://github.com/imNebula/icloud-hide-my-email-browser-extension), itself a fork of Dimitrios Dedoussis's original. It keeps every feature and adds a few.

> Veil is an independent project. It is not affiliated with, endorsed or sponsored by Apple Inc. iCloud and Hide My Email are trademarks of Apple Inc.

## Features

- **Popup**: proposes a fresh address for the current site. Regenerate it, add a label and an optional note, reserve it, then copy it or fill it into the page.
- **My addresses**: fuzzy search across label, address and note, with matches highlighted. Filter by all, active or inactive. The details view shows label, forward-to address, creation date and note.
- **Manage**: edit the label and note, deactivate (with undo), reactivate, and delete (inactive addresses only, after a confirmation).
- **In-field button**: when you focus an email field on any site, a small button appears inside it. One click reserves an address labelled with the site and fills it in. It works with React, Vue and Angular forms, is isolated in a Shadow DOM, and picks a light or dark style to match the page.
- **Right-click menu**: on any text field, "Generate and fill a hidden address" (also copies it to the clipboard).
- **Settings**: choose the forward-to address, toggle both autofill methods, pick the theme (system, light or dark) and the language.
- **15 languages**: English, Russian, Ukrainian, Spanish, Portuguese (Brazil), Italian, German, French, Polish, Turkish, Arabic, Persian, Simplified Chinese, Japanese and Korean. Arabic and Persian get a full right-to-left layout. Strings live in `crates/core/locales/*.json`, and the build fails if a translation drops a placeholder.
- **Session sync**: Veil picks up your iCloud.com sign-in automatically, follows sign-outs, and supports iCloud.com.cn.
- **Welcome page** with a quick start and troubleshooting tips (passkeys, Advanced Data Protection, Firefox containers). In Firefox it also has a button to grant host access.

## Speed

The UI and the background logic are Rust compiled to WebAssembly (`hme_ui.wasm` ≈ 600 KB, `hme_background.wasm` ≈ 210 KB, compiled by the browser in a few milliseconds). The popup never waits for the network before it draws. It renders from the stored session and a memory-only `storage.session` cache, then refreshes from iCloud in parallel. Measured with `e2e/perf.mjs`, iCloud mocked at 300 ms latency:

| popup | WASM ready | usable (address shown) | list + count |
| --- | --- | --- | --- |
| before caching | 31 ms | 664 ms | 664 ms |
| now, repeat open | 23 ms | **44 ms** | **44 ms** |
| now, first open per browser session | — | ≈ 330 ms (one round-trip) | ≈ 330 ms |

## How it works

```
crates/
  core/        pure Rust: iCloud API (as data), models, message protocol,
               fuzzy search, ru/en strings; unit-tested with `cargo test`
  web/         WASM runtime: fetch-based iCloud client, chrome.*/browser.*
               bindings, typed storage
  background/  service worker logic (WASM): session sync, context menu,
               generate/reserve for content scripts
  ui/          popup, options and welcome pages (Leptos → WASM)
extension/
  static/      manifest assets, HTML shells, background.js bootstrap,
               content.js (the in-page button; tiny plain JS on purpose)
  styles/      design tokens + components (Astryx-based "Postmark" theme)
xtask/         `cargo xtask build|package`: wasm-bindgen, wasm-opt,
               manifests for Chrome and Firefox, icons, zips
e2e/           Playwright smoke test and startup benchmark (mocked iCloud)
```

Veil talks to the same endpoints as icloud.com (`setup.icloud.com/setup/ws/1/validate`, then `premiummailsettings` `/v2/hme/list` and `/v1/hme/{generate,reserve,updateMetaData,deactivate,reactivate,delete,updateForwardTo}`). It uses the cookies of your signed-in iCloud.com session. A `declarativeNetRequest` rule sets the `Origin`/`Referer` headers that iCloud expects. There is no server: nothing leaves your browser except the requests to Apple.

## Build

Requirements: Rust **1.93.1** with the `wasm32-unknown-unknown` target (pinned in `rust-toolchain.toml`; rustup picks it up), `wasm-bindgen-cli` **0.2.130** (must match `Cargo.toml`), and `wasm-opt` from binaryen **133** for release builds. `--dev` builds skip `wasm-opt`.

```bash
cargo install wasm-bindgen-cli --version 0.2.130 --locked
brew install binaryen          # must be version 133, or use scripts/build-for-amo.sh

cargo xtask build              # → dist/chrome, dist/firefox
cargo xtask package            # + dist/veil-{chrome,firefox,source}-<version>.zip
cargo test -p hme-core         # unit tests
```

### Reproducible build (for addons.mozilla.org reviewers)

Release builds are byte-for-byte reproducible. `rust-toolchain.toml` pins the compiler, `Cargo.lock` pins every crate, the WASM tools are pinned by version, and source paths are remapped so the machine that builds doesn't matter. One command downloads the pinned tools and builds:

```bash
sh scripts/build-for-amo.sh    # or: npm run build-for-amo
```

It runs on Linux (x86_64, aarch64) and macOS and needs `curl`, `tar` and a C linker for Rust's build tools. On Ubuntu 24.04 that means `sudo apt-get install -y build-essential curl`. If rustup is missing, the script installs it with the official installer. wasm-opt 133 and wasm-bindgen 0.2.130 are downloaded from their GitHub releases into `./.tools`. The result is `dist/firefox`, identical to the submitted package. Node.js is not used by the build. No other files are generated.

What produces each file in `dist/firefox`:
- `wasm/*.wasm`, `wasm/*.js`, `wasm/snippets/`: Rust crates in `crates/` compiled to WebAssembly, then `wasm-bindgen` (JS glue) and `wasm-opt -Oz`.
- `content.js`: `extension/static/content.js` with `extension/styles/inpage.css` inlined as a string.
- `icons/*.png`: rendered from `extension/icon.svg` by `xtask` (resvg).
- `manifest.json`: generated by `xtask/src/main.rs`.
- Everything else is copied unchanged from `extension/static` and `extension/styles`.

Load it:
- **Chrome / Edge / Brave**: open `chrome://extensions`, enable Developer mode, click "Load unpacked" and choose `dist/chrome`.
- **Firefox** (140+): open `about:debugging#/runtime/this-firefox`, click "Load Temporary Add-on…" and choose `dist/firefox/manifest.json`.

### End-to-end test

```bash
cargo xtask build
cd e2e && npm i && npx playwright install chromium && npm test
```

The test loads the extension into Chromium and mocks iCloud, so no account is needed. It then exercises every feature and writes screenshots to `e2e/shots/`.

`npm run test:firefox` does the same in the installed Firefox (140+) through WebDriver; Selenium fetches geckodriver. A local proxy sends `*.icloud.com` to the mock and refuses everything else. Screenshots go to `e2e/shots-firefox/`.

### Security model

- **Content scripts are untrusted.** The background only accepts two requests from web pages: "is there a session" (answered without account data) and "create an address for this site". It derives the label from the sender tab's URL, never from the message. Sign-out, the address list and every edit are only reachable from the extension's own pages.
- **In-field button.** Nothing is sent to iCloud until a real, left-button click lands on the button after it has been on screen and still for 500 ms. The button sits in the top layer (`popover`), its styles are re-asserted every frame, and on Chromium the click also requires IntersectionObserver v2 to report it visible. No address enters the page before it is filled in. The button only appears in the top frame or same-origin frames; any frame can still use the right-click menu.
- **Headers.** Chromium uses `declarativeNetRequest`, which only applies to requests from initiators the extension has host access to (itself and iCloud). Firefox grants content-script match patterns as host permissions, so there a blocking `webRequest` listener sets `Origin`/`Referer` on the extension's own requests only.
- **Storage.** `storage.local` holds only settings and the iCloud service URLs; the account name and Apple ID stay in memory. The address list cache is in memory-only `storage.session`. Stored URLs are re-validated as `https://*.icloud.com(.cn)` on load.
- **Permissions.** `activeTab` instead of `tabs`, so there is no "read your browsing history" prompt.

### Store review notes

- `web-ext lint` on `dist/firefox`: 0 errors, 1 warning (`UNSAFE_VAR_ASSIGNMENT` in `wasm/hme_ui.js`). It comes from Leptos's DOM renderer, which clones view templates through `<template>.innerHTML`. The markup is static and compiled into the WASM module; no user or page data ever reaches it. Icons and all user text are created with DOM APIs.
- Extension pages need `'wasm-unsafe-eval'` in `content_security_policy.extension_pages` to instantiate WebAssembly. That is the only CSP relaxation.
- The content script is plain JavaScript and never loads WASM, so pages pay almost nothing for it.

## License

MIT — see [LICENSE](LICENSE). Includes the original copyright of Dimitrios Dedoussis.
