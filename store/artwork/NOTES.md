# Veil — Chrome Web Store artwork

Rebuild everything with `node store/artwork/render.mjs` from the repo root. The script writes
one fixed-size HTML page per artwork to `src/<lang>/`, then screenshots it with Playwright at
`deviceScaleFactor: 1` into `<lang>/`. Shared styles are in `src/artwork.css`; copy is the
`COPY` object at the top of `render.mjs`.

The UI in every frame is the real extension: the 2x captures from `store/raw/<lang>/` shown at
1x (380×560 popups, 640×520 in-page form). Nothing is redrawn. The browser toolbar around it
is a generic drawing, not any real browser.

| File | Size |
| --- | --- |
| `<lang>/screenshot-1.png` … `screenshot-5.png` | 1280×800 |
| `<lang>/promo-small.png` | 440×280 |
| `<lang>/promo-marquee.png` | 1400×560 |

## Known gap

`store/raw/ru/inpage-ready.png` and `inpage-success.png` were captured without the Veil button
and tooltip (both show the already-filled field). Until they are re-captured, the Russian
frames 1 and 2 use the English in-page captures, so the tooltip text in `ru/screenshot-2.png`
is English. Remove `INPAGE_FALLBACK` in `render.mjs` once the Russian captures are fixed.

## Design method

Audit, direction and pass order follow **unslop by Damir Sokolovsky (CC BY 4.0)**,
github.com/heydamir/unslop. Its scripts were not run.

- **Audit.** There was no earlier artwork, so the audit was run on the usual store-art
  defaults and on the first render: gradient backdrops, tilted floating mockups, glow, badge
  pills, icon-and-three-features rows, hype headlines. None are used. Fixed after the first
  render: a form cropped mid-word behind the popup, a capture overflowing the toolbar, an
  "after" crop that cut through a field, and a three-line headline in frame 3.
- **Direction.** Discreet, exact, postal. Quiet graphite frame, ink type, a generic toolbar.
  The one bright element in each screenshot is the manila address label inside the real UI.
  The promo tiles invert that: the whole tile is label paper with the ink mark.
- **Copy.** Headlines say what the product does in its own nouns, no periods, no hype. The
  Russian lines are written separately, not translated word for word.
