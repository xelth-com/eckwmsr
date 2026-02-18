# Task: Scraper Admin UI + /S/ Proxy + Full Import Implementation

## Status: DONE

## Changes Applied

### 1. `scraper/server.js`
- **Debug mode**: `"debug": true` in POST body launches headed Chromium with 600ms slowMo
- **`_from_env` support**: send `"_from_env": true` to use server-side env vars instead of providing creds
- **GET `/debug`**: returns service status and all available endpoints (JSON)
- **Full OPAL parser**: restored from `eckwmsgo` — all fields (contacts, dates, dimensions, receiver, etc.)
- **Full DHL CSV parser**: added `international_number`, `billing_number`, `delivered_to_*`, `services`, row guard

### 2. `src/handlers/scraper_proxy.rs` (new file)
- Reverse proxy: `/S/*` → `http://127.0.0.1:3211/*`
- JWT auth validated inline (Bearer token required)
- `Authorization` header stripped before forwarding to scraper
- 10MB body limit, 300s timeout

### 3. `src/main.rs`
- Added routes: `GET/POST /S` and `/S/*path` → `scraper_proxy::proxy_handler`
- `routing::any` imported

### 4. `src/handlers/mod.rs`
- Added `pub mod scraper_proxy;`

### 5. `src/handlers/delivery.rs`
- `trigger_opal_import`: real implementation — calls `/api/opal/fetch`, saves to `stock_picking_delivery`, creates/updates `sync_history`
- `trigger_dhl_import`: same for DHL
- Helper functions: `ensure_carrier`, `finish_sync`, `map_opal_status`, `map_dhl_status`, `parse_opal_datetime`

### 6. `src/db.rs`
- `setup_password` changed from `Option<String>` to `Arc<RwLock<Option<String>>>`
- Added `cleanup_setup_if_real_users()` — deletes setup account and clears in-memory password once real user exists

### 7. `src/handlers/auth.rs`
- `setup_status` reads from `RwLock`

### 8. `src/handlers/admin_users.rs`
- After creating a user, calls `cleanup_setup_if_real_users()`

### 9. `web/src/routes/login/+page.svelte`
- On mount: fetches `/E/auth/setup-status`
- If `needsSetup: true`: shows yellow "Initial Setup" banner with credentials, pre-fills form
- Shows "Rust Edition" (not "GO Edition") when setup is done

### 10. `web/src/routes/dashboard/shipping/+page.svelte`
- New tab **Scraper Admin** with:
  - Service status indicator (online/offline dot)
  - Check Status button → calls `GET /S/debug`
  - Endpoint badges list
  - OPAL card: limit selector, Debug toggle, Run Fetch button, JSON result viewer
  - DHL card: same layout
  - Credentials note at bottom

## Validation
- `cargo check`: 0 errors
- `npm run build`: success
- `GET /S/debug` via JWT proxy: 200 OK
- `POST /S/api/opal/fetch` with `_from_env: true`: returns OPAL orders

## Servers
- Rust: `http://127.0.0.1:3210`
- Node scraper: `http://127.0.0.1:3211`
- Proxy: `http://127.0.0.1:3210/S/` → `http://127.0.0.1:3211/`

