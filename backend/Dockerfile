# ---------- Base ----------
FROM rust:slim AS base

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# ---------- Dependency cache ----------
FROM base AS deps

# Copy manifest files to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true
RUN rm -rf src

# Install sqlx CLI for migrations (used by dev + migrate service)
RUN cargo install sqlx-cli --no-default-features --features postgres

# ---------- Development ----------
FROM base AS development

# Copy sqlx CLI from deps
COPY --from=deps /usr/local/cargo/bin/sqlx /usr/local/cargo/bin/sqlx
COPY --from=deps /usr/local/cargo/bin/cargo-sqlx /usr/local/cargo/bin/cargo-sqlx
COPY . .

WORKDIR /app

ENV SQLX_OFFLINE=true

# Run dev server
CMD ["sh", "-c", "sqlx migrate run && cargo run"]

# ---------- Runtime (Production) ----------
FROM rust:slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    libz1 \
    libzstd1 \
    libgcc-s1 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy project source
COPY . .

# Use offline metadata if present (.sqlx/) to avoid build-time DB dependency
ENV SQLX_OFFLINE=false

RUN cargo build --release

# Build and run in release mode
CMD ["sh", "-c", "sqlx migrate run && cargo run --release"]
