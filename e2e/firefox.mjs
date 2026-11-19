// Firefox smoke test: installs dist/firefox into a real Firefox (WebDriver;
// Selenium Manager fetches geckodriver) with iCloud mocked.
//
//   cargo xtask build && cd e2e && node firefox.mjs
//   FIREFOX_PATH=/path/to/firefox HEADED=1 node firefox.mjs
//
// Firefox talks to a local HTTPS proxy: CONNECTs to *.icloud.com(.cn) go to
// a mock iCloud with a throwaway certificate, everything else is refused, so
// the test never reaches the internet. Screenshots land in e2e/shots-firefox/.

import { Builder, By, until, Origin } from 'selenium-webdriver';
import firefox from 'selenium-webdriver/firefox.js';
import http from 'node:http';
import https from 'node:https';
import net from 'node:net';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, '..');
const EXT_DIR = path.join(ROOT, 'dist/firefox');
const SHOTS = path.join(HERE, 'shots-firefox');
const ADDON_ID = 'veil-hide-my-email@limeflash.github.io';
const UUID = '7e1c5a52-8d9e-4b6a-9f3e-5a1d2c3b4e5f';
const EXT = `moz-extension://${UUID}/`;
const PMS = 'https://p00-maildomainws.icloud.com';
const pause = (ms) => new Promise((r) => setTimeout(r, ms));
fs.rmSync(SHOTS, { recursive: true, force: true });
fs.mkdirSync(SHOTS, { recursive: true });

let failures = 0;
const check = (ok, what) => {
  console.log(`${ok ? '✓' : '✗'} ${what}`);
  if (!ok) failures++;
};

// ---- mock iCloud (HTTPS, self-signed)
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'veil-ff-'));
execFileSync('openssl', [
  'req', '-x509', '-newkey', 'rsa:2048', '-nodes', '-days', '1', '-subj', '/CN=icloud.com',
  '-addext', 'subjectAltName=DNS:*.icloud.com,DNS:*.icloud.com.cn',
  '-keyout', path.join(tmp, 'key.pem'), '-out', path.join(tmp, 'cert.pem'),
], { stdio: 'ignore' });

const day = 86_400_000;
const mock = {
  signedIn: true,
  generated: 0,
  requests: [],
  emails: [
    ['GitHub', 'quiet_plover.4k', true, 2, 'Open source'],
    ['Figma', 'marble.sails_0q', true, 5, ''],
    ['Booking.com', 'north.kite_5d', true, 12, ''],
    ['Newsletter', 'amber_wren.x1', false, 22, ''],
  ].map(([label, local, isActive, ago, note], i) => ({
    anonymousId: `id-${i}`, hme: `${local}@icloud.com`, label, note, isActive,
    forwardToEmail: 'me@example.com', createTimestamp: Date.now() - ago * day,
  })),
};

const icloud = https.createServer(
  { key: fs.readFileSync(path.join(tmp, 'key.pem')), cert: fs.readFileSync(path.join(tmp, 'cert.pem')) },
  (req, res) => {
    let raw = '';
    req.on('data', (c) => (raw += c));
    req.on('end', () => {
      const u = new URL(req.url, `https://${req.headers.host}`);
      mock.requests.push({ method: req.method, host: u.host, path: u.pathname, origin: req.headers.origin, referer: req.headers.referer, body: raw });
      const cors = {
        'access-control-allow-origin': req.headers.origin ?? '*',
        'access-control-allow-credentials': 'true',
        'access-control-allow-headers': 'content-type',
      };
      if (req.method === 'OPTIONS') return res.writeHead(204, cors).end();
      const send = (status, body) => res.writeHead(status, { ...cors, 'content-type': 'application/json' }).end(JSON.stringify(body));
      if (u.pathname.endsWith('/validate')) {
        if (!mock.signedIn) return send(421, {});
        return send(200, { dsInfo: { fullName: 'Test User', appleId: 'test@example.com' }, webservices: { premiummailsettings: { url: PMS } } });
      }
      if (!mock.signedIn) return send(421, {});
      if (u.pathname === '/v2/hme/list') {
        return send(200, { success: true, result: { hmeEmails: mock.emails, forwardToEmails: ['me@example.com', 'work@example.com'], selectedForwardTo: 'me@example.com' } });
      }
      if (u.pathname === '/v1/hme/generate') return send(200, { success: true, result: { hme: `fox_trot.${++mock.generated}k@icloud.com` } });
      if (u.pathname === '/v1/hme/reserve') {
        const b = JSON.parse(raw || '{}');
        const e = { anonymousId: `new-${mock.generated}`, hme: b.hme, label: b.label, note: b.note, isActive: true, forwardToEmail: 'me@example.com', createTimestamp: Date.now() };
        mock.emails.unshift(e);
        return send(200, { success: true, result: { hme: e } });
      }
      return send(200, { success: true, result: {} });
    });
  },
);
await new Promise((r) => icloud.listen(0, '127.0.0.1', r));

// ---- CONNECT proxy: iCloud → mock, anything else refused
const proxy = http.createServer((_q, r) => r.writeHead(403).end());
const refused = new Set();
proxy.on('connect', (req, client, head) => {
  const host = req.url.replace(/:\d+$/, '');
  if (!/(^|\.)icloud\.com(\.cn)?$/.test(host)) {
    refused.add(host);
    client.end('HTTP/1.1 403 Forbidden\r\n\r\n');
    return;
  }
  const upstream = net.connect(icloud.address().port, '127.0.0.1', () => {
    client.write('HTTP/1.1 200 Connection Established\r\n\r\n');
    upstream.write(head);
    upstream.pipe(client);
    client.pipe(upstream);
  });
  upstream.on('error', () => client.destroy());
  client.on('error', () => upstream.destroy());
});
await new Promise((r) => proxy.listen(0, '127.0.0.1', r));

// ---- a third-party sign-up page (plain HTTP, direct)
const SITE_HTML = `<!doctype html><meta charset="utf-8"><title>Sign up</title>
<style>body{font:16px system-ui;margin:120px 40px 40px}input{display:block;width:340px;padding:10px;font-size:16px}</style>
<label>Email<input id="email" type="email"></label><p><textarea id="bio" rows="3" cols="40"></textarea>`;
const site = http.createServer((_q, r) => r.writeHead(200, { 'content-type': 'text/html' }).end(SITE_HTML));
await new Promise((r) => site.listen(0, '127.0.0.1', r));
const SITE = `http://127.0.0.1:${site.address().port}/signup`;

// ---- Firefox
const xpi = path.join(tmp, 'veil.zip');
execFileSync('zip', ['-qr', xpi, '.'], { cwd: EXT_DIR });

const opts = new firefox.Options()
  .setPreference('extensions.webextensions.uuids', JSON.stringify({ [ADDON_ID]: UUID }))
  .setPreference('network.proxy.type', 1)
  .setPreference('network.proxy.ssl', '127.0.0.1')
  .setPreference('network.proxy.ssl_port', proxy.address().port)
  .setPreference('network.proxy.no_proxies_on', '')
  .setPreference('network.trr.mode', 5)
  .setPreference('browser.shell.checkDefaultBrowser', false)
  .setPreference('extensions.webextensions.remote', false)
  .setAcceptInsecureCerts(true)
  .windowSize({ width: 1000, height: 800 });
if (!process.env.HEADED) opts.addArguments('-headless');
if (process.env.FIREFOX_PATH) opts.setBinary(process.env.FIREFOX_PATH);
else if (fs.existsSync('/Applications/Firefox.app/Contents/MacOS/firefox')) opts.setBinary('/Applications/Firefox.app/Contents/MacOS/firefox');

// Chrome-context scripts (see page() below) need system access.
process.env.MOZ_REMOTE_ALLOW_SYSTEM_ACCESS = '1';
const driver = await new Builder().forBrowser('firefox').setFirefoxOptions(opts).build();
const caps = await driver.getCapabilities();
console.log(`Firefox ${caps.get('browserVersion')}`);

const shot = async (name) => fs.writeFileSync(path.join(SHOTS, `${name}.png`), await driver.takeScreenshot(), 'base64');
const $ = (css, ms = 8000) => driver.wait(until.elementLocated(By.css(css)), ms);

// Extension pages are a privileged scope for WebDriver, so they are driven
// from the browser chrome: with extensions.webextensions.remote off they run
// in the parent process, where chrome JS can reach their DOM directly.
async function page(body, ...args) {
  await driver.setContext(firefox.Context.CHROME);
  try {
    const r = await driver.executeAsyncScript(
      `const done = arguments[arguments.length - 1];
       const args = Array.from(arguments).slice(0, -1);
       const b = gBrowser.selectedBrowser;
       const doc = b.contentDocument, win = b.contentWindow;
       const browser = win.wrappedJSObject.browser;
       const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
       const waitFor = async (sel, ms = 8000, ok = (e) => true) => {
         const t = Date.now();
         while (Date.now() - t < ms) {
           const e = doc.querySelector(sel);
           if (e && ok(e)) return e;
           await sleep(50);
         }
         throw new Error('timed out waiting for ' + sel);
       };
       const type = (el, text) => {
         el.focus();
         el.value = text;
         el.dispatchEvent(new win.Event('input', { bubbles: true }));
         el.dispatchEvent(new win.Event('change', { bubbles: true }));
       };
       const snap = () => {
         const w = Math.min(doc.documentElement.scrollWidth, win.innerWidth), h = doc.documentElement.scrollHeight;
         const c = document.createElementNS('http://www.w3.org/1999/xhtml', 'canvas');
         c.width = w * 2; c.height = h * 2;
         const ctx = c.getContext('2d');
         ctx.scale(2, 2);
         ctx.drawWindow(win, 0, 0, w, h, 'white');
         return c.toDataURL('image/png').split(',')[1];
       };
       (async () => { ${body} })().then((v) => done({ ok: true, v }), (e) => done({ ok: false, e: String(e) }));`,
      ...args,
    );
    if (!r.ok) throw new Error(r.e);
    return r.v;
  } finally {
    await driver.setContext(firefox.Context.CONTENT);
  }
}
const pageShot = async (name) => fs.writeFileSync(path.join(SHOTS, `${name}.png`), await page('return snap();'), 'base64');
const open = async (file, ready) => {
  await driver.get(`${EXT}${file}`);
  await page(`await waitFor(args[0], 10000);`, ready);
  await pause(400);
};

try {
  await driver.installAddon(xpi, true);
  await pause(2000);
  const handles = await driver.getAllWindowHandles();
  let welcome = false;
  for (const h of handles) {
    await driver.switchTo().window(h);
    if ((await driver.getCurrentUrl()).includes('welcome.html')) welcome = h;
  }
  check(Boolean(welcome), 'welcome page opens on install');
  if (welcome) {
    await driver.switchTo().window(welcome);
    await page(`await waitFor('.v-hero__title');`);
    await pause(300);
    await pageShot('welcome');
  }

  // ---- permissions as Firefox granted them on install
  await open('options.html', '.v-radio');
  const perms = await page(`return JSON.parse(JSON.stringify(await browser.permissions.getAll()));`);
  console.log(`  granted origins: ${perms.origins.join(' ')}`);
  check(perms.origins.some((o) => o.includes('icloud.com')), 'iCloud host access granted on install');
  check(perms.origins.some((o) => /^(https?|\*):\/\/\*\//.test(o) || o === '<all_urls>'), 'all-sites access (content script) granted on install');
  await pageShot('options-light');

  // ---- popup: signed in, proposal, reserve
  await driver.manage().window().setRect({ width: 420, height: 680 });
  await open('popup.html', '.v-address');
  const proposed = await page(`return (await waitFor('.v-address', 8000, (e) => e.textContent.includes('@icloud.com'))).textContent.replace(/\s+/g, '');`);
  check(/^fox_trot\.\d+k@icloud\.com$/.test(proposed), `popup proposes ${proposed}`);
  const icloudReqs = mock.requests.filter((r) => r.method !== 'OPTIONS');
  check(icloudReqs.length > 0 && icloudReqs.every((r) => r.origin === 'https://www.icloud.com'), `Origin rewritten to https://www.icloud.com on ${icloudReqs.length} requests`);
  check(icloudReqs.every((r) => r.referer === 'https://www.icloud.com/'), 'Referer rewritten too');
  await pageShot('popup-generate-light');

  await page(`type(await waitFor('#gen-label'), 'orbit.fit');
              type(await waitFor('#gen-note'), 'Firefox test');
              await sleep(100);
              (await waitFor('button[type=submit]')).click();
              await waitFor('.v-address-card--success');`);
  const reserve = mock.requests.find((r) => r.path === '/v1/hme/reserve');
  const body = reserve ? JSON.parse(reserve.body) : {};
  check(body.label === 'orbit.fit' && body.note === 'Firefox test', `reserve sends label and note (${body.label}, ${body.note})`);
  await pause(500);
  await pageShot('popup-reserved-light');

  // ---- My addresses, search, details
  const listed = await page(`(await waitFor('.v-popup__footer .v-btn')).click();
              await waitFor('.v-list .v-list-item:not(.v-list-item--skeleton)');
              await sleep(300);
              return doc.querySelectorAll('.v-list .v-list-item').length;`);
  check(listed === mock.emails.length, `list shows ${listed}/${mock.emails.length} addresses`);
  await pageShot('popup-manage-light');
  const found = await page(`type(await waitFor('input[type=search]'), 'fig');
              await sleep(300);
              const n = doc.querySelectorAll('.v-list .v-list-item').length;
              type(doc.querySelector('input[type=search]'), '');
              await sleep(300);
              doc.querySelector('.v-list .v-list-item').click();
              await waitFor('.v-popup__header--nav');
              return n;`);
  check(found === 1, `fuzzy search narrows to ${found}`);
  await pause(300);
  await pageShot('popup-details-light');

  // ---- dark theme + RTL language
  await page(`const { options = {} } = await browser.storage.local.get('options');
              await browser.storage.local.set(Cu.cloneInto({ options: { ...JSON.parse(JSON.stringify(options)), theme: 'dark', language: 'ar' } }, win));`);
  await open('popup.html', '.v-address');
  await pause(300);
  check((await page(`return doc.documentElement.dir;`)) === 'rtl', 'Arabic switches the popup to RTL');
  await pageShot('popup-generate-dark-ar');
  await page(`const { options = {} } = await browser.storage.local.get('options');
              await browser.storage.local.set(Cu.cloneInto({ options: { ...JSON.parse(JSON.stringify(options)), theme: 'system', language: 'auto' } }, win));`);

  // ---- in-field button on a third-party page
  await driver.manage().window().setRect({ width: 900, height: 600 });
  await driver.get(SITE);
  await pause(400);
  const before = mock.generated;
  await (await $('#email')).click();
  await pause(1200);
  check(mock.generated === before, 'focusing a field makes no iCloud request');
  const place = await driver.executeScript(`
    const host = [...document.documentElement.children].find((e) => e.hasAttribute('popover'));
    if (!host) return null;
    const r = host.getBoundingClientRect();
    return { x: Math.round(r.x + r.width / 2), y: Math.round(r.y + r.height / 2), w: r.width, open: host.matches(':popover-open') };`);
  check(place && place.w === 22, `in-field button shown (${place ? `${place.w}px, popover ${place.open ? 'open' : 'closed'}` : 'missing'})`);
  if (place) {
    await driver.actions().move({ x: place.x, y: place.y, origin: Origin.VIEWPORT }).perform();
    await pause(900);
    await shot('inpage-ready');
    await driver.actions().move({ x: place.x, y: place.y, origin: Origin.VIEWPORT }).press().release().perform();
    await driver
      .wait(async () => (await driver.executeScript(`return document.querySelector('#email').value`)).endsWith('@icloud.com'), 6000)
      .catch(() => {});
    const filled = await driver.executeScript(`return document.querySelector('#email').value`);
    check(filled.endsWith('@icloud.com'), `in-field button filled "${filled}"`);
    check(mock.emails[0].hme === filled && mock.emails[0].label === '127.0.0.1', 'reserved with the site as label');
    await pause(300);
    await shot('inpage-success');
  }

  // ---- signed out
  mock.signedIn = false;
  await driver.manage().window().setRect({ width: 420, height: 680 });
  await open('options.html', '.v-select');
  await pageShot('options-signedout-light');
  await page(`await browser.storage.local.remove('clientState'); await browser.storage.session?.clear();`);
  await open('popup.html', '.v-intro__title').then(() => check(true, 'signed-out screen'), (e) => check(false, `signed-out screen (${e.message})`));
  await pageShot('popup-signedout-light');
} catch (e) {
  failures++;
  console.error('✗ crashed:', e);
  await shot('crash').catch(() => {});
} finally {
  if (refused.size) console.log(`(proxy refused: ${[...refused].join(', ')})`);
  await driver.quit();
  icloud.close();
  proxy.close();
  site.close();
  fs.rmSync(tmp, { recursive: true, force: true });
}

console.log(failures ? `\n${failures} check(s) failed` : '\nAll Firefox checks passed');
process.exit(failures ? 1 : 0);
