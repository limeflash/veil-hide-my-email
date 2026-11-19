//! Russian and English UI strings.
//!
//! Every string is a variant of [`T`], so a missing translation is a compile
//! error rather than a blank label. Strings with `{}` are templates for
//! [`fill`].

use crate::model::LanguagePref;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lang {
    Ru,
    #[default]
    En,
}

impl Lang {
    /// Resolves the user preference against the browser UI language
    /// (e.g. `"ru-RU"`).
    pub fn resolve(pref: LanguagePref, browser_lang: &str) -> Lang {
        match pref {
            LanguagePref::Ru => Lang::Ru,
            LanguagePref::En => Lang::En,
            LanguagePref::Auto => {
                let l = browser_lang.to_ascii_lowercase();
                // Russian, plus languages whose speakers usually read Russian
                // better than English.
                if ["ru", "be", "uk", "kk", "ky", "tg"]
                    .iter()
                    .any(|p| l == *p || l.starts_with(&format!("{p}-")))
                {
                    Lang::Ru
                } else {
                    Lang::En
                }
            }
        }
    }

    /// BCP-47 tag for `<html lang>` and date formatting.
    pub fn tag(self) -> &'static str {
        match self {
            Lang::Ru => "ru",
            Lang::En => "en",
        }
    }
}

macro_rules! strings {
    ($($key:ident => $ru:expr, $en:expr;)*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum T { $($key,)* }

        impl T {
            pub const ALL: &'static [T] = &[$(T::$key,)*];

            pub fn get(self, lang: Lang) -> &'static str {
                match (self, lang) {
                    $(
                        (T::$key, Lang::Ru) => $ru,
                        (T::$key, Lang::En) => $en,
                    )*
                }
            }
        }
    };
}

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

strings! {
    // Brand
    AppName => "Veil", "Veil";
    AppTagline => "Hide My Email для iCloud", "Hide My Email for iCloud";

    // Background: context menu, notifications
    MenuSignedOut => "Veil: сначала войдите в iCloud", "Veil: sign in to iCloud first";
    MenuLoading => "Veil: проверяю вход в iCloud…", "Veil: checking iCloud sign-in…";
    MenuGenerate => "Создать и вставить скрытый адрес", "Generate and fill a hidden address";
    NotifyReadyTitle => "Veil готов к работе", "Veil is ready";
    NotifyReadyBody => "Вход в iCloud найден — можно создавать скрытые адреса.", "Found your iCloud session — you can create hidden addresses now.";
    ErrSignedOut => "Сначала войдите в iCloud", "Sign in to iCloud first";
    ErrGeneric => "Что-то пошло не так", "Something went wrong";
    ErrNoHmeService => "Hide My Email недоступен для этого аккаунта (нужна подписка iCloud+)", "Hide My Email isn't available for this account (iCloud+ required)";
    ErrNetwork => "Нет связи с iCloud", "Can't reach iCloud";

    // Common
    Loading => "Загрузка…", "Loading…";
    CheckingSession => "Проверяем сессию iCloud", "Checking your iCloud session";
    Copy => "Копировать", "Copy";
    Copied => "Скопировано", "Copied";
    FillOnPage => "Вставить на сайт", "Fill on page";
    FilledOnPage => "Адрес вставлен", "Filled in";
    NothingToFill => "На странице нет поля для email", "No email field on this page";
    Save => "Сохранить", "Save";
    Saved => "Сохранено", "Saved";
    Cancel => "Отмена", "Cancel";
    Back => "Назад", "Back";
    Retry => "Повторить", "Retry";
    TryAgain => "Попробовать снова", "Try again";
    Settings => "Настройки", "Settings";
    SignOut => "Выйти", "Sign out";
    SignedIn => "Вход выполнен", "Signed in";
    SignedOut => "Вход не выполнен", "Signed out";

    // Popup: signed out
    SignInTitle => "Войдите в iCloud", "Sign in to iCloud";
    SignInSubtitle => "Veil работает через вашу сессию на iCloud.com. Пароль расширению не нужен.", "Veil works through your iCloud.com session. It never needs your password.";
    OpenICloud => "Открыть iCloud.com", "Open iCloud.com";
    CheckAgain => "Я уже вошёл — проверить снова", "I'm signed in — check again";
    TipTrust => "Войдите с **паролем и кодом 2FA**, затем выберите «Доверять этому браузеру»", "Sign in with **password + 2FA code**, then choose “Trust this browser”";
    TipKeepSignedIn => "Отметьте **«Оставаться в системе»** — иначе сессия быстро сбросится", "Tick **“Keep me signed in”** — otherwise the session drops quickly";
    TipNoPasskey => "**Не входите через Passkey** — такую сессию расширение не увидит", "**Don't sign in with a passkey** — the extension can't see that session";
    TipAdp => "Включена Расширенная защита данных? Разрешите **«Доступ к данным iCloud в интернете»**", "Advanced Data Protection on? Allow **“Access iCloud Data on the Web”**";
    TipFirefoxContainers => "Multi-Account Containers: входите в **контейнере по умолчанию**", "Multi-Account Containers: sign in in the **default container**";

    // Popup: generate
    EyebrowNew => "Новый адрес", "New address";
    EyebrowPicking => "Подбираем адрес", "Picking an address";
    GenerateFor => "для {}", "for {}";
    Regenerate => "Сгенерировать другой адрес", "Generate another address";
    Generating => "Генерируем адрес", "Generating address";
    ForwardsTo => "Пересылка на {}", "Forwards to {}";
    LabelAndForward => "Метка {} · на {}", "Label {} · to {}";
    Label => "Метка", "Label";
    LabelPlaceholder => "Например: github.com", "E.g. github.com";
    LabelRequired => "Укажите метку — по ней вы найдёте адрес", "Add a label — it's how you'll find this address";
    Note => "Заметка", "Note";
    NoteOptional => "необязательно", "optional";
    NotePlaceholder => "Например: рассылки и уведомления", "E.g. newsletters and notifications";
    UseAddress => "Использовать адрес", "Use this address";
    Reserved => "Адрес сохранён", "Address saved";
    ReserveFailed => "Не удалось сохранить адрес", "Couldn't save the address";
    GenerateFailed => "Не удалось создать адрес", "Couldn't create an address";
    CreateAnother => "Создать ещё один", "Create another";
    ManageAddresses => "Мои адреса", "My addresses";
    NewAddress => "Новый адрес", "New address";

    // Popup: manage
    SearchAria => "Поиск", "Search";
    SearchPlaceholder => "Поиск", "Search";
    ClearSearch => "Очистить поиск", "Clear search";
    FilterAria => "Фильтр", "Filter";
    FilterAll => "Все", "All";
    FilterActive => "Активные", "Active";
    FilterInactive => "Отключённые", "Inactive";
    Active => "Активен", "Active";
    Inactive => "Отключён", "Inactive";
    EmptyTitle => "Пока нет адресов", "No addresses yet";
    EmptyBody => "Создайте первый — он появится здесь", "Create your first one — it will show up here";
    NoResultsTitle => "Ничего не найдено", "Nothing found";
    NoResultsBody => "По запросу «{}» совпадений нет. Проверьте написание или сбросьте поиск.", "Nothing matches “{}”. Check the spelling or reset the search.";
    ResetSearch => "Сбросить поиск", "Reset search";
    LoadFailed => "Не удалось загрузить адреса", "Couldn't load your addresses";
    BackToList => "Назад к списку", "Back to the list";
    CreatedOn => "Создан {}", "Created {}";
    DetailsTitle => "Сведения", "Details";
    EditingTitle => "Изменение", "Editing";
    Edit => "Изменить", "Edit";
    AddressField => "Адрес", "Address";
    ForwardTo => "Пересылка", "Forwards to";
    Created => "Создан", "Created";
    NoNote => "Нет заметки", "No note";
    CopyAddress => "Копировать адрес", "Copy address";
    NotForwarding => "Письма на этот адрес не пересылаются", "Mail to this address is not forwarded";
    Deactivate => "Отключить адрес", "Deactivate address";
    Reactivate => "Включить снова", "Turn back on";
    Delete => "Удалить", "Delete";
    Undo => "Отменить", "Undo";
    DeleteConfirmTitle => "Удалить адрес навсегда?", "Delete this address forever?";
    DeleteConfirmBody => "{} исчезнет из iCloud. Восстановить его не получится.", "{} will be removed from iCloud. It can't be restored.";
    Deactivated => "Адрес отключён", "Address deactivated";
    Reactivated => "Адрес снова активен", "Address is active again";
    Deleted => "Адрес удалён", "Address deleted";

    // Options
    OptionsTitle => "Настройки", "Settings";
    RegionGlobal => "iCloud.com", "iCloud.com";
    RegionChina => "iCloud.com.cn", "iCloud.com.cn";
    FwdTitle => "Пересылка", "Forwarding";
    FwdDesc => "Куда приходят письма, отправленные на ваши скрытые адреса", "Where mail sent to your hidden addresses is delivered";
    FwdHint => "Это настройка вашего аккаунта iCloud", "This is a setting of your iCloud account";
    FwdCurrent => "Сейчас", "Current";
    FwdLoading => "Загружаем адреса из iCloud…", "Loading addresses from iCloud…";
    FwdSignedOut => "Адреса для пересылки появятся после входа на iCloud.com", "Forwarding addresses appear after you sign in at iCloud.com";
    FwdSaved => "Адрес пересылки сохранён", "Forwarding address saved";
    AutofillTitle => "Автозаполнение", "Autofill";
    AutofillDesc => "Как Veil предлагает адрес на сайтах", "How Veil offers an address on websites";
    AutofillButton => "Кнопка в полях email", "Button in email fields";
    AutofillButtonDesc => "Появляется в поле, когда оно в фокусе", "Appears in the field when it's focused";
    AutofillMenu => "Пункт в контекстном меню", "Context-menu item";
    AutofillMenuDesc => "Правый клик по полю → «Создать и вставить скрытый адрес»", "Right-click a field → “Generate and fill a hidden address”";
    AppearanceTitle => "Оформление", "Appearance";
    Theme => "Тема", "Theme";
    ThemeDesc => "Для окна расширения и этих страниц", "For the popup and these pages";
    ThemeSystem => "Системная", "System";
    ThemeLight => "Светлая", "Light";
    ThemeDark => "Тёмная", "Dark";
    Language => "Язык", "Language";
    LanguageDesc => "«Авто» берёт язык браузера", "“Auto” follows the browser language";
    LangAuto => "Авто", "Auto";
    LangRu => "Русский", "Русский";
    LangEn => "English", "English";
    AboutTitle => "О расширении", "About";
    Disclaimer => "**Veil — независимый проект.** Он не связан с Apple Inc. и не одобрен ею. iCloud и Hide My Email — товарные знаки Apple Inc.", "**Veil is an independent project.** It is not affiliated with or endorsed by Apple Inc. iCloud and Hide My Email are trademarks of Apple Inc.";
    Credits => "Основан на оригинальном расширении Dimitrios Dedoussis и форке {}.", "Based on the original extension by Dimitrios Dedoussis and the {} fork.";
    OpenSource => "Открытый исходный код", "Open source";
    SourceCode => "Исходный код", "Source code";

    // Welcome
    WelcomeTitle => "Свой адрес для каждого сайта", "A separate address for every site";
    WelcomeLead => "Veil установлен. Он создаёт адреса Hide My Email для iCloud прямо в браузере. Осталось три шага.", "Veil is installed. It creates iCloud Hide My Email addresses right in your browser. Three steps to go.";
    Step1Title => "Войдите на iCloud.com", "Sign in to iCloud.com";
    Step1Body => "С паролем и кодом 2FA. Отметьте «Оставаться в системе».", "With your password and 2FA code. Tick “Keep me signed in”.";
    Step2Title => "Закрепите Veil на панели", "Pin Veil to the toolbar";
    Step2Body => "Откройте меню расширений в панели браузера и нажмите на булавку рядом с Veil", "Open the extensions menu in the toolbar and click the pin next to Veil";
    Step3Title => "Создавайте адреса", "Create addresses";
    Step3Body => "Из окна расширения, кнопкой в поле email или через правый клик по полю", "From the popup, the button in email fields, or by right-clicking a field";
    GrantTitle => "Разрешите доступ к icloud.com", "Allow access to icloud.com";
    FirefoxOnly => "Только Firefox", "Firefox only";
    GrantBody => "Firefox не выдаёт расширениям доступ к сайтам автоматически. Без него Veil не увидит вашу сессию.", "Firefox doesn't grant site access to extensions automatically. Without it Veil can't see your session.";
    GrantButton => "Разрешить доступ", "Grant access";
    Granted => "Доступ разрешён", "Access granted";
    FaqTitle => "Если что-то не работает", "If something doesn't work";
    FaqSignedInQ => "Уже вошли в iCloud?", "Already signed in to iCloud?";
    FaqSignedInA => "Ничего делать не нужно — откройте Veil, он подхватит текущую сессию.", "Nothing to do — open Veil and it picks up the current session.";
    FaqKeepQ => "Нужна ли галочка «Оставаться в системе»?", "Do I need “Keep me signed in”?";
    FaqKeepA => "Да. Без неё iCloud быстро завершает сессию, и Veil снова попросит войти. При входе также выберите **«Доверять этому браузеру»**.", "Yes. Without it iCloud ends the session quickly and Veil asks you to sign in again. Also choose **“Trust this browser”** when signing in.";
    FaqPasskeyQ => "Не входите через Passkey", "Don't sign in with a passkey";
    FaqPasskeyA => "Сессию после входа по Passkey расширение не видит. Войдите с паролем и кодом двухфакторной аутентификации.", "The extension can't see a session created by a passkey sign-in. Use your password and two-factor code.";
    FaqAdpQ => "Расширенная защита данных", "Advanced Data Protection";
    FaqAdpA => "Если она включена, разрешите в настройках iCloud на устройстве **«Доступ к данным iCloud в интернете»**.", "If it's on, allow **“Access iCloud Data on the Web”** in iCloud settings on your device.";
    FaqContainersQ => "Контейнеры в Firefox", "Firefox containers";
    FaqContainersA => "С Multi-Account Containers входите в iCloud в контейнере по умолчанию — сессии из других контейнеров Veil не видит.", "With Multi-Account Containers, sign in to iCloud in the default container — Veil can't see sessions from other containers.";
    SampleForward => "me@example.com", "me@example.com";
    Footnote => "Veil — независимый проект с открытым исходным кодом. Не связан с Apple Inc.", "Veil is an independent open-source project. Not affiliated with Apple Inc.";

    // In-page button (content script reads these through the background)
    InpageClickToFill => "Нажмите, чтобы вставить", "Click to fill";
    InpageGenerating => "Создаю скрытый адрес…", "Creating a hidden address…";
    InpageReserving => "Сохраняю адрес…", "Saving the address…";
    InpageFilled => "Готово — адрес вставлен", "Done — address filled";
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
        assert_eq!(Lang::resolve(LanguagePref::En, "ru"), Lang::En);
    }

    #[test]
    fn templates() {
        assert_eq!(T::GenerateFor.fill(Lang::En, &["github.com"]), "for github.com");
        assert_eq!(fill("{} and {}", &["a"]), "a and ");
    }

    #[test]
    fn every_string_is_translated() {
        for t in T::ALL {
            assert!(!t.get(Lang::Ru).is_empty(), "{t:?} ru");
            assert!(!t.get(Lang::En).is_empty(), "{t:?} en");
        }
    }
}
