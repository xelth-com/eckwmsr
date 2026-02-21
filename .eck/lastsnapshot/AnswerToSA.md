# Exact Online Integration Stub — Done

## What was added to `scraper/server.js`

### `exactLogin(page, targetUrl, username, password)`
Handles authentication on `start.exactonline.de`:
- Navigates to the target URL and waits for SPA to settle
- Dismisses cookie/consent banners if present
- Supports both single-step and two-step (username → next → password) login flows
- Falls back gracefully with warnings if fields are not found

### `POST /api/exact/inventory/fetch`
Stub endpoint:
- Logs in via `exactLogin`
- Returns `current_url` and a 500-char `text_preview` of the dashboard
- `// TODO` comment marks where inventory table navigation and parsing will go

### `POST /api/exact/quotation/create`
Stub endpoint:
- Logs in via `exactLogin`
- Returns `current_url`
- `// TODO` comment marks where Sales → Quotations → New navigation will go

### `/debug` updated
Both new routes are listed in the debug endpoint's `endpoints` array.

## Next steps
1. Test login flow against live `start.exactonline.de` with real credentials
2. Map exact URL paths for inventory page and quotation form
3. Implement parsing/form-fill logic in the stub TODOs
