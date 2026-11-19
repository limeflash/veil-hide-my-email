// Service-worker bootstrap. MV3 only wakes a worker for events whose
// listeners were registered synchronously during the first run, and
// service-worker modules can't use top-level await — so every listener is
// registered here, right away, and forwards to the Rust/WASM module once it
// has loaded. All logic lives in crates/background.

import init, * as bg from './wasm/hme_background.js';

const api = globalThis.browser ?? globalThis.chrome;

const ready = init({
  module_or_path: new URL('./wasm/hme_background_bg.wasm', import.meta.url),
}).then(() => bg);

const run = (task) =>
  ready.then(task).catch((e) => console.error('[veil]', e));

// ---- messages from the popup and content scripts --------------------------

api.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (!message || typeof message.type !== 'string') return false;
  const task =
    message.type === 'inpageStrings'
      ? ready.then((b) => b.inpage_strings())
      : ready.then((b) => b.on_message(message));
  task.then(sendResponse, (e) => sendResponse({ ok: false, error: String(e) }));
  return true; // keep the channel open for the async reply
});

// ---- lifecycle ------------------------------------------------------------

api.runtime.onInstalled.addListener((details) =>
  run((b) => b.on_installed(details.reason)),
);
api.runtime.onStartup.addListener(() => run((b) => b.on_startup()));

api.storage.onChanged.addListener((changes, area) => {
  if (area === 'local' && 'options' in changes) {
    run((b) => b.on_options_changed());
  }
});

// ---- right-click menu -----------------------------------------------------

(api.contextMenus ?? api.menus).onClicked.addListener((info, tab) =>
  run((b) =>
    b.on_menu_click(
      String(info.menuItemId),
      info.pageUrl ?? tab?.url ?? null,
      tab?.id ?? null,
      info.frameId ?? null,
    ),
  ),
);

// ---- follow the icloud.com session ----------------------------------------

const SETUP_HOSTS = ['https://setup.icloud.com', 'https://setup.icloud.com.cn'];
const AUTH_URLS = SETUP_HOSTS.flatMap((h) =>
  ['accountLogin', 'validate', 'refreshWebAuth'].map((a) => `${h}/setup/ws/*/${a}*`),
);
const LOGOUT_URLS = SETUP_HOSTS.map((h) => `${h}/setup/ws/*/logout*`);

// Our own /validate calls hit these listeners too; skip them.
const fromExtension = (d) =>
  /(chrome|moz)-extension:\/\//.test(`${d.initiator ?? ''} ${d.originUrl ?? ''}`);

api.webRequest.onResponseStarted.addListener(
  (d) => run((b) => b.on_auth_response(d.url, d.statusCode, fromExtension(d))),
  { urls: AUTH_URLS },
);

api.webRequest.onResponseStarted.addListener(
  (d) => run((b) => b.on_logout_response(d.statusCode)),
  { urls: LOGOUT_URLS },
);

api.tabs.onUpdated.addListener((_id, change, tab) => {
  if (change.status === 'complete' && tab.url) {
    run((b) => b.on_tab_complete(tab.url));
  }
});

run((b) => b.boot());
