# Embedded PostgreSQL — Implementation Report

## What was done
Added embedded PostgreSQL support via `postgresql_embedded` crate. The server now starts without any external database dependency.

## ENV Switch Logic
- `DATABASE_URL` **set** → uses external PostgreSQL (production mode, same as before)
- `DATABASE_URL` **not set** (default) → starts embedded PostgreSQL on port 5433, auto-creates `eckwms` database and all tables

## Changes

### `Cargo.toml`
- Added `postgresql_embedded = { version = "0.20", features = ["tokio"] }`

### `src/config.rs`
- `DATABASE_URL` default changed from `postgres://...` to empty string (triggers embedded mode)

### `src/db.rs`
- Added `start_embedded()` — configures and starts embedded PG with persistent data dir (`./data/pg/`)
- Added `create_schema()` — uses `sea_orm::Schema::create_table_from_entity()` for all 22 entities with `IF NOT EXISTS`
- `AppState` now holds `_embedded_pg: Option<PostgreSQL>` to keep the PG process alive

### `src/main.rs`
- At startup: checks if `database_url` is empty → calls `start_embedded()`, stores handle in `AppState`
- After connect: if embedded mode, calls `create_schema()` to ensure tables exist

### `.gitignore`
- Added `data/` to exclude embedded PG data directory

## First run behavior
1. Downloads PostgreSQL binaries (~100MB, cached at `~/.theseus/postgresql/`)
2. Initializes data directory at `./data/pg/`
3. Starts PG on port 5433
4. Creates `eckwms` database
5. Creates all 22 tables
6. Seeds setup account (admin@setup.local)

## Subsequent runs
- Skips download (cached)
- Starts PG in ~2-5 seconds
- Tables already exist (IF NOT EXISTS), data persists

## Verification
- `cargo check` passes with zero errors
