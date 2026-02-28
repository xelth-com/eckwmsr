# Session Handover — Direct-First Sync + UUID Migration + Device Pairing

## What was done this session

### 1. Direct-First, Relay-as-Fallback Sync Delivery
- `push_user_to_peers()` in `src/handlers/admin_users.rs` — 3-tier: Direct HTTP → WS signal → Relay
- `is_peer_connected()` added to `MeshHub` in `src/handlers/mesh_ws.rs`
- `is_url_directly_reachable()` helper — checks if URL is not localhost/loopback

### 2. Fixed MeshClient double /E prefix
- `src/sync/mesh_client.rs` — URLs were `{base_url}/E/mesh/*` but `base_url` already contains `/E`
- Changed to `{base_url}/mesh/*`

### 3. Startup sync + full-pull for users
- `src/sync/engine.rs` — `full_pull_from_peer()` bypasses merkle tree (empty ids = return all)
- `src/handlers/mesh_sync.rs` — `pull_handler` now returns all entities when `ids` is empty
- `src/handlers/sync.rs` — `POST /api/sync/peers` endpoint for manual mesh sync
- `src/main.rs` — startup task pulls users from all known peers on boot

### 4. Device status endpoint
- `src/handlers/device.rs` — `GET /api/status` for PDA heartbeat (JWT protected)
- Returns device status, updates `last_seen_at`, includes `enc_key` for active devices

### 5. UUID instance_id auto-generation
- `src/config.rs` — `ensure_uuid_instance_id()` generates UUID v4 and **writes it back to .env**
- `src/utils/identity.rs` — updates `server_identity.json` when instance_id changes
- Old string IDs (`rust_dev_local`, `production_pda_repair`) replaced with real UUIDs

## Current state

### Servers
- **Local** (`localhost:3210`): running, UUID `f4356a29-1935-4d13-bdb9-39571508e8a8`
- **Production** (`pda.repair`): running, UUID `e3f6a705-751b-4846-bed3-a2d399290867`
- Both deployed with latest code

### Device
- **Ranger2** (`7079f94ca9373a43`) registered on local server as `active`
- Home instance = local UUID
- PDA connected but user reports it doesn't show as connected on server side

### Database (local)
- `mesh_nodes`: has old entry `production_pda_repair` (string ID, not UUID) — needs cleanup
- `registered_devices`: Ranger2 is active, last_seen 04:24
- Embedded PG on port 5433: running (started manually via `pg_ctl`)
- `.env` has `DATABASE_URL=postgres://eckwms@localhost:5433/eckwms`

### Database (production)
- `mesh_nodes`: empty (cleared during migration)
- `registered_devices`: empty
- User `dmytro` exists in `user_auths`

## Known issues to investigate
1. **PDA says connected but server doesn't show it** — device IS in DB as active. Problem may be in dashboard UI not showing devices, or WebSocket `/ws` connection not establishing
2. **Old mesh_node entry** on local still has `production_pda_repair` (string) — should be updated to UUID or deleted
3. **Embedded PG zombie processes** on Windows — 8 postgres.exe processes from old sessions can't be killed from current session. Started PG manually with `pg_ctl`. May need Windows restart to clean up.
4. **TECH_DEBT items 10-13** in `.eck/TECH_DEBT.md` — merkle checksums for users, outbound WS client, periodic sync, base_url coupling

## Key files changed
| File | Summary |
|------|---------|
| `src/config.rs` | UUID auto-generation + .env persistence |
| `src/utils/identity.rs` | Auto-update identity when instance_id changes |
| `src/handlers/admin_users.rs` | 3-tier push (direct → WS → relay) |
| `src/handlers/mesh_ws.rs` | `is_peer_connected()` |
| `src/handlers/mesh_sync.rs` | Empty ids = return all entities |
| `src/handlers/sync.rs` | `POST /api/sync/peers` |
| `src/handlers/device.rs` | `GET /api/status` |
| `src/sync/engine.rs` | `full_pull_from_peer()` |
| `src/sync/mesh_client.rs` | Fixed double /E prefix |
| `src/main.rs` | Startup sync task, `/api/sync/peers` route, `/api/status` route |
