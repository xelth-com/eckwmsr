# Report: Add error summary badges + "Copy for AI" to scraper cards
**Executor:** Claude Opus 4.6
**Status:** SUCCESS

## Changes:
- `web/src/routes/dashboard/scrapers/+page.svelte`:
  - Added `summarizeError()` — classifies errors into short labels (Timeout, Auth failed, Not found, Network error, etc.)
  - Added `copyScraperError(provider, result)` — copies structured markdown with eckWMS system context + error details for AI analysis
  - Enhanced `copyDebugInfo(sync)` — now includes system context prompt and short error summary
  - All 5 error result-boxes (OPAL, DHL, Exact, Zoho tickets, Zoho threads) now show:
    - Short error badge (e.g. "Not found", "Timeout", "Auth failed")
    - Green "Copy for AI" button
    - Full error detail in monospace block with red left border
  - Changed default `zohoLimit` from 50 to 10, added 500/1000 options
  - New CSS: `.error-row`, `.error-badge`, `.error-detail`
