# Report: Fix Zoho Attachment Downloads & Unicode Mojibake
**Executor:** Claude Opus 4.6
**Status:** SUCCESS

## Task 1: Fix Attachment Downloads
- Rewrote `scraper/server.js` download-attachment endpoint: replaced `page.evaluate(fetch())` with Playwright's native `context.request.get()` — handles large binary files safely in Node.js, avoids DOM memory limits
- Added `context` parameter to `runScraper()` callback so endpoints can use native API context
- Previous fix (orgId) preserved: Zoho API requires `orgId=20078282365` on all requests

## Task 2: Fix Unicode Mojibake
- **Root cause**: The scraper returns correct UTF-8 — the mojibake was a data artifact from previous imports that went through Python on Windows (cp1252 stdin encoding corrupted the UTF-8 bytes)
- Added `fixMojibake()` function to `scraper/server.js` as a safety net: scans for Windows-1252 double-encoding patterns (e.g. Ã¶→ö, â€ž→„) and fixes them in-place, preserving already-correct Unicode chars
- Applied to both single-ticket and bulk thread fetch endpoints
- Re-imported ticket 53451000033454145 with correct UTF-8 — all German chars now display correctly (ö, ü, ä, ß, „, ", –)

## Files Changed
- `scraper/server.js`: Added fixMojibake(), rewrote download-attachment to use context.request.get(), passed context to runScraper callback
- `src/handlers/support.rs`: Upsert path now downloads attachments, added serde alias for AttachmentRef
