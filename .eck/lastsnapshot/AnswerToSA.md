# Report: Add ticket count selector to Zoho scraper UI
**Executor:** Claude Opus 4.6
**Status:** SUCCESS

## Changes:
- Modified `web/src/routes/dashboard/scrapers/+page.svelte`:
  - Changed default `zohoLimit` from 50 to 10 (for testing)
  - Added 500 and 1000 options to the ticket limit dropdown
- Rebuilt frontend (`npm run build`)
