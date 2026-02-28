# Report: Split RMA and Repairs Frontend
**Executor:** Claude Sonnet 4.6
**Status:** SUCCESS
**Changes:**
- Modified `web/src/routes/dashboard/+layout.svelte` — added "Repairs" sidebar link after RMA
- Modified `web/src/routes/dashboard/rma/+page.svelte` — filter fetch to `/rma?type=rma`
- Modified `web/src/routes/dashboard/rma/[id]/+page.svelte` — added `orderType: 'rma'` to formData defaults
- Modified `web/src/routes/dashboard/support/[id]/+page.svelte` — added `createRepair()` function, "Create Repair" button, `.repair-btn` styles, `.summary-actions` wrapper with dual "Use for new RMA/Repair" buttons
- Created `web/src/routes/dashboard/repairs/+page.svelte` — Repair Orders list page (fetches `/rma?type=repair`)
- Created `web/src/routes/dashboard/repairs/[id]/+page.svelte` — Repair Order detail/create page with repair-specific fields (repairNotes, laborHours, serialNumber)
