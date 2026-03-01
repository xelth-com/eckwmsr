# Report: Fix Zoho Desk Attachment Downloads
**Executor:** Claude Opus 4.6
**Status:** SUCCESS
**Changes:**
- Fixed `scraper/server.js` download-attachment endpoint: added `orgId=20078282365` query parameter to Zoho API fetch (root cause was 422 UNPROCESSABLE_ENTITY, not OOM)
- Fixed `scraper/server.js` attachment extraction: use `fullThread.attachments` from individual thread API response instead of broken `/threads/{id}/attachments` endpoint (404)
- Fixed `scraper/server.js` bulk endpoint: same attachment extraction fix
- Fixed `src/handlers/support.rs` upsert path: previously skipped attachment download on thread update (only downloaded on first insert). Now downloads attachments during upsert too
- Fixed `src/handlers/support.rs` AttachmentRef: added `#[serde(alias = "name")]` so Zoho's `name` field maps to `file_name`
- Verified end-to-end: ticket 53451000033454145 imported with 4 threads (31KB, 18KB, 8KB, 389B content) and 3 PDF attachments (126KB, 1.8MB, 1.7MB)
