# Reqstly

Reqstly is in a big-bang rewrite with strict phase ordering.

## Rewrite Order
1. Backend (Supabase DB + Supabase Auth integration)
2. Backend hardening and tests
3. Frontend rewrite (SvelteKit)
4. Observability stack

## Phase Snapshot (March 6, 2026)
- Phase 0 (Infra Foundation): complete
- Phase 1 (Backend Core): complete
- Phase 2 (Backend Hardening): complete
- Phase 3 (Frontend Rewrite): in progress (closeout hardening)
- Phase 4 (Observability): not started

## Active Stack
- Supabase full self-hosted stack (`infra/supabase/`):
  - Kong, Auth, PostgREST, Realtime, Storage, Postgres, Studio, Functions, Analytics, Vector, Supavisor
- Reqstly backend (Rust + Axum + SQLx)
- Reqstly frontend (SvelteKit + Bun + Tailwind + shadcn-svelte)
- Caddy TLS reverse proxy (`https://localhost`, `https://api.localhost`, `https://supabase.localhost`)
- Redis
- Docker Compose (Supabase base + Reqstly overlays)

## Repository Layout
```text
reqstly/
├── backend/
├── frontend/
├── docs/
│   ├── PLAN.md
│   ├── frontend_functionality.md
│   └── improvements.md
├── infra/
│   ├── supabase/
│   ├── docker-compose.dev.yml
│   ├── docker-compose.yml
│   └── proxy/caddy/
├── scripts/
│   ├── setup-dev.sh
│   ├── up-dev.sh
│   ├── up-prod.sh
│   ├── smoke-check.sh
│   ├── test-backend.sh
│   └── reset-dev-db.sh
├── AGENTS.md
├── .env.example
├── .env.local.example
└── .env.local
```

## Local Development

### Quick Setup (fresh clone)
```bash
./scripts/setup-dev.sh
```

`setup-dev.sh` will:
- create `.env.local` from `.env.local.example` (unless it already exists)
- generate local Supabase dev secrets (`JWT_SECRET`, `ANON_KEY`, `SERVICE_ROLE_KEY`, and related keys)
- generate dev TLS certificates in `infra/proxy/caddy/certs/dev`

Useful options:
- `ROTATE_KEYS=1 ./scripts/setup-dev.sh` rotates secrets in existing `.env.local`
- `FORCE=1 ./scripts/setup-dev.sh` recreates `.env.local` from template and generates new secrets
- `ENV_FILE=/path/to/.env.local ./scripts/setup-dev.sh` writes env to a custom path

### Prerequisites
- Docker with Compose plugin
- `curl`
- `openssl` (for local TLS certificate generation)
- `bun` (for frontend checks/builds)
- Docker VM capacity for full Supabase stack:
  - minimum recommended: 4 GB RAM, 2 CPU
  - preferred for smoother Supabase stack startup: 8 GB RAM, 4 CPU

### Start Dev Stack
```bash
./scripts/up-dev.sh
```

`up-dev.sh` waits for compose service health and backend health before returning.

Common options:
- `COMPOSE_BUILD=1 ./scripts/up-dev.sh` to force rebuild
- `COMPOSE_FORCE_RECREATE=1 ./scripts/up-dev.sh backend` to recreate selected services and reconcile health
- `BACKEND_HEALTH_TIMEOUT=1200 ./scripts/up-dev.sh` to increase backend health wait timeout

### Frontend Checks (Local)
```bash
cd frontend
bun install --frozen-lockfile
bun run check
bun run build
```

### Backend Tests (Local)
```bash
./scripts/test-backend.sh
```

### Health Checks
```bash
curl -kfsS https://localhost/health
curl -kfsS https://api.localhost/health
curl -kfsS https://supabase.localhost/health
```

### Stop Stack
```bash
docker compose --env-file .env.local -f infra/supabase/docker-compose.yml -f infra/docker-compose.dev.yml down
```

## Production (Manual)
Use `.env` production values, then:
```bash
./scripts/up-prod.sh
```

## CI/CD
CI runs backend and frontend checks.

Deploy workflow is staging-first:
1. Build and push backend/frontend images
2. Deploy staging
3. Run staging smoke checks
4. Deploy production

Production promotion must not bypass staging validation.

## Tooling Conventions
- Prefer `bun` over `npm`
- Prefer `bunx` over `npx`
- Only use `npm`/`npx` when explicitly requested or when blocked on Bun tooling

## Documentation
- Rewrite plan and phase gates: [docs/PLAN.md](docs/PLAN.md)
- Frontend/API mapping: [docs/frontend_functionality.md](docs/frontend_functionality.md)
- Phase 3 pre-close backlog: [docs/improvements.md](docs/improvements.md)
- Repository agent rules: [AGENTS.md](AGENTS.md)
- Infra details: [infra/README.md](infra/README.md)

## License
MIT
