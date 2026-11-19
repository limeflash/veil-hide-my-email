//! `cargo xtask` — builds the extension for Chromium and Firefox.
//!
//! ```text
//! cargo xtask build              release build into dist/chrome and dist/firefox
//! cargo xtask build --dev        unoptimised, faster to compile
//! cargo xtask package            build + dist/veil-<browser>-<version>.zip
//!                                + dist/veil-source-<version>.zip (for AMO review)
//! ```
//!
//! Needs `wasm-bindgen` (same version as the crate, see Cargo.toml) and, for
//! release builds, `wasm-opt` from binaryen version 133. Release builds are
//! reproducible: the same sources and tool versions give byte-identical files.

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
/// binaryen release used for published builds (reviewers rebuild with it).
const WASM_OPT: &str = "133";
/// Left out of the source archive: not needed to build the extension.
const SOURCE_EXCLUDE: [&str; 3] = ["design/", "e2e/", "store/"];

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
            let source = root().join(format!("dist/veil-source-{VERSION}.zip"));
            zip_source(&source)?;
            println!("📦 {}", source.strip_prefix(root())?.display());
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

fn build(dev: bool) -> Result<Vec<(Browser, PathBuf)>> {
    let root = root();
    let profile = if dev { "debug" } else { "release" };

    // 1. Rust → wasm32
    if !dev {
        check_wasm_opt()?;
    }
    let mut cargo = Command::new(env!("CARGO"));
    cargo
        .current_dir(&root)
        .args(["build", "--locked", "--target", "wasm32-unknown-unknown", "-p", "hme-background", "-p", "hme-ui"]);
    if !dev {
        cargo.arg("--release");
    }
    // Source paths end up in panic messages. Remapping this checkout and the
    // cargo registry to fixed names makes the WASM identical on any machine.
    // (These flags replace the ones in .cargo/config.toml, so repeat the cfg.)
    let cargo_home = std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cargo")))
        .context("neither CARGO_HOME nor HOME is set")?;
    // With the rust-src component installed, rustc swaps its virtual
    // /rustc/<commit> paths for the local copy; map them back.
    let rustc = |args: &[&str]| -> Result<String> {
        let out = Command::new("rustc").current_dir(&root).args(args).output().context("rustc")?;
        Ok(String::from_utf8(out.stdout)?.trim().to_owned())
    };
    let sysroot = rustc(&["--print", "sysroot"])?;
    let version = rustc(&["-vV"])?;
    let commit = version
        .lines()
        .find_map(|l| l.strip_prefix("commit-hash: "))
        .context("rustc -vV has no commit-hash")?;
    let rustflags = [
        "--cfg".to_owned(),
        r#"getrandom_backend="wasm_js""#.to_owned(),
        format!("--remap-path-prefix={}=/veil", root.display()),
        format!("--remap-path-prefix={}=/cargo", cargo_home.display()),
        format!("--remap-path-prefix={sysroot}/lib/rustlib/src/rust=/rustc/{commit}"),
    ];
    cargo.env("CARGO_ENCODED_RUSTFLAGS", rustflags.join("\x1f"));
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
            optimize(&out)?;
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

/// Release builds must use the pinned binaryen, or they would not match the
/// published packages.
fn check_wasm_opt() -> Result<()> {
    let found = Command::new("wasm-opt")
        .arg("--version")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .unwrap_or_default();
    // "wasm-opt version 133" (some builds append " (version_133)")
    if found.split_whitespace().nth(2) != Some(WASM_OPT) {
        bail!(
            "need wasm-opt {WASM_OPT}, found {found:?}: download binaryen-version_{WASM_OPT} from \
             https://github.com/WebAssembly/binaryen/releases/tag/version_{WASM_OPT} \
             (or use --dev for an unoptimised build)"
        );
    }
    Ok(())
}

fn optimize(wasm: &Path) -> Result<()> {
    let tmp = wasm.with_extension("opt.wasm");
    let ok = Command::new("wasm-opt")
        .arg(wasm)
        .args([
            "-Oz",
            // The producers section names the tool builds (release binaries of
            // wasm-bindgen add a commit hash); drop it so any build matches.
            "--strip-producers",
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
    if !ok {
        let _ = fs::remove_file(&tmp);
        bail!("wasm-opt failed on {}", wasm.display());
    }
    fs::rename(&tmp, wasm)?;
    Ok(())
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
                // 140 is the first release with the built-in data consent
                // prompt. Declared: addresses, notes and the forwarding email
                // (personal info) and the site's domain used as a label
                // (browsing activity) are sent to Apple's iCloud, and only there.
                json!({ "gecko": {
                    "id": "veil-hide-my-email@limeflash.github.io",
                    "strict_min_version": "140.0",
                    "data_collection_permissions": {
                        "required": ["personallyIdentifyingInfo", "browsingActivity"]
                    }
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

/// Every file git knows about (committed or not, minus ignored ones), except
/// SOURCE_EXCLUDE: what Mozilla's reviewers need to rebuild dist/firefox.
fn zip_source(out: &Path) -> Result<()> {
    use zip::write::SimpleFileOptions;
    let root = root();
    let listed = Command::new("git")
        .current_dir(&root)
        .args(["ls-files", "-z", "--cached", "--others", "--exclude-standard"])
        .output()
        .context("git ls-files")?;
    if !listed.status.success() {
        bail!("git ls-files failed: the source archive is built from the git checkout");
    }
    let mut files: Vec<String> = String::from_utf8(listed.stdout)?
        .split('\0')
        .filter(|f| !f.is_empty() && !SOURCE_EXCLUDE.iter().any(|p| f.starts_with(p)))
        .filter(|f| root.join(f).is_file())
        .map(str::to_owned)
        .collect();
    files.sort();
    files.dedup();
    let mut zip = zip::ZipWriter::new(fs::File::create(out)?);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for f in files {
        let mode = if f.ends_with(".sh") { 0o755 } else { 0o644 };
        zip.start_file(f.as_str(), opts.unix_permissions(mode))?;
        zip.write_all(&fs::read(root.join(&f))?)?;
    }
    zip.finish()?;
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
