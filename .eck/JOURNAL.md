# Development Journal

## 2026-02-15 — Session: Frontend + Sync + Setup Account
- type: fix+feat
- scope: frontend, sync, auth
- **Fixes**: DB type mismatches (numeric→float8, timestamp→timestamptz), removed non-existent columns from picking/move_line models, added missing endpoints (mesh/nodes, odoo/pickings), error logging
- **Features**: Merkle Tree sync, Conflict Resolver (VC→Priority→LWW), auto-created setup account with random password on first run
- **Server**: RUST_LOG=info, column type conversions, user d.suro@inbody.com
- **Verified**: Dashboard ✅, Inventory ✅, Warehouse ✅, Shipping ✅

## 2026-02-15 — Phase 10: SPA Static File Server
- type: feat
- scope: frontend
- Updated `web.rs` with `/E/` prefix stripping, MIME resolution, immutable caching for hashed assets, SPA fallback
- Created `IMPLEMENTATION_SUMMARY.md` documenting full architecture
- Updated all `.eck/` stub files with real project content

## 2026-02-15 — Phase 9.3: Workflows and Endpoints
- type: feat
- scope: workflows
- Added `RepairService` for intake_save processing and ProductAlias linking
- Added `/api/repair/event` handler with inventory reconciliation (PDA as source of truth)
- Added `/api/inventory/discrepancies` endpoints with stats aggregation
- Added `/E/rma` CRUD endpoints for unified orders table

## 2026-02-15 — Phase 9.2: Sea-ORM Models
- type: feat
- scope: models
- Added `Order` model (RMA + repair via `order_type` field)
- Added `DeviceIntake` for Android PDA intake flow
- Added `InventoryDiscrepancy` for QC count mismatches
- Added `Document` for generic workflow logs

## 2026-02-15 — Phase 9.1: PDF Label Generation
- type: feat
- scope: print
- Added `printpdf`, `qrcode`, `crc32fast` dependencies
- Ported AES-192-GCM encryption with custom Base32 encoding from Go
- Ported label generator with QR puzzle layout (3 QR codes + checksum + serial)
- Created `POST /api/print/labels` endpoint

## 2026-02-14 — Phases 1-8
- type: feat
- scope: project
- Initial project scaffold through delivery provider integration
- Core: auth, warehouse, scan, sync, AI, file store, picking, delivery
