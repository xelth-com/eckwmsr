# PDA Device Pairing — Implementation Complete

## What was done

Ported PDA device pairing from Go (`eckwmsgo`) to Rust (`eckwmsr`). The Android PDA client can now pair with the Rust server using QR codes.

## Files created

| File | Purpose |
|------|---------|
| `src/models/registered_device.rs` | SeaORM entity for `registered_devices` table |
| `src/handlers/device.rs` | All device pairing + admin management endpoints |
| `src/utils/identity.rs` | Ed25519 keypair management + signature verification |

## Files modified

| File | Change |
|------|--------|
| `Cargo.toml` | Added `ed25519-dalek`, `image`, `local-ip-address` |
| `src/models/mod.rs` | Added `registered_device` module |
| `src/handlers/mod.rs` | Added `device` module |
| `src/utils/mod.rs` | Added `identity` module |
| `src/utils/auth.rs` | Added `generate_invite_token()`, `validate_invite_token()`, `generate_device_token()` |
| `src/db.rs` | Added `server_identity` field to `AppState` |
| `src/main.rs` | Registered all new routes, initialized server identity |

## Frontend

Copied SvelteKit source from `eckwmsgo/web/src/` to `eckwmsr/web/src/`. Built with `npm run build`. The devices page at `/E/dashboard/devices` is now available.

## New API Endpoints

### Public (no JWT)
- `POST /E/api/internal/register-device` — Device registration with Ed25519 signature verification

### Protected (JWT required)
- `GET /E/api/internal/pairing-qr` — Generate pairing QR code (PNG)
- `GET /E/api/internal/pairing-qr?type=vip` — Auto-approve QR (24h invite token)
- `GET /E/api/admin/devices` — List all devices
- `PUT /E/api/admin/devices/:id/status` — Update device status
- `PUT /E/api/admin/devices/:id/home` — Update home node
- `DELETE /E/api/admin/devices/:id` — Soft delete device
- `POST /E/api/admin/devices/:id/restore` — Restore deleted device

## Build status

`cargo build` — SUCCESS (warnings only, pre-existing)
`npm run build` — SUCCESS

## Next steps

- Deploy to production
- Test with Android PDA: scan QR, verify registration, approve device
