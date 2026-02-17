# Phase 13.1: Mesh WebSocket Infrastructure

## What was done
Implemented server-to-server WebSocket signaling for live mesh sync.

## Changes

### New: `src/handlers/mesh_ws.rs`
- **`MeshHub`** — manages peer connections via `broadcast::channel`:
  - `broadcast(signal)` — send to all connected peers
  - `notify_update(sender, entity_type, entity_id)` — convenience for entity change signals
  - `register/unregister` — track connected peers
  - `subscribe()` — get a receiver for hub signals
  - `peer_count()` — monitoring
- **`MeshSignal`** — JSON message protocol:
  - `type`: "HELLO", "UPDATE", "PING"
  - `senderId`: originating instance
  - `entityType` / `entityId`: optional, for UPDATE signals
- **`mesh_ws_handler`** — Axum WebSocket handler at `/E/mesh/ws?instance_id=...`:
  - Accepts connection, sends HELLO
  - Bidirectional: forwards hub broadcasts to peer, reads incoming signals from peer
  - Re-broadcasts received UPDATE signals to other peers
  - Properly splits socket with `futures_util::{SinkExt, StreamExt}`
  - Uses `tokio::select!` for clean shutdown

### Updated: `src/db.rs`
- Added `mesh_hub: MeshHub` to `AppState`

### Updated: `src/handlers/mod.rs`
- Added `pub mod mesh_ws;`

### Updated: `src/main.rs`
- Initialized `MeshHub::new()`
- Added to `AppState` construction
- Registered `/ws` route under `/E/mesh` nest (public, no JWT)

## Endpoint
`GET /E/mesh/ws?instance_id=<peer_instance_id>` — WebSocket upgrade

## Signal Protocol
```json
{"type":"HELLO","senderId":"rust_dev_node"}
{"type":"UPDATE","senderId":"rust_dev_node","entityType":"product","entityId":"123"}
{"type":"PING","senderId":"rust_dev_node"}
```

## Verification
- `cargo check` — zero errors


[SYSTEM: EMBEDDED]
