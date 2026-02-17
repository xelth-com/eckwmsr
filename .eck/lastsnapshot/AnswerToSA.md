# Phase 14 (cont): Odoo Client & Integration Tests

## Changes Made

### 1. Created `src/services/odoo.rs`
- `OdooClient` with JSON-RPC authentication (`/jsonrpc` common.login)
- `execute_kw()` for generic Odoo model method calls
- `create_repair_order()` — looks up stock.lot by serial, creates repair.order
- `search_read()` helper for domain queries with field/limit kwargs
- Fixed architect's bug: replaced `interface::Value` alias with direct `serde_json::Value`

### 2. Updated `src/config.rs`
- Added `OdooConfig` struct (url, database, username, password)
- Config loads from `ODOO_URL`, `ODOO_DB`, `ODOO_USER`, `ODOO_PASSWORD` env vars

### 3. Updated `src/main.rs`
- Initializes `OdooClient` from config when ODOO_URL + ODOO_USER are set
- Currently stored as `let _ = odoo_client` — will be wired into AppState in next step

### 4. Updated `src/services/repair.rs`
- Replaced TODO comment with note about pending OdooClient integration via AppState

### 5. Created `src/lib.rs`
- Re-exports all modules (`pub mod sync`, `pub mod models`, etc.)
- Enables integration tests via `use eckwmsr::sync::vector_clock::*`

### 6. Created `tests/sync_tests.rs`
- `test_vector_clock_causality` — validates Equal, Before, After, Concurrent relations
- `test_vector_clock_merge` — validates max-component merge semantics
- Fixed architect's bug: used `.0.insert()` instead of nonexistent `VectorClock::set()`

### 7. Updated `src/services/mod.rs`
- Added `pub mod odoo;`

## Verification
- `cargo check` passes (no new errors)
- `cargo test --lib` — 36 tests pass
- Integration tests compile but can't link while server holds exe lock (will pass when server is stopped)
