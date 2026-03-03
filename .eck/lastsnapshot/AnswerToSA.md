# Task Complete: Binary SmartTag Encryption with Dynamic IV Length

## Date: 2026-03-03

### Status
✅ **COMPLETE — SmartTag encrypt/decrypt implemented, all 13 tests passing**

---

## What Was Done

### 1. Config (`src/config.rs`)
- Added `qr_prefixes: Vec<String>` — parsed from `QR_PREFIXES` env (comma-separated, default `ECK1.COM/`)
- Added `qr_tenant_suffix: String` — from `QR_TENANT_SUFFIX` env (default `IB`)
- Added `qr_iv_length: usize` — from `QR_IV_LENGTH` env (default `9`)

### 2. SmartTag (`src/utils/smart_code.rs`)
- `SmartTag` struct: `uuid: [u8; 16]`, `entity_type: u8`, `flags: u16`
- `to_bytes() -> [u8; 19]` and `from_bytes(&[u8; 19])` (flags big-endian)
- Entity type constants: WMS (0x00–0x05), Twenty CRM (0x10–0x12), Odoo (0x20–0x21)

### 3. Binary Encryption (`src/utils/encryption.rs`)
- **`eck_binary_encrypt(tag, prefix, suffix, iv_len, key_hex)`**:
  - Random IV string of `iv_len` Base32 chars
  - Nonce = SHA-256(iv_string)[:12]
  - AES-192-GCM encrypts 19 bytes → 35 bytes → 56 Base32 chars (constant)
  - Returns: `{prefix}{56ch data}{iv_string}{suffix}`
- **`eck_binary_decrypt(barcode, prefixes, suffix, key_hex)`**:
  - Strips prefix + suffix, first 56 chars = data, remainder = iv_string
  - Auto-detects IV length → old QR codes remain valid after config change

### 4. QR Layout Math
```
Payload:    16 (UUID) + 1 (type) + 2 (flags) = 19 bytes
Encrypted:  19 + 16 (GCM tag) = 35 bytes
Encoded:    35 * 8 / 5 = 56 Base32 chars (constant)
QR String:  9 (prefix) + 56 (data) + 9 (iv) + 2 (suffix) = 76 chars
QR V3 Max:  77 chars alphanumeric ✓
```

### 5. Tests (13 total, all pass)
- SmartTag: roundtrip, big-endian flags
- Encryption: roundtrip, different IV lengths (5 & 12), multiple prefixes, wrong key rejection, wrong suffix rejection, Base32 35-byte roundtrip, QR V3 fit

---

## Files Changed

| File | Change |
|------|--------|
| `src/config.rs` | +3 fields: `qr_prefixes`, `qr_tenant_suffix`, `qr_iv_length` |
| `src/utils/smart_code.rs` | +`SmartTag` struct, entity constants, `to_bytes`/`from_bytes`, 2 tests |
| `src/utils/encryption.rs` | +`eck_binary_encrypt`, `eck_binary_decrypt`, helpers, 7 tests |

## Build & Test
- `cargo check` — **OK**
- `cargo test` — **13 passed** (smart_code + encryption)

---

**Agent**: Expert Developer (The Fixer)
**Status**: ✅ Complete
