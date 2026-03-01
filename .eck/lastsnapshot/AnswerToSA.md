# Report: Fix Zoho session management + Bulk import
**Executor:** Claude Opus 4.6
**Status:** SUCCESS

## Changes:

### 1. Persistent browser context (scraper/server.js)
- `runScraper()` now uses `chromium.launchPersistentContext()` with `.browser-data/` directory
- Cookies/sessions survive across requests — no repeated logins
- Added request serialization lock to avoid concurrent browser conflicts

### 2. Smart Zoho login (scraper/server.js)
- `zohoLogin()` first checks if session cookies are valid via a lightweight API call
- If cookies work → skips login entirely (no navigation, no credentials)
- If expired → navigates to Desk, tries login only if not blocked
- Detects `signin-block` URL and throws clear error instead of retrying
- `zohoApi()` ensures page is on correct domain before fetch

### 3. Bulk thread fetch endpoint (scraper/server.js)
- `POST /api/zoho/ticket-threads-bulk` — accepts `ticketIds[]`, single login, 1.5s pause between tickets
- Frontend `importAllTickets()` uses bulk endpoint instead of N separate calls

### How to initialize after signin-block expires:
1. Go to Scrapers page
2. Enable "Debug (headed)" checkbox
3. Click "Fetch Tickets" — Playwright opens visible browser
4. Complete Zoho login manually (if captcha/2FA)
5. Cookies saved to `.browser-data/` — all subsequent requests use saved session
