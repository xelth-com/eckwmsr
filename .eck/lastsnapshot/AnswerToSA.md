# Report: Split RMA/Repairs + Fix Zoho Import Data Loss
**Executor:** Claude Opus 4.6
**Status:** SUCCESS

## Part 1: Split RMA and Repairs Frontend

**Changes:**
- Modified `web/src/routes/dashboard/+layout.svelte` — added "Repairs" sidebar link after RMA
- Modified `web/src/routes/dashboard/rma/+page.svelte` — filter fetch to `/rma?type=rma`
- Modified `web/src/routes/dashboard/rma/[id]/+page.svelte` — added `orderType: 'rma'` to formData defaults
- Modified `web/src/routes/dashboard/support/[id]/+page.svelte` — added `createRepair()` function, "Create Repair" button (blue), `.repair-btn` styles, `.summary-actions` wrapper with dual "Use for new RMA/Repair" buttons in AI summary panel
- Created `web/src/routes/dashboard/repairs/+page.svelte` — Repair Orders list page (fetches `/rma?type=repair`)
- Created `web/src/routes/dashboard/repairs/[id]/+page.svelte` — Repair Order detail/create page with repair-specific fields (repairNotes, laborHours, serialNumber), blue linked-ticket banner

**Verified in browser:**
- Sidebar shows "Repairs" link, highlights correctly
- Support ticket detail shows both "Create RMA" and "Create Repair" buttons
- "Create RMA" → navigates to `/rma/new?ticketId=...` with linked banner, pre-filled issue
- "Create Repair" → navigates to `/repairs/new?ticketId=...` with linked banner, repair-specific fields (labor hours, repair notes, serial number, status/priority selectors)
- Backend already supports `order_type` field and `?type=` query filter, auto-generates `REP-XXXXX` prefix for repairs

## Part 2: Fix Zoho Ticket Import — Missing Metadata & Content

**Root cause:** When importing Zoho threads via Scrapers → "Fetch Threads" → "Save to System", the frontend only sent `ticketId` + `threads` array, but NOT the `ticket` metadata object (subject, status, contact info). The backend stores `payload.ticket` which was always `null`. Additionally, the scraper endpoint `/api/zoho/ticket-threads` didn't fetch ticket metadata — only thread data.

**Symptoms:** Imported tickets showed "(no subject)", no customer name, "Unknown" status, and sometimes "(no content)" when the thread `content` field was null (web form submissions only have `summary`).

**Fixes applied:**

1. **`scraper/server.js`** — `/api/zoho/ticket-threads` endpoint now also fetches ticket metadata via `GET /tickets/{id}?include=contacts` and returns it as `ticket` field alongside `threads`

2. **`web/src/routes/dashboard/scrapers/+page.svelte`** — `importThreadsToSystem()` now passes `ticket` from the thread fetch result (or falls back to matching ticket from the ticket list) to the import API

3. **`src/handlers/support.rs`** — Added `summary: Option<String>` to `ThreadData` struct; content now falls back to `summary` if `content` is null: `thread.content.as_deref().or(thread.summary.as_deref())`

**Verified in browser:**
- Fetched threads for ticket `53451000033474015` ("Ersatz") — 4 threads in 17.9s
- Saved to system → 4 documents created
- Support list: subject "Ersatz", status "Pending Agent Answer", 8 threads shown
- Ticket detail: full email thread content visible (German text with inbound/outbound messages), "Create RMA" and "Create Repair" buttons work
- Old ticket `53451000033589085` still shows "(no subject)" because it was imported before the fix — re-importing would fix it
