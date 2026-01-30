# Development stage
FROM rust:1.83-slim as development

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies
RUN cargo build || true
RUN rm -rf src

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build the application
RUN cargo build

# Development runner
CMD ["cargo", "run"]

# Production builder stage
FROM rust:1.83-slim as builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy source and manifests
COPY . .

# Build the application in release mode
RUN cargo build --release

# Production stage
FROM debian:bookworm-slim as production

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 appuser

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/reqstly_backend /app/reqstly_backend

# Copy migrations
COPY --from=builder /app/migrations /app/migrations

# Change ownership
RUN chown -R appuser:appuser /app

USER appuser

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run the application
CMD ["/app/reqstly_backend"]
