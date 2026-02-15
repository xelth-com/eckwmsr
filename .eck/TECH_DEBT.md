# Tech Debt

## Server-side manual changes (NOT in git — applied to pda.repair DB directly)
- All `numeric` columns → `double precision` (23 columns)
- All `timestamp without time zone` → `timestamptz` (3 columns)
- `RUST_LOG=info` added to `/var/www/eckwmsr/.env`
- User `d.suro@inbody.com` created in `user_auths`

## Active TODOs in Code

1. **`src/handlers/ai.rs:114`** — `TODO: Use ToolService to link alias in DB (Phase 6.3)`
   AI barcode resolution identifies unknown barcodes but doesn't persist the ProductAlias yet.

2. **`src/services/repair.rs:72`** — `TODO: Sync to Odoo repair.order`
   DeviceIntake processing saves locally but doesn't create the corresponding repair.order in Odoo via XML-RPC.

## Structural Issues

3. **No database migrations** — Sea-ORM models assume tables already exist (created by Go/GORM auto-migrate). Should add `sea-orm-migration` for standalone deployment.

4. **No tests** — No unit or integration tests exist yet. Priority areas:
   - Encryption round-trip (encrypt → decrypt)
   - Inventory reconciliation logic (process_inventory_count)
   - PDF label generation (basic sanity check)
   - JWT auth middleware

5. **printpdf/image version conflict** — `printpdf` 0.7 depends on `image` 0.24, while `qrcode` 0.14 depends on `image` 0.25. Workaround: QR codes are rendered to raw grayscale pixel buffers and fed to `printpdf::ImageXObject` directly, bypassing the `image` crate entirely. If printpdf upgrades to image 0.25+, this can be simplified.

6. **Delivery scraping fragility** — DHL/OPAL integration uses `thirtyfour` WebDriver scraping which is inherently brittle. Portal UI changes will break the selectors.

7. **In-memory filtering for repair events** — `list_repair_events` fetches all `repair_log` documents and filters in Rust memory. Should use PostgreSQL JSONB containment (`@>`) operator via Sea-ORM raw query for efficiency at scale.

8. **Dead code warnings** — ~50 warnings from pre-existing unused code (smart_code functions, sync utilities, delivery provider fields). Not harmful but should be cleaned up.

9. **DeliveryService not in AppState** — `delivery_service` is created in `main()` but dropped immediately (comment says "Phase 8.3"). The delivery handlers likely work independently but the service should be wired into AppState for proper lifecycle management.
