# Report: Improve Support Tickets Presentation and Related Tickets Detection
**Executor:** Claude Opus 4.6
**Status:** SUCCESS
**Changes:**
- Modified `src/handlers/support.rs`: Added `ticket_number`, `email`, `phone` fields to `TicketSummary` struct. Improved customer name extraction with `firstName`/`lastName` fallback before falling back to `from` field.
- Modified `web/src/routes/dashboard/support/+page.svelte`: Display short `ticket_number` instead of raw ticketId, show email/phone under customer name in the table.
- Modified `web/src/routes/dashboard/support/[id]/+page.svelte`: Display short ticket number in header badge, structured customer contact box with email/phone. Added "Related Tickets" banner that cross-references all imported tickets by matching email, phone, or private domain name.
