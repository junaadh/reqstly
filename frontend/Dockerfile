FROM oven/bun:1.2.23 AS bun-bin

FROM node:20.19.5-bookworm-slim AS base
WORKDIR /app
COPY --from=bun-bin /usr/local/bin/bun /usr/local/bin/bun
COPY --from=bun-bin /usr/local/bin/bunx /usr/local/bin/bunx

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
CMD ["bun", "run", "dev:docker"]

FROM deps AS builder
COPY . .
RUN bun run build

FROM base AS runtime
WORKDIR /app
ENV NODE_ENV=production
ENV HOST=0.0.0.0
ENV PORT=3000
ENV ORIGIN=https://localhost
RUN groupadd --system appgroup && useradd --system --gid appgroup --create-home appuser
COPY --from=deps /app/node_modules ./node_modules
COPY --from=builder /app/build ./build
COPY --from=builder /app/package.json ./package.json
USER appuser
EXPOSE 3000
HEALTHCHECK --interval=10s --timeout=5s --retries=10 CMD node -e "fetch('http://127.0.0.1:' + (process.env.PORT || '3000') + '/').then((r)=>process.exit((r.status>=200&&r.status<400)?0:1)).catch(()=>process.exit(1))"
CMD ["bun", "run", "start"]
