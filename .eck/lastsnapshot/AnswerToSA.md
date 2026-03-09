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
