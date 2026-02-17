# Stage 1: Build Frontend
FROM node:18-alpine AS frontend-builder
WORKDIR /app/web
COPY web/package.json web/package-lock.json ./
RUN npm ci
COPY web/ .
ENV BASE_PATH=/E
RUN npm run build

# Stage 2: Build Backend
FROM rust:1.77-slim-bookworm AS backend-builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
# Create dummy main to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

COPY src/ src/
COPY scripts/ scripts/
COPY --from=frontend-builder /app/web/build web/build

# Touch main.rs to force rebuild with real source
RUN touch src/main.rs
RUN cargo build --release

# Stage 3: Runtime
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=backend-builder /app/target/release/eckwmsr ./eckwms
RUN mkdir -p data/pg data/filestore

ENV PORT=3210
ENV RUST_LOG=info
ENV HTTP_PATH_PREFIX=/E

EXPOSE 3210

CMD ["./eckwms"]
