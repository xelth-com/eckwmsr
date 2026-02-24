# feat(support, scraper): Implement generic support thread import with CAS attachments

## What was done

### Step A — Node.js scraper: `POST /api/zoho/download-attachment`
**File:** `scraper/server.js`

Added a new Playwright endpoint that:
1. Logs into Zoho Desk using session credentials (from `.env` via `_from_env: true`)
2. Uses `page.evaluate()` with `fetch(..., { credentials: 'include' })` to download the attachment file buffer inside the browser context (carrying Zoho session cookies)
3. Converts the `ArrayBuffer` to base64 in chunks of 8192 bytes
4. Returns `{ success, base64, mimeType, fileName }` to the caller

### Step B — Rust handler: `POST /api/support/import-thread`
**New file:** `src/handlers/support.rs`

For each thread:
- Saves to `documents` table with `type = "support_thread"`, full metadata in JSONB payload
- Calls scraper to download each attachment, saves to CAS via `file_store.save_file()`
- Links files to the document via `entity_attachments` (res_model="document")

Registered in `src/handlers/mod.rs` and `src/main.rs` (JWT-protected).

### Step C — Svelte UI
**File:** `web/src/routes/dashboard/scrapers/+page.svelte`

Added "Save to System" button inside the thread results box, import state variables,
`importThreadsToSystem()` function, and result display with error reporting.

## Verification
- `cargo check` passes clean (no new errors)
