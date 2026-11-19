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
- **Settings**: choose the forward-to address, toggle both autofill methods, pick the theme (system, light or dark) and the language (auto, Russian or English).
- **Session sync**: Veil picks up your iCloud.com sign-in automatically, follows sign-outs, and supports iCloud.com.cn.
- **Welcome page** with a quick start and troubleshooting tips (passkeys, Advanced Data Protection, Firefox containers). In Firefox it also has a button to grant host access.

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
  styles/      design tokens + components (Astryx-based "Moonveil" theme)
xtask/         `cargo xtask build|package`: wasm-bindgen, wasm-opt,
               manifests for Chrome and Firefox, icons, zips
e2e/           Playwright smoke test with a mocked iCloud API
```

Veil talks to the same endpoints as icloud.com (`setup.icloud.com/setup/ws/1/validate`, then `premiummailsettings` `/v2/hme/list` and `/v1/hme/{generate,reserve,updateMetaData,deactivate,reactivate,delete,updateForwardTo}`). It uses the cookies of your signed-in iCloud.com session. A `declarativeNetRequest` rule sets the `Origin`/`Referer` headers that iCloud expects. There is no server: nothing leaves your browser except the requests to Apple.

## Build

Requirements: Rust (stable) with the `wasm32-unknown-unknown` target, `wasm-bindgen-cli` **0.2.130** (must match `Cargo.toml`), and optionally `wasm-opt` (binaryen) for smaller files.

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.130 --locked
brew install binaryen          # optional

cargo xtask build              # → dist/chrome, dist/firefox
cargo xtask package            # + dist/veil-{chrome,firefox}-<version>.zip
cargo test -p hme-core         # unit tests
```

Load it:
- **Chrome / Edge / Brave**: open `chrome://extensions`, enable Developer mode, click "Load unpacked" and choose `dist/chrome`.
- **Firefox**: open `about:debugging#/runtime/this-firefox`, click "Load Temporary Add-on…" and choose `dist/firefox/manifest.json`.

### End-to-end test

```bash
cargo xtask build
cd e2e && npm i && npx playwright install chromium && npm test
```

The test loads the extension into Chromium and mocks iCloud, so no account is needed. It then exercises every feature and writes screenshots to `e2e/shots/`.

## License

MIT — see [LICENSE](LICENSE). Includes the original copyright of Dimitrios Dedoussis.
