# Veil — Chrome Web Store artwork

Rebuild everything with `node store/artwork/render.mjs` from the repo root. The script writes
one fixed-size HTML page per artwork to `src/<lang>/`, then screenshots it with Playwright at
`deviceScaleFactor: 1` into `<lang>/`. Shared styles are in `src/artwork.css`; copy is the
`COPY` object at the top of `render.mjs`. Headlines carry `data-fit="maxWidth,maxHeight"` and
shrink until they fit, so a longer language keeps the layout.

| File | Size | Direction |
| --- | --- | --- |
| `<lang>/screenshot-1.png` | 1280×800 | Mail counter (dark) |
| `<lang>/screenshot-2.png` | 1280×800 | Big and bold, paper + ink block |
| `<lang>/screenshot-3.png` | 1280×800 | Big and bold, ink + paper block |
| `<lang>/screenshot-4.png` | 1280×800 | Big and bold, light grey + ink block |
| `<lang>/screenshot-5.png` | 1280×800 | Big and bold, graphite + paper block |
| `<lang>/promo-small.png` | 440×280 | Big and bold |
| `<lang>/promo-marquee.png` | 1400×560 | Mail counter (dark) |

- **Mail counter.** A dark counter scattered with manila mailing labels, the real popup
  standing among them. The labels are illustrations for made-up sites (`SITES` in
  `render.mjs`); none is a real brand. Keep it that way.
- **Big and bold.** One flat colour field per frame cut by one diagonal block, a headline up to
  118px, and the real UI scaled 1.2–1.6x with a tilt of at most 3°.
- The UI in every frame is the real extension: the 2x captures from `store/raw/<lang>/`,
  cropped and scaled. Nothing is redrawn. Frame order alternates dark, light, dark, light, dark.
- `concepts/` and `concepts.mjs` are the three directions shown before this one was chosen.

Design method: audit, direction and pass order follow **unslop by Damir Sokolovsky
(CC BY 4.0)**, github.com/heydamir/unslop. Its scripts were not run. No gradients, glow, fake
3D, emoji or hype words; colour is paper `#E9D8B4`, ink `#221C12` and graphite only.
