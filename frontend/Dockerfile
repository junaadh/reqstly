FROM oven/bun:1.2.23-alpine AS base
WORKDIR /app

FROM base AS deps
COPY package.json bun.lock ./
RUN bun install --frozen-lockfile --ignore-scripts

FROM base AS development
ENV NODE_ENV=development
ENV HOST=0.0.0.0
ENV PORT=3000
COPY package.json bun.lock ./
RUN bun install --frozen-lockfile --ignore-scripts
COPY . .
CMD ["bun", "run", "dev", "--host", "0.0.0.0", "--port", "3000"]

FROM deps AS builder
COPY . .
RUN bun run build

FROM node:20.19.5-alpine AS runtime
WORKDIR /app
ENV NODE_ENV=production
ENV HOST=0.0.0.0
ENV PORT=3000
ENV ORIGIN=https://localhost
RUN addgroup -S appgroup && adduser -S appuser -G appgroup
COPY --from=deps /app/node_modules ./node_modules
COPY --from=builder /app/build ./build
COPY --from=builder /app/package.json ./package.json
USER appuser
EXPOSE 3000
HEALTHCHECK --interval=10s --timeout=5s --retries=10 CMD wget -qO- "http://127.0.0.1:${PORT}/" >/dev/null || exit 1
CMD ["node", "build"]
