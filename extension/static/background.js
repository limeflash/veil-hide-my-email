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

const OWN_URL = api.runtime.getURL('');

// ---- messages from the popup and content scripts --------------------------

api.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (sender.id !== api.runtime.id) return false;
  if (!message || typeof message.type !== 'string') return false;
  // Extension pages (popup, options, welcome) vs. content scripts in web pages.
  const fromExtensionPage = typeof sender.url === 'string' && sender.url.startsWith(OWN_URL);
  const tabUrl = fromExtensionPage ? null : sender.tab?.url ?? null;
  const task =
    message.type === 'inpageStrings'
      ? ready.then((b) => b.inpage_strings())
      : ready.then((b) => b.on_message(message, fromExtensionPage, tabUrl));
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
// Only icloud.com's own sign-in/out calls count, not pages that merely
// navigate to or embed those URLs.
const fromICloud = (d) =>
  d.method === 'POST' &&
  /^https:\/\/([a-z0-9-]+\.)*icloud\.com(\.cn)?(\/|$)/.test(d.initiator ?? d.originUrl ?? '');
const XHR = { types: ['xmlhttprequest'] };

api.webRequest.onResponseStarted.addListener(
  (d) => {
    if (fromICloud(d)) run((b) => b.on_auth_response(d.url, d.statusCode, fromExtension(d)));
  },
  { urls: AUTH_URLS, ...XHR },
);

api.webRequest.onResponseStarted.addListener(
  (d) => {
    if (fromICloud(d)) run((b) => b.on_logout_response(d.statusCode));
  },
  { urls: LOGOUT_URLS, ...XHR },
);

// Firefox: set the Origin/Referer iCloud expects, on the extension's own
// requests only (Chromium does this with the DNR rules in rules.json).
if (OWN_URL.startsWith('moz-extension://') && api.webRequest.onBeforeSendHeaders) {
  api.webRequest.onBeforeSendHeaders.addListener(
    (d) => {
      if (!(d.originUrl ?? '').startsWith(OWN_URL)) return {};
      const origin = new URL(d.url).hostname.endsWith('.icloud.com.cn')
        ? 'https://www.icloud.com.cn'
        : 'https://www.icloud.com';
      const requestHeaders = d.requestHeaders.filter((h) => !/^(origin|referer)$/i.test(h.name));
      requestHeaders.push({ name: 'Origin', value: origin }, { name: 'Referer', value: `${origin}/` });
      return { requestHeaders };
    },
    { urls: ['https://*.icloud.com/*', 'https://*.icloud.com.cn/*'], ...XHR },
    ['blocking', 'requestHeaders'],
  );
}

// No tabs.onUpdated listener on purpose: it would wake the worker on every
// page load. The content script on icloud.com pages sends syncAuth instead.

run((b) => b.boot());
