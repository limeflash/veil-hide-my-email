// End-to-end smoke test for the built extension (dist/chrome).
//
// Loads the extension into Playwright's Chromium, replaces iCloud with an
// in-memory mock (no real account involved), and walks through every
// feature: session pickup, generate + reserve, list/search/filter, details,
// edit, deactivate + undo, delete, options, the in-field button and the
// popup's "fill on page". Screenshots go to e2e/shots/.
//
//   cargo xtask build && (cd e2e && npm i && npm test)
//   SIGNED_OUT=1 npm test   # only the signed-out screens
//   BROWSER_PATH="/Applications/Brave Browser.app/Contents/MacOS/Brave Browser" npm test

import { chromium } from 'playwright';
import http from 'node:http';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

process.env.PW_EXPERIMENTAL_SERVICE_WORKER_NETWORK_EVENTS = '1';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const EXT = path.resolve(HERE, '../dist/chrome');
const SHOTS = path.join(HERE, 'shots');
const LANG = process.env.LANG_UI ?? 'ru';
fs.mkdirSync(SHOTS, { recursive: true });

let failures = 0;
const check = (ok, what) => {
  console.log(`${ok ? '✓' : '✗'} ${what}`);
  if (!ok) failures++;
};

// ---------------------------------------------------------------- mock iCloud

const PMS = 'https://p00-maildomainws.icloud.com';
const now = Date.now();
const day = 86_400_000;
const mock = {
  signedIn: !process.env.SIGNED_OUT,
  forwardTo: ['me@example.com', 'work@example.com'],
  selected: 'me@example.com',
  emails: [
    ['GitHub', 'quiet_plover.4k', true, 2, 'Опенсорс и релизы'],
    ['Figma', 'marble.sails_0q', true, 5, ''],
    ['Ozon', 'tidal-fern.72', true, 12, ''],
    ['Рассылка Хабра', 'amber_wren.x1', false, 22, ''],
    ['Booking.com', 'north.kite_5d', true, 36, 'Отели'],
    ['DigitalOcean', 'slate_heron.2c', true, 60, ''],
    ['Авито', 'dusk.lark_3h', false, 69, ''],
  ].map(([label, local, isActive, ago, note], i) => ({
    anonymousId: `id-${i}`,
    hme: `${local}@icloud.com`,
    label,
    note,
    isActive,
    origin: 'ON_DEMAND',
    domain: '',
    forwardToEmail: 'me@example.com',
    createTimestamp: now - ago * day,
    recipientMailId: '',
  })),
  generated: 0,
  fail: {}, // e.g. { generate: true } → the next generate answers HTTP 500
};

const json = (route, body, status = 200) =>
  route.fulfill({ status, contentType: 'application/json', body: JSON.stringify(body) });

async function handleICloud(route) {
  const url = new URL(route.request().url());
  const body = route.request().postData() ? JSON.parse(route.request().postData()) : {};
  const p = url.pathname;
  if (url.host.startsWith('setup.icloud.com')) {
    if (p.endsWith('/validate')) {
      return mock.signedIn
        ? json(route, {
            dsInfo: { fullName: 'Test User', appleId: 'test@example.com' },
            webservices: { premiummailsettings: { url: PMS, status: 'active' } },
          })
        : json(route, { error: 'Missing X-APPLE-WEBAUTH-TOKEN cookie' }, 421);
    }
    if (p.endsWith('/logout')) {
      mock.signedIn = false;
      return json(route, { success: true });
    }
  }
  if (url.origin === PMS) {
    const find = () => mock.emails.find((e) => e.anonymousId === body.anonymousId);
    switch (p) {
      case '/v2/hme/list':
        if (mock.fail.list) {
          delete mock.fail.list;
          return json(route, { error: 'boom' }, 500);
        }
        return json(route, {
          success: true,
          result: { hmeEmails: mock.emails, forwardToEmails: mock.forwardTo, selectedForwardTo: mock.selected },
        });
      case '/v1/hme/generate':
        if (mock.fail.generate) {
          delete mock.fail.generate;
          return json(route, { error: 'boom' }, 500);
        }
        mock.generated++;
        return json(route, { success: true, result: { hme: `fresh_owl.${mock.generated}k@icloud.com` } });
      case '/v1/hme/reserve': {
        const e = {
          anonymousId: `new-${mock.generated}`,
          hme: body.hme,
          label: body.label,
          note: body.note,
          isActive: true,
          origin: 'ON_DEMAND',
          domain: '',
          forwardToEmail: mock.selected,
          createTimestamp: Date.now(),
          recipientMailId: '',
        };
        mock.emails.unshift(e);
        return json(route, { success: true, result: { hme: e } });
      }
      case '/v1/hme/deactivate':
        find().isActive = false;
        return json(route, { success: true, result: {} });
      case '/v1/hme/reactivate':
        find().isActive = true;
        return json(route, { success: true, result: {} });
      case '/v1/hme/delete':
        mock.emails = mock.emails.filter((e) => e.anonymousId !== body.anonymousId);
        return json(route, { success: true, result: {} });
      case '/v1/hme/updateMetaData':
        Object.assign(find(), { label: body.label, note: body.note });
        return json(route, { success: true, result: {} });
      case '/v1/hme/updateForwardTo':
        mock.selected = body.forwardToEmail;
        return json(route, { success: true, result: {} });
    }
  }
  return route.fulfill({ status: 404, body: 'not mocked' });
}

// ------------------------------------------------------- a third-party site

const SITE_HTML = `<!doctype html><html><head><meta charset="utf-8"><title>Sign up</title>
<style>body{font:16px system-ui;margin:60px;background:#fff} input{display:block;width:320px;padding:10px;margin:8px 0 20px;font-size:16px}</style>
</head><body><h1>Create account</h1><form>
<label>Name<input id="name" name="name"></label>
<label>Email<input id="email" type="email" name="email" autocomplete="email"></label>
</form></body></html>`;
const site = http.createServer((_req, res) => res.writeHead(200, { 'content-type': 'text/html' }).end(SITE_HTML));
await new Promise((r) => site.listen(0, '127.0.0.1', r));
const SITE = `http://127.0.0.1:${site.address().port}/signup`;

// ------------------------------------------------------------------- browser

const userDataDir = fs.mkdtempSync(path.join(os.tmpdir(), 'veil-e2e-'));
// BROWSER_PATH=/path/to/chrome-or-brave uses a local Chromium build instead
// of Playwright's own download.
const context = await chromium.launchPersistentContext(userDataDir, {
  ...(process.env.BROWSER_PATH ? { executablePath: process.env.BROWSER_PATH } : { channel: 'chromium' }),
  headless: true,
  locale: LANG,
  args: [`--disable-extensions-except=${EXT}`, `--load-extension=${EXT}`, `--lang=${LANG}`],
  viewport: { width: 380, height: 560 },
});
await context.route(/https:\/\/[^/]*icloud\.com(\.cn)?\//, handleICloud);

const swErrors = [];
let [sw] = context.serviceWorkers();
sw ??= await context.waitForEvent('serviceworker');
const extId = new URL(sw.url()).host;
check(!!extId, `service worker started (${extId})`);

const pageErrors = [];
context.on('page', (p) => p.on('pageerror', (e) => pageErrors.push(`${p.url()}: ${e.message}`)));

const url = (p) => `chrome-extension://${extId}/${p}`;
const shot = async (page, name) => page.screenshot({ path: path.join(SHOTS, `${name}.png`) });
const pause = (ms) => new Promise((r) => setTimeout(r, ms));

// Welcome tab opens on install.
const isWelcome = (p) => p.url().includes('welcome.html');
const welcome =
  context.pages().find(isWelcome) ??
  (await context.waitForEvent('page', { predicate: isWelcome, timeout: 10_000 }).catch(() => context.pages().find(isWelcome) ?? null));
check(!!welcome, 'welcome page opens on install');

const popup = await context.newPage();
// Mocked HTTP 500s are logged by the browser itself; they're expected.
popup.on('console', (m) => m.type() === 'error' && !m.text().startsWith('Failed to load resource') && pageErrors.push(`popup console: ${m.text()}`));

async function openPopup(scheme) {
  await popup.emulateMedia({ colorScheme: scheme });
  await popup.goto(url('popup.html'));
  await popup.waitForSelector('.v-popup:not([aria-busy="true"])', { timeout: 15_000 });
}

for (const scheme of ['dark', 'light']) {
  await openPopup(scheme);
  if (!mock.signedIn) {
    check(await popup.isVisible('.v-intro__title'), `[${scheme}] signed-out screen`);
    await shot(popup, `popup-signed-out-${scheme}`);
    continue;
  }
  await popup.waitForSelector('.v-address');
  await pause(300);
  await shot(popup, `popup-generate-${scheme}`);
}

if (mock.signedIn) {
  // ---- generate + reserve
  await openPopup('dark');
  await popup.waitForSelector('.v-address');
  const proposed = await popup.textContent('.v-address');
  check(/@icloud\.com$/.test(proposed), `address proposed: ${proposed}`);
  await popup.click('.v-address-card .v-icon-btn');
  await popup.waitForFunction((prev) => document.querySelector('.v-address')?.textContent !== prev, proposed);
  check(true, 'regenerate gives another address');
  await popup.fill('#gen-label', '');
  await popup.click('button[type=submit]');
  check(await popup.isVisible('.v-field__error'), 'empty label is rejected');
  await popup.fill('#gen-label', 'example.org');
  await popup.fill('#gen-note', 'e2e note');
  await popup.click('button[type=submit]');
  await popup.waitForSelector('.v-address-card--success');
  check(mock.emails[0].label === 'example.org' && mock.emails[0].note === 'e2e note', 'reserve sent label + note');
  await pause(400);
  await shot(popup, 'popup-reserved-dark');

  // ---- API errors are shown and don't leave the popup stuck
  mock.fail.generate = true;
  await popup.click('.v-btn--ghost.v-btn--block'); // "Create another"
  await popup.waitForSelector('.v-alert--error', { timeout: 5000 });
  check(true, 'generate error is shown');
  await popup.click('.v-address-card .v-icon-btn');
  await popup.waitForSelector('.v-address', { timeout: 5000 });
  check(!(await popup.isVisible('.v-alert--error')), 'regenerate recovers after an error');

  // ---- manage (first load fails, retry works)
  mock.fail.list = true;
  await popup.click('.v-popup__footer .v-btn >> nth=0');
  await popup.waitForSelector('.v-empty--error', { timeout: 5000 });
  check(true, 'list error is shown');
  await popup.click('.v-empty--error .v-btn');
  await popup.waitForSelector('.v-list .v-list-item:not(.v-list-item--skeleton)');
  const rows = await popup.locator('.v-list .v-list-item').count();
  check(rows === mock.emails.length, `list shows ${rows} addresses`);
  await shot(popup, 'popup-manage-dark');
  await popup.fill('input[type=search]', 'git');
  await pause(100);
  const titles = await popup.locator('.v-list-item__title').allTextContents();
  check(titles.length >= 2 && titles[0].startsWith('GitHub'), `fuzzy search "git": ${titles.join(', ')}`);
  check((await popup.locator('mark.v-mark').count()) > 0, 'search hits are highlighted');
  await shot(popup, 'popup-search-dark');
  await popup.click('.v-segmented__item >> nth=2'); // inactive
  await popup.fill('input[type=search]', '');
  const inactive = await popup.locator('.v-list .v-list-item').count();
  check(inactive === mock.emails.filter((e) => !e.isActive).length, `inactive filter: ${inactive}`);
  await popup.click('.v-segmented__item >> nth=0');

  // details → edit
  await popup.click('.v-list-item:has-text("GitHub")');
  await popup.waitForSelector('.v-popup__header--nav');
  await shot(popup, 'popup-details-dark');
  await popup.click('.v-section-title .v-btn');
  await popup.fill('#edit-label', 'GitHub — личный');
  await popup.click('form button[type=submit]');
  await popup.waitForSelector('.v-section-title .v-btn');
  check(mock.emails.find((e) => e.hme.startsWith('quiet_plover')).label === 'GitHub — личный', 'label edited');

  // deactivate + undo, then deactivate + delete
  await popup.click('.v-btn--block:has(svg)'); // "Deactivate address"
  await popup.waitForSelector('.v-address-card.is-inactive');
  check(!mock.emails.find((e) => e.hme.startsWith('quiet_plover')).isActive, 'deactivated');
  await popup.click('.v-toast__action');
  await popup.waitForSelector('.v-address-card:not(.is-inactive)');
  check(mock.emails.find((e) => e.hme.startsWith('quiet_plover')).isActive, 'undo reactivated');
  await popup.click('.v-btn--block:has(svg)');
  await popup.waitForSelector('.v-actions .v-btn--danger');
  await popup.click('.v-actions .v-btn--danger');
  await popup.waitForSelector('.v-sheet');
  await pause(300);
  await shot(popup, 'popup-delete-confirm-dark');
  await popup.click('.v-sheet .v-btn--danger');
  await popup.waitForSelector('.v-list');
  check(!mock.emails.some((e) => e.hme.startsWith('quiet_plover')), 'deleted');

  await popup.emulateMedia({ colorScheme: 'light' });
  await pause(600);
  await shot(popup, 'popup-manage-light');

  // ---- options
  const options = await context.newPage();
  await options.setViewportSize({ width: 1100, height: 1300 });
  for (const scheme of ['dark', 'light']) {
    await options.emulateMedia({ colorScheme: scheme });
    await options.goto(url('options.html'));
    await options.waitForSelector('.v-radio');
    await pause(200);
    await options.screenshot({ path: path.join(SHOTS, `options-${scheme}.png`), fullPage: true });
  }
  await options.click('.v-radio:has-text("work@example.com")');
  await options.click('.v-card__footer .v-btn--primary');
  await options.waitForFunction(() => document.querySelector('.v-card__footer .v-btn--primary')?.disabled);
  check(mock.selected === 'work@example.com', 'forward-to updated');
  await options.click('.v-segmented__item:has-text("Тёмная"), .v-segmented__item:has-text("Dark")');
  await pause(200);
  const stored = await sw.evaluate(() => chrome.storage.local.get('options'));
  check(stored.options?.theme === 'dark', 'theme saved to storage');
  check((await options.getAttribute('html', 'data-theme')) === 'dark', 'theme applied live');
  await options.click('.v-setting:has(.v-toggle) >> nth=0');
  await pause(200);
  const stored2 = await sw.evaluate(() => chrome.storage.local.get('options'));
  check(stored2.options?.autofill?.button === false, 'autofill button toggle saved');
  await options.click('.v-setting:has(.v-toggle) >> nth=0'); // back on
  await options.click('.v-segmented__item >> nth=0'); // theme: system

  // ---- languages (right-to-left included)
  for (const lang of ['ar', 'ja']) {
    await options.selectOption('select.v-select', lang);
    await pause(300);
    const dir = await options.getAttribute('html', 'dir');
    check((await options.getAttribute('html', 'lang')) === lang && dir === (lang === 'ar' ? 'rtl' : 'ltr'), `language ${lang} applied (dir=${dir})`);
    await popup.goto(url('popup.html'));
    await popup.waitForSelector('.v-address');
    await pause(400);
    await shot(popup, `popup-${lang}`);
  }
  await options.screenshot({ path: path.join(SHOTS, 'options-ja.png'), fullPage: true });
  await options.selectOption('select.v-select', 'auto');

  // ---- in-field button on a third-party page
  const page = await context.newPage();
  await page.setViewportSize({ width: 900, height: 500 });
  await page.goto(SITE);
  await pause(300);
  const generatedBefore = mock.generated;
  await page.click('#email');
  await pause(1200);
  check(mock.generated === generatedBefore, 'focusing a field makes no iCloud request');
  await page.screenshot({ path: path.join(SHOTS, 'inpage-ready.png') });
  // The button lives in a closed shadow root; click where it is drawn.
  // Like a person: hover first (the tooltip opens), then click.
  const box = await page.locator('#email').boundingBox();
  await page.mouse.move(box.x + box.width - 6 - 14, box.y + box.height / 2);
  await pause(900);
  await page.mouse.click(box.x + box.width - 6 - 14, box.y + box.height / 2);
  await page.waitForFunction(() => document.querySelector('#email').value.endsWith('@icloud.com'), null, { timeout: 5000 });
  const filled = await page.inputValue('#email');
  check(filled.endsWith('@icloud.com'), `in-field button filled ${filled}`);
  check(mock.emails[0].hme === filled && mock.emails[0].label === '127.0.0.1', 'reserved with site label');
  await pause(300);
  await page.screenshot({ path: path.join(SHOTS, 'inpage-success.png') });

  // ---- popup "fill on page"
  await page.fill('#email', '');
  await page.click('#email');
  await page.bringToFront();
  const tabId = await sw.evaluate(async () => (await chrome.tabs.query({ active: true, lastFocusedWindow: true }))[0]?.id);
  const reply = await sw.evaluate(
    ([id, hme]) => chrome.tabs.sendMessage(id, { type: 'autofill', hme }, { frameId: 0 }),
    [tabId, 'popup_fill@icloud.com'],
  );
  check(reply?.filled === 1 && (await page.inputValue('#email')) === 'popup_fill@icloud.com', 'popup → page autofill');

  // ---- sign out
  await popup.bringToFront();
  await popup.goto(url('popup.html'));
  await popup.waitForSelector('.v-popup__footer');
  await popup.click('.v-popup__footer .v-btn >> nth=1');
  await popup.waitForSelector('.v-intro__title');
  check(true, 'sign out → sign-in screen');
  await shot(popup, 'popup-signed-out-after-logout');
}

if (welcome) {
  await welcome.setViewportSize({ width: 1000, height: 1300 });
  for (const scheme of ['dark', 'light']) {
    await welcome.emulateMedia({ colorScheme: scheme });
    await welcome.reload();
    await welcome.waitForSelector('.v-hero__title');
    await welcome.screenshot({ path: path.join(SHOTS, `welcome-${scheme}.png`), fullPage: true });
  }
}

check(pageErrors.length === 0, `no page errors${pageErrors.length ? ':\n  ' + pageErrors.join('\n  ') : ''}`);
check(swErrors.length === 0, 'no service worker errors');

await context.close();
site.close();
fs.rmSync(userDataDir, { recursive: true, force: true });
console.log(failures ? `\n${failures} check(s) failed` : '\nall checks passed');
process.exit(failures ? 1 : 0);
