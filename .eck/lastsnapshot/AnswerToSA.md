# Task Complete: Murmur3 CAS Verification + Idempotency (Rust Server)

## Date: 2026-03-03

### Status
✅ **COMPLETE — Server verifies content hash, deduplicates by CAS UUID**

---

## What Was Done

### Dependency
- Added `murmur3 = "0.5"` to Cargo.toml

### FileStore (`services/filestore.rs`)
- **`content_hash_uuid(data: &[u8]) -> Uuid`**: Computes deterministic UUID from file bytes using MurmurHash3 x64_128 (seed=0)
- **Updated `save_file()`**: Now accepts `claimed_id: Option<&str>`
  - If provided: verifies `claimed_id == computed_hash`. Returns **400 Bad Request** on mismatch (data corruption)
  - Deduplication: checks by UUID first (new CAS), then falls back to SHA-256 hash (backward compat with old uploads)
  - File record `id` is now the deterministic CAS UUID (not random v4)
- **Cross-platform test**: Asserts matching UUIDs with Kotlin reference vectors

### Upload Handler (`handlers/file.rs`)
- Extracts `imageId` from multipart form data
- Passes it as `claimed_id` to `save_file()`
- CAS mismatch returns 400, other errors return 500

### Backward Compatibility
- Old uploads (with SHA-256 hash) are still deduplicated via hash column lookup
- `save_file()` without `claimed_id` (None) skips verification — works for server-side imports (support scraper)

---

## Files Changed

| File | Change |
|------|--------|
| `Cargo.toml` | +`murmur3 = "0.5"` |
| `src/services/filestore.rs` | +`content_hash_uuid()`, updated `save_file()` with CAS verification + idempotency |
| `src/handlers/file.rs` | Extract `imageId` from multipart, pass to save_file, 400 on CAS mismatch |
| `src/handlers/support.rs` | Added `None` for new `claimed_id` parameter |

## Build & Test
- `cargo check` — **OK** (50 pre-existing warnings)
- `cargo test test_content_hash` — **1 passed**

## Cross-Platform Reference Vectors
```
"test"  -> ac7d28cc-74bd-e19d-9a12-8231f9bd4d82
"hello" -> cbd8a7b3-41bd-9b02-5b1e-906a48ae1d19
""      -> 00000000-0000-0000-0000-000000000000
```

---

**Agent**: Expert Developer (The Fixer)
**Status**: ✅ Complete
