// Thin, promise-based facade over the WebExtension API, imported by the Rust
// side through wasm-bindgen. Firefox exposes `browser.*` (promises), Chromium
// exposes `chrome.*` (promises in MV3). Keep this file logic-free: every
// decision lives in Rust.

const api = globalThis.browser ?? globalThis.chrome;

export function isFirefox() {
  return api.runtime.getURL('').startsWith('moz-extension://');
}

export function uiLanguage() {
  try {
    return api.i18n.getUILanguage();
  } catch {
    return globalThis.navigator?.language ?? 'en';
  }
}

export function getURL(path) {
  return api.runtime.getURL(path);
}

// ---- storage.local --------------------------------------------------------

export async function storageGet(key) {
  const result = await api.storage.local.get(key);
  return result[key] ?? null;
}

export async function storageSet(key, value) {
  if (value === undefined || value === null) {
    await api.storage.local.remove(key);
  } else {
    await api.storage.local.set({ [key]: value });
  }
}

// ---- storage.session (memory only; cleared when the browser closes) --------

const memory = new Map();

export async function sessionGet(key) {
  if (!api.storage.session) return memory.get(key) ?? null;
  const result = await api.storage.session.get(key);
  return result[key] ?? null;
}

export async function sessionSet(key, value) {
  if (!api.storage.session) {
    value === undefined || value === null ? memory.delete(key) : memory.set(key, value);
  } else if (value === undefined || value === null) {
    await api.storage.session.remove(key);
  } else {
    await api.storage.session.set({ [key]: value });
  }
}

/// Calls `cb(key, newValue)` for every change in storage.local.
export function onStorageChanged(cb) {
  api.storage.onChanged.addListener((changes, area) => {
    if (area !== 'local') return;
    for (const [key, change] of Object.entries(changes)) {
      cb(key, change.newValue ?? null);
    }
  });
}

// ---- messaging ------------------------------------------------------------

export function sendMessage(message) {
  return api.runtime.sendMessage(message);
}

export async function sendToTab(tabId, message, frameId) {
  const options = frameId === undefined || frameId === null ? undefined : { frameId };
  try {
    return await api.tabs.sendMessage(tabId, message, options);
  } catch (e) {
    // No content script in that tab (chrome:// pages, the Web Store…).
    return null;
  }
}

// ---- tabs -----------------------------------------------------------------

export async function activeTab() {
  const [tab] = await api.tabs.query({ active: true, lastFocusedWindow: true });
  return tab ?? null;
}

export function openTab(url) {
  return api.tabs.create({ url });
}

export function openOptionsPage() {
  return api.runtime.openOptionsPage();
}

// ---- context menu ---------------------------------------------------------

const menus = () => api.contextMenus ?? api.menus;

export async function menuRecreate(id, title, enabled, visible) {
  await menus().removeAll();
  await new Promise((resolve) => {
    const documentUrlPatterns = ['http://*/*', 'https://*/*'];
    menus().create({ id, title, enabled, visible, contexts: ['editable'], documentUrlPatterns }, () => {
      void api.runtime.lastError;
      resolve();
    });
  });
}

export async function menuUpdate(id, title, enabled, visible) {
  try {
    await menus().update(id, { title, enabled, visible });
  } catch {
    // The item doesn't exist yet (e.g. right after an update on Firefox).
    await menuRecreate(id, title, enabled, visible);
  }
}

// ---- misc -----------------------------------------------------------------

export async function notify(title, message) {
  try {
    // A fixed id replaces an earlier notification instead of stacking.
    await api.notifications.create('veil', {
      type: 'basic',
      title,
      message,
      iconUrl: api.runtime.getURL('icons/icon-128.png'),
    });
  } catch {
    // Notifications may be disabled by the OS; not worth failing over.
  }
}

export async function copyText(text) {
  await navigator.clipboard.writeText(text);
}

export const ICLOUD_ORIGINS = ['https://*.icloud.com/*', 'https://*.icloud.com.cn/*'];

export function hasHostAccess() {
  return api.permissions.contains({ origins: ICLOUD_ORIGINS });
}

export function requestHostAccess() {
  return api.permissions.request({ origins: ICLOUD_ORIGINS });
}
