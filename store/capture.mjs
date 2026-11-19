// Raw captures of the real extension UI for the store artwork.
//
//   BROWSER_PATH=… node store/capture.mjs   (run from the repo root after `cargo xtask build`)
//
// Writes store/raw/<lang>/*.png at 2× scale. iCloud is mocked with
// realistic-looking data; nothing touches a real account.

import { createRequire } from 'node:module';
import http from 'node:http';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, '..');
const require = createRequire(path.join(ROOT, 'e2e/package.json'));
const { chromium } = require('playwright');

process.env.PW_EXPERIMENTAL_SERVICE_WORKER_NETWORK_EVENTS = '1';
const EXT = path.join(ROOT, 'dist/chrome');
const LANGS = (process.env.LANGS ?? 'en,ru').split(',');
const PMS = 'https://p00-maildomainws.icloud.com';
const day = 86_400_000;
const now = Date.now();

const LISTS = {
  en: [
    ['GitHub', 'quiet_plover.4k', true, 1, 'Open source, release notes'],
    ['Figma', 'marble.sails_0q', true, 4, ''],
    ['Booking.com', 'north.kite_5d', true, 9, 'Summer trip'],
    ['The Verge newsletter', 'amber_wren.x1', false, 17, ''],
    ['Steam', 'pale_otter.9m', true, 30, ''],
    ['Notion', 'slate_heron.2c', true, 44, ''],
    ['Coupon site', 'dusk.lark_3h', false, 61, 'Too much spam'],
    ['Airbnb', 'cedar_finch.7p', true, 75, ''],
  ],
  ru: [
    ['GitHub', 'quiet_plover.4k', true, 1, 'Опенсорс и релизы'],
    ['Figma', 'marble.sails_0q', true, 4, ''],
    ['Ozon', 'north.kite_5d', true, 9, 'Покупки'],
    ['Рассылка Хабра', 'amber_wren.x1', false, 17, ''],
    ['Steam', 'pale_otter.9m', true, 30, ''],
    ['Notion', 'slate_heron.2c', true, 44, ''],
    ['Сайт с купонами', 'dusk.lark_3h', false, 61, 'Слишком много спама'],
    ['Авито', 'cedar_finch.7p', true, 75, ''],
  ],
};

let emails = [];
const setList = (lang) => {
  emails = (LISTS[lang] ?? LISTS.en).map(([label, local, isActive, ago, note], i) => ({
    anonymousId: `id-${i}`,
    hme: `${local}@icloud.com`,
    label,
    note,
    isActive,
    forwardToEmail: 'jane.doe@example.com',
    createTimestamp: now - ago * day,
  }));
};
const proposals = ['velvet_crane.8r', 'misty_harbor.3v', 'amber_quill.5t', 'silent_fjord.2m'];
let generated = 0;

async function icloud(route) {
  const u = new URL(route.request().url());
  const body = (b) => route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(b) });
  if (u.pathname.endsWith('/validate')) {
    return body({ dsInfo: { fullName: 'Jane Doe', appleId: 'jane.doe@example.com' }, webservices: { premiummailsettings: { url: PMS } } });
  }
  if (u.pathname === '/v2/hme/list') {
    return body({ success: true, result: { hmeEmails: emails, forwardToEmails: ['jane.doe@example.com', 'jane@work.example'], selectedForwardTo: 'jane.doe@example.com' } });
  }
  if (u.pathname === '/v1/hme/generate') {
    return body({ success: true, result: { hme: `${proposals[generated++ % proposals.length]}@icloud.com` } });
  }
  if (u.pathname === '/v1/hme/reserve') {
    const req = JSON.parse(route.request().postData() || '{}');
    const e = { anonymousId: `new-${generated}`, hme: req.hme, label: req.label, note: req.note, isActive: true, forwardToEmail: 'jane.doe@example.com', createTimestamp: Date.now() };
    emails.unshift(e);
    return body({ success: true, result: { hme: e } });
  }
  return body({ success: true, result: {} });
}

// A neutral sign-up page for the in-field button (artwork can restage it).
const SIGNUP = `<!doctype html><meta charset="utf-8"><title>Sign up</title>
<style>
  body{margin:0;min-height:100vh;display:grid;place-items:center;background:#f4f4f2;font:16px/1.4 -apple-system,BlinkMacSystemFont,"Segoe UI",Inter,sans-serif;color:#1d1d1b}
  .card{width:420px;background:#fff;border-radius:16px;padding:32px;box-shadow:0 1px 2px rgba(0,0,0,.06),0 12px 40px rgba(0,0,0,.08)}
  h1{font-size:22px;margin:0 0 20px} label{display:block;font-size:13px;color:#5d5d58;margin:14px 0 6px}
  input{box-sizing:border-box;width:100%;height:44px;border:1px solid #d8d8d3;border-radius:10px;padding:0 12px;font:inherit}
  input:focus{outline:2px solid #2a6df4;outline-offset:-1px;border-color:transparent}
  button.go{margin-top:22px;width:100%;height:44px;border:0;border-radius:10px;background:#1d1d1b;color:#fff;font:600 15px inherit}
</style>
<div class="card"><h1 id="t">Create your account</h1>
<label for="name" id="ln">Name</label><input id="name" value="Jane Doe">
<label for="email" id="le">Email</label><input id="email" type="email" autocomplete="email">
<button class="go" id="go">Continue</button></div>
<script>
  if (location.hash === '#ru') { t.textContent = 'Создайте аккаунт'; ln.textContent = 'Имя'; le.textContent = 'Почта'; go.textContent = 'Продолжить'; document.getElementById('name').value = 'Анна Иванова'; }
</script>`;
const site = http.createServer((_q, r) => r.writeHead(200, { 'content-type': 'text/html' }).end(SIGNUP));
await new Promise((r) => site.listen(0, '127.0.0.1', r));
const SITE = `http://127.0.0.1:${site.address().port}/signup`;

const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'veil-store-'));
const ctx = await chromium.launchPersistentContext(dir, {
  ...(process.env.BROWSER_PATH ? { executablePath: process.env.BROWSER_PATH } : { channel: 'chromium' }),
  headless: true,
  args: [`--disable-extensions-except=${EXT}`, `--load-extension=${EXT}`],
  viewport: { width: 380, height: 560 },
  deviceScaleFactor: 2,
});
await ctx.route(/https:\/\/[^/]*icloud\.com(\.cn)?\//, icloud);
let [sw] = ctx.serviceWorkers();
sw ??= await ctx.waitForEvent('serviceworker');
const id = new URL(sw.url()).host;
const url = (p) => `chrome-extension://${id}/${p}`;
const pause = (ms) => new Promise((r) => setTimeout(r, ms));
await pause(1500);
for (const p of ctx.pages()) if (p.url().includes('welcome.html')) await p.close();

const setOptions = (language, theme) =>
  sw.evaluate(async ([language, theme]) => {
    const { options = {} } = await chrome.storage.local.get('options');
    await chrome.storage.local.set({ options: { ...options, language, theme } });
  }, [language, theme]);
const fresh = () => sw.evaluate(() => chrome.storage.session.clear());

const popup = await ctx.newPage();
const page = await ctx.newPage();
const web = await ctx.newPage();
await web.setViewportSize({ width: 640, height: 520 });

for (const lang of LANGS) {
  const out = path.join(HERE, 'raw', lang);
  fs.rmSync(out, { recursive: true, force: true });
  fs.mkdirSync(out, { recursive: true });
  const shot = (p, name, opts) => p.screenshot({ path: path.join(out, `${name}.png`), ...opts });
  setList(lang);

  for (const theme of ['light', 'dark']) {
    await setOptions(lang, theme);
    await fresh();
    await popup.goto(url('popup.html'));
    await popup.waitForSelector('.v-address');
    await popup.fill('#gen-label', 'github.com');
    await pause(400);
    await shot(popup, `popup-generate-${theme}`);

    await popup.fill('#gen-note', lang === 'ru' ? 'Аккаунт для опенсорса' : 'Open source account');
    await popup.click('button[type=submit]');
    await popup.waitForSelector('.v-address-card--success');
    await pause(700);
    await shot(popup, `popup-reserved-${theme}`);

    await popup.click('.v-popup__footer .v-btn >> nth=0');
    await popup.waitForSelector('.v-list .v-list-item:not(.v-list-item--skeleton)');
    await pause(500);
    await shot(popup, `popup-manage-${theme}`);

    await popup.fill('input[type=search]', 'not');
    await pause(300);
    await shot(popup, `popup-search-${theme}`);
    await popup.fill('input[type=search]', '');

    await popup.click('.v-list-item:has-text("GitHub") >> nth=-1');
    await popup.waitForSelector('.v-popup__header--nav');
    await pause(500);
    await shot(popup, `popup-details-${theme}`);
    setList(lang);
  }
  await setOptions(lang, 'light');

  await page.setViewportSize({ width: 1100, height: 900 });
  for (const theme of ['light', 'dark']) {
    await setOptions(lang, theme);
    await page.goto(url('options.html'));
    await page.waitForSelector('.v-radio');
    await pause(400);
    await shot(page, `options-${theme}`, { fullPage: true });
  }
  await setOptions(lang, 'light');
  await page.goto(url('welcome.html'));
  await page.waitForSelector('.v-hero__title');
  await pause(400);
  await shot(page, 'welcome-light', { fullPage: true });

  // In-field button: ready (tooltip) and success (address filled)
  await web.goto(`${SITE}#${lang}`);
  await web.click('#email');
  await pause(900);
  const box = await web.locator('#email').boundingBox();
  await web.mouse.move(box.x + box.width - 20, box.y + box.height / 2);
  await pause(700);
  await shot(web, 'inpage-ready');
  await web.mouse.click(box.x + box.width - 20, box.y + box.height / 2);
  await web.waitForFunction(() => document.querySelector('#email').value.endsWith('@icloud.com'));
  await pause(250);
  await shot(web, 'inpage-success');
  console.log(`✓ ${lang}: ${fs.readdirSync(out).length} files`);
}

await setOptions('auto', 'system');
await ctx.close();
site.close();
fs.rmSync(dir, { recursive: true, force: true });
