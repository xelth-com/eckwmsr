# eckWMS (Rust Edition)

High-performance Warehouse Management System with built-in Mesh Sync, AI capabilities, and Embedded Database.

## Quick Start

### Option A: Zero-Dependency (Embedded DB)
No external database required. Just run the binary.

```bash
# 1. Build
cargo build --release

# 2. Run (starts on port 3210)
./target/release/eckwmsr
```

The server will:
- Auto-download and start PostgreSQL (in `./data/pg`)
- Create all database tables
- Generate a setup account (`admin@setup.local`)
- Serve the embedded frontend at `http://localhost:3210/E/`

### Option B: Docker

```bash
docker build -t eckwms .
docker run -p 3210:3210 -v $(pwd)/data:/app/data eckwms
```

## Mesh Synchronization

eckWMS supports peer-to-peer syncing between servers.

1. **Generate Pairing Code** on the "Host" server (Dashboard -> Devices -> Servers).
2. **Enter Code** on the "Client" server.
3. Servers exchange keys and begin syncing changes via WebSocket + HTTP.

## AI Features

To enable AI features (barcode analysis, OCR), set the API key in `.env`:
```env
GEMINI_API_KEY=your_key_here
```

## Architecture

- **Backend**: Rust (Axum, SeaORM, Tokio)
- **Frontend**: SvelteKit (Embedded)
- **Database**: PostgreSQL (Embedded or External)
- **Sync**: Merkle Tree based diffing + AES-256 encryption

## Development

```bash
# Run backend
cargo run

# Run frontend (separate terminal)
cd web && npm run dev
```
