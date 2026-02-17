# Phase 14 Final: Odoo Client Wired into AppState

## Changes Made

### 1. `src/db.rs` — Added `odoo_client` to AppState
- New field: `pub odoo_client: Option<tokio::sync::Mutex<OdooClient>>`
- Wrapped in `Mutex` because `authenticate()` takes `&mut self` (sets UID)
- `None` when Odoo env vars not configured — zero overhead

### 2. `src/main.rs` — Passes OdooClient into AppState
- Replaced `let _ = odoo_client` with `odoo_mutex` wrapping into `tokio::sync::Mutex`
- Added `odoo_client: odoo_mutex` to AppState construction

### 3. `src/services/repair.rs` — Calls Odoo on intake
- After saving `DeviceIntake`, checks `state.odoo_client`
- If present: authenticates, calls `create_repair_order()`
- If absent: logs "Not configured, skipping" (no-op)
- Errors are logged but don't block the intake flow

## Flow
```
Android PDA -> intake_save event -> handlers/repair.rs
  -> RepairService::process_intake()
    -> Save DeviceIntake to local DB
    -> Create ProductAlias (HWB <-> Serial)
    -> If Odoo configured:
         Lock mutex -> authenticate -> create_repair_order()
    -> If not: log and skip
```

## Verification
- `cargo check` — passes, no new warnings
- `cargo test --lib` — 36 tests pass


[SYSTEM: EMBEDDED]
