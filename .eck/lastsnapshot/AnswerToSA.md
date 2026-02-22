# Zoho Desk stub в eckwmsr scraper — Done

## Изменения в `scraper/server.js`

### `zohoLogin(page, targetUrl, username, password)`
- Навигация → ждёт редиректа на `accounts.zoho`
- Шаг 1: `#login_id` → `#nextbtn`
- Шаг 2: `#password` → `#nextbtn`
- Обрабатывает popup "Remind me later" если появится
- Если уже залогинен — пропускает

### `POST /api/zoho/tickets`
- Берёт credentials из body или `ZOHO_EMAIL`/`ZOHO_PASSWORD` из env (`_from_env: true`)
- Логинится через `zohoLogin`
- После логина явно навигирует на `/agent/` если попал на portal
- Возвращает `current_url` + `text_preview` (1000 chars)

### `/debug` обновлён
Новый маршрут виден в Scraper Admin tab фронтенда.

## Примечание
В `eckwmsr/.env` уже добавлены `ZOHO_EMAIL`, `ZOHO_PASSWORD`, `ZOHO_URL`.
Полная реализация синка тикетов живёт в `zoho-clicker/scraper/` (PG, pagination, incremental sync).


[SYSTEM: EMBEDDED]
