# feat(support, ui): Build Support module UI for viewing imported tickets and attachments

## What was done

### Step 1 — Rust: New read endpoints in `src/handlers/support.rs`

**`GET /api/support/tickets`** (`list_tickets`)
- Fetches all documents with `type = "support_thread"` from the DB
- Groups by `payload.ticketId` in memory using a HashMap
- Returns `Vec<TicketSummary>` (ticketId, subject, status, customer, thread_count, latest_update)
- Sorted newest-first by `createdTime` string (ISO 8601 sorts lexicographically)

**`GET /api/support/tickets/:ticket_id/threads`** (`get_ticket_threads`)
- Fetches all `support_thread` documents, filters by `payload.ticketId == ticket_id` in Rust
- Returns the documents sorted oldest-first (conversation order)
- Each document carries the full payload including `content` (HTML body) and metadata

Both endpoints registered in `src/main.rs` alongside the existing import route.

### Step 2 — Layout: Support nav link
Added a "Support" link in `web/src/routes/dashboard/+layout.svelte` after RMA Requests.

### Step 3 — Ticket List (`web/src/routes/dashboard/support/+page.svelte`)
- Calls `GET /api/support/tickets` on mount
- Table: Ticket ID, Subject, Customer, Status (colored badge), Thread count, Latest Update
- Clicking a row navigates to `/dashboard/support/{ticketId}`
- Empty state directs user to the Scrapers page import flow

### Step 4 — Ticket Detail (`web/src/routes/dashboard/support/[id]/+page.svelte`)
- Route param `id` = Zoho ticketId
- Fetches threads from `GET /api/support/tickets/{id}/threads`
- Loads attachments per thread via existing `GET /api/attachments/document/{docUUID}` in parallel
- Renders email HTML with `{@html thread.payload.content}` in a scrollable container
  - Applied `:global()` CSS rules to constrain images, tables, links inside `{@html}` content
- Inbound/outbound threads visually distinguished by colored left border + direction badge
- Attachments shown as clickable pills; images get a 40×40px thumbnail preview
- All file links point to `GET /api/files/{file_id}` (public CAS endpoint)

## Verification
- `cargo check` passes clean
