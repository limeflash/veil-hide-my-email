//! UI strings in ten languages.
//!
//! The strings live in `crates/core/locales/<lang>.json`; `build.rs` turns
//! them into the [`T`] enum and [`T::get`], so looking a string up is a plain
//! `match` on static data. Strings with `{}` are templates for [`fill`], and
//! `**text**` marks bold runs (see `rich` in the UI crate).

use crate::model::LanguagePref;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lang {
    #[default]
    En,
    Ru,
    Es,
    De,
    Fr,
    Tr,
    Ar,
    Fa,
    Zh,
    Ja,
}

impl Lang {
    pub const ALL: [Lang; 10] = [
        Lang::En,
        Lang::Ru,
        Lang::Es,
        Lang::De,
        Lang::Fr,
        Lang::Tr,
        Lang::Ar,
        Lang::Fa,
        Lang::Zh,
        Lang::Ja,
    ];

    /// Resolves the user preference against the browser UI language
    /// (e.g. `"ru-RU"`, `"zh-CN"`, `"pt-BR"`).
    pub fn resolve(pref: LanguagePref, browser_lang: &str) -> Lang {
        if let Some(lang) = pref.lang() {
            return lang;
        }
        let l = browser_lang.to_ascii_lowercase().replace('_', "-");
        let primary = l.split('-').next().unwrap_or_default();
        match primary {
            // Russian, plus languages whose speakers usually read Russian
            // better than English.
            "ru" | "be" | "uk" | "kk" | "ky" | "tg" => Lang::Ru,
            "es" => Lang::Es,
            "de" => Lang::De,
            "fr" => Lang::Fr,
            "tr" => Lang::Tr,
            "ar" => Lang::Ar,
            "fa" => Lang::Fa,
            "zh" => Lang::Zh,
            "ja" => Lang::Ja,
            _ => Lang::En,
        }
    }

    /// BCP-47 tag for `<html lang>` and date formatting.
    pub fn tag(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Ru => "ru",
            Lang::Es => "es",
            Lang::De => "de",
            Lang::Fr => "fr",
            Lang::Tr => "tr",
            Lang::Ar => "ar",
            Lang::Fa => "fa",
            Lang::Zh => "zh-CN",
            Lang::Ja => "ja",
        }
    }

    /// Right-to-left scripts: Arabic and Persian.
    pub fn is_rtl(self) -> bool {
        matches!(self, Lang::Ar | Lang::Fa)
    }

    /// The language's own name, for the language picker.
    pub fn native_name(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::Ru => "Русский",
            Lang::Es => "Español",
            Lang::De => "Deutsch",
            Lang::Fr => "Français",
            Lang::Tr => "Türkçe",
            Lang::Ar => "العربية",
            Lang::Fa => "فارسی",
            Lang::Zh => "简体中文",
            Lang::Ja => "日本語",
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/i18n_gen.rs"));

/// Replaces each `{}` in `template` with the next argument.
pub fn fill(template: &str, args: &[&str]) -> String {
    let mut out = String::with_capacity(template.len() + 32);
    let mut args = args.iter();
    let mut rest = template;
    while let Some(i) = rest.find("{}") {
        out.push_str(&rest[..i]);
        out.push_str(args.next().copied().unwrap_or(""));
        rest = &rest[i + 2..];
    }
    out.push_str(rest);
    out
}

impl T {
    pub fn fill(self, lang: Lang, args: &[&str]) -> String {
        fill(self.get(lang), args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_language() {
        assert_eq!(Lang::resolve(LanguagePref::Auto, "ru-RU"), Lang::Ru);
        assert_eq!(Lang::resolve(LanguagePref::Auto, "uk"), Lang::Ru);
        assert_eq!(Lang::resolve(LanguagePref::Auto, "en-US"), Lang::En);
        assert_eq!(Lang::resolve(LanguagePref::Auto, "rup"), Lang::En);
        assert_eq!(Lang::resolve(LanguagePref::Auto, "zh_TW"), Lang::Zh);
        assert_eq!(Lang::resolve(LanguagePref::Auto, "fa-IR"), Lang::Fa);
        assert_eq!(Lang::resolve(LanguagePref::Auto, "pt-BR"), Lang::En);
        assert_eq!(Lang::resolve(LanguagePref::En, "ru"), Lang::En);
        assert_eq!(Lang::resolve(LanguagePref::Ja, "ru"), Lang::Ja);
        assert!(Lang::Ar.is_rtl() && Lang::Fa.is_rtl() && !Lang::Tr.is_rtl());
    }

    #[test]
    fn templates() {
        assert_eq!(T::GenerateFor.fill(Lang::En, &["github.com"]), "for github.com");
        assert_eq!(fill("{} and {}", &["a"]), "a and ");
    }

    #[test]
    fn every_string_is_non_empty() {
        for lang in Lang::ALL {
            for t in T::ALL {
                assert!(!t.get(lang).trim().is_empty(), "{t:?} {lang:?}");
            }
        }
    }

    /// Fails while any locale still falls back to English.
    #[test]
    fn every_language_is_complete() {
        assert!(MISSING.is_empty(), "{} untranslated strings, e.g. {:?}", MISSING.len(), &MISSING[..MISSING.len().min(5)]);
    }
}
