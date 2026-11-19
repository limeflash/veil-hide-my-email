# Chrome Web Store submission — Veil

Everything to paste into the [Developer Dashboard](https://chrome.google.com/webstore/devconsole), field by field.
Package: `cargo xtask package` → `dist/veil-chrome-<version>.zip`.

## Store listing

**Name** (from the manifest, localized): `Veil — Hide My Email for iCloud` / `Veil — Hide My Email для iCloud`

**Summary** (manifest description, ≤132 chars):
- EN: Create and manage iCloud+ Hide My Email addresses in any browser. Autofill on sign-up forms, right-click menu, fuzzy search.
- RU: Создавайте и управляйте адресами iCloud+ «Скрыть e-mail» в любом браузере. Автозаполнение форм, контекстное меню, быстрый поиск.

**Category:** Privacy & Security (or Productivity → Tools)

**Language:** English (default). The extension name and summary are localized in 15 languages through `_locales`; the long description above can be added for each language in the dashboard.

### Description — English

Veil gives every website its own iCloud+ Hide My Email address, so your real inbox stays private. Mail sent to those addresses still reaches you.

• One click in the popup: Veil proposes a random @icloud.com address for the site you're on. Add a label and a note, then save it.
• Autofill: a small Veil button appears inside email fields. Click it and a fresh address is reserved for that site and filled in. Works with React, Vue and Angular forms.
• Right-click any text field → "Generate and fill a hidden address".
• My addresses: instant fuzzy search across labels, addresses and notes, filter active or inactive, edit labels and notes, deactivate (with undo), reactivate and delete.
• Settings: choose the forwarding address, turn each autofill method on or off, light/dark/system theme.
• 15 languages: English, Русский, Українська, Español, Português, Italiano, Deutsch, Français, Polski, Türkçe, العربية, فارسی, 简体中文, 日本語, 한국어.
• Fast and light: written in Rust and compiled to WebAssembly. The popup opens in milliseconds.
• Private by design: no servers, no analytics. Veil only talks to Apple's iCloud, using the iCloud.com session already in your browser. It never sees your password.
• Works with iCloud.com and iCloud.com.cn.

Requires an iCloud+ subscription and signing in at icloud.com in this browser (password + two-factor code; tick "Keep me signed in").

Open source (MIT): https://github.com/limeflash/veil-hide-my-email

Veil is an independent project. It is not affiliated with, endorsed or sponsored by Apple Inc. iCloud and Hide My Email are trademarks of Apple Inc.

### Description — Russian

Veil создаёт для каждого сайта отдельный адрес iCloud+ «Скрыть e-mail», и ваш настоящий ящик остаётся скрытым. Письма на эти адреса приходят вам как обычно.

• Один клик в окне расширения: Veil предлагает случайный адрес @icloud.com для текущего сайта. Добавьте метку и заметку и сохраните.
• Автозаполнение: в полях email появляется маленькая кнопка Veil. Нажмите её, и для сайта будет создан и вставлен новый адрес. Работает с формами на React, Vue и Angular.
• Правый клик по любому полю → «Создать и вставить скрытый адрес».
• Мои адреса: мгновенный поиск по меткам, адресам и заметкам, фильтр активных и отключённых, редактирование, отключение (с отменой), повторное включение и удаление.
• Настройки: адрес пересылки, включение и выключение автозаполнения, светлая, тёмная или системная тема.
• 15 языков: русский, украинский, английский, испанский, португальский, итальянский, немецкий, французский, польский, турецкий, арабский, фарси, китайский, японский и корейский.
• Быстро и легко: написано на Rust и скомпилировано в WebAssembly, окно открывается за миллисекунды.
• Приватность: никаких серверов и аналитики. Veil обращается только к iCloud Apple через вашу сессию iCloud.com в браузере. Пароль он не видит.
• Работает с iCloud.com и iCloud.com.cn.

Нужна подписка iCloud+ и вход на icloud.com в этом браузере (пароль и код 2FA, отметьте «Оставаться в системе»).

Открытый исходный код (MIT): https://github.com/limeflash/veil-hide-my-email

Veil — независимый проект, не связанный с Apple Inc. и не одобренный ею. iCloud и Hide My Email — товарные знаки Apple Inc.

### Graphics
- Store icon 128×128: `dist/chrome/icons/icon-128.png`
- Screenshots 1280×800 (1–5): `store/screenshots/*.png`
- Small promo tile 440×280: `store/promo-440x280.png`

## Privacy practices tab

**Single purpose:**
Create and manage the user's iCloud+ Hide My Email addresses and fill them into email fields on websites.

**Permission justifications:**
- `storage`: saves the user's settings and the pointer to their iCloud session (local), and caches the address list in memory (session) so the popup opens instantly.
- `declarativeNetRequest`: sets the Origin/Referer headers that Apple's iCloud API requires on the extension's own requests to *.icloud.com.
- `contextMenus`: adds the "Generate and fill a hidden address" item to the right-click menu of text fields.
- `activeTab`: when the user opens the popup, reads the current tab's URL so a new address can be labelled with the site's domain, and sends the address to that tab when the user chooses "Fill on page".
- `webRequest`: observes (without blocking or modifying) responses of icloud.com's sign-in and sign-out requests, so the extension knows when the user has signed in or out.
- `notifications`: shows one notification when the iCloud sign-in is detected and the extension is ready.
- `clipboardWrite`: copies an address to the clipboard when the user clicks Copy or uses the right-click menu.
- Host permissions `https://*.icloud.com/*`, `https://*.icloud.com.cn/*`: the extension calls Apple's iCloud Hide My Email API with the user's existing iCloud.com session. These are the only servers it contacts.
- Content script on all http/https pages: finds email input fields to show the autofill button and fills an address when the user asks. It does not read or transmit page content.

**Remote code:** No. All code, including the WebAssembly modules, is in the package.

**Data usage:** tick "Personally identifiable information" (the user's Apple ID email and Hide My Email addresses are handled on the device to provide the feature). Do not tick anything else. Then certify all three statements:
- I do not sell or transfer user data to third parties, outside of the approved use cases.
- I do not use or transfer user data for purposes that are unrelated to my item's single purpose.
- I do not use or transfer user data to determine creditworthiness or for lending purposes.

**Privacy policy URL:** https://github.com/limeflash/veil-hide-my-email/blob/main/PRIVACY.md

## Distribution
Public, all regions. Free.

## Notes for review (optional field)
Veil needs an iCloud+ account signed in at icloud.com. It has no server and only calls Apple's iCloud endpoints. Source: https://github.com/limeflash/veil-hide-my-email. Build: `cargo xtask package`.
