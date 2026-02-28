# Session: Relay Pairing Fix + Three-State Health Check (2026-02-28)

## What was done

### 1. Fixed relay channel mismatch in pairing (commit d4958fe)
- **Bug**: `push_packet()` used server's real `mesh_id` but `pull_packets_for()` used `routing_id` as mesh_id — packets never found
- **Fix**: Added `push_packet_to_channel()` to `RelayClient` — uses `channel_id` as both mesh_id and target_id
- Updated all 3 push calls in `PairingService` (offer, response, approval) to use the new method
- **Verified**: Full pairing flow tested via Chrome — production created code 927-222, local entered it, approval modal appeared on production, both sides saved each other

### 2. Three-state peer health check (commit 70218f8)
- **Was**: Binary online/offline, 60s interval, skipped peers without `base_url` (NAT peers always red)
- **Now**: Three statuses with smart fallback:
  - `active` (green) — direct HTTP ping OK, or relay says online for NAT peers
  - `degraded` (yellow) — direct ping just failed, waiting 5 min grace period before asking relay
  - `offline` (red) — both direct ping and relay (9eck.com) confirm down
- Health check interval: 30s (was 60s)
- Relay is only queried when needed (NAT peers or degraded > 5 min) — no unnecessary relay traffic
- NAT peers (no `base_url`): checked via relay `GET /E/mesh/{mesh_id}/status`

### 3. Frontend three-color status
- Devices page table: green dot "Online" / yellow dot "Unstable" / red dot "Offline"
- Sidebar `MeshStatus.svelte`: green/yellow/red node backgrounds and status dots
- API `GET /mesh/nodes` now returns `status` field ("active"/"degraded"/"offline") alongside `is_online` boolean

## Verified behavior
- Local → pda.repair: `status: "active"` (direct HTTP ping OK)
- Production → f4356a29 (NAT, no base_url): `status: "active"` (relay confirms online)
- Both servers deployed and running

## Key files changed
| File | Summary |
|------|---------|
| `src/sync/relay_client.rs` | `push_packet_to_channel()` — pairing-specific push with channel_id routing |
| `src/services/pairing.rs` | All 3 push calls switched to `push_packet_to_channel()` |
| `src/main.rs` | Rewritten health check: 30s interval, three-state logic, relay fallback |
| `src/handlers/mesh.rs` | `NodeInfo.status` field, removed time-based `is_online` calc |
| `src/models/mesh_node.rs` | Comment update: status = "active"/"degraded"/"offline" |
| `web/src/lib/components/MeshStatus.svelte` | Yellow degraded styling in sidebar |
| `web/src/routes/dashboard/devices/+page.svelte` | Yellow "Unstable" in table + CSS |


[SYSTEM: EMBEDDED]
