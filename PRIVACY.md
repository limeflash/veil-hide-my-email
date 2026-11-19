# Veil — Privacy Policy

_Last updated: 19 November 2026_

Veil is a browser extension that creates and manages iCloud+ Hide My Email addresses. This page explains what the extension does with your data. In short, nothing leaves your browser except the requests to Apple that you ask for.

## What Veil does not do

- It has no server. The developer receives no data from you.
- No analytics, telemetry, crash reporting, ads or tracking.
- It does not sell, share or transfer your data to anyone.
- It does not read, record or send the content of the pages you visit.

## Network requests

Veil talks only to Apple's iCloud services (`setup.icloud.com`, `*.icloud.com` and, for iCloud China, `*.icloud.com.cn`). It does so to check that you are signed in, to generate, reserve, list, edit, deactivate or delete your Hide My Email addresses, and to change your forwarding address. These requests use the iCloud.com session that already exists in your browser. Veil never sees or stores your Apple ID password. A browser rule sets the `Origin`/`Referer` headers that iCloud expects on these requests.

## Data stored in your browser

| What | Where | Why | Removed when |
| --- | --- | --- | --- |
| Settings (autofill toggles, theme, language) | `storage.local` | Remember your choices | You uninstall Veil |
| iCloud service address, and the account name and Apple ID returned by iCloud | `storage.local` | Talk to the right iCloud server; show who is signed in | You sign out, iCloud ends the session, or you uninstall |
| Your address list and the address currently proposed in the popup | `storage.session` (memory only) | Open the popup instantly | You sign out or close the browser |

None of this is synced to other devices by Veil, and the content scripts that run on websites cannot read any of it.

## On websites

On web pages, Veil's content script looks for email input fields so it can show its button there. When you click the button, use the right-click menu, or choose "Fill on page" in the popup, Veil reserves an address labelled with the site's domain and writes it into the field. The site then receives that Hide My Email address, which is the point. Your real email address is never written into pages.

## Clipboard and notifications

Veil writes to the clipboard only when you click Copy, or when the right-click menu fills an address (it copies it too). It shows a single system notification when it detects your iCloud sign-in.

## Your control

Sign out from the popup to clear the stored session and caches. Uninstalling Veil removes everything it stored. Your Hide My Email addresses themselves live in your iCloud account and can be managed at icloud.com.

## Contact

Questions or concerns: open an issue at https://github.com/limeflash/veil-hide-my-email/issues.

Veil is an independent open-source project. It is not affiliated with, endorsed or sponsored by Apple Inc. iCloud and Hide My Email are trademarks of Apple Inc.

---

# Veil — Политика конфиденциальности (RU)

Veil не имеет сервера и ничего не отправляет разработчику: никакой аналитики, телеметрии и рекламы, данные никому не продаются и не передаются. Расширение обращается только к сервисам Apple iCloud (`setup.icloud.com`, `*.icloud.com`, `*.icloud.com.cn`). Оно делает это, чтобы проверить вход и выполнить действия, которые вы запросили: создать, сохранить, показать, изменить, отключить или удалить адреса и сменить адрес пересылки. Для этого используется уже существующая сессия iCloud.com в браузере. Пароль Apple ID расширение не видит и не хранит.

В браузере хранятся настройки и данные сессии iCloud (адрес сервиса, имя и Apple ID из ответа iCloud) в `storage.local`. Список адресов и текущий предложенный адрес лежат в `storage.session`: только в памяти, они стираются при выходе из аккаунта или закрытии браузера. Скрипт на сайтах ищет поля email, чтобы показать кнопку. Адрес он вставляет только по вашему действию и содержимое страниц не читает и не отправляет. В буфер обмена Veil пишет только по кнопке «Копировать» или при вставке из контекстного меню.

Выход из аккаунта в окне расширения очищает сессию и кэш, удаление расширения стирает всё. Вопросы: https://github.com/limeflash/veil-hide-my-email/issues. Veil — независимый проект, не связанный с Apple Inc.
