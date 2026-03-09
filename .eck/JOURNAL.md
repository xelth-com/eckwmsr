# Development Journal










## 2026-03-09 — Agent Report

# Task Complete: Implement Sea-ORM Migrations

## Date: 2026-03-09

### Status
✅ **COMPLETE — Sea-ORM migration system implemented. Compiles clean.**

---

## What Was Done

### Summary
Transitioned the project from reflective `db::create_schema` (which used `Schema::create_table_from_entity` with `IF NOT EXISTS`) to a declarative, incremental migration system using `sea-orm-migration`. Wiped the embedded PostgreSQL data directory for a clean start with the new migration-based schema.

### Files Changed

| File | Change |
|------|--------|
| `Cargo.toml` | Added `[workspace]` section, `migration` crate dependency |
| `migration/Cargo.toml` | New — migration crate with `sea-orm-migration` dependency |
| `migration/src/lib.rs` | New — `Migrator` struct implementing `MigratorTrait` |
| `migration/src/m20260309_000001_initial_schema.rs` | New — initial migration with all 24 tables |
| `src/main.rs` | Replaced `db::create_schema()` with `Migrator::up()` |
| `src/db.rs` | Removed `create_schema` function and unused imports (`ConnectionTrait`, `Schema`, `models`) |
| `.eck/AGENT_MIGRATIONS.md` | New — protocol doc for future agents creating migrations |
| `data/pg` | Deleted — old embedded PG data directory |

### Tables in Initial Migration (24 total)
`user_auths`, `product_product`, `product_aliases`, `stock_location`, `stock_quant`, `entity_checksums`, `res_partner`, `stock_picking`, `stock_move_line`, `warehouse_racks`, `file_resources`, `entity_attachments`, `delivery_carrier`, `stock_picking_delivery`, `delivery_tracking`, `sync_history`, `orders`, `device_intakes`, `inventory_discrepancy`, `documents`, `mesh_nodes`, `registered_devices`, `items`, `order_item_events`

### Key Details
- All FK constraints preserved (stock_picking → stock_location, stock_move_line → stock_picking/product/location, etc.)
- Indexes on: `delivery_tracking.picking_delivery_id`, `sync_history.provider/status`, `orders.order_type/customer_name/product_sku/serial_number`, `order_item_events.order_id/item_id`
- Unique constraints on: `stock_location.barcode`, `stock_picking.name`, `stock_picking_delivery.picking_id`, `delivery_carrier.provider_code`, `orders.order_number`, `items.primary_barcode`, `file_resources.hash`
- `down()` method drops all tables in reverse FK-dependency order
- `documents` table PK column is `document_id` (matching existing model's `column_name` annotation)
- Legacy exceptions preserved: `orders.item_id` as `Option<i32>`, `device_intakes.odoo_repair_id` as `i64`

### Compilation
- `cargo check` — ✅ clean (0 errors, only pre-existing warnings)

---

**Agent**: Expert Developer (The Fixer)
**Status**: ✅ Complete

## 2026-03-09 — Agent Report

# Task Complete: Implement Periodic Merkle Catch-up Sync (Mesh Stage C)

## Date: 2026-03-09

### Status
✅ **COMPLETE — Periodic catch-up sync task added. Compiles clean.**

---

## What Was Done

### Summary
Added a background task in `src/main.rs` that runs every 5 minutes, iterating over all known mesh peers and invoking Merkle tree diffing (`sync_with_peer`) for all syncable entity types. This ensures eventual consistency for nodes recovering from offline status or network partitions.

### Files Changed

| File | Change |
|------|--------|
| `src/main.rs` | Added periodic Merkle catch-up sync background task |

### How It Works

1. Waits 60 seconds after startup (avoids clashing with the startup full-pull).
2. Every 5 minutes, fetches all `mesh_nodes` from the database.
3. Skips self and nodes without a `base_url`.
4. For each peer, runs `sync_with_peer` across 10 entity types: `user`, `order`, `document`, `file_resource`, `attachment`, `item`, `order_item_event`, `product`, `location`, `shipment`.
5. Failures logged at `debug` level (peers may be offline).

### Entity Types Synced
`user`, `order`, `document`, `file_resource`, `attachment`, `item`, `order_item_event`, `product`, `location`, `shipment`

### Compilation
- `cargo check` — ✅ clean (0 errors, only pre-existing warnings)

---

**Agent**: Expert Developer (The Fixer)
**Status**: ✅ Complete

## 2026-03-09 — Agent Report

# Task Complete: Implement Outbound WebSocket Client (Mesh Stage B)

## Date: 2026-03-09

### Status
✅ **COMPLETE — Outbound WS client loop implemented. Compiles clean.**

---

## What Was Done

### Summary
Implemented an active outbound WebSocket connection loop so that nodes behind NAT can proactively connect to known mesh peers, enabling bidirectional real-time sync without requiring inbound connections.

### Files Changed

| File | Change |
|------|--------|
| `Cargo.toml` | Added `tokio-tungstenite = "0.21"` with `rustls-tls-webpki-roots` |
| `src/sync/ws_client.rs` | **NEW** — Outbound WS client module |
| `src/sync/mod.rs` | Added `pub mod ws_client` |
| `src/main.rs` | Spawned `start_outbound_ws_loop` background task |

### How It Works

1. **Background loop** (`start_outbound_ws_loop`) runs every 30 seconds.
2. Reads all `mesh_nodes` from the database.
3. Skips nodes that are already connected, have no `base_url`, or are self.
4. Converts HTTP(S) base_url to WS(S) URL → `{base}/mesh/ws?instance_id={our_id}`.
5. Connects via `tokio-tungstenite::connect_async`.
6. On success: registers peer in `MeshHub`, sends HELLO, spawns bidirectional relay tasks (mirrors server-side `handle_mesh_socket` logic).
7. On UPDATE signals: re-broadcasts to hub and triggers entity sync via `sync_engine`.
8. On disconnect: unregisters peer, logs warning. Next tick will attempt reconnection.

### Architecture Notes
- Mirrors the server-side handler in `src/handlers/mesh_ws.rs` for consistency.
- Connection failures are logged at `debug` level (peers may be offline — not an error).
- Each peer connection is spawned as an independent task — one slow/failing peer doesn't block others.

### Compilation
- `cargo check` — ✅ clean (0 errors, only pre-existing warnings)

---

**Agent**: Expert Developer (The Fixer)
**Status**: ✅ Complete

## 2026-03-09 — Agent Report

# Task Complete: Migrate All Legacy Integer IDs to UUID

## Date: 2026-03-09

### Status
✅ **COMPLETE — All entity PKs and FKs are now UUID. Compiles clean. DB wipe required on next start.**

---

## What Was Done

### Summary
Migrated all Sea-ORM models from legacy `i64`/`i32` primary keys to `Uuid`. Updated all handlers, sync engine, DTOs, and route optimization to use UUID types throughout.

### Models Updated (14 files)

| Model | PK Change | FK Changes |
|-------|-----------|------------|
| `product.rs` | `i64` → `Uuid` | — |
| `location.rs` | `i64` → `Uuid` | `location_id: Option<Uuid>` |
| `quant.rs` | `i64` → `Uuid` | `product_id`, `location_id`, `lot_id`, `package_id` → `Uuid` |
| `picking.rs` | `i64` → `Uuid` | `location_id`, `location_dest_id`, `picking_type_id`, `partner_id` → `Uuid` |
| `move_line.rs` | `i64` → `Uuid` | `picking_id`, `product_id`, `location_id`, `location_dest_id`, `package_id`, `result_package_id`, `lot_id` → `Uuid` |
| `rack.rs` | `i64` → `Uuid` | `warehouse_id`, `mapped_location_id` → `Uuid` |
| `partner.rs` | `i64` → `Uuid` | `state_id`, `country_id` → `Uuid` |
| `delivery_carrier.rs` | `i64` → `Uuid` | — |
| `stock_picking_delivery.rs` | `i64` → `Uuid` | `picking_id`, `carrier_id` → `Uuid` |
| `delivery_tracking.rs` | `i64` → `Uuid` | `picking_delivery_id` → `Uuid` |
| `device_intake.rs` | `i32` → `Uuid` | `odoo_repair_id` kept as `i64` (external) |
| `checksum.rs` | `i64` → `Uuid` | — |
| `product_alias.rs` | `i32` → `Uuid` | — |
| `inventory_discrepancy.rs` | — (already Uuid) | `product_id`, `location_id` → `Uuid` |

### Handlers & Services Updated (6+ files)

| File | Changes |
|------|---------|
| `handlers/picking.rs` | All DTOs and Path params `i64` → `Uuid`, HashMap keys → `Uuid` |
| `handlers/delivery.rs` | Path params, `CreateShipmentReq`, `ensure_carrier` return type → `Uuid` |
| `handlers/repair.rs` | Stub product/quant creation uses `Uuid::new_v4()`, location_id → `Uuid` |
| `handlers/warehouse.rs` | `qty_map` HashMap key → `Uuid` |
| `handlers/mesh_sync.rs` | pull_handler: products/locations/shipments now use UUID parsing |
| `sync/engine.rs` | perform_push: products/locations/shipments use UUID parsing |
| `utils/route.rs` | `PickStop.rack_id` and `line_ids` → `Uuid`, tests updated |
| `utils/checksum.rs` | `upsert_checksum` inserts with `Uuid::new_v4()` |

### Database
- **Embedded PG data must be wiped** — `create_schema` will recreate all tables with UUID columns on next server start.
- No migration needed — dev environment, clean slate.

### Compilation
- `cargo check` — ✅ clean (0 errors, only pre-existing warnings)

---

**Agent**: Expert Developer (The Fixer)
**Status**: ✅ Complete

## 2026-03-09 — Agent Report

# Task Complete: Entity Checksums for Merkle Tree Sync (Stage A)

## Date: 2026-03-09

### Status
✅ **COMPLETE — Entity checksums are now recorded on every local mutation and incoming sync.**

---

## What Was Done

### Summary
Created `record_payload_checksums` in `src/utils/checksum.rs` and hooked it into all three sync paths: `push_to_all_peers`, `push_handler`, and `apply_pull_response`. Every entity that flows through the sync pipeline now has its SHA-256 checksum upserted into `entity_checksums`.

### Changes (4 files)

| File | Change |
|------|--------|
| `src/utils/checksum.rs` | Added `record_payload_checksums()` and `upsert_checksum()` — iterates all 10 entity arrays in PushPayload, computes hash via ChecksumCalculator, upserts into entity_checksums |
| `src/handlers/mesh_sync.rs` | Hooked checksums into `push_to_all_peers` (before peer iteration) and `push_handler` (before consuming payload) |
| `src/sync/engine.rs` | Hooked checksums into `apply_pull_response` (clones PullResponse into PushPayload before upserts) |
| `src/models/checksum.rs` | Fixed `id` type from `i32` to `i64` to match DB schema (`bigint`) |

### Design

- **Upsert strategy**: Select by `(entity_type, entity_id)` → update if hash changed, insert if new. Uses the existing `idx_entity_lookup` btree index.
- **Checksum calculation**: Reuses `ChecksumCalculator::compute_checksum()` which strips timestamp fields and produces deterministic SHA-256 hashes.
- **Non-blocking errors**: Checksum failures are logged as warnings but never block entity sync operations.
- **All 10 entity types covered**: product, location, shipment, user, order, document, file_resource, attachment, item, order_item_event.

### Note on entity_checksums.id type
The production DB uses `bigint` (auto-increment sequence). The Rust model previously had `i32` — fixed to `i64`. If UUID PKs are desired, that requires a DB migration (ALTER TABLE + drop sequence).

### Compilation
- `cargo check` — ✅ clean (no new warnings or errors)

---

**Agent**: Expert Developer (The Fixer)
**Status**: ✅ Complete

## 2026-03-09 — Agent Report

# Task Complete: Implement Twenty CRM Write-back API

## Date: 2026-03-09

### Status
✅ **COMPLETE — CRM entities can now be updated via `PUT /api/crm/:entity_type/:id`.**

---

## What Was Done

### Summary
Extended `TwentyClient` with PATCH methods and created a new CRM handler to expose a generic write-back endpoint for the PDA client.

### Changes (4 files)

| File | Change |
|------|--------|
| `src/services/twenty.rs` | Added `update_entity`, `update_company`, `update_person`, `update_opportunity` methods using PATCH |
| `src/handlers/crm.rs` | **New** — `update_entity` handler: validates entity type, delegates to TwentyClient |
| `src/handlers/mod.rs` | Added `pub mod crm` |
| `src/main.rs` | Added route `/crm/:entity_type/:id` (PUT) to protected API routes |

### API

```
PUT /api/crm/:entity_type/:id
Authorization: Bearer <jwt>
Content-Type: application/json

Body: { ...fields to update... }
```

Supported entity types: `company`, `person`, `opportunity`.

Returns the updated entity JSON from Twenty CRM, or appropriate error status codes:
- `400` — unsupported entity type
- `503` — Twenty CRM not configured
- `500` — upstream error

### Compilation
- `cargo check` — ✅ clean (no new warnings or errors)

---

**Agent**: Expert Developer (The Fixer)
**Status**: ✅ Complete

## 2026-03-08 — Agent Report

# Task Complete: Integrate V2 SmartTag Decryption into Global Scanner

## Date: 2026-03-09

### Status
✅ **COMPLETE — Encrypted QR codes are now transparently decrypted and routed through the existing scan pipeline.**

---

## What Was Done

### Summary
Modified `handle_scan` in `src/handlers/scan.rs` to intercept encrypted V2 SmartTag QR codes (e.g. `ECK1.COM/...`), decrypt them using `eck_binary_decrypt`, map the binary entity type to an internal routing prefix, and feed the result back into the existing resolution pipeline.

### Changes (1 file)

| File | Change |
|------|--------|
| `src/handlers/scan.rs` | Added SmartTag decryption interception block + entity type mapping |

### Details

1. **Imports added**: `eck_binary_decrypt`, entity type constants (`ENTITY_WMS_ITEM`, `ENTITY_WMS_BOX`, `ENTITY_WMS_LOCATION`, `ENTITY_TWENTY_COMPANY`, `ENTITY_TWENTY_PERSON`, `ENTITY_TWENTY_OPPORTUNITY`), `warn` from tracing.

2. **Decryption interception** (before `try_twenty_lookup`): If the barcode starts with any configured `qr_prefixes` (e.g. `ECK1.COM/`), it calls `eck_binary_decrypt` with the app config's prefixes, tenant suffix, and `ENC_KEY` from env. On failure, returns an immediate error response.

3. **Entity type mapping**: On successful decryption, the `SmartTag.entity_type` byte is mapped to internal routing strings:
   - `0x00` (WMS_ITEM) → `i-{uuid}`
   - `0x01` (WMS_BOX) → `b-{uuid}`
   - `0x02` (WMS_LOCATION) → `p-{uuid}`
   - `0x10` (TWENTY_COMPANY) → `company-{uuid}`
   - `0x11` (TWENTY_PERSON) → `person-{uuid}`
   - `0x12` (TWENTY_OPPORTUNITY) → `opp-{uuid}`
   - Fallback → `unknown-{uuid}`

4. **Seamless routing**: By overwriting the `barcode` variable with the mapped string (e.g. `company-UUID`), the rest of `handle_scan` (Twenty CRM lookup, V2 UUID parser, legacy search) picks it up automatically — zero structural changes needed downstream.

### Compilation
- `cargo check` — ✅ clean (no new warnings or errors)

---

**Agent**: Expert Developer (The Fixer)
**Status**: ✅ Complete

## 2026-03-08 — Agent Report

# Task Complete: Dynamic Repair Schemas & Excel Sync

## Date: 2026-03-08

### Status
✅ **COMPLETE — Excel bidirectional sync + dynamic metadata renderer on Repair Detail page**

---

## What Was Done

### 1. Excel Sync Backend (`scraper/server.js`)
- Added `exceljs` dependency for reading/writing `.xlsm` files (preserves VBA macros)
- **`GET /api/excel/info`** — file metadata: path, size, repair count, last repair number
- **`POST /api/excel/read`** — reads repairs from "Körperanalyse" sheet, returns JSON (newest first, with limit/offset)
- **`POST /api/excel/write-row`** — adds or updates a row by repair number (with automatic `.bak` backup)
- Handles Excel formula cells (shared formulas), hyperlink cells, rich text, Date objects
- Column mapping (`EXCEL_COL` object) easily replaceable for other Excel files
- 30+ mapped fields: ticket#, repair#, warranty, error description, troubleshooting, defective parts (6 slots), firmware before/after, model, serial, customer, dates, completion status

### 2. Excel Sync UI (`web/src/routes/dashboard/scrapers/+page.svelte`)
- New "Excel Reparaturliste" section (orange themed) on Scrapers page
- **Info button** — shows repair count, last repair number, file modification date
- **Import tab** (Excel → DB): Read Excel → table with checkboxes → Import selected to orders table
- **Export tab** (DB → Excel): Load repairs from DB → select → Write to Excel file
- All operations manual, with preview, nothing automatic
- Import creates `orders` records with `type=repair`, maps all Excel fields to order fields + metadata JSONB

### 3. Dynamic Repair Schemas (`web/src/routes/dashboard/repairs/[id]/+page.svelte`)
- **Replaced Parts**: tag-based editor bound to `partsUsed` JSON array (add/remove with Enter key support)
- **Dynamic Attributes**: renders all `metadata` JSONB fields as editable form inputs
  - Nested objects (e.g. `fwBefore: {kernel, digital, analog}`) → grouped sub-fields with header
  - Boolean values → checkboxes
  - Strings/numbers → text inputs
  - System keys (`ticketId`, `trackingNumber`, `importedFromExcel`, `excelRow`) hidden from display
- **Add Custom Field**: key/value input with type inference (true/false → boolean, numbers → number)
- `formatKey()` converts camelCase to Title Case for display

### 4. Config
- `.env`: Added `EXCEL_REPAIR_FILE` path (relative to project root)
- `scraper/package.json`: Added `exceljs` dependency

---

## Files Changed

| File | Change |
|------|--------|
| `scraper/server.js` | +ExcelJS import, +EXCEL_COL mapping, +3 endpoints (info/read/write-row), +cellVal with formula/hyperlink/richtext handling |
| `scraper/package.json` | +`exceljs` dependency |
| `web/src/routes/dashboard/scrapers/+page.svelte` | +Excel Sync section (state, functions, UI, CSS) |
| `web/src/routes/dashboard/repairs/[id]/+page.svelte` | Full rewrite: +partsUsed tags, +dynamic metadata grid, +custom field adder |
| `.env` | +`EXCEL_REPAIR_FILE` |
| `.eck/JOURNAL.md` | +2026-03-08 entry |

## Build
- `npm run build` — **OK** (SvelteKit static adapter)
- Excel endpoints tested: info returns 1224 repairs, read returns correct data with proper cell parsing

---

**Agent**: Expert Developer (The Fixer)
**Status**: ✅ Complete

## 2026-03-08 — feat(ui): Dynamic Repair Schemas & Excel Sync

- **Excel Sync** (Scrapers page): New "Excel Reparaturliste" section with Import (Excel→DB) and Export (DB→Excel) tabs
  - Backend: `GET /api/excel/info`, `POST /api/excel/read`, `POST /api/excel/write-row` in scraper/server.js (exceljs)
  - Handles formulas, hyperlinks, rich text, dates from .xlsm files
  - Mapping for 30+ columns from "Körperanalyse" sheet — easily replaceable for other Excel files
  - Backup `.bak.YYYY-MM-DD` created before each write
- **Dynamic Repair Schemas** (Repair Detail page): JSONB metadata rendered as editable dynamic form fields
  - Nested objects (fwBefore, fwAfter) rendered as grouped sub-fields
  - Boolean fields as checkboxes, strings as text inputs
  - "Add Custom Field" for arbitrary metadata
  - "Replaced Parts" tag editor bound to partsUsed JSON array
  - System keys (ticketId, trackingNumber) hidden from dynamic grid

## 2026-03-04 — feat(architecture): Murmur3 CAS, Binary SmartTags & Twenty CRM
- **Infrastructure**: Fixed Nginx 413 payload limit for `pda.repair` (set to 50M).
- **CAS Pipeline**: Migrated file storage to MurmurHash3 (x64_128) deterministic UUIDs. Uploads are now strictly idempotent.
- **Crypto Engine**: Implemented V2 Binary SmartTags (19 bytes: 16b UUID + 1b Type + 2b Flags). Added support for dynamic IV length and multi-domain `QR_PREFIXES` via `.env`. Output is locked to exactly 56 chars of Base32 data + dynamic IV.
- **CRM Integration**: Added `TwentyClient` to fetch entities from Twenty CRM REST API. Updated `handle_scan` to route `company-` and `person-` UUIDs to Twenty CRM.

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
