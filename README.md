# Fork Zapret

Современный десктопный лаунчер для [zapret](https://github.com/Flowseal/zapret-discord-youtube) (сборка Flowseal) на **Tauri 2 + Rust + SvelteKit 2 (Svelte 5)**.

**Автор и разработчик:** [W3kiss](https://github.com/W3kiss)

## Возможности

- Большая кнопка подключения в стиле VPN, выбор стратегии (`general`, `ALT` … `ALT13`, `FAKE TLS`, `SIMPLE FAKE`, `EXP`).
- **Без консольных окон.** Rust сам разбирает `.bat`, извлекает аргументы и запускает `winws.exe` напрямую с флагом `CREATE_NO_WINDOW`. Так же скрыто запускаются все вспомогательные утилиты (`netsh`, `sc`, `taskkill`, `schtasks`).
- **Подобрать ALT.** Приложение по очереди запускает стратегии и проверяет HTTPS-доступ к адресам из `utils/targets.txt` (Discord, YouTube, Google, Cloudflare). Первая рабочая стратегия выбирается и сохраняется.
- **Game Filter и IPSet (`loaded` / `none` / `any`).** Логика `service.bat` повторена на Rust 1:1, состояние хранится в тех же файлах (`utils/game_filter.enabled`, `lists/ipset-all.txt`).
- **Smart Mode.** zapret работает только пока открыт Discord или браузер. Список процессов можно настроить.
- Глобальная горячая клавиша (по умолчанию `Ctrl+Shift+Z`), трей со статусом, сворачивание в трей.
- Автозапуск через планировщик задач с наивысшими правами, без запроса UAC.
- **Обновление zapret из GitHub Releases.** Перед обновлением создаётся резервная копия, пользовательские списки сохраняются. Откат к копии и сброс настроек делаются из настроек.
- Редактор `list-general*.txt`, `list-exclude-user.txt`, `ipset-*.txt` с мгновенным сохранением.
- Языки: русский, английский, украинский, казахский, немецкий. Темы: **Star** (анимированные звёзды, следят за курсором), Моно, Бирюза, Фиолет, Киберпанк, Изумруд, AMOLED, Светлая.
- Звуковые эффекты (Web Audio), динамический аватар автора с GitHub.

## Структура

```
Fork Zapret/
├─ Zapret/                 ← сборка Flowseal (встраивается в установщик)
└─ zapret-launcher/
   ├─ src/                 ← SvelteKit: страницы, компоненты, i18n, темы
   └─ src-tauri/
      ├─ src/              ← Rust: strategy, process, controller, autopick,
      │                      updater, ipset, game_filter, lists, smart_mode,
      │                      autostart, tray, hotkey, commands
      ├─ icons/            ← иконки приложения и трея (app-icon.svg — исходник)
      └─ app.manifest      ← requireAdministrator
```

При первом запуске встроенная папка `Zapret` копируется в `%LOCALAPPDATA%\com.w3kiss.forkzapret\zapret`. Там же лежат резервные копии (`backups\`), конфигурация (`config.json`) и журнал `winws.log`.

## Сборка

Требуется Node.js 20+, Rust (stable, MSVC) и WebView2 (есть в Windows 10/11).

```powershell
cd "Fork Zapret\zapret-launcher"
npm install
npm run tauri build      # установщик: src-tauri\target\release\bundle\nsis\
```

Режим разработки: `npm run tauri dev`. Терминал нужно открыть **от имени администратора**, потому что в манифест встроен `requireAdministrator`.

Пересоздать иконки из исходника: `npm run icons`. Иконки трея `tray-on.png` и `tray-off.png` (64×64) лежат отдельно.

## Лицензия

MIT © W3kiss. zapret © bol-van, сборка © Flowseal — распространяются по их лицензиям.

> Fork Zapret — неофициальный лаунчер. Используйте его в соответствии с законодательством вашей страны.
