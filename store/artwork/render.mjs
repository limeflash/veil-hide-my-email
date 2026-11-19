// Chrome Web Store artwork for Veil.
//
//   node store/artwork/render.mjs            (from the repo root)
//
// 1. Writes one HTML file per artwork, at its exact pixel size, to store/artwork/src/<lang>/.
// 2. Screenshots each with Playwright (deviceScaleFactor 1) to store/artwork/<lang>/*.png.
// The UI in the frames is the real extension, captured at 2x by store/capture.mjs into
// store/raw/<lang>/ and shown here at 1x. Nothing is redrawn.

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
    tagline: 'Hide My Email for iCloud',
    s1: ['A new address for every site', 'One click in the toolbar. Mail still lands in your real inbox.'],
    s2: ['Right inside the email field', 'Veil adds a button to sign-up forms. Click it and the address is filled in.'],
    s2cap: 'after one click',
    s3: ['Every address, one search away', 'Search by label, address or note. Filter active and inactive.'],
    s4: ['Too much mail? Turn the address off', 'Give each address a label and a note. Deactivate it, bring it back, or delete it.'],
    s5: ['Just you and iCloud', null],
    facts: [['servers', 'None of ours'], ['talks to', 'Apple’s iCloud only'], ['code', 'Open source, MIT'], ['languages', '15'], ['themes', 'Light and dark']],
    promo: 'A new address for every site',
  },
  ru: {
    tagline: 'Hide My Email для iCloud',
    s1: ['Свой адрес для каждого сайта', 'Один клик на панели браузера. Письма придут на ваш настоящий ящик.'],
    s2: ['Кнопка прямо в поле email', 'Veil добавляет её в формы регистрации. Нажмите, и адрес подставится сам.'],
    s2cap: 'после клика',
    s3: ['Все адреса под рукой', 'Поиск по метке, адресу и заметке. Фильтр активных и отключённых.'],
    s4: ['Надоели письма? Отключите адрес', 'Подпишите адрес меткой и заметкой. Отключите, верните или удалите.'],
    s5: ['Только вы и iCloud', null],
    facts: [['серверы', 'Своих нет'], ['обмен данными', 'Только с iCloud от Apple'], ['код', 'Открытый, лицензия MIT'], ['языки', '15'], ['темы', 'Светлая и тёмная']],
    promo: 'Свой адрес для каждого сайта',
  },
};

// The filled Veil mark (design/handoff/icons.md, "Logo (filled)").
const FLAP = 'M5 5h14a3 3 0 0 1 2.6 1.5L12 13.2 2.4 6.5A3 3 0 0 1 5 5z';
const BODY = 'M2 9.3l8.9 6.2a1.9 1.9 0 0 0 2.2 0L22 9.3V16a3 3 0 0 1-3 3H5a3 3 0 0 1-3-3z';
const glyph = (size, color = '#221C12') =>
  `<svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="${color}" aria-hidden="true"><path d="${FLAP}"/><path d="${BODY}"/></svg>`;
const icon = (size) =>
  `<svg width="${size}" height="${size}" viewBox="0 0 128 128" aria-hidden="true"><rect x="4" y="4" width="120" height="120" rx="28" fill="#E9D8B4"/><g transform="translate(16 16) scale(4)" fill="#221C12"><path d="${FLAP}"/><path d="${BODY}"/></g></svg>`;

const line = (d) => `<svg viewBox="0 0 24 24" aria-hidden="true"><path d="${d}"/></svg>`;
const bar = (url) => `<div class="bar">
  ${line('m15 18-6-6 6-6')}${line('m9 18 6-6-6-6')}${line('M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8M3 3v5h5')}
  <div class="url"><svg viewBox="0 0 24 24" aria-hidden="true"><rect x="5" y="11" width="14" height="9" rx="2"/><path d="M8 11V8a4 4 0 0 1 8 0v3"/></svg>${url}</div>
  <span class="ext ext--on">${icon(20)}</span>
  ${line('M12 5h.01M12 12h.01M12 19h.01')}
</div>`;

const copyBlock = (c, [title, sub], width, extra = '') => `<div class="copy" style="width: ${width}px">
  <div class="brand">${icon(36)}Veil</div>
  <h1>${title}</h1>
  ${sub ? `<p class="sub">${sub}</p>` : ''}${extra}
</div>`;

const doc = (lang, cls, body) => `<!doctype html>
<html lang="${lang}">
<head><meta charset="utf-8"><link rel="stylesheet" href="../artwork.css"></head>
<body class="${cls}">
${body}
</body>
</html>
`;

// store/raw/ru/inpage-*.png were captured without the Veil button and tooltip (both files show
// the already-filled field), so the Russian frames borrow the English in-page captures until
// the Russian ones are re-captured. Delete this map then.
const INPAGE_FALLBACK = { ru: 'en' };
const raw = (lang, name) => `../../../raw/${name.startsWith('inpage-') ? INPAGE_FALLBACK[lang] ?? lang : lang}/${name}.png`;
// Window geometry: the Veil toolbar icon sits 76px from the frame's right edge; the popup hangs under it.
const POP_X = 836, POP_Y = 132;

function pages(lang) {
  const c = COPY[lang];
  const r = (n) => raw(lang, n);
  return {
    'screenshot-1': [1280, 800, doc(lang, 'shot', `
${copyBlock(c, c.s1, 400)}
<div class="win" style="left: 520px">${bar('github.com/signup')}
  <div class="page"><img src="${r('inpage-ready')}" width="640" height="520" style="left: -44px; top: 30px" alt=""></div>
</div>
<img class="popup" src="${r('popup-generate-light')}" style="left: ${POP_X}px; top: ${POP_Y}px" alt="">`)],

    'screenshot-2': [1280, 800, doc(lang, 'shot', `
${copyBlock(c, c.s2, 420)}
<div class="win" style="left: 548px">${bar('example.com/signup')}
  <div class="page"><img src="${r('inpage-ready')}" width="640" height="520" style="left: 46px; top: -20px" alt=""></div>
</div>
<span class="cap" style="left: 792px; top: 580px">${c.s2cap}</span>
<div class="card" style="left: 792px; top: 608px; width: 440px; height: 164px"><i style="width: 440px; height: 164px; background-image: url('${r('inpage-success')}'); background-size: 640px 520px; background-position: -100px -176px"></i></div>`)],

    'screenshot-3': [1280, 800, doc(lang, 'shot', `
${copyBlock(c, c.s3, 320)}
<div class="win" style="left: 420px">${bar('github.com')}
  <div class="page"></div>
</div>
<img class="popup" src="${r('popup-search-light')}" style="left: 440px; top: 196px" alt="">
<img class="popup" src="${r('popup-manage-light')}" style="left: ${POP_X}px; top: ${POP_Y}px" alt="">`)],

    'screenshot-4': [1280, 800, doc(lang, 'shot dark', `
${copyBlock(c, c.s4, 560)}
<div class="win" style="left: 720px">${bar('github.com')}
  <div class="page"></div>
</div>
<img class="popup" src="${r('popup-details-dark')}" style="left: ${POP_X}px; top: ${POP_Y}px" alt="">`)],

    'screenshot-5': [1280, 800, doc(lang, 'shot', `
${copyBlock(c, c.s5, 340, `<dl class="facts">${c.facts.map(([k, v]) => `<dt>${k}</dt><dd>${v}</dd>`).join('')}</dl>`)}
<div class="win" style="left: 440px">${bar('github.com/signup')}
  <div class="page"></div>
</div>
<img class="popup" src="${r('popup-reserved-dark')}" style="left: 460px; top: 196px" alt="">
<img class="popup" src="${r('popup-generate-light')}" style="left: ${POP_X}px; top: ${POP_Y}px" alt="">`)],

    'promo-small': [440, 280, doc(lang, 'small paper', `
<div style="position: absolute; inset: 0; display: flex; flex-direction: column; justify-content: center; gap: 14px; padding: 0 40px">
  <div class="lockup">${glyph(64)}<span class="word" style="font-size: 60px; line-height: 64px">Veil</span></div>
  <div style="font-size: 22px; line-height: 28px; color: var(--text-2)">${c.tagline}</div>
</div>`)],

    'promo-marquee': [1400, 560, doc(lang, 'marquee paper', `
<div style="position: absolute; left: 96px; top: 0; bottom: 0; width: 640px; display: flex; flex-direction: column; justify-content: center; gap: 24px">
  <div class="lockup">${glyph(44)}<span class="word" style="font-size: 40px; line-height: 44px">Veil</span></div>
  <h1 style="font-size: 68px; line-height: 72px">${c.promo}</h1>
  <p class="sub" style="font-size: 26px; line-height: 34px">${c.tagline}</p>
</div>
<img class="popup" src="${r('popup-generate-dark')}" style="left: 900px; top: 64px; box-shadow: 0 1px 2px rgba(34, 28, 18, 0.12), 0 24px 56px rgba(34, 28, 18, 0.28)" alt="">`)],
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
    await page.evaluate(async () => {
      await document.fonts.ready;
      await Promise.all([...document.images].map((i) => i.decode().catch(() => {})));
    });
    const broken = await page.evaluate(() => [...document.images].filter((i) => !i.naturalWidth).map((i) => i.src));
    if (broken.length) throw new Error(`${lang}/${name}: missing images ${broken.join(', ')}`);
    await page.screenshot({ path: path.join(HERE, lang, `${name}.png`), type: 'png', omitBackground: false, clip: { x: 0, y: 0, width, height } });
    await page.close();
    console.log(`${lang}/${name}.png ${width}x${height}`);
  }
}
await browser.close();
