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
