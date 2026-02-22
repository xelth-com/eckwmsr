# Zoho Desk Scraper — Done

## Разведка (Chrome MCP + Playwright probe)
- Zoho login: `#login_id` + `#nextbtn` (Weiter), затем `#password` + `#nextbtn` (Anmelden)
- После логина Zoho бросает на portal — нужна явная навигация на `/agent/`
- Внутренний API: `GET /supportapi/zd/inbodyeu/api/v1/tickets?departmentId=53451000019414029&orgId=20078282365`
- Auth: сессионные cookies (без отдельного token) — работает через `fetch(..., {credentials:'include'})` из Playwright page context

## Создано: `zoho-clicker/scraper/`

### Endpoints
| Method | Path | Описание |
|--------|------|----------|
| POST | `/api/zoho/sync` | Синк. `{mode:"full"\|"incremental", fetchDescriptions:bool}` |
| GET | `/api/zoho/tickets` | Поиск. `?status_type=Open&search=...&assignee=...&limit=50` |
| GET | `/api/zoho/tickets/:id` | Один тикет по id или ticketNumber |
| GET | `/api/zoho/stats` | Статистика по статусам и агентам |

### Стратегия синка
- **Full**: все тикеты постранично (50/страница, ~19 запросов для 939 тикетов)
- **Incremental**: только open тикеты — closed в БД не трогаются
- БД: SQLite `zoho_tickets.db`, таблицы `tickets` + `ticket_threads`

### Результат первого синка
- 939 тикетов загружено за один full sync
- 55 открытых, 742 закрытых (Closed + Auto closed)
- Агенты: Yeon Woo Jeong (514), Stephan Hwei (217), Artug Özbakir (111), Dmytro Surovtsev (67)

## Exact Online (eckwmsr/scraper) — статус
- Login flow исправлен: `[id="LoginForm$UserName"]` → Weiter → Azure B2C → `#password` → `#next`
- Блокер: 2FA (TOTP) при каждой новой Playwright сессии
- Решение: storage state — один раз войти вручную, сохранить cookies на 7 дней


[SYSTEM: EMBEDDED]
