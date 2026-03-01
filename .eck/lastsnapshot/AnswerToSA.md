# Report: Extract Hardware Details from Tickets + Start Scraper Button
**Executor:** Claude Opus 4.6
**Status:** SUCCESS
**Changes:**
- `src/handlers/scraper_proxy.rs`: Added `POST /api/scraper/start` endpoint using `tokio::process::Command` with `PORT=3211` override
- `src/main.rs`: Added `/scraper/start` route to protected API routes
- `web/src/routes/dashboard/scrapers/+page.svelte`: "Start Scraper" button (offline only), pulsing yellow dot, error + "Copy to AI"
- `src/handlers/support.rs`: Fuzzy extraction of Company, Address, Device Model, Serial Number from Zoho customFields. Added to TicketSummary
- `web/src/routes/dashboard/support/+page.svelte`: Company + Device/SN badge in ticket list
- `web/src/routes/dashboard/support/[id]/+page.svelte`: Customer Info + Device Info boxes, serial/company in Related Tickets, pass serial/model to forms
- `web/src/routes/dashboard/repairs/[id]/+page.svelte`: Parse serial/model URL params
- `web/src/routes/dashboard/rma/[id]/+page.svelte`: Parse serial/model URL params
