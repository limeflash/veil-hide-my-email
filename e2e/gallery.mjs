// Screenshots of every screen in every language, for design review.
//
//   BROWSER_PATH=… node gallery.mjs        → e2e/gallery/<lang>-<screen>.png
//
// iCloud is mocked (no account needed). Each language is forced through the
// extension's own setting, so this is exactly what users see.

import { chromium } from 'playwright';
import http from 'node:http';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

process.env.PW_EXPERIMENTAL_SERVICE_WORKER_NETWORK_EVENTS = '1';
const HERE = path.dirname(fileURLToPath(import.meta.url));
const EXT = path.resolve(HERE, '../dist/chrome');
const OUT = path.join(HERE, 'gallery');
const LANGS = (process.env.LANGS ?? 'en,ru,es,de,fr,tr,ar,fa,zh,ja').split(',');
fs.rmSync(OUT, { recursive: true, force: true });
fs.mkdirSync(OUT, { recursive: true });

const PMS = 'https://p00-maildomainws.icloud.com';
const day = 86_400_000;
const now = Date.now();
const emails = [
  ['GitHub', 'quiet_plover.4k', true, 2, 'Open source and release notes'],
  ['Figma', 'marble.sails_0q', true, 5, ''],
  ['Booking.com', 'north.kite_5d', true, 12, ''],
  ['Newsletter', 'amber_wren.x1', false, 22, ''],
  ['Steam', 'pale_otter.9m', true, 36, ''],
  ['DigitalOcean', 'slate_heron.2c', true, 60, ''],
  ['Twitch', 'dusk.lark_3h', false, 69, ''],
].map(([label, local, isActive, ago, note], i) => ({
  anonymousId: `id-${i}`,
  hme: `${local}@icloud.com`,
  label,
  note,
  isActive,
  forwardToEmail: 'me@example.com',
  createTimestamp: now - ago * day,
}));
let generated = 0;

async function icloud(route) {
  const u = new URL(route.request().url());
  const body = (b, status = 200) => route.fulfill({ status, contentType: 'application/json', body: JSON.stringify(b) });
  if (u.pathname.endsWith('/validate')) {
    return body({ dsInfo: { fullName: 'Test User', appleId: 'test@example.com' }, webservices: { premiummailsettings: { url: PMS } } });
  }
  if (u.pathname === '/v2/hme/list') {
    return body({ success: true, result: { hmeEmails: emails, forwardToEmails: ['me@example.com', 'work@example.com'], selectedForwardTo: 'me@example.com' } });
  }
  if (u.pathname === '/v1/hme/generate') return body({ success: true, result: { hme: `fresh_owl.${++generated}k@icloud.com` } });
  return body({ success: true, result: {} });
}

const SITE_HTML = `<!doctype html><meta charset="utf-8"><style>body{font:16px system-ui;margin:120px 40px 40px;background:#fff}
input{display:block;width:340px;padding:10px;font-size:16px}</style><label>Email<input id="email" type="email"></label>`;
const site = http.createServer((_q, r) => r.writeHead(200, { 'content-type': 'text/html' }).end(SITE_HTML));
await new Promise((r) => site.listen(0, '127.0.0.1', r));
const SITE = `http://127.0.0.1:${site.address().port}/`;

const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'veil-gallery-'));
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

const popup = await ctx.newPage();
const page = await ctx.newPage();
const site_ = await ctx.newPage();
await site_.setViewportSize({ width: 520, height: 260 });

const shot = (p, name, opts = {}) => p.screenshot({ path: path.join(OUT, `${name}.png`), ...opts });
const setOptions = (language, theme) =>
  sw.evaluate(async ([language, theme]) => {
    const { options = {} } = await chrome.storage.local.get('options');
    await chrome.storage.local.set({ options: { ...options, language, theme } });
  }, [language, theme]);

for (const lang of LANGS) {
  for (const theme of ['light', 'dark']) {
    await setOptions(lang, theme);
    await popup.goto(url('popup.html'));
    await popup.waitForSelector('.v-address');
    await pause(500);
    await shot(popup, `${lang}-popup-generate-${theme}`);
  }
  await setOptions(lang, 'light');

  // My addresses → details → delete confirmation of an inactive one
  await popup.goto(url('popup.html'));
  await popup.waitForSelector('.v-popup__footer');
  await popup.click('.v-popup__footer .v-btn >> nth=0');
  await popup.waitForSelector('.v-list .v-list-item:not(.v-list-item--skeleton)');
  await pause(400);
  await shot(popup, `${lang}-popup-manage-light`);
  await popup.click('.v-list-item >> nth=0');
  await popup.waitForSelector('.v-popup__header--nav');
  await pause(400);
  await shot(popup, `${lang}-popup-details-light`);

  // Signed-out screen (rendered with the session cleared, then restored)
  const state = await sw.evaluate(() => chrome.storage.local.get('clientState'));
  await sw.evaluate(() => chrome.storage.local.remove('clientState'));
  await ctx.unroute(/https:\/\/[^/]*icloud\.com(\.cn)?\//);
  await ctx.route(/https:\/\/[^/]*icloud\.com(\.cn)?\//, (r) => r.fulfill({ status: 421, body: '{}' }));
  await popup.goto(url('popup.html'));
  await popup.waitForSelector('.v-intro__title');
  await pause(300);
  await shot(popup, `${lang}-popup-signedout-light`);
  await ctx.unroute(/https:\/\/[^/]*icloud\.com(\.cn)?\//);
  await ctx.route(/https:\/\/[^/]*icloud\.com(\.cn)?\//, icloud);
  await sw.evaluate((s) => chrome.storage.local.set(s), state);

  // Options and welcome pages
  await page.setViewportSize({ width: 1000, height: 900 });
  await page.goto(url('options.html'));
  await page.waitForSelector('.v-radio');
  await pause(300);
  await shot(page, `${lang}-options-light`, { fullPage: true });
  await page.goto(url('welcome.html'));
  await page.waitForSelector('.v-hero__title');
  await pause(300);
  await shot(page, `${lang}-welcome-light`, { fullPage: true });

  // In-page button with its tooltip open
  await site_.goto(SITE);
  await site_.click('#email');
  await pause(800);
  const box = await site_.locator('#email').boundingBox();
  await site_.mouse.move(box.x + box.width - 6 - 14, box.y + box.height / 2);
  await pause(700);
  await shot(site_, `${lang}-inpage-light`);
  await site_.mouse.move(5, 5);
  console.log(`✓ ${lang}`);
}

await setOptions('auto', 'system');
await ctx.close();
site.close();
fs.rmSync(dir, { recursive: true, force: true });
console.log(`${fs.readdirSync(OUT).length} screenshots in ${OUT}`);
