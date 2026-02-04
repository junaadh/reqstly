# Development stage
FROM oven/bun:1 AS development

WORKDIR /app

# Copy package files
COPY package.json bun.lock ./

# Install dependencies
RUN bun install

# Copy source code
COPY . .

# Expose Vite dev server port
EXPOSE 5173

# Development command
CMD ["bun", "run", "dev", "--host"]

# Builder stage
FROM oven/bun:1 AS builder

WORKDIR /app

# Copy package files
COPY package.json bun.lock ./

# Install dependencies
RUN bun install --frozen-lockfile

# Copy source code
COPY . .

# Build the application
RUN bun run build

# Production stage
FROM caddy:2-alpine AS production

# Copy built assets from builder
COPY --from=builder /app/dist /usr/share/caddy

# Caddy configuration for SPA
COPY <<EOF /etc/caddy/Caddyfile
:80 {
    root * /usr/share/caddy
    encode gzip
    try_files {path} /index.html
    file_server
}
EOF
