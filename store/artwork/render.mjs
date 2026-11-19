// Chrome Web Store artwork for Veil. Frame 1 and the marquee are "Mail counter" (dark counter,
// manila labels for made-up sites); frames 2–5 and the small tile are "Big and bold".
//
//   node store/artwork/render.mjs            (from the repo root)
//
// 1. Writes one HTML file per artwork, at its exact pixel size, to store/artwork/src/<lang>/.
// 2. Screenshots each with Playwright (deviceScaleFactor 1) to store/artwork/<lang>/*.png.
// The UI in the frames is the real extension: the 2x captures that store/capture.mjs writes to
// store/raw/<lang>/, cropped, scaled up and at most tilted 3°. Nothing is redrawn.
// Headlines carry data-fit="maxWidth,maxHeight": the page shrinks them until they fit, so longer
// languages keep the same layout without hyphenation.

import { createRequire } from 'node:module';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, '../..');
const require = createRequire(path.join(ROOT, 'e2e/package.json'));
const { chromium } = require('playwright');
const BROWSER = process.env.BROWSER_PATH ?? '/Applications/Brave Browser.app/Contents/MacOS/Brave Browser';

const COPY = {
  en: {
    h1: 'A new address for every site', s1: 'Each one gets its own label. Mail still reaches you.',
    h2: 'Right<br>inside<br>the email<br>field', s2: 'One click on any sign-up form.',
    h3: 'Find any<br>address', s3: 'Search by label, address or note.',
    h4: 'Too much<br>mail? Turn<br>it off', s4: 'Deactivate, bring back or delete.',
    h5: 'Just you<br>and iCloud',
    facts: ['No Veil servers', 'Talks only to Apple’s iCloud', 'Open source, MIT', '15 languages', 'Light and dark'],
    marquee: 'A new address for every site', tagline: 'Hide My Email for iCloud', small: 'Hide My Email',
  },
  ru: {
    h1: 'Свой адрес каждому сайту', s1: 'У каждого своя метка. Письма всё равно придут вам.',
    h2: 'Кнопка<br>прямо<br>в поле', s2: 'Один клик в любой форме регистрации.',
    h3: 'Все адреса<br>под рукой', s3: 'Поиск по метке, адресу и заметке.',
    h4: 'Надоели<br>письма?<br>Отключите', s4: 'Отключите, верните или удалите адрес.',
    h5: 'Только вы<br>и iCloud',
    facts: ['Своих серверов нет', 'Данные идут только в iCloud', 'Открытый код, MIT', '15 языков', 'Светлая и тёмная темы'],
    marquee: 'Свой адрес каждому сайту', tagline: 'Hide My Email для iCloud', small: 'Hide My Email',
  },
};

// The filled Veil mark (design/handoff/icons.md, "Logo (filled)").
const FLAP = 'M5 5h14a3 3 0 0 1 2.6 1.5L12 13.2 2.4 6.5A3 3 0 0 1 5 5z';
const BODY = 'M2 9.3l8.9 6.2a1.9 1.9 0 0 0 2.2 0L22 9.3V16a3 3 0 0 1-3 3H5a3 3 0 0 1-3-3z';
const glyph = (size) => `<svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="${FLAP}"/><path d="${BODY}"/></svg>`;
const lockup = (x = 64, y = 52) => `<div class="lockup" style="left: ${x}px; top: ${y}px">${glyph(36)}Veil</div>`;

// Made-up sites for the mailing labels: no real brands, so nothing third-party is implied.
const SITES = [
  ['mosslane.shop', 'quiet_plover.4k'], ['paperboat.news', 'marble.sails_0q'], ['bramble.travel', 'north.kite_5d'],
  ['kettle.app', 'slate_heron.2c'], ['quietroom.co', 'cedar_finch.7p'], ['lumen.games', 'pale_otter.9m'],
  ['saltmarsh.cafe', 'amber_wren.x1'], ['orbit.fit', 'misty_harbor.3v'], ['tinyforge.dev', 'silent_fjord.2m'],
];
const tag = (i, x, y, rot, cls = '', extra = '') =>
  `<div class="tag ${cls}" style="left: ${x}px; top: ${y}px; transform: rotate(${rot}deg); ${extra}"><b>${SITES[i][0]}</b><span>${SITES[i][1]}@icloud.com</span></div>`;

const POP = [380, 560];   // popup capture, CSS px at 1x
const FORM = [640, 520];  // in-page capture, CSS px at 1x

const doc = (lang, cls, body) => `<!doctype html>
<html lang="${lang}">
<head><meta charset="utf-8"><link rel="stylesheet" href="../artwork.css"></head>
<body class="${cls}">
${body}
</body>
</html>
`;

function pages(lang) {
  const c = COPY[lang];
  const raw = (n) => `../../../raw/${lang}/${n}.png`;
  // whole capture at `scale`, tilted `rot` degrees around its top-left corner
  const ui = (name, base, scale, x, y, rot = 0, radius = 12) =>
    `<img class="ui" src="${raw(name)}" width="${base[0] * scale}" height="${base[1] * scale}" style="left: ${x}px; top: ${y}px; border-radius: ${radius * scale}px; transform: rotate(${rot}deg)" alt="">`;
  // a w×h region (CSS px at 1x, origin sx,sy) of a capture, shown at `scale`
  const crop = (name, base, sx, sy, w, h, scale, x, y, radius = 12, rot = 0) =>
    `<div class="crop" style="left: ${x}px; top: ${y}px; width: ${w * scale}px; height: ${h * scale}px; border-radius: ${radius * scale}px; transform: rotate(${rot}deg); background-image: url('${raw(name)}'); background-size: ${base[0] * scale}px ${base[1] * scale}px; background-position: ${-sx * scale}px ${-sy * scale}px"></div>`;
  const h1 = (html, x, y, maxW, maxH) => `<h1 data-fit="${maxW},${maxH}" style="left: ${x}px; top: ${y}px">${html}</h1>`;
  const sub = (text, x, y, w) => `<p class="sub" style="left: ${x}px; top: ${y}px; width: ${w}px">${text}</p>`;

  return {
    // 1 · Mail counter · labels for made-up sites, the real popup among them
    'screenshot-1': [1280, 800, doc(lang, 'shot counter', `
<div class="abs" style="left: 72px; top: 64px; width: 620px; display: flex; flex-direction: column; gap: 20px">
  <div class="lockup" style="position: static">${glyph(30)}Veil</div>
  <h1 style="position: static">${c.h1}</h1>
  <p class="sub" style="position: static">${c.s1}</p>
</div>
${tag(6, -60, 470, -14, 'lo')}${tag(5, 150, 600, 9)}${tag(3, 30, 640, -5, 'hi')}${tag(4, 330, 470, 13, 'lo')}
${tag(2, 420, 640, -9)}${tag(8, 600, 720, 6, 'hi')}${tag(1, 250, 380, -3, 'hi')}${tag(7, 560, 500, 4, 'lo')}
${tag(0, 520, 318, -7, '', 'width: 340px')}
<img class="ui" src="${raw('popup-generate-light')}" width="437" height="644" style="left: 736px; top: 96px; border-radius: 14px; box-shadow: 0 2px 4px rgba(0,0,0,.4), 0 30px 70px rgba(0,0,0,.6)" alt="">`)],

    // 2 · paper + ink block · the button inside the email field, then the filled field
    'screenshot-2': [1280, 800, doc(lang, 'shot on-paper', `
<div class="block fall"></div>
${lockup()}
${h1(c.h2, 60, 140, 400, 440)}
${sub(c.s2, 64, 624, 340)}
${crop('inpage-ready', FORM, 78, 94, 484, 332, 1.5, 506, 56, 14)}
${crop('inpage-success', FORM, 100, 246, 440, 94, 1.5, 580, 610, 10)}`)],

    // 3 · ink + paper block on the left · the list and a search
    'screenshot-3': [1280, 800, doc(lang, 'shot on-ink', `
<div class="block left"></div>
${ui('popup-manage-light', POP, 1.5, 70, 84, -2)}
${crop('popup-search-light', POP, 0, 56, 380, 250, 1.4, 270, 430, 12)}
${lockup(840, 52)}
${h1(c.h3, 836, 140, 400, 300)}
${sub(c.s3, 840, 400, 360)}`)],

    // 4 · mist + ink block · just deactivated: the inactive card, Turn back on / Delete, and the
    //     "Address deactivated · Undo" toast. The popup is shifted up so the toast is in frame.
    'screenshot-4': [1280, 800, doc(lang, 'shot on-mist-ink', `
<div class="block rise"></div>
${lockup()}
${h1(c.h4, 60, 140, 450, 440)}
${sub(c.s4, 64, 624, 340)}
${ui('popup-deactivated-light', POP, 1.6, 630, -112, 2)}`)],

    // 5 · graphite + paper block · privacy facts, both themes
    'screenshot-5': [1280, 800, doc(lang, 'shot on-graphite', `
<div class="block rise"></div>
${lockup()}
${h1(c.h5, 60, 140, 460, 220)}
<ul class="facts" style="left: 64px; top: 400px">${c.facts.map((f) => `<li>${f}</li>`).join('')}</ul>
${ui('popup-reserved-dark', POP, 1.2, 566, 200, -2)}
${ui('popup-generate-light', POP, 1.2, 800, 70, 2)}`)],

    // small tile · Big and bold: the name on paper, the mark on one ink block
    'promo-small': [440, 280, doc(lang, 'small on-paper', `
<div class="block" style="clip-path: polygon(62% 0, 100% 0, 100% 100%, 46% 100%)"></div>
<div class="abs" style="left: 32px; top: 84px; font-size: 84px; line-height: 84px; font-weight: 600; letter-spacing: -0.05em">Veil</div>
<div class="abs" style="left: 34px; top: 176px; font-size: 21px; line-height: 28px; font-weight: 500">${c.small}</div>
<div class="abs" style="left: 284px; top: 82px; color: var(--paper)">${glyph(116)}</div>`)],

    // marquee · Mail counter
    'promo-marquee': [1400, 560, doc(lang, 'marquee counter', `
<div class="abs" style="left: 88px; top: 0; bottom: 0; width: 540px; display: flex; flex-direction: column; justify-content: center; gap: 22px">
  <div class="lockup" style="position: static; font-size: 30px">${glyph(38)}Veil</div>
  <h1 style="position: static; font-size: 72px">${c.marquee}</h1>
  <p class="sub" style="position: static; font-size: 26px; line-height: 34px">${c.tagline}</p>
</div>
${tag(6, 640, 40, -11, 'lo')}${tag(3, 700, 150, 6, 'hi')}${tag(2, 620, 270, -6)}${tag(4, 720, 390, 9, 'lo')}${tag(5, 610, 480, -4, 'hi')}
${tag(1, 1120, 30, 8, 'hi')}${tag(7, 1150, 180, -7)}${tag(8, 1130, 330, 5, 'lo')}${tag(0, 1160, 460, -9, 'hi')}
<img class="ui" src="${raw('popup-generate-light')}" width="380" height="560" style="left: 880px; top: 56px; border-radius: 12px; transform: rotate(-3deg); transform-origin: 50% 50%; box-shadow: 0 2px 4px rgba(0,0,0,.4), 0 30px 70px rgba(0,0,0,.6)" alt="">`)],
  };
}

const langs = (process.env.LANGS ?? 'en,ru').split(',');
const browser = await chromium.launch({ executablePath: BROWSER, headless: true });
for (const lang of langs) {
  fs.mkdirSync(path.join(HERE, 'src', lang), { recursive: true });
  fs.mkdirSync(path.join(HERE, lang), { recursive: true });
  for (const [name, [width, height, html]] of Object.entries(pages(lang))) {
    const src = path.join(HERE, 'src', lang, `${name}.html`);
    fs.writeFileSync(src, html);
    const page = await browser.newPage({ viewport: { width, height }, deviceScaleFactor: 1 });
    await page.goto(pathToFileURL(src).href);
    const fitted = await page.evaluate(async () => {
      await document.fonts.ready;
      await Promise.all([...document.images].map((i) => i.decode().catch(() => {})));
      const out = [];
      for (const el of document.querySelectorAll('[data-fit]')) {
        const [maxW, maxH] = el.dataset.fit.split(',').map(Number);
        let size = parseFloat(getComputedStyle(el).fontSize);
        while (size > 40 && (el.offsetWidth > maxW || el.offsetHeight > maxH)) { size -= 2; el.style.fontSize = `${size}px`; }
        out.push(size);
      }
      return out;
    });
    const broken = await page.evaluate(() => [...document.images].filter((i) => !i.naturalWidth).map((i) => i.src));
    if (broken.length) throw new Error(`${lang}/${name}: missing images ${broken.join(', ')}`);
    await page.screenshot({ path: path.join(HERE, lang, `${name}.png`), type: 'png', omitBackground: false, clip: { x: 0, y: 0, width, height } });
    await page.close();
    console.log(`${lang}/${name}.png ${width}x${height}${fitted.length ? `  headline ${fitted.join('/')}px` : ''}`);
  }
}
await browser.close();
