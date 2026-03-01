# Report: Add "Import All" bulk import for Zoho tickets
**Executor:** Claude Opus 4.6
**Status:** SUCCESS
**Changes:**
- Modified `web/src/routes/dashboard/scrapers/+page.svelte`:
  - Added `importAllTickets()` function — iterates over all fetched tickets, fetches threads for each via scraper, saves to system via `/api/support/import-thread`
  - Shows live progress (`3/10: fetching threads for #12345…`)
  - Added "📥 Import All to Support" button after the ticket list JSON viewer
  - Result box shows total imported threads, skipped tickets (no threads), and per-ticket errors
- Added `copyForAI()` button to support ticket detail (previous task)
- Frontend build: OK
