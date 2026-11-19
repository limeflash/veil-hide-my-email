// Veil content script: the in-field button and the right-click fill.
//
// Plain JS on purpose: it runs on every page, so it stays tiny and never
// loads WASM there. All iCloud logic (sessions, generating, reserving) is in
// the Rust background worker; this file finds email fields, draws the button
// in a closed Shadow DOM inside the top layer, and writes values.
//
// Security model: pages can focus fields, restyle our host element and stack
// things around it. So nothing touches the network until a real, left-button
// click on a button that has been visibly on screen for a moment, and the
// address only enters the page DOM once it has been filled in.
(() => {
  'use strict';

  const api = globalThis.browser ?? globalThis.chrome;
  if (!api?.runtime?.id || globalThis.__veilContentLoaded) return;
  globalThis.__veilContentLoaded = true;

  // Replaced with the contents of styles/inpage.css by `cargo xtask build`.
  const INPAGE_CSS = /*__INPAGE_CSS__*/'';

  const BUTTON = 28;
  const INSET = 6;
  const SUCCESS_MS = 1200;
  const ERROR_MS = 3500;
  const DWELL_MS = 500; // the button must sit still this long before a click counts

  // ---- DOM access that page markup can't clobber (<form name="activeElement">…)
  const D = Document.prototype;
  const doc = document;
  const getter = (proto, name) => Object.getOwnPropertyDescriptor(proto, name).get;
  const activeOf = getter(D, 'activeElement');
  const rootOf = getter(D, 'documentElement');
  const qsa = (sel) => D.querySelectorAll.call(doc, sel);
  const make = (tag) => D.createElement.call(doc, tag);
  const makeNS = (ns, tag) => D.createElementNS.call(doc, ns, tag);
  const hasPopover = typeof HTMLElement.prototype.showPopover === 'function';

  // The button only appears in the top frame or same-origin frames; any
  // frame can still use the right-click menu (that click is the browser's).
  const buttonFrame = (() => {
    try {
      return window.top === window || window.top.location.origin === location.origin;
    } catch {
      return false;
    }
  })();

  let alive = true;
  const isAlive = () => alive && !!api.runtime?.id;

  const send = async (message) => {
    if (!isAlive()) {
      teardown();
      return { ok: false };
    }
    try {
      return await api.runtime.sendMessage(message);
    } catch (e) {
      if (!api.runtime?.id) teardown(); // extension updated or removed
      return { ok: false, error: String(e?.message ?? e) };
    }
  };

  let strings = null;
  const FALLBACK_STRINGS = {
    clickToFill: 'Create and fill a hidden address',
    generating: 'Creating a hidden address…',
    filled: 'Done — address filled',
    signedOut: 'Sign in to iCloud first',
    appName: 'Veil',
    dir: 'ltr',
  };
  const loadStrings = async () => {
    if (!strings) {
      const s = await send({ type: 'inpageStrings' });
      strings = { ...FALLBACK_STRINGS, ...(s && !('ok' in s) ? s : {}) };
    }
    return strings;
  };

  const deepActiveElement = () => {
    let el = activeOf.call(doc);
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
    return /e-?mail|почт|пошт|correo|courriel|e-?posta|邮箱|メール|이메일|ایمیل|بريد/.test(hints);
  };

  const isEditable = (el) =>
    !!el && (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.isContentEditable);

  const isVisibleField = (el) => {
    if (!el?.isConnected) return false;
    const r = el.getBoundingClientRect();
    // Tiny or hidden fields are usually honeypots.
    if (r.width < 30 || r.height < 10) return false;
    const cs = getComputedStyle(el);
    return cs.display !== 'none' && cs.visibility !== 'hidden' && Number(cs.opacity) >= 0.5;
  };

  const setNativeValue = (el, value) => {
    const proto = el.tagName === 'TEXTAREA' ? HTMLTextAreaElement.prototype : HTMLInputElement.prototype;
    const setter = Object.getOwnPropertyDescriptor(proto, 'value')?.set;
    if (setter) setter.call(el, value);
    else el.value = value;
  };

  const notify = (el) => {
    el.dispatchEvent(new Event('input', { bubbles: true, composed: true }));
    el.dispatchEvent(new Event('change', { bubbles: true }));
  };

  /**
   * Writes `value` so React/Vue/Angular forms notice it. Single-line inputs
   * get the whole value; textareas and rich editors get it at the caret, so
   * existing text is kept. Focus is never stolen from where the user is.
   */
  const fill = (el, value) => {
    if (!el?.isConnected) return false;
    if (el.isContentEditable) {
      el.focus({ preventScroll: true });
      document.execCommand('insertText', false, value);
      return true;
    }
    if (el.tagName === 'TEXTAREA') {
      const start = el.selectionStart ?? el.value.length;
      const end = el.selectionEnd ?? start;
      el.setRangeText(value, start, end, 'end');
      notify(el);
      return true;
    }
    setNativeValue(el, value);
    notify(el);
    return true;
  };

  const copy = (text) => navigator.clipboard?.writeText(text).catch(() => {});

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
    const svg = makeNS(SVG_NS, 'svg');
    svg.setAttribute('viewBox', '0 0 24 24');
    svg.setAttribute('aria-hidden', 'true');
    svg.setAttribute('class', `veil-icon veil-icon--${modifier}`);
    for (const [tag, attrs] of shapes) {
      const node = makeNS(SVG_NS, tag);
      for (const [k, v] of Object.entries(attrs)) node.setAttribute(k, v);
      svg.appendChild(node);
    }
    return svg;
  };

  const ICONS = {
    logo: [
      ['path', { d: 'M5 5h14a3 3 0 0 1 2.6 1.5L12 13.2 2.4 6.5A3 3 0 0 1 5 5z' }],
      ['path', { d: 'M2 9.3l8.9 6.2a1.9 1.9 0 0 0 2.2 0L22 9.3V16a3 3 0 0 1-3 3H5a3 3 0 0 1-3-3z' }],
    ],
    check: [['path', { d: 'M20 6 9 17l-5-5' }]],
    alert: [['path', { d: 'M12 7v6M12 17h.01' }]],
  };

  const el = (tag, cls, text) => {
    const n = make(tag);
    if (cls) n.className = cls;
    if (text) n.textContent = text;
    return n;
  };

  /** Inline !important styles for the host; re-applied every frame. */
  const hostStyle = (top, left, shown) =>
    [
      'all:initial', 'position:fixed', 'z-index:2147483647', 'display:block',
      `top:${top}px`, `left:${left}px`, 'right:auto', 'bottom:auto',
      `width:${BUTTON}px`, `height:${BUTTON}px`, 'margin:0', 'padding:0', 'border:0',
      'background:transparent', 'overflow:visible', 'opacity:1', 'transform:none',
      'filter:none', 'clip-path:none', 'mask:none', 'pointer-events:auto',
      `visibility:${shown ? 'visible' : 'hidden'}`, 'color-scheme:normal',
    ]
      .map((d) => `${d} !important`)
      .join(';');

  class Widget {
    constructor(target) {
      this.target = target;
      this.busy = false;
      this.stableSince = performance.now();
      this.visibleToUser = null; // IntersectionObserver v2 verdict, when available
      this.pos = { top: -1, left: -1, shown: false };

      this.host = make('div');
      if (hasPopover) this.host.setAttribute('popover', 'manual');
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
      this.btn.append(
        svgIcon('logo', ICONS.logo),
        svgIcon('check', ICONS.check),
        svgIcon('alert', ICONS.alert),
        el('span', 'veil-spinner'),
      );
      this.tip = el('span', 'veil-tip');
      this.tip.id = 'veil-tip';
      this.tip.setAttribute('role', 'tooltip');
      this.root.append(this.btn, this.tip);
      shadow.appendChild(this.root);

      const scheme = schemeFor(target);
      if (scheme) this.root.dataset.scheme = scheme;
      if (strings?.dir === 'rtl') this.root.dir = 'rtl';

      // Keep focus in the field while pressing the button.
      this.btn.addEventListener('pointerdown', (e) => {
        e.preventDefault();
        e.stopPropagation();
        if (this.canAccept(e)) onPress(this);
      });
      this.btn.addEventListener('mousedown', (e) => e.preventDefault());
      this.btn.addEventListener('click', (e) => e.preventDefault());

      // Chromium: ask the compositor whether the button is really visible
      // (not covered, not faded, not transformed by the page).
      try {
        this.io = new IntersectionObserver(
          (entries) => {
            const last = entries[entries.length - 1];
            if (typeof last.isVisible === 'boolean') this.visibleToUser = last.isVisible;
          },
          { trackVisibility: true, delay: 100 },
        );
        this.io.observe(this.btn);
      } catch {
        this.io = null;
      }

      this.mount();
      this.track();
    }

    mount() {
      const root = rootOf.call(doc) || doc.body;
      if (this.host.parentNode !== root) root.appendChild(this.host);
      if (hasPopover && !this.host.matches(':popover-open')) {
        try {
          this.host.showPopover();
        } catch {
          /* not connected yet */
        }
      }
    }

    /** Anti-clickjacking checks for a press on the button. */
    canAccept(e) {
      if (!e.isTrusted || e.button !== 0) return false;
      if (performance.now() - this.stableSince < DWELL_MS) return false;
      if (this.visibleToUser === false) return false;
      const top = D.elementsFromPoint.call(doc, e.clientX, e.clientY)[0];
      return top === this.host;
    }

    track() {
      const step = () => {
        if (!this.host) return;
        if (!this.target.isConnected && !this.busy) {
          detach();
          return;
        }
        const shown = isVisibleField(this.target);
        let { top, left } = this.pos;
        if (shown) {
          const r = this.target.getBoundingClientRect();
          const rtl = getComputedStyle(this.target).direction === 'rtl';
          top = Math.round(r.top + (r.height - BUTTON) / 2);
          left = Math.round(rtl ? r.left + INSET : r.right - BUTTON - INSET);
          this.tip.classList.toggle('veil-tip--below', r.top < 64);
        }
        if (Math.abs(top - this.pos.top) > 2 || Math.abs(left - this.pos.left) > 2 || shown !== this.pos.shown) {
          this.stableSince = performance.now();
        }
        this.pos = { top, left, shown };
        // Undo anything the page did to the host (styles, parent, popover).
        const css = hostStyle(top, left, shown);
        if (this.host.style.cssText !== css) this.host.style.cssText = css;
        if (!this.host.isConnected || this.host.parentNode !== (rootOf.call(doc) || doc.body)) this.mount();
        if (hasPopover && !this.host.matches(':popover-open')) this.mount();
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
      this.btn.setAttribute('aria-label', [title, text].filter(Boolean).join(' '));
      this.tip.classList.toggle('is-open', open);
    }

    destroy() {
      cancelAnimationFrame(this.frame);
      clearTimeout(this.timer);
      this.io?.disconnect();
      try {
        if (hasPopover) this.host.hidePopover();
      } catch {
        /* already hidden */
      }
      this.host.remove();
      this.host = null;
    }
  }

  // ------------------------------------------------------------------ state

  let widget = null;
  let lastEmailInput = null; // last email field the user focused
  let buttonEnabled = true;
  let signedIn = false;

  const detach = () => {
    widget?.destroy();
    widget = null;
  };

  const attach = (target) => {
    if (widget?.target === target) return widget;
    if (widget?.busy) return null; // let the running request finish first
    detach();
    widget = new Widget(target);
    return widget;
  };

  const showIdle = async (w) => {
    const s = await loadStrings();
    if (w === widget && !w.busy) w.set('ready', { title: s.clickToFill });
  };

  const showError = async (w, reply) => {
    const s = await loadStrings();
    const message = reply?.signedOut ? s.signedOut : reply?.error || s.signedOut;
    w.set('error', { title: s.appName, text: message, error: true, open: true });
    clearTimeout(w.timer);
    w.timer = setTimeout(() => {
      if (w !== widget) return;
      if (deepActiveElement() === w.target) showIdle(w);
      else detach();
    }, ERROR_MS);
  };

  const succeed = (w, s, hme) => {
    w.set('success', { title: s.filled, addr: hme, open: true });
    clearTimeout(w.timer);
    w.timer = setTimeout(() => w === widget && detach(), SUCCESS_MS);
  };

  /** One click: the background generates, reserves and returns an address. */
  async function onPress(w) {
    if (w.busy || w.state === 'success') return;
    const s = await loadStrings();
    w.busy = true;
    w.set('reserving', { title: s.generating, open: true });
    const reply = await send({ type: 'createForSite' });
    w.busy = false;
    if (reply?.ok && reply.hme) {
      // Fill the field the press was for, even if focus moved meanwhile.
      if (!fill(w.target, reply.hme)) copy(reply.hme);
      if (w === widget) succeed(w, s, reply.hme);
    } else if (w === widget) {
      showError(w, reply);
    }
  }

  const buttonAllowed = () => buttonFrame && buttonEnabled && signedIn && isAlive();

  const maybeAttach = (target) => {
    if (!isEmailInput(target)) return;
    lastEmailInput = target;
    if (buttonAllowed() && isVisibleField(target)) {
      const w = attach(target);
      if (w) loadStrings().then(() => showIdle(w));
    }
  };

  // ------------------------------------------------------------- listeners

  const readSettings = (items) => {
    buttonEnabled = items?.options?.autofill?.button !== false;
    signedIn = !!items?.clientState;
    if (!buttonAllowed() && !widget?.busy) detach();
  };
  api.storage.local.get(['options', 'clientState']).then((items) => {
    readSettings(items);
    // A field that already had focus when the script loaded.
    const active = deepActiveElement();
    if (active) maybeAttach(active);
  });
  const onStorage = (changes, area) => {
    if (area !== 'local' || !(changes.options || changes.clientState)) return;
    api.storage.local.get(['options', 'clientState']).then(readSettings);
  };
  api.storage.onChanged.addListener(onStorage);

  const onFocusIn = (e) => maybeAttach(e.composedPath?.()[0] ?? e.target);

  const onFocusOut = () => {
    const w = widget;
    setTimeout(() => {
      if (w && w === widget && !w.busy && w.state !== 'success' && w.state !== 'error' && deepActiveElement() !== w.target) {
        detach();
      }
    }, 150);
  };

  const onKeyDown = (e) => {
    if (e.key === 'Escape' && widget && !widget.busy) detach();
  };

  window.addEventListener('focusin', onFocusIn, true);
  window.addEventListener('focusout', onFocusOut, true);
  window.addEventListener('keydown', onKeyDown, true);

  // From the popup ("Fill on page", sent to the top frame only) and from the
  // background (right-click menu, sent to the clicked frame).
  const onMessage = (message, _sender, sendResponse) => {
    if (message?.type === 'autofill') {
      // Exactly one field: the one the user last focused, or the only email
      // field on the page. Never every matching input.
      let target = lastEmailInput?.isConnected && isVisibleField(lastEmailInput) ? lastEmailInput : null;
      if (!target) {
        const fields = [...qsa('input')].filter((i) => isEmailInput(i) && isVisibleField(i));
        if (fields.length === 1) target = fields[0];
      }
      if (!target) return false; // no answer → the popup says "no field"
      fill(target, message.hme);
      if (widget?.target === target && !widget.busy) detach();
      sendResponse({ filled: 1 });
      return false;
    }

    if (message?.type === 'contextFill') {
      // Acknowledge the start, so the background only reserves an address
      // when there is a page to put it in.
      if (message.phase === 'loading') sendResponse({ ok: true });
      handleContextFill(message);
    }
    return false;
  };
  api.runtime.onMessage.addListener(onMessage);

  let contextTarget = null;

  const handleContextFill = async ({ phase, text, copy: shouldCopy }) => {
    const s = await loadStrings();
    if (phase === 'loading') {
      const active = deepActiveElement();
      contextTarget = isEditable(active) ? active : null;
      if (contextTarget && isVisibleField(contextTarget) && !widget?.busy) {
        const w = attach(contextTarget);
        if (w) {
          w.busy = true;
          w.set('reserving', { title: s.generating, open: true });
        }
      }
      return;
    }
    const target = contextTarget ?? deepActiveElement();
    contextTarget = null;
    if (widget && widget.target === target) widget.busy = false;
    if (phase === 'done' && text) {
      if (isEditable(target)) fill(target, text);
      if (shouldCopy) copy(text);
      if (widget?.target === target) succeed(widget, s, text);
    } else if (phase === 'error') {
      const w = widget?.target === target ? widget : isEditable(target) && isVisibleField(target) ? attach(target) : null;
      if (w) showError(w, { error: text });
    }
  };

  /** The extension was updated or removed: drop everything. */
  function teardown() {
    if (!alive) return;
    alive = false;
    detach();
    window.removeEventListener('focusin', onFocusIn, true);
    window.removeEventListener('focusout', onFocusOut, true);
    window.removeEventListener('keydown', onKeyDown, true);
    try {
      api.storage.onChanged.removeListener(onStorage);
      api.runtime.onMessage.removeListener(onMessage);
    } catch {
      /* context already gone */
    }
  }

  // Tell the background about a fresh icloud.com session.
  if (/(^|\.)icloud\.com(\.cn)?$/.test(location.hostname)) {
    send({ type: 'syncAuth' });
  }
})();
