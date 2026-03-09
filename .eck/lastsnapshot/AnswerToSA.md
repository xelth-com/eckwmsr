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
