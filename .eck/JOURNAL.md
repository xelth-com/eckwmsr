# Development Journal

## 2026-03-03 — feat(crypto): Binary SmartTag encryption with dynamic IV length

### What Was Done
- **SmartTag struct** (`smart_code.rs`): 16-byte UUID + 1-byte entity_type + 2-byte flags (big-endian). Entity type constants for WMS (0x00–0x05), Twenty CRM (0x10–0x12), Odoo (0x20–0x21).
- **`eck_binary_encrypt`** (`encryption.rs`): Serializes 19-byte SmartTag → AES-192-GCM encrypt → 35 bytes → 56 Base32 chars. Random IV string of configurable length, nonce derived via SHA-256(iv_string)[:12].
- **`eck_binary_decrypt`** (`encryption.rs`): Strips prefix/suffix, takes first 56 chars as data, remainder as iv_string. Auto-detects IV length — old QR codes stay valid if `QR_IV_LENGTH` changes.
- **Config** (`config.rs`): Added `qr_prefixes` (comma-separated, env `QR_PREFIXES`), `qr_tenant_suffix` (env `QR_TENANT_SUFFIX`), `qr_iv_length` (env `QR_IV_LENGTH`).
- **Tests**: 7 new encryption tests + 2 SmartTag tests, all passing. Verifies roundtrip, multi-prefix, wrong key/suffix rejection, QR Version 3 fit (76 chars ≤ 77 max).

### QR String Layout
`[PREFIX 9ch][DATA 56ch][IV 9ch][SUFFIX 2ch]` = 76 chars (fits QR V3 Alphanumeric)



## 2026-03-03 — Agent Report

# Task Complete: Murmur3 CAS Verification + Idempotency (Rust Server)

## Date: 2026-03-03

### Status
✅ **COMPLETE — Server verifies content hash, deduplicates by CAS UUID**

---

## What Was Done

### Dependency
- Added `murmur3 = "0.5"` to Cargo.toml

### FileStore (`services/filestore.rs`)
- **`content_hash_uuid(data: &[u8]) -> Uuid`**: Computes deterministic UUID from file bytes using MurmurHash3 x64_128 (seed=0)
- **Updated `save_file()`**: Now accepts `claimed_id: Option<&str>`
  - If provided: verifies `claimed_id == computed_hash`. Returns **400 Bad Request** on mismatch (data corruption)
  - Deduplication: checks by UUID first (new CAS), then falls back to SHA-256 hash (backward compat with old uploads)
  - File record `id` is now the deterministic CAS UUID (not random v4)
- **Cross-platform test**: Asserts matching UUIDs with Kotlin reference vectors

### Upload Handler (`handlers/file.rs`)
- Extracts `imageId` from multipart form data
- Passes it as `claimed_id` to `save_file()`
- CAS mismatch returns 400, other errors return 500

### Backward Compatibility
- Old uploads (with SHA-256 hash) are still deduplicated via hash column lookup
- `save_file()` without `claimed_id` (None) skips verification — works for server-side imports (support scraper)

---

## Files Changed

| File | Change |
|------|--------|
| `Cargo.toml` | +`murmur3 = "0.5"` |
| `src/services/filestore.rs` | +`content_hash_uuid()`, updated `save_file()` with CAS verification + idempotency |
| `src/handlers/file.rs` | Extract `imageId` from multipart, pass to save_file, 400 on CAS mismatch |
| `src/handlers/support.rs` | Added `None` for new `claimed_id` parameter |

## Build & Test
- `cargo check` — **OK** (50 pre-existing warnings)
- `cargo test test_content_hash` — **1 passed**

## Cross-Platform Reference Vectors
```
"test"  -> ac7d28cc-74bd-e19d-9a12-8231f9bd4d82
"hello" -> cbd8a7b3-41bd-9b02-5b1e-906a48ae1d19
""      -> 00000000-0000-0000-0000-000000000000
```

---

**Agent**: Expert Developer (The Fixer)
**Status**: ✅ Complete

## 2026-03-03 — Agent Report

# Report: Nginx Fix for Photo Upload (413 Entity Too Large)

## Date: 2026-03-03

### Status
✅ **COMPLETE — Production fix applied**

---

## Problem
Photos uploaded from Android PDA via global failover (pda.repair) were rejected with **413 Request Entity Too Large**. The Nginx config for `pda.repair` had no `client_max_body_size` set, defaulting to 1MB. Repair photos are 850KB-1MB, so anything slightly over 1MB was blocked.

This was discovered when investigating why only 2 out of 10-15 photos reached the server. The other causes were on the Android side (slot_N.webp overwriting — fixed in eckwms-movFast).

## What Was Done
- Added `client_max_body_size 50M` to the `location ~ ^/E/` block in `/etc/nginx/sites-available/pda.repair.conf`
- `nginx -t` passed, `systemctl reload nginx` applied

## Verification
```
$ grep client_max_body_size /etc/nginx/sites-available/pda.repair.conf
        client_max_body_size 50M;
```

Other eck*.com configs already had 100M — only pda.repair was missing it.

---

## No Code Changes
The Rust server (`eckwmsr`) was not modified. The fix was purely Nginx configuration on the production server.

---

**Agent**: Expert Developer (The Fixer)
**Status**: ✅ Complete

## 2026-03-02 — feat(sync): mesh sync for orders, documents, files, attachments

- **order.id migrated from i32 to UUID** — distributed-safe, no auto-increment conflicts between servers
- **4 new entity types in mesh sync**: order, document, file_resource, attachment
- Added `SyncableOrder`, `SyncableDocument`, `SyncableFileResource` wrapper structs (base64 avatar_data, include deleted_at)
- Extended `PullResponse`, `PushPayload`, `pull_handler`, `push_handler` with new entity vectors
- Extended `SyncEngine`: upsert methods, relay process_*_packet, apply_pull_response, perform_push
- Extended `MeshClient::push_entities` with 4 new Vec params, added `fetch_file(hash)` for lazy CAS fetch
- **Push-on-write**: repair events, orders (create/update), file uploads + attachments push to all mesh peers
- **3-tier push**: Direct HTTP → WebSocket signal → Relay fallback (same pattern as user sync)
- **Startup sync** now pulls all 5 entity types (user, order, document, file_resource, attachment)
- **`/mesh/file/:hash`** endpoint for peer lazy-fetch of full CAS file content
- DB migration: `ALTER TABLE orders ALTER COLUMN id TYPE uuid` on both local and prod

## 2026-03-02 — feat(repair): Auto-create Repair Order on PDA slot bind

- Added `device_bound` event trigger in Android `MainScreenViewModel` when a repair slot is bound to a device barcode
- Added Rust backend interceptor in `handlers/repair.rs` for `device_bound` event type
- Added `RepairService::process_device_bind()` in `services/repair.rs` — checks for existing active orders (not completed/cancelled) by serial number, creates a new pending `repair` order if none exist
- Order number format: `REP-YYYYMMDD-XXXX` (random suffix)
- Fixed NOT NULL constraint error: all required order fields now explicitly set (customer_email, parts_used, metadata, etc.)

## 2026-03-02 — feat(pairing): include mesh peer URLs in QR candidates

- `handlers/device.rs` `generate_pairing_qr()` now queries `mesh_nodes` for non-offline peers
- Peer `base_url`s appended to connection candidates in QR code
- Clients discover all servers in the mesh during pairing, not just the host

## 2026-03-02 — fix(android): remove hardcoded pda.repair, filter link-local IPs

- `SettingsManager.kt`: server_url and global_server_url defaults changed from `pda.repair/E` to empty string
- Migration v2: clears any saved pda.repair URLs from legacy installs
- `ScanRecoveryViewModel.kt`: 169.254.x.x addresses filtered from pairing candidates when real IPs available
- `NetworkUtils.kt`: `isLinkLocalAddress` check added to skip 169.254.x.x in device IP detection
- `NetworkPanelSheet.kt`: shows "NOT PAIRED" (grey) when server URLs are empty

## 2026-03-02 — ops: prod DB schema migration for file_resources

- Added columns: `hash` (unique), `width`, `height`, `avatar_data`, `context` to `file_resources` on prod
- Fixed NOT NULL constraints on `mime_type`, `size`, `source_instance`, timestamps
- Cleaned legacy data (349K stub file_resources from Go era)



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
