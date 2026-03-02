# Development Journal


## 2026-03-02 — feat(repair): Auto-create Repair Order on PDA slot bind

- Added `device_bound` event trigger in Android `MainScreenViewModel` when a repair slot is bound to a device barcode
- Added Rust backend interceptor in `handlers/repair.rs` for `device_bound` event type
- Added `RepairService::process_device_bind()` in `services/repair.rs` — checks for existing active orders (not completed/cancelled) by serial number, creates a new pending `repair` order if none exist
- Order number format: `REP-YYYYMMDD-XXXX` (random suffix)



## 2026-03-01 — Agent Report

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

## 2026-03-01 — Agent Report

# Report: Status Colors, Shipping Fix, and Repair Workflow Integration
**Executor:** Claude Opus 4.6
**Status:** SUCCESS
**Changes:**
- Updated `statusClass` in Support list (`+page.svelte`) and detail (`[id]/+page.svelte`) to detect "Pending Agent Answer" (urgent/red) and "Research Period" (research/blue) with matching CSS styles.
- Fixed snake_case vs camelCase parsing bug in `shipping/+page.svelte`: changed `shipment.rawResponse` to `shipment.raw_response` and `shipment.trackingNumber` to `shipment.tracking_number`. This resolved the "UNKNOWN"/"Pending..." display issue.
- Added "Repair" button to the Shipments list actions column, routing to `/dashboard/repairs/new` with tracking number, customer name, and issue pre-filled via URL params.
- Updated `repairs/[id]/+page.svelte` to parse `tracking` URL param, store it in `metadata.trackingNumber`, and display a "Linked Shipment" banner alongside the existing ticket link.

## 2026-03-01 — Agent Report

# Report: Improve Support Tickets Presentation and Related Tickets Detection
**Executor:** Claude Opus 4.6
**Status:** SUCCESS
**Changes:**
- Modified `src/handlers/support.rs`: Added `ticket_number`, `email`, `phone` fields to `TicketSummary` struct. Improved customer name extraction with `firstName`/`lastName` fallback before falling back to `from` field.
- Modified `web/src/routes/dashboard/support/+page.svelte`: Display short `ticket_number` instead of raw ticketId, show email/phone under customer name in the table.
- Modified `web/src/routes/dashboard/support/[id]/+page.svelte`: Display short ticket number in header badge, structured customer contact box with email/phone. Added "Related Tickets" banner that cross-references all imported tickets by matching email, phone, or private domain name.

## 2026-03-01 — Fix Zoho Attachment Downloads & Unicode Mojibake
- type: fix
- scope: scraper, support, backend
- **Attachment download fix**: Root cause was missing `orgId=20078282365` query parameter (422 error, not OOM). Rewrote `download-attachment` endpoint to use Playwright's native `context.request.get()` instead of `page.evaluate(fetch())` — handles large binaries safely without DOM memory limits.
- **Attachment extraction fix**: Zoho `/threads/{id}/attachments` endpoint returns 404. Attachments come from individual thread API response (`fullThread.attachments` field). Fixed both single and bulk endpoints.
- **Upsert deduplication**: `import_thread` now deduplicates by ticketId+threadId. Both insert and update paths download attachments (previously upsert skipped attachments). Added `serde(alias = "name")` to AttachmentRef for Zoho compat.
- **Unicode mojibake investigation**: Scraper always returned correct UTF-8. The mojibake in DB came from a previous import that piped JSON through Python on Windows (cp1252 stdin silently corrupted UTF-8 bytes). Added `fixMojibake()` safety net function that detects/reverses CP1252 double-encoding patterns in-place.
- **Re-imported** ticket 53451000033454145: 4 threads with full HTML + 3 PDF attachments (126KB, 1.8MB, 1.7MB), all German chars correct.
- **Verified**: „Vereinbarung", Körperanalysegerätes, füllen, Außerhalb — all display correctly.

## 2026-02-28 — Relay Pairing Fix + Three-State Health Check
- type: fix+feat
- scope: pairing, mesh, frontend
- **Pairing relay fix**: `push_packet()` used server's `mesh_id` but `pull_packets_for()` used `routing_id` as mesh_id — packets never matched. Added `push_packet_to_channel()` that uses `channel_id` for both. All 3 pairing pushes (offer, response, approval) updated.
- **Pairing verified end-to-end**: production created code, local entered it, approval modal appeared, both sides saved each other as mesh_nodes. Tested via Chrome automation.
- **Three-state health**: `active` (green, direct ping OK) / `degraded` (yellow, direct ping failed < 5 min) / `offline` (red, both direct and relay confirm down). NAT peers checked via relay only.
- **Health check interval**: 30s (was 60s). Relay queried only when needed.
- **Frontend**: three-color dots in sidebar and Mesh Servers table. "Unstable" label for degraded.
- **Deployed**: both local and production updated and verified.

## 2026-02-28 — Mutual Peer Health Check + Identity Display
- type: feat+fix
- scope: mesh, pairing, frontend
- **Mutual verification**: health check pings `GET /mesh/status?peer_id=<our_id>`, peer responds `known: true/false`. Only marks online if mutual recognition.
- **Pairing base_url exchange**: both sides exchange real `base_url` and `instance_name` during pairing flow.
- **Health endpoint**: `GET /E/health` returns `built_at` (compile-time) and `started_at`.
- **Dashboard identity**: sidebar shows "This Server" with green dot. Mesh Servers tab shows identity card.

## 2026-02-27 — Direct-First Sync + UUID Migration + Device Pairing
- type: feat+fix
- scope: sync, config, devices
- **Direct-first sync**: `push_user_to_peers()` tries Direct HTTP > WS signal > Relay (3-tier fallback).
- **MeshClient double /E fix**: base_url already contains `/E`, URLs were doubling it.
- **Startup sync**: full-pull users from all known peers on boot.
- **Device status endpoint**: `GET /api/status` for PDA heartbeat (JWT protected).
- **UUID instance_id**: auto-generates UUID v4, writes back to `.env`. Old string IDs replaced.

## 2026-02-16 — Phase 2: WebSocket + Heartbeat + Mesh Networking
- type: feat+fix
- scope: sync, mesh, websocket
- **WebSocket**: `/E/ws` handler with tokio broadcast channel, `DEVICE_IDENTIFY` handshake.
- **DB Fixes**: numeric->float8, sync_history id type, stock_picking_delivery.label_data.
- **Config**: `mesh_id` = sha256(SYNC_NETWORK_KEY)[:16], `base_url` field.
- **mesh_node model**: `mesh_nodes` table + Sea-ORM model.
- **RelayClient**: rewrite with mesh_id in all relay calls.
- **Heartbeat**: background task every 5 min. Verified on production.
- **Pairing endpoints**: `POST /api/pairing/approve` and `/api/pairing/finalize`.

## 2026-02-15 — Session: Frontend + Sync + Setup Account
- type: fix+feat
- scope: frontend, sync, auth
- **Fixes**: DB type mismatches (numeric->float8, timestamp->timestamptz), removed non-existent columns, added missing endpoints, error logging.
- **Features**: Merkle Tree sync, Conflict Resolver (VC>Priority>LWW), setup account with random password.

## 2026-02-15 — Phase 10: SPA Static File Server
- type: feat
- scope: frontend
- rust-embed static server with /E/ prefix, immutable caching, SPA fallback.

## 2026-02-15 — Phase 9: RMA, Repair & Print
- type: feat
- scope: workflows, models, print
- PDF label generation, Sea-ORM models (Order, DeviceIntake, InventoryDiscrepancy, Document), RepairService, RMA CRUD.

## 2026-02-14 — Phases 1-8
- type: feat
- scope: project
- Initial scaffold through delivery provider integration.
