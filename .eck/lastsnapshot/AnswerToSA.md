# Task: Extract delivery scrapers to Node.js Playwright microservice

## Status: DONE

## Changes Applied
1. **Created** `scraper/package.json` — Node.js package manifest (express + playwright)
2. **Created** `scraper/server.js` — Express server on port 3211 with `/api/dhl/create` and `/api/opal/create` endpoints using Playwright
3. **Removed** `thirtyfour = "0.35"` from `Cargo.toml`
4. **Removed** `pub mod webdriver;` from `src/utils/mod.rs`
5. **Deleted** `src/utils/webdriver.rs`
6. **Rewrote** `src/services/delivery_dhl.rs` — now POSTs to `http://127.0.0.1:3211/api/dhl/create`
7. **Rewrote** `src/services/delivery_opal.rs` — now POSTs to `http://127.0.0.1:3211/api/opal/create`

## Port
Scraper microservice runs on **3211** (user request, changed from plan's 3005).

## Validation
- `cargo check` passes with 0 errors, warnings only (pre-existing dead code)

## Next Steps
```
cd scraper && npm install && npx playwright install chromium
```
