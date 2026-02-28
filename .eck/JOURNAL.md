# Development Journal

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
