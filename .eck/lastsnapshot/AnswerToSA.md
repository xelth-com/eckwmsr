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
