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

# ---------- Builder ----------
FROM base AS builder

# Copy built dependencies to speed up build
COPY --from=deps /app/target /app/target
COPY . .

# Install sqlx CLI for migrations
RUN cargo install sqlx-cli --no-default-features --features postgres

RUN cargo sqlx prepare

# Build the application in release mode
RUN cargo build --release

# ---------- Development ----------
FROM base AS development

# Copy sqlx CLI from deps
COPY --from=deps /usr/local/cargo/bin/sqlx /usr/local/cargo/bin/sqlx
COPY . .

WORKDIR /app

# Run dev server
CMD ["cargo", "run"]

# ---------- Runtime (Production) ----------
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the release binary from builder
COPY --from=builder /app/target/release/reqstly_backend /app/reqstly_backend

# Copy migrations (needed for sqlx at runtime if you run migrations)
COPY --from=builder /app/migrations /app/migrations

EXPOSE 3000

# Run the backend in production
CMD ["/app/reqstly_backend"]
