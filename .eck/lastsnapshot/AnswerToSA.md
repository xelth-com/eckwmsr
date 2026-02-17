# Connectivity Hub — Unified Scanners + Servers UI

## What was done
Integrated Server Pairing into the Dashboard's Devices page with a tabbed interface (Scanners / Mesh Servers), added config persistence API and mesh node deletion.

## Backend Changes

### New: `src/handlers/config.rs`
- `POST /api/admin/config/save-key` — validates 64-hex-char key and writes `SYNC_NETWORK_KEY` to `.env` file (creates or updates)

### Updated: `src/handlers/mesh.rs`
- `DELETE /api/admin/mesh/:id` — deletes a mesh node by instance_id

### Updated: `src/main.rs`
- Registered `/admin/config/save-key` and `/admin/mesh/:id` routes under protected API

### Updated: `src/handlers/mod.rs`
- Added `pub mod config;`

## Frontend Changes

### Replaced: `web/src/routes/dashboard/devices/+page.svelte`
- **Tabbed layout**: "Scanners (PDAs)" and "Mesh Servers" tabs
- **Scanners tab**: QR pairing, device list with status/approve/block/delete, home node selector
- **Servers tab**:
  - "Invite Server" button → generates code, polls for peer, approve modal
  - "Join Network" input → enter code, connect, poll for approval, save network key
  - Server list with role badges (master/peer), online status, unpair button
- Pairing modal with full Host/Client workflow

## Verification
- `cargo check` — zero errors
- `npm run build` — frontend compiles successfully
