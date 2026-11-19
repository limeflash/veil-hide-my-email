// Popup startup timing with a simulated iCloud round-trip.
//
//   LATENCY=300 BROWSER_PATH=… node perf.mjs
//
// Reports, per popup open (median of RUNS):
//   wasm   – boot.js finished instantiating the UI module
//   shell  – the popup chrome is on screen (header rendered)
//   ready  – a generated address is visible (the screen is usable)
//   list   – the address count badge is filled in

import { chromium } from 'playwright';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

process.env.PW_EXPERIMENTAL_SERVICE_WORKER_NETWORK_EVENTS = '1';
const HERE = path.dirname(fileURLToPath(import.meta.url));
const EXT = path.resolve(HERE, '../dist/chrome');
const LATENCY = Number(process.env.LATENCY ?? 300);
const RUNS = Number(process.env.RUNS ?? 7);
const PMS = 'https://p00-maildomainws.icloud.com';

const emails = Array.from({ length: 120 }, (_, i) => ({
  anonymousId: `id-${i}`,
  hme: `addr_${i}.x@icloud.com`,
  label: `site-${i}.com`,
  note: '',
  isActive: i % 7 !== 0,
  forwardToEmail: 'me@example.com',
  createTimestamp: Date.now() - i * 3_600_000,
}));

const delay = (ms) => new Promise((r) => setTimeout(r, ms));
async function icloud(route) {
  const u = new URL(route.request().url());
  await delay(LATENCY);
  const body = (b) => route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(b) });
  if (u.pathname.endsWith('/validate')) {
    return body({ dsInfo: { fullName: 'T', appleId: 't@example.com' }, webservices: { premiummailsettings: { url: PMS } } });
  }
  if (u.pathname === '/v2/hme/list') {
    return body({ success: true, result: { hmeEmails: emails, forwardToEmails: ['me@example.com'], selectedForwardTo: 'me@example.com' } });
  }
  if (u.pathname === '/v1/hme/generate') return body({ success: true, result: { hme: 'fresh.1@icloud.com' } });
  return route.fulfill({ status: 404, body: '' });
}

const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'veil-perf-'));
const ctx = await chromium.launchPersistentContext(dir, {
  ...(process.env.BROWSER_PATH ? { executablePath: process.env.BROWSER_PATH } : { channel: 'chromium' }),
  headless: true,
  args: [`--disable-extensions-except=${EXT}`, `--load-extension=${EXT}`, '--lang=ru'],
  viewport: { width: 380, height: 560 },
});
await ctx.route(/https:\/\/[^/]*icloud\.com(\.cn)?\//, icloud);
let [sw] = ctx.serviceWorkers();
sw ??= await ctx.waitForEvent('serviceworker');
const id = new URL(sw.url()).host;
await delay(1500); // let install-time work settle

const page = await ctx.newPage();
const samples = { wasm: [], shell: [], ready: [], list: [] };
for (let i = 0; i < RUNS + 1; i++) {
  const t0 = Date.now();
  await page.goto(`chrome-extension://${id}/popup.html`, { waitUntil: 'commit' });
  const marks = await page.evaluate(async () => {
    const wait = (sel) =>
      new Promise((res) => {
        const tick = () => (document.querySelector(sel) ? res(performance.now()) : requestAnimationFrame(tick));
        tick();
      });
    const [shell, ready, list] = await Promise.all([
      wait('.v-popup__header'),
      wait('.v-address'),
      wait('.v-popup__footer .v-badge'),
    ]);
    const wasm = performance.getEntriesByName('veil:wasm-ready')[0]?.startTime ?? NaN;
    return { wasm, shell, ready, list };
  });
  if (i === 0) continue; // first open warms the HTTP cache
  for (const k of Object.keys(samples)) samples[k].push(marks[k]);
}
const median = (a) => a.sort((x, y) => x - y)[Math.floor(a.length / 2)];
console.log(`iCloud latency ${LATENCY} ms, ${RUNS} runs (ms from navigation start):`);
for (const [k, v] of Object.entries(samples)) console.log(`  ${k.padEnd(6)} ${median(v).toFixed(0)}`);
await ctx.close();
fs.rmSync(dir, { recursive: true, force: true });
