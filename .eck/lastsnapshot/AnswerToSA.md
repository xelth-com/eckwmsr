# Phase 13.2: Merkle Tree Sync Logic & API

## What was done
Implemented Merkle tree-based sync API for efficient O(log n) data difference detection between mesh servers.

## Changes

### Updated: `src/sync/merkle_tree.rs`
- Added `MerkleRequest` struct (entity_type, level, bucket)
- Added `MerkleTreeService` — builds Merkle trees from `entity_checksums` table via SeaORM
  - `get_root()` — Level 0: fetches all checksums, groups by bucket, returns bucket hashes
  - `get_bucket()` — Level 1: returns individual entity_id -> hash pairs for a specific bucket
- Added Serialize/Deserialize derives to `MerkleNode` for JSON API

### New: `src/handlers/mesh_sync.rs`
- `POST /mesh/merkle` — Returns Merkle tree state for comparison (level 0 = root, level 1 = bucket)
- `POST /mesh/pull` — Fetch specific entities by ID (product, location, shipment)
- `POST /mesh/push` — Apply incoming entities with upsert (ON CONFLICT UPDATE)

### Updated: `src/handlers/mod.rs`
- Added `pub mod mesh_sync;`

### Updated: `src/main.rs`
- Registered `/merkle`, `/pull`, `/push` routes under `/E/mesh` nest

## Sync Protocol
1. Server A calls `POST /mesh/merkle` on Server B with `{entity_type: "product", level: 0}`
2. Compare root hashes — if equal, in sync. If not, compare bucket hashes.
3. For mismatched buckets, call level 1 to get individual entity hashes.
4. Call `/mesh/pull` with IDs of entities that differ.
5. Call `/mesh/push` to send entities the other side is missing.

## Verification
- `cargo check` — zero errors
- `cargo test` — 36 tests pass (including existing Merkle tree tests)
