//! `cargo xtask` — builds the extension for Chromium and Firefox.
//!
//! ```text
//! cargo xtask build              release build into dist/chrome and dist/firefox
//! cargo xtask build --dev        unoptimised, faster to compile
//! cargo xtask package            build + dist/veil-<browser>-<version>.zip
//! ```
//!
//! Needs `wasm-bindgen` (same version as the crate, see Cargo.toml) and,
//! optionally, `wasm-opt` from binaryen.

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const WASM_CRATES: [&str; 2] = ["hme_background", "hme_ui"];
const ICON_SIZES: [u32; 4] = [16, 32, 48, 128];
/// Must equal the `wasm-bindgen` crate version pinned in Cargo.toml.
const WASM_BINDGEN: &str = "0.2.130";

#[derive(Clone, Copy, PartialEq)]
enum Browser {
    Chrome,
    Firefox,
}

impl Browser {
    fn name(self) -> &'static str {
        match self {
            Browser::Chrome => "chrome",
            Browser::Firefox => "firefox",
        }
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let dev = args.iter().any(|a| a == "--dev");
    match args.first().map(String::as_str) {
        Some("build") => build(dev).map(|_| ()),
        Some("package") => {
            let dirs = build(dev)?;
            for (browser, dir) in dirs {
                let out = root().join(format!("dist/veil-{}-{VERSION}.zip", browser.name()));
                zip_dir(&dir, &out)?;
                println!("📦 {}", out.strip_prefix(root())?.display());
            }
            Ok(())
        }
        _ => {
            eprintln!("usage: cargo xtask <build|package> [--dev]");
            std::process::exit(2);
        }
    }
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

fn run(cmd: &mut Command) -> Result<()> {
    let status = cmd.status().with_context(|| format!("failed to start {cmd:?}"))?;
    if !status.success() {
        bail!("{cmd:?} exited with {status}");
    }
    Ok(())
}

fn has_tool(name: &str) -> bool {
    Command::new(name).arg("--version").output().is_ok_and(|o| o.status.success())
}

fn build(dev: bool) -> Result<Vec<(Browser, PathBuf)>> {
    let root = root();
    let profile = if dev { "debug" } else { "release" };

    // 1. Rust → wasm32
    let mut cargo = Command::new(env!("CARGO"));
    cargo
        .current_dir(&root)
        .args(["build", "--locked", "--target", "wasm32-unknown-unknown", "-p", "hme-background", "-p", "hme-ui"]);
    if !dev {
        cargo.arg("--release");
    }
    run(&mut cargo)?;

    // 2. wasm-bindgen (+ wasm-opt) into a staging dir shared by both browsers
    let bindgen_version = Command::new("wasm-bindgen")
        .arg("--version")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .unwrap_or_default();
    if bindgen_version != format!("wasm-bindgen {WASM_BINDGEN}") {
        bail!(
            "need wasm-bindgen {WASM_BINDGEN}, found {:?}: cargo install wasm-bindgen-cli --version {WASM_BINDGEN} --locked",
            bindgen_version
        );
    }
    let staging = root.join("target/veil-wasm");
    let _ = fs::remove_dir_all(&staging);
    fs::create_dir_all(&staging)?;
    for krate in WASM_CRATES {
        let wasm = root.join(format!("target/wasm32-unknown-unknown/{profile}/{krate}.wasm"));
        run(Command::new("wasm-bindgen")
            .arg(&wasm)
            .args(["--target", "web", "--no-typescript", "--out-dir"])
            .arg(&staging))?;
        let out = staging.join(format!("{krate}_bg.wasm"));
        if !dev {
            optimize(&out);
        }
        let size = fs::metadata(&out)?.len();
        println!("🦀 {krate}.wasm  {:.0} KB", size as f64 / 1024.0);
    }

    // 3. assemble each browser's directory
    let inpage_css = fs::read_to_string(root.join("extension/styles/inpage.css"))
        .context("extension/styles/inpage.css")?;
    let icon_svg = fs::read(root.join("extension/icon.svg")).context("extension/icon.svg")?;

    let mut dirs = Vec::new();
    for browser in [Browser::Chrome, Browser::Firefox] {
        let dir = root.join("dist").join(browser.name());
        let _ = fs::remove_dir_all(&dir);
        copy_dir(&root.join("extension/static"), &dir)?;
        copy_dir(&root.join("extension/styles"), &dir.join("styles"))?;
        fs::remove_file(dir.join("styles/inpage.css"))?;
        copy_dir(&staging, &dir.join("wasm"))?;

        // The in-page button lives in a Shadow DOM; its CSS is inlined into
        // the content script so pages can't see or block it.
        let content = fs::read_to_string(dir.join("content.js"))?;
        let marker = "/*__INPAGE_CSS__*/''";
        if !content.contains(marker) {
            bail!("content.js lost its {marker} marker");
        }
        fs::write(
            dir.join("content.js"),
            content.replace(marker, &serde_json::to_string(&inpage_css)?),
        )?;

        render_icons(&icon_svg, &dir.join("icons"))?;
        fs::write(
            dir.join("manifest.json"),
            serde_json::to_string_pretty(&manifest(browser))?,
        )?;
        println!("✅ dist/{}", browser.name());
        dirs.push((browser, dir));
    }
    Ok(dirs)
}

fn optimize(wasm: &Path) {
    if !has_tool("wasm-opt") {
        println!("ℹ️  wasm-opt not found, skipping size optimisation (brew install binaryen)");
        return;
    }
    let tmp = wasm.with_extension("opt.wasm");
    let ok = Command::new("wasm-opt")
        .arg(wasm)
        .args([
            "-Oz",
            "--enable-bulk-memory",
            "--enable-nontrapping-float-to-int",
            "--enable-sign-ext",
            "--enable-reference-types",
            "--enable-multivalue",
            "--enable-mutable-globals",
            "-o",
        ])
        .arg(&tmp)
        .status()
        .is_ok_and(|s| s.success());
    if ok {
        let _ = fs::rename(&tmp, wasm);
    } else {
        let _ = fs::remove_file(&tmp);
        println!("⚠️  wasm-opt failed on {}, keeping the unoptimised file", wasm.display());
    }
}

fn manifest(browser: Browser) -> Value {
    let icons: serde_json::Map<String, Value> = ICON_SIZES
        .iter()
        .map(|s| (s.to_string(), json!(format!("icons/icon-{s}.png"))))
        .collect();
    let mut m = json!({
        "manifest_version": 3,
        "name": "__MSG_extName__",
        "short_name": "__MSG_extShortName__",
        "description": "__MSG_extDescription__",
        "default_locale": "en",
        "version": VERSION,
        "homepage_url": "https://github.com/limeflash/veil-hide-my-email",
        "icons": icons,
        "action": {
            "default_popup": "popup.html",
            "default_title": "__MSG_actionTitle__",
            "default_icon": { "16": "icons/icon-16.png", "32": "icons/icon-32.png" }
        },
        "options_ui": { "page": "options.html", "open_in_tab": true },
        "content_scripts": [{
            "matches": ["http://*/*", "https://*/*"],
            "js": ["content.js"],
            "all_frames": true,
            "run_at": "document_idle"
        }],
        "permissions": [
            "storage", "activeTab", "contextMenus", "webRequest", "notifications", "clipboardWrite"
        ],
        "host_permissions": ["https://*.icloud.com/*", "https://*.icloud.com.cn/*"],
        "content_security_policy": {
            "extension_pages": "script-src 'self' 'wasm-unsafe-eval'; object-src 'self'"
        }
    });
    let obj = m.as_object_mut().unwrap();
    match browser {
        Browser::Chrome => {
            // Chromium only applies these rules to requests whose initiator the
            // extension has host access to, i.e. its own and iCloud's.
            obj.insert(
                "declarative_net_request".into(),
                json!({ "rule_resources": [{ "id": "icloud_headers", "enabled": true, "path": "rules.json" }] }),
            );
            push_permission(obj, "declarativeNetRequest");
            obj.insert(
                "background".into(),
                json!({ "service_worker": "background.js", "type": "module" }),
            );
            obj.insert("minimum_chrome_version".into(), json!("111"));
        }
        Browser::Firefox => {
            // Firefox grants content-script match patterns as host permissions,
            // which would let static DNR rules touch other sites' requests to
            // iCloud. Headers are set instead by a blocking webRequest listener
            // in background.js that only handles the extension's own requests.
            push_permission(obj, "webRequestBlocking");
            obj.insert(
                "background".into(),
                json!({ "scripts": ["background.js"], "type": "module" }),
            );
            obj.insert(
                "browser_specific_settings".into(),
                json!({ "gecko": {
                    "id": "veil-hide-my-email@limeflash.github.io",
                    "strict_min_version": "128.0",
                    "data_collection_permissions": { "required": ["none"] }
                }}),
            );
        }
    }
    m
}

fn push_permission(obj: &mut serde_json::Map<String, Value>, permission: &str) {
    if let Some(Value::Array(list)) = obj.get_mut("permissions") {
        list.push(json!(permission));
    }
}

fn render_icons(svg: &[u8], out: &Path) -> Result<()> {
    use resvg::{tiny_skia, usvg};
    fs::create_dir_all(out)?;
    let tree = usvg::Tree::from_data(svg, &usvg::Options::default()).context("parse icon.svg")?;
    for size in ICON_SIZES {
        let mut pixmap = tiny_skia::Pixmap::new(size, size).context("pixmap")?;
        let scale = size as f32 / tree.size().width().max(tree.size().height());
        resvg::render(&tree, tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
        pixmap.save_png(out.join(format!("icon-{size}.png")))?;
    }
    Ok(())
}

fn copy_dir(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from).with_context(|| from.display().to_string())? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else if entry.file_name() != ".DS_Store" {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

fn zip_dir(dir: &Path, out: &Path) -> Result<()> {
    use zip::write::SimpleFileOptions;
    let file = fs::File::create(out)?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let mut entries: Vec<_> = fs::read_dir(&d)?.collect::<Result<_, _>>()?;
        entries.sort_by_key(|e| e.path());
        for e in entries {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                let name = p.strip_prefix(dir)?.to_string_lossy().replace('\\', "/");
                zip.start_file(name, opts)?;
                zip.write_all(&fs::read(&p)?)?;
            }
        }
    }
    zip.finish()?;
    Ok(())
}
