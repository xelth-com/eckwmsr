# Report: Extract Manufacturing Date & Warranty Status
**Executor:** Claude Opus 4.6
**Status:** SUCCESS
**Changes:**
- `src/handlers/support.rs`: Added `manufacturing_date` field to `TicketSummary`, extracted from Zoho ticket `customFields` (keys: herstellungsdatum, manufacturing date, manufacturing)
- `web/src/routes/dashboard/support/+page.svelte`:
  - Split "Subject / Device" column into separate "Subject" and "Device & Warranty" columns
  - Added `getWarrantyStatus()` function with 4-tier logic: <2y Warranty (green), 2-2.3y Likely Warranty (yellow), 2.3-2.5y Goodwill (orange), >2.5y Out of Warranty (gray)
  - Warranty badge displayed below device info in the list
- `web/src/routes/dashboard/support/[id]/+page.svelte`: Added manufacturing date display (de-DE locale) and warranty badge to the Device Info box on ticket detail page

**Build:** Rust `cargo check` OK (warnings only), SvelteKit `npm run build` OK


[SYSTEM: EMBEDDED]
