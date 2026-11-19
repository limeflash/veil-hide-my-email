// Loads the Rust/WASM UI and mounts the page named in <body data-page>.
import init, { mount } from './wasm/hme_ui.js';

await init({ module_or_path: new URL('./wasm/hme_ui_bg.wasm', import.meta.url) });
performance.mark('veil:wasm-ready');
await mount(document.body.dataset.page);
