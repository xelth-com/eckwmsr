# Direct-First, Relay-as-Fallback Sync Delivery

## What was done

Implemented 3-tier delivery for `push_user_to_peers()`:

1. **Direct HTTP** — if peer has a reachable `base_url` (not localhost/loopback), push via `MeshClient.push_entities()` to `POST /E/mesh/push`
2. **WebSocket signal** — if peer is connected to our WS hub, send UPDATE signal so they pull from us
3. **Relay fallback** — encrypted packet via 9eck.com, only used when direct and WS paths fail

## Files modified

| File | Change |
|------|--------|
| `src/handlers/mesh_ws.rs` | Added `is_peer_connected()` method to `MeshHub` |
| `src/handlers/admin_users.rs` | Added `is_url_directly_reachable()` helper, rewrote `push_user_to_peers()` with 3-tier delivery |

## Delivery matrix

| From → To | Path |
|-----------|------|
| Any → peer with public URL | Direct HTTP first |
| Any → peer on WS (no public URL) | WebSocket UPDATE signal |
| Any → unreachable peer | Relay via 9eck.com |

## Build

Compiles successfully with `cargo build`.


[SYSTEM: EMBEDDED]
