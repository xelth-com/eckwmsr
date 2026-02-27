# feat(workflows, ai): Link Support tickets to RMA workflow with AI summarization

## What was done

### Step 1 — Rust: `POST /api/support/tickets/:ticket_id/summary`
**File:** `src/handlers/support.rs` (`summarize_ticket`)

- Checks `state.ai_client` — returns 503 if not configured
- Fetches and filters threads by ticketId (same pattern as `get_ticket_threads`)
- Strips HTML tags with `regex::Regex` → builds a labelled plain-text transcript per thread (direction | from | time)
- Calls `ai.generate_content(system_prompt, transcript)` with primary→fallback model routing
- Returns `{"summary": "<text>"}` or appropriate HTTP error
- Registered in `main.rs`: `POST /api/support/tickets/:ticket_id/summary`

### Step 2 — Support detail UI (`web/.../support/[id]/+page.svelte`)

Added:
- `customerEmail` reactive variable (from Zoho contact)
- `summary`, `isSummarizing`, `summaryError` state
- `generateSummary()` — calls the new summary endpoint, handles loading/error states
- `createRMA()` — builds a URL with `URLSearchParams` (`ticketId`, `name`, `email`, `issue`) and navigates to `/dashboard/rma/new?...`
- **"✨ Summarize with AI"** button in the ticket header meta row
- **"📋 Create RMA"** button beside it
- **Summary panel** appears below the header once a summary exists; includes a "→ Use as issue description in new RMA" shortcut button

### Step 3 — RMA detail UI (`web/.../rma/[id]/+page.svelte`)

Added:
- `import { base } from '$app/paths'`
- `metadata: {}` field in formData initial state
- `onMount` reads `$page.url.searchParams` when `isNew`:
  - `ticketId` → stored in `formData.metadata.ticketId`
  - `name` / `email` / `issue` → pre-fill customer name, email, issue description
- **Linked ticket banner** (green border section, full-width) shown when `formData.metadata?.ticketId` is present
  - Contains link back to the source Support ticket detail page
  - Persists on existing RMAs if their `metadata.ticketId` was stored on creation

## Verification
- `cargo check` passes clean


[SYSTEM: EMBEDDED]
