# Roadmap

## Completed Phases

### Phase 1-4: Core Foundation
- [x] Project scaffold (Cargo.toml, config, db connection)
- [x] Sea-ORM models for Odoo entities (product, location, quant, picking, move_line, rack, partner)
- [x] JWT auth middleware and login handler
- [x] Warehouse/items listing, barcode scan handler

### Phase 5: Sync Engine
- [x] Vector clocks, security layer (AES-256-GCM)
- [x] Relay client with encrypted sync packets
- [x] Sync trigger and push handlers

### Phase 6: AI & Vision
- [x] Gemini API integration with primary/fallback model routing
- [x] Image analysis endpoint (box condition assessment, OCR)
- [x] AI chat response handler

### Phase 7: File Store & Attachments
- [x] Content-addressable file store (CAS)
- [x] Image upload handler
- [x] Entity attachment listing

### Phase 8: Delivery Providers
- [x] DHL and OPAL Kurier WebDriver scraping services
- [x] Shipment CRUD, carrier listing, delivery sync history
- [x] Sea-ORM models: delivery_carrier, stock_picking_delivery, delivery_tracking, sync_history

### Phase 9: RMA, Repair & Print
- [x] 9.1: PDF label generation (printpdf + qrcode)
- [x] 9.2: Sea-ORM models (order, device_intake, inventory_discrepancy, document)
- [x] 9.3: Repair/intake workflow, inventory reconciliation, RMA CRUD

### Phase 10: SPA Frontend Serving
- [x] rust-embed static file server with /E/ prefix stripping
- [x] SvelteKit asset caching (immutable for hashed files)
- [x] SPA fallback routing to index.html
- [x] Implementation summary documentation

### Phase 11: Production Hardening (2026-02-15)
- [x] Fix DB model↔table mismatches (numeric, timestamp, missing columns)
- [x] Add missing frontend endpoints (mesh/nodes, mesh/status, odoo/pickings)
- [x] Error logging in all handlers
- [x] Setup account with random password on first run (/auth/setup-status)
- [x] Merkle Tree for efficient sync diffing
- [x] Conflict Resolver (Vector Clock → Source Priority → LWW)

## Future Work

### Phase 12: Server Pairing System (NEXT PRIORITY)
- [ ] **Pairing Code Generation**: Server A generates one-time code (e.g. `ECK-A7F3-9X2K`), displayed on frontend Settings page
- [ ] **Pairing Request**: Server B enters code → sends pairing request via Blind Relay (or direct URL if known)
- [ ] **Mutual Consent**: Server A shows popup "Server B wants to connect — Accept/Deny?". Both sides must approve.
- [ ] **Key Exchange**: On mutual approval, servers exchange sync encryption keys and instance_ids
- [ ] **Direct Connect Fallback**: If Blind Relay is down, user can enter direct URL (`https://server-b.example.com/E/`) + pairing code to connect peer-to-peer
- [ ] **Mesh Registry**: Replace stub in `handlers/mesh.rs` with real in-memory registry of paired nodes
- [ ] **Security**: Pairing codes expire after 5 minutes. Failed attempts are rate-limited. No unsolicited connection attempts visible to users.

### Phase 13: Live Mesh Sync
- [ ] WebSocket endpoint `/E/mesh/ws` for real-time bidirectional sync
- [ ] Integrate Merkle Tree into sync engine pull/push cycle
- [ ] Integrate Conflict Resolver into entity apply logic
- [ ] Mesh node heartbeat and online/offline detection
- [ ] Frontend "Connected Servers" sidebar shows actual paired nodes with status

### Phase 14: Polish
- [ ] Database migrations (Sea-ORM Migrator) for standalone deployment
- [ ] Odoo XML-RPC sync for repair.order creation
- [ ] AI tool service for alias linking (Phase 6.3 TODO)
- [ ] Unit and integration tests
- [ ] Production Docker image
- [ ] Remove setup account automatically when first real user is created via frontend
