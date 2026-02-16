# Admin User Management API — Implementation Report

## What was done
Implemented the missing Admin User CRUD API that the dashboard frontend (`/dashboard/users`) expects.

## Changes

### New file: `src/handlers/admin_users.rs`
- **GET /api/admin/users** — Lists all non-deleted users, returns `SafeUser` DTO (never exposes password/pin hashes)
- **POST /api/admin/users** — Creates user with bcrypt-hashed password, validates required fields, returns 201
- **PUT /api/admin/users/:id** — Partial update (name, role, email, pin, isActive, password), skips empty strings
- **DELETE /api/admin/users/:id** — Soft delete via `deleted_at` timestamp (matches Go behavior)

### Modified: `src/handlers/mod.rs`
- Added `pub mod admin_users;`

### Modified: `src/main.rs`
- Registered routes under protected API group:
  - `/admin/users` (GET + POST)
  - `/admin/users/:id` (PUT + DELETE)

## Key decisions
- Soft delete matching Go version (sets `deleted_at` instead of row removal)
- All queries filter `deleted_at IS NULL` to hide soft-deleted users
- `SafeUser` DTO includes `hasPin` (computed), `preferredLanguage` to match Go response shape
- Create returns HTTP 201 (not 200) matching Go behavior
- Duplicate key errors return 409 Conflict

## Verification
- `cargo check` passes with zero errors
- `cargo build` compiles successfully (binary write blocked by running process — not a code issue)
