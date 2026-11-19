// Veil store artwork — concept directions A, B, C (EN only, screenshot-1 + marquee each).
//
//   node store/artwork/concepts.mjs      (from the repo root)
//
// Writes store/artwork/src/concepts/<name>.html, renders store/artwork/concepts/<name>.png.
// Real UI only: the popups and form are the 2x captures in store/raw/en, cropped and scaled.

import { createRequire } from 'node:module';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, '../..');
const require = createRequire(path.join(ROOT, 'e2e/package.json'));
const { chromium } = require('playwright');
const BROWSER = process.env.BROWSER_PATH ?? '/Applications/Brave Browser.app/Contents/MacOS/Brave Browser';

const FLAP = 'M5 5h14a3 3 0 0 1 2.6 1.5L12 13.2 2.4 6.5A3 3 0 0 1 5 5z';
const BODY = 'M2 9.3l8.9 6.2a1.9 1.9 0 0 0 2.2 0L22 9.3V16a3 3 0 0 1-3 3H5a3 3 0 0 1-3-3z';
const glyph = (size, color) => `<svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="${color}" aria-hidden="true"><path d="${FLAP}"/><path d="${BODY}"/></svg>`;
const raw = (n) => `../../../raw/en/${n}.png`;
const tag = (site, addr, x, y, rot, cls = '', extra = '') => `<div class="tag ${cls}" style="left: ${x}px; top: ${y}px; transform: rotate(${rot}deg); ${extra}"><b>${site}</b><span>${addr}@icloud.com</span></div>`;
// crop(name, sx, sy, w, h, scale, x, y): a w×h (CSS px at 1x) region of a capture, shown at `scale`
const crop = (name, base, sx, sy, w, h, scale, x, y, extra = '') => `<div class="crop" style="left: ${x}px; top: ${y}px; width: ${w * scale}px; height: ${h * scale}px; background-image: url('${raw(name)}'); background-size: ${base[0] * scale}px ${base[1] * scale}px; background-position: ${-sx * scale}px ${-sy * scale}px; ${extra}"></div>`;
const POP = [380, 560], FORM = [640, 520];
const doc = (cls, style, body) => `<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><link rel="stylesheet" href="concepts.css"><style>${style}</style></head>
<body class="${cls}">
${body}
</body>
</html>
`;

const SITES = [
  ['github.com', 'quiet_plover.4k'], ['figma.com', 'marble.sails_0q'], ['booking.com', 'north.kite_5d'],
  ['notion.so', 'slate_heron.2c'], ['airbnb.com', 'cedar_finch.7p'], ['steampowered.com', 'pale_otter.9m'],
  ['theverge.com', 'amber_wren.x1'], ['spotify.com', 'misty_harbor.3v'], ['duolingo.com', 'silent_fjord.2m'],
];
const t = (i, x, y, rot, cls, extra) => tag(SITES[i][0], SITES[i][1], x, y, rot, cls, extra);

const pages = {
  // ------------------------------------------------------------ A: Mail counter
  'a-screenshot-1': [1280, 800, doc('shot', `
    body { background: #17130D; color: #F3E7CB; }
    h1 { font-size: 68px; line-height: 70px; }
    .sub { margin: 0; font-size: 23px; line-height: 32px; color: #B9AA8A; }`, `
<div class="abs" style="left: 72px; top: 64px; width: 600px; display: flex; flex-direction: column; gap: 20px">
  <div class="lockup">${glyph(30, '#E9D8B4')}Veil</div>
  <h1>A new address for every site</h1>
  <p class="sub">Each one gets its own label. Mail still reaches you.</p>
</div>
${t(6, -60, 470, -14, 'lo')}${t(5, 150, 600, 9, '')}${t(3, 30, 640, -5, 'hi')}${t(4, 330, 470, 13, 'lo')}
${t(2, 420, 640, -9, '')}${t(8, 600, 720, 6, 'hi')}${t(1, 250, 380, -3, 'hi')}${t(7, 560, 500, 4, 'lo')}
${t(0, 520, 318, -7, '', 'z-index: 1; width: 340px')}
<img class="ui" src="${raw('popup-generate-light')}" width="437" height="644" style="left: 800px; top: 96px; z-index: 2; box-shadow: 0 2px 4px rgba(0,0,0,.4), 0 30px 70px rgba(0,0,0,.6)" alt="">`)],

  'a-marquee': [1400, 560, doc('marquee', `
    body { background: #17130D; color: #F3E7CB; }
    h1 { font-size: 72px; line-height: 74px; }
    .sub { margin: 0; font-size: 26px; line-height: 34px; color: #B9AA8A; }`, `
<div class="abs" style="left: 88px; top: 0; bottom: 0; width: 560px; display: flex; flex-direction: column; justify-content: center; gap: 22px">
  <div class="lockup" style="font-size: 30px">${glyph(38, '#E9D8B4')}Veil</div>
  <h1>A new address for every site</h1>
  <p class="sub">Hide My Email for iCloud</p>
</div>
${t(6, 640, 40, -11, 'lo')}${t(3, 700, 150, 6, 'hi')}${t(2, 620, 270, -6, '')}${t(4, 720, 390, 9, 'lo')}${t(5, 610, 480, -4, 'hi')}
${t(1, 1120, 30, 8, 'hi')}${t(7, 1150, 180, -7, '')}${t(8, 1130, 330, 5, 'lo')}${t(0, 1160, 460, -9, 'hi')}
<img class="ui" src="${raw('popup-generate-light')}" width="380" height="560" style="left: 880px; top: 56px; z-index: 2; transform: rotate(-3deg); box-shadow: 0 2px 4px rgba(0,0,0,.4), 0 30px 70px rgba(0,0,0,.6)" alt="">`)],

  // ------------------------------------------------------------ B: Big and bold
  'b-screenshot-1': [1280, 800, doc('shot', `
    body { background: #E9D8B4; color: #221C12; }
    .field { position: absolute; inset: 0; background: #221C12; clip-path: polygon(52% 0, 100% 0, 100% 100%, 34% 100%); }
    h1 { font-size: 118px; line-height: 104px; font-weight: 600; letter-spacing: -0.05em; text-wrap: nowrap; }
    .sub { margin: 0; font-size: 26px; line-height: 34px; font-weight: 500; }`, `
<div class="field"></div>
<div class="abs lockup" style="left: 64px; top: 56px; font-size: 28px">${glyph(36, '#221C12')}Veil</div>
<h1 class="abs" style="left: 60px; top: 150px">A new<br>address<br>for every<br>site</h1>
<p class="abs sub" style="left: 64px; top: 620px; width: 330px">One click. Your real inbox stays private.</p>
<img class="ui" src="${raw('popup-generate-light')}" width="646" height="952" style="left: 590px; top: 84px; border-radius: 20px; transform: rotate(3deg); transform-origin: 0 0; box-shadow: 0 40px 90px rgba(0,0,0,.5)" alt="">`)],

  'b-marquee': [1400, 560, doc('marquee', `
    body { background: #E9D8B4; color: #221C12; }
    .field { position: absolute; inset: 0; background: #221C12; clip-path: polygon(62% 0, 100% 0, 100% 100%, 50% 100%); }
    h1 { font-size: 112px; line-height: 100px; font-weight: 600; letter-spacing: -0.05em; text-wrap: nowrap; }`, `
<div class="field"></div>
<div class="abs lockup" style="left: 72px; top: 56px; font-size: 30px">${glyph(38, '#221C12')}Veil</div>
<h1 class="abs" style="left: 68px; top: 150px">A new address<br>for every site</h1>
<p class="abs" style="left: 72px; top: 400px; margin: 0; font-size: 28px; line-height: 36px; font-weight: 500">Hide My Email for iCloud</p>
<img class="ui" src="${raw('popup-generate-light')}" width="570" height="840" style="left: 800px; top: 56px; border-radius: 18px; transform: rotate(4deg); transform-origin: 0 0; box-shadow: 0 40px 90px rgba(0,0,0,.5)" alt="">`)],

  // ------------------------------------------------------------ C: One address, one story
  'c-screenshot-1': [1280, 800, doc('shot', `
    body { background: #F1EDE4; color: #221C12; }
    h1 { font-size: 64px; line-height: 66px; }
    .step { font-family: var(--mono); font-size: 17px; line-height: 24px; color: #5C5140; }
    .sub { margin: 0; font-size: 23px; line-height: 32px; color: #5C5140; }
    .route { position: absolute; inset: 0; }
    .stop { position: absolute; font-family: var(--mono); font-size: 15px; color: #5C5140; }`, `
<div class="abs" style="left: 72px; top: 0; bottom: 0; width: 400px; display: flex; flex-direction: column; justify-content: center; gap: 20px">
  <div class="lockup">${glyph(30, '#221C12')}Veil</div>
  <div class="step">1 of 5 · create</div>
  <h1>One click, one new address</h1>
  <p class="sub">Follow it from the popup to your inbox.</p>
</div>
<div class="abs" style="left: 1150px; top: 300px; width: 640px; height: 520px; border-radius: 0; overflow: hidden"><img src="${raw('inpage-plain')}" width="640" height="520" style="display: block; margin-left: -60px" alt=""></div>
<svg class="route" viewBox="0 0 1280 800" aria-hidden="true">
  <path d="M 880 218 C 1010 218, 1040 330, 990 420 S 1020 610, 1280 590" fill="none" stroke="#221C12" stroke-width="5" stroke-linecap="round" stroke-dasharray="0.1 16"/>
  <circle cx="880" cy="218" r="9" fill="#221C12"/>
</svg>
<div class="abs" style="left: 968px; top: 398px; width: 52px; height: 52px; border-radius: 14px; background: #E9D8B4; display: grid; place-items: center; box-shadow: 0 1px 2px rgba(34,28,18,.2), 0 10px 24px rgba(34,28,18,.22); transform: rotate(-8deg)">${glyph(32, '#221C12')}</div>
<span class="stop" style="left: 900px; top: 716px">next: the sign-up form →</span>
<img class="ui" src="${raw('popup-generate-light')}" width="380" height="560" style="left: 500px; top: 120px; box-shadow: 0 1px 2px rgba(34,28,18,.12), 0 24px 56px rgba(34,28,18,.22)" alt="">`)],

  'c-marquee': [1400, 560, doc('marquee', `
    body { background: #F1EDE4; color: #221C12; }
    h1 { font-size: 54px; line-height: 58px; }
    .route { position: absolute; inset: 0; }
    .stop { position: absolute; font-family: var(--mono); font-size: 16px; line-height: 22px; color: #5C5140; }
    .stop b { display: block; font-family: var(--sans); font-size: 22px; line-height: 28px; font-weight: 500; color: #221C12; }
    .crop { box-shadow: 0 1px 2px rgba(34,28,18,.12), 0 16px 40px rgba(34,28,18,.2); }`, `
<div class="abs" style="left: 72px; top: 48px; display: flex; align-items: center; gap: 28px">
  <div class="lockup" style="font-size: 28px">${glyph(36, '#221C12')}Veil</div>
  <h1>One address, from sign-up to switch-off</h1>
</div>
<svg class="route" viewBox="0 0 1400 560" aria-hidden="true">
  <path d="M 60 330 C 230 250, 300 410, 470 330 S 720 250, 880 330 S 1130 410, 1360 330" fill="none" stroke="#221C12" stroke-width="5" stroke-linecap="round" stroke-dasharray="0.1 16"/>
</svg>
${crop('popup-generate-light', POP, 16, 72, 348, 114, 0.8, 60, 284)}
${crop('inpage-success', FORM, 104, 287, 432, 48, 0.65, 400, 314, 'border-radius: 8px')}
<div class="tag" style="left: 740px; top: 288px; width: 280px; --hole: #F1EDE4; box-shadow: 0 1px 2px rgba(34,28,18,.2), 0 14px 32px rgba(34,28,18,.22)"><b>Your inbox</b><span style="white-space: nowrap">jane.doe@example.com</span></div>
${crop('popup-details-light', POP, 16, 405, 348, 40, 0.8, 1080, 314, 'border-radius: 8px')}
<span class="stop" style="left: 60px; top: 424px"><b>Create</b>one click in the popup</span>
<span class="stop" style="left: 400px; top: 424px"><b>Fill</b>straight into the form</span>
<span class="stop" style="left: 740px; top: 424px"><b>Receive</b>mail is forwarded to you</span>
<span class="stop" style="left: 1080px; top: 424px"><b>Turn off</b>when you've had enough</span>`)],
};

const browser = await chromium.launch({ executablePath: BROWSER, headless: true });
for (const [name, [width, height, html]] of Object.entries(pages)) {
  const src = path.join(HERE, 'src/concepts', `${name}.html`);
  fs.writeFileSync(src, html);
  const page = await browser.newPage({ viewport: { width, height }, deviceScaleFactor: 1 });
  await page.goto(pathToFileURL(src).href);
  await page.evaluate(async () => { await document.fonts.ready; await Promise.all([...document.images].map((i) => i.decode().catch(() => {}))); });
  const broken = await page.evaluate(() => [...document.images].filter((i) => !i.naturalWidth).map((i) => i.src));
  if (broken.length) throw new Error(`${name}: missing images ${broken.join(', ')}`);
  await page.screenshot({ path: path.join(HERE, 'concepts', `${name}.png`), type: 'png', clip: { x: 0, y: 0, width, height } });
  await page.close();
  console.log(`concepts/${name}.png ${width}x${height}`);
}
await browser.close();
