# Phase 13.3: Active Mesh Synchronization

## What was done
Implemented the full active sync pipeline: MeshClient for HTTP peer communication, SyncEngine.sync_with_peer for Merkle-based diff+pull+push, and WebSocket-triggered sync on UPDATE signals.

## Changes

### New: `src/sync/mesh_client.rs`
- `MeshClient` — HTTP client for talking to peer mesh nodes:
  - `get_merkle_root(entity_type)` → POST /E/mesh/merkle level 0
  - `get_merkle_bucket(entity_type, bucket)` → POST /E/mesh/merkle level 1
  - `pull_entities(entity_type, ids)` → POST /E/mesh/pull
  - `push_entities(products, locations, shipments)` → POST /E/mesh/push

### Updated: `src/sync/engine.rs`
- `sync_with_peer(peer_url, entity_type)` — full orchestration:
  1. Fetch local + remote Merkle roots
  2. If roots match → no sync needed (O(1) check)
  3. Compare bucket hashes → find differing buckets
  4. Drill into differing buckets → find specific entity IDs
  5. Pull missing/changed entities from peer
  6. Push local-only entities to peer
- `apply_pull_response()` — upserts received products/locations/shipments
- `perform_push()` — queries local DB and pushes to peer

### Updated: `src/sync/merkle_tree.rs`
- `compare_trees` now takes `BTreeMap, BTreeMap` (both sides same type)
- Handles bidirectional diff: items present on both sides with different hashes go into both pull and push lists
- `MerkleRequest` now derives `Serialize` for client usage

### Updated: `src/handlers/mesh_ws.rs`
- UPDATE signal handler now triggers `sync_with_peer` via `tokio::spawn`
- Looks up peer's `base_url` from `mesh_nodes` table
- Spawns async task so WebSocket handler isn't blocked

### Updated: `src/handlers/mesh_sync.rs`
- `PullRequest`, `PullResponse`, `PushPayload` now derive both Serialize + Deserialize

### Updated: `src/sync/mod.rs`
- Added `pub mod mesh_client;`

## Data Flow
```
Peer A changes product → MeshHub.notify_update("product", "123")
  → WebSocket sends UPDATE to Peer B
  → Peer B receives UPDATE in mesh_ws handler
  → Spawns sync_with_peer(peer_a_url, "product")
  → Merkle root comparison → bucket diff → entity diff
  → Pull changed products from A, Push local-only to A
```

## Verification
- `cargo check` — zero errors
- `cargo test` — 36 tests pass
