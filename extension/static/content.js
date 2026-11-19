// Veil content script: the in-field button and the right-click fill.
//
// Plain JS on purpose — it runs on every page, so it stays tiny and never
// loads WASM there. All decisions about iCloud (sessions, generating,
// reserving) happen in the Rust background worker; this file only finds email
// fields, draws the button inside a closed Shadow DOM and fills values in.
(() => {
  'use strict';

  const api = globalThis.browser ?? globalThis.chrome;
  if (!api?.runtime?.id || globalThis.__veilContentLoaded) return;
  globalThis.__veilContentLoaded = true;

  // Replaced with the contents of styles/inpage.css by `cargo xtask build`.
  const INPAGE_CSS = /*__INPAGE_CSS__*/'';

  const BUTTON = 28;
  const INSET = 6;
  const SUCCESS_MS = 900;
  const ERROR_MS = 3500;

  // ---------------------------------------------------------------- helpers

  const send = (message) =>
    api.runtime.sendMessage(message).catch((e) => ({ ok: false, error: String(e?.message ?? e) }));

  let strings = null;
  const FALLBACK_STRINGS = {
    clickToFill: 'Click to fill',
    generating: 'Creating a hidden address…',
    reserving: 'Saving the address…',
    filled: 'Done — address filled',
    signedOut: 'Sign in to iCloud first',
    appName: 'Veil',
  };
  const loadStrings = async () => {
    if (!strings) {
      const s = await send({ type: 'inpageStrings' });
      strings = { ...FALLBACK_STRINGS, ...(s && !('ok' in s) ? s : {}) };
    }
    return strings;
  };

  const deepActiveElement = () => {
    let el = document.activeElement;
    while (el?.shadowRoot?.activeElement) el = el.shadowRoot.activeElement;
    return el;
  };

  const NON_TEXT_TYPES = new Set([
    'hidden', 'password', 'checkbox', 'radio', 'submit', 'button', 'reset', 'file',
    'image', 'range', 'color', 'date', 'datetime-local', 'month', 'week', 'time',
    'number', 'tel', 'url', 'search',
  ]);

  const isEmailInput = (el) => {
    if (!el || el.tagName !== 'INPUT' || el.disabled || el.readOnly) return false;
    const type = (el.getAttribute('type') || 'text').toLowerCase();
    if (type === 'email') return true;
    if (NON_TEXT_TYPES.has(type)) return false;
    const autocomplete = (el.getAttribute('autocomplete') || '').toLowerCase();
    if (/\bemail\b/.test(autocomplete)) return true;
    if ((el.getAttribute('inputmode') || '').toLowerCase() === 'email') return true;
    const hints = ['name', 'id', 'placeholder', 'aria-label']
      .map((a) => el.getAttribute(a) || '')
      .join(' ')
      .toLowerCase();
    return /e-?mail|почт/.test(hints);
  };

  const isEditable = (el) =>
    !!el && (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.isContentEditable);

  const isVisibleField = (el) => {
    if (!el?.isConnected) return false;
    const r = el.getBoundingClientRect();
    // Tiny or hidden fields are usually honeypots.
    if (r.width < 30 || r.height < 10) return false;
    const cs = getComputedStyle(el);
    return cs.display !== 'none' && cs.visibility !== 'hidden' && cs.opacity !== '0';
  };

  /** Writes a value so that React/Vue/Angular forms notice it. */
  const fill = (el, value) => {
    if (!el) return false;
    el.focus({ preventScroll: true });
    if (el.isContentEditable) {
      document.execCommand('selectAll', false);
      document.execCommand('insertText', false, value);
      return true;
    }
    const proto = el.tagName === 'TEXTAREA' ? HTMLTextAreaElement.prototype : HTMLInputElement.prototype;
    const setter = Object.getOwnPropertyDescriptor(proto, 'value')?.set;
    if (setter) setter.call(el, value);
    else el.value = value;
    el.dispatchEvent(new Event('input', { bubbles: true, composed: true }));
    el.dispatchEvent(new Event('change', { bubbles: true }));
    return true;
  };

  const copy = (text) => navigator.clipboard?.writeText(text).catch(() => {});

  /** Label stored with the address: the top-level site, without "www.". */
  const siteLabel = () => {
    let host = location.hostname;
    try {
      if (window.top !== window) {
        const origins = location.ancestorOrigins;
        if (origins?.length) host = new URL(origins[origins.length - 1]).hostname;
      }
    } catch {
      /* cross-origin parent: keep our own host */
    }
    return host.replace(/^www\./, '');
  };

  /** Luminance of the field's effective background, to pick the button scheme. */
  const schemeFor = (el) => {
    for (let n = el; n && n.nodeType === 1; n = n.parentElement) {
      const m = getComputedStyle(n).backgroundColor.match(/rgba?\(([^)]+)\)/);
      if (!m) continue;
      const [r, g, b, a = 1] = m[1].split(/[ ,/]+/).filter(Boolean).map(Number);
      if (a < 0.5) continue;
      const lum = (0.2126 * r + 0.7152 * g + 0.0722 * b) / 255;
      return lum < 0.45 ? 'dark' : 'light';
    }
    return null; // let prefers-color-scheme decide
  };

  // ----------------------------------------------------------------- widget

  const SVG_NS = 'http://www.w3.org/2000/svg';
  const svgIcon = (modifier, shapes) => {
    const svg = document.createElementNS(SVG_NS, 'svg');
    svg.setAttribute('viewBox', '0 0 24 24');
    svg.setAttribute('aria-hidden', 'true');
    svg.setAttribute('class', `veil-icon veil-icon--${modifier}`);
    for (const [tag, attrs] of shapes) {
      const node = document.createElementNS(SVG_NS, tag);
      for (const [k, v] of Object.entries(attrs)) node.setAttribute(k, v);
      svg.appendChild(node);
    }
    return svg;
  };

  const ICONS = {
    logo: [
      ['rect', { x: '3.5', y: '6', width: '17', height: '12.5', rx: '3' }],
      ['path', { d: 'm7.5 10 4.5 4 4.5-4' }],
    ],
    check: [['path', { d: 'M5 12.5l4.5 4.5L19 7.5' }]],
    alert: [
      ['circle', { cx: '12', cy: '12', r: '9' }],
      ['path', { d: 'M12 7.5v5' }],
      ['path', { d: 'M12 16.25v.01' }],
    ],
  };

  const el = (tag, cls, text) => {
    const n = document.createElement(tag);
    if (cls) n.className = cls;
    if (text) n.textContent = text;
    return n;
  };

  class Widget {
    constructor(target) {
      this.target = target;
      this.host = document.createElement('div');
      this.host.style.cssText =
        `all:initial;position:fixed;z-index:2147483647;top:0;left:0;width:${BUTTON}px;height:${BUTTON}px;display:block;`;
      const shadow = this.host.attachShadow({ mode: 'closed' });
      try {
        const sheet = new CSSStyleSheet();
        sheet.replaceSync(INPAGE_CSS);
        shadow.adoptedStyleSheets = [sheet];
      } catch {
        shadow.appendChild(el('style', null, INPAGE_CSS));
      }
      this.root = el('span', 'veil-root');
      this.btn = el('button', 'veil-btn');
      this.btn.type = 'button';
      this.btn.setAttribute('aria-describedby', 'veil-tip');
      this.btn.append(svgIcon('logo', ICONS.logo), svgIcon('check', ICONS.check), svgIcon('alert', ICONS.alert), el('span', 'veil-spinner'));
      this.tip = el('span', 'veil-tip');
      this.tip.id = 'veil-tip';
      this.tip.setAttribute('role', 'tooltip');
      this.root.append(this.btn, this.tip);
      shadow.appendChild(this.root);

      const scheme = schemeFor(target);
      if (scheme) this.root.dataset.scheme = scheme;

      // Keep focus in the field while pressing the button.
      this.btn.addEventListener('mousedown', (e) => {
        e.preventDefault();
        e.stopPropagation();
        onPress(this);
      });
      this.btn.addEventListener('click', (e) => e.preventDefault());

      (document.documentElement || document.body).appendChild(this.host);
      this.track();
    }

    track() {
      const step = () => {
        if (!this.host.isConnected) return;
        if (!isVisibleField(this.target)) {
          this.host.style.setProperty('display', 'none', 'important');
        } else {
          const r = this.target.getBoundingClientRect();
          this.host.style.setProperty('display', 'block', 'important');
          this.host.style.top = `${Math.round(r.top + (r.height - BUTTON) / 2)}px`;
          this.host.style.left = `${Math.round(r.right - BUTTON - INSET)}px`;
          this.tip.classList.toggle('veil-tip--below', r.top < 64);
        }
        this.frame = requestAnimationFrame(step);
      };
      step();
    }

    set(state, { title = '', addr = '', text = '', error = false, open = false } = {}) {
      this.state = state;
      this.btn.dataset.state = state;
      this.btn.setAttribute('aria-busy', String(state === 'loading' || state === 'reserving'));
      this.tip.replaceChildren();
      this.tip.classList.toggle('veil-tip--error', error);
      if (title) this.tip.append(el('span', 'veil-tip__title', title));
      if (addr) this.tip.append(el('span', 'veil-tip__addr', addr));
      if (text) this.tip.append(el('span', 'veil-tip__text', text));
      this.btn.setAttribute('aria-label', [title, addr, text].filter(Boolean).join(' '));
      this.tip.classList.toggle('is-open', open);
    }

    destroy() {
      cancelAnimationFrame(this.frame);
      clearTimeout(this.timer);
      this.host.remove();
    }
  }

  // ------------------------------------------------------------------ state

  let widget = null;
  let candidate = null; // generated but not yet reserved, reused across focuses
  let lastEmailInput = null;
  let buttonEnabled = true;
  let request = 0; // guards against stale replies

  const detach = () => {
    widget?.destroy();
    widget = null;
  };

  const attach = (target) => {
    if (widget?.target === target) return widget;
    detach();
    widget = new Widget(target);
    return widget;
  };

  const showError = async (w, reply) => {
    const s = await loadStrings();
    const message = reply?.signedOut ? s.signedOut : reply?.error || s.signedOut;
    w.set('error', { title: s.appName, text: message, error: true, open: true });
    clearTimeout(w.timer);
    w.timer = setTimeout(() => w === widget && w.tip.classList.remove('is-open'), ERROR_MS);
  };

  const generate = async (w) => {
    const s = await loadStrings();
    if (candidate) {
      w.set('ready', { title: s.clickToFill, addr: candidate });
      return;
    }
    const id = ++request;
    w.set('loading', { title: s.generating });
    const reply = await send({ type: 'generate' });
    if (id !== request || w !== widget) return;
    if (reply?.ok && reply.hme) {
      candidate = reply.hme;
      w.set('ready', { title: s.clickToFill, addr: candidate });
    } else {
      showError(w, reply);
    }
  };

  const reserve = async (w) => {
    const s = await loadStrings();
    const hme = candidate;
    const id = ++request;
    w.set('reserving', { title: s.reserving, addr: hme, open: true });
    const reply = await send({ type: 'reserve', hme, label: siteLabel() });
    if (id !== request || w !== widget) return;
    if (reply?.ok && reply.hme) {
      candidate = null;
      fill(w.target, reply.hme);
      succeed(w, s, reply.hme);
    } else {
      // The proposal may have expired; get a fresh one next time.
      candidate = null;
      showError(w, reply);
    }
  };

  const succeed = (w, s, hme) => {
    w.set('success', { title: s.filled, addr: hme, open: true });
    w.timer = setTimeout(() => w === widget && detach(), SUCCESS_MS);
  };

  function onPress(w) {
    if (w.state === 'ready') reserve(w);
    else if (w.state === 'error') generate(w);
  }

  // ------------------------------------------------------------- listeners

  api.storage.local.get('options').then((r) => {
    buttonEnabled = r?.options?.autofill?.button !== false;
  });
  api.storage.onChanged.addListener((changes, area) => {
    if (area === 'local' && changes.options) {
      buttonEnabled = changes.options.newValue?.autofill?.button !== false;
      if (!buttonEnabled) detach();
    }
  });

  window.addEventListener(
    'focusin',
    (e) => {
      const target = e.composedPath?.()[0] ?? e.target;
      if (!isEmailInput(target)) return;
      lastEmailInput = target;
      if (buttonEnabled && isVisibleField(target)) generate(attach(target));
    },
    true,
  );

  window.addEventListener(
    'focusout',
    () => {
      const w = widget;
      setTimeout(() => {
        if (w && w === widget && deepActiveElement() !== w.target && w.state !== 'reserving' && w.state !== 'success') {
          detach();
        }
      }, 150);
    },
    true,
  );

  window.addEventListener(
    'keydown',
    (e) => {
      if (e.key === 'Escape' && widget) detach();
    },
    true,
  );

  // From the popup ("Fill on page") and the background (right-click menu).
  api.runtime.onMessage.addListener((message, _sender, sendResponse) => {
    if (message?.type === 'autofill') {
      const targets =
        lastEmailInput?.isConnected && isVisibleField(lastEmailInput)
          ? [lastEmailInput]
          : [...document.querySelectorAll('input')].filter((i) => isEmailInput(i) && isVisibleField(i));
      if (targets.length === 0) return false; // let a frame that has a field answer
      targets.forEach((t) => fill(t, message.hme));
      if (widget && targets.includes(widget.target)) detach();
      sendResponse({ filled: targets.length });
      return false;
    }

    if (message?.type === 'contextFill') {
      handleContextFill(message);
    }
    return false;
  });

  let contextTarget = null;

  const handleContextFill = async ({ phase, text, copy: shouldCopy }) => {
    const s = await loadStrings();
    if (phase === 'loading') {
      const active = deepActiveElement();
      contextTarget = isEditable(active) ? active : null;
      if (contextTarget && isVisibleField(contextTarget)) {
        request++; // cancel any in-field generation
        attach(contextTarget).set('loading', { title: s.generating, open: true });
      }
      return;
    }
    const target = contextTarget ?? deepActiveElement();
    contextTarget = null;
    if (phase === 'done' && text) {
      if (isEditable(target)) fill(target, text);
      if (shouldCopy) copy(text);
      if (widget) succeed(widget, s, text);
    } else if (phase === 'error') {
      const w = widget ?? (isEditable(target) && isVisibleField(target) ? attach(target) : null);
      if (w) showError(w, { error: text });
    }
  };

  // Tell the background about a fresh icloud.com session.
  if (/(^|\.)icloud\.com(\.cn)?$/.test(location.hostname)) {
    send({ type: 'syncAuth' });
  }
})();
