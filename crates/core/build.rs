//! Generates the translation table (`T` + `T::get`) from `locales/*.json`.
//!
//! `en.json` defines the keys. Every other locale must keep each key's `{}`
//! placeholders and `**bold**` markers (a mismatch fails the build). Missing
//! keys fall back to English with a warning, and `cargo test` lists them.

use serde_json::{Map, Value};
use std::fmt::Write as _;
use std::{env, fs, path::Path};

/// (file stem, `Lang` variant). Order = `Lang::ALL`.
const LANGS: &[(&str, &str)] = &[
    ("en", "En"),
    ("ru", "Ru"),
    ("es", "Es"),
    ("de", "De"),
    ("fr", "Fr"),
    ("tr", "Tr"),
    ("ar", "Ar"),
    ("fa", "Fa"),
    ("zh", "Zh"),
    ("ja", "Ja"),
];

fn load(stem: &str) -> Option<Map<String, Value>> {
    let path = format!("locales/{stem}.json");
    println!("cargo:rerun-if-changed={path}");
    let text = fs::read_to_string(&path).ok()?;
    match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(m)) => Some(m),
        Ok(_) => panic!("{path}: expected a JSON object"),
        Err(e) => panic!("{path}: {e}"),
    }
}

fn shape(s: &str) -> (usize, usize) {
    (s.matches("{}").count(), s.matches("**").count())
}

fn main() {
    println!("cargo:rerun-if-changed=locales");
    let en = load("en").expect("locales/en.json is required");
    let keys: Vec<&String> = en.keys().collect();

    let mut out = String::new();
    out.push_str("/// Every UI string. Generated from `locales/*.json` by build.rs.\n");
    out.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum T {\n");
    for k in &keys {
        writeln!(out, "    {k},").unwrap();
    }
    out.push_str("}\n\nimpl T {\n    pub const ALL: &'static [T] = &[");
    for k in &keys {
        write!(out, "T::{k}, ").unwrap();
    }
    out.push_str("];\n\n    pub fn get(self, lang: Lang) -> &'static str {\n        match lang {\n");

    let mut missing: Vec<(String, String)> = Vec::new();
    for (stem, variant) in LANGS {
        let table = if *stem == "en" { Some(en.clone()) } else { load(stem) };
        writeln!(out, "            Lang::{variant} => match self {{").unwrap();
        for k in &keys {
            let english = en[*k].as_str().unwrap_or_else(|| panic!("en.json: {k} is not a string"));
            let text = match table.as_ref().and_then(|t| t.get(*k)).and_then(Value::as_str) {
                Some(s) if !s.trim().is_empty() => {
                    if shape(s) != shape(english) {
                        panic!(
                            "locales/{stem}.json: {k} must keep {} `{{}}` and {} `**` like English, got {:?}",
                            shape(english).0,
                            shape(english).1,
                            s
                        );
                    }
                    s
                }
                _ => {
                    missing.push((stem.to_string(), k.to_string()));
                    english
                }
            };
            writeln!(out, "                T::{k} => {text:?},").unwrap();
        }
        out.push_str("            },\n");
        if let Some(t) = &table {
            for extra in t.keys().filter(|k| !en.contains_key(*k)) {
                println!("cargo:warning=locales/{stem}.json has unknown key {extra}");
            }
        }
    }
    out.push_str("        }\n    }\n}\n\n/// (language, key) pairs that fell back to English.\npub const MISSING: &[(&str, &str)] = &[\n");
    for (l, k) in &missing {
        writeln!(out, "    ({l:?}, {k:?}),").unwrap();
    }
    out.push_str("];\n");
    let langs_missing: std::collections::BTreeSet<_> = missing.iter().map(|(l, _)| l.as_str()).collect();
    if !langs_missing.is_empty() {
        println!("cargo:warning=incomplete translations (English fallback): {langs_missing:?}");
    }

    let dest = Path::new(&env::var("OUT_DIR").unwrap()).join("i18n_gen.rs");
    fs::write(dest, out).unwrap();
}
