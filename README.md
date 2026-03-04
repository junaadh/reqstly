# Reqstly

Reqstly is currently in a **big-bang rewrite** phase.

## Current Status
The repository has been intentionally reset to rebuild in a strict order:
1. Backend (depends on Supabase DB + Supabase Auth)
2. Backend testing/hardening
3. Frontend
4. Observability stack (last)

At this moment:
- `backend/` is reintroduced and in active Phase 1 development
- `frontend/` is intentionally removed
- Prometheus/Loki/Grafana are intentionally removed
- Infra baseline is active (full Supabase docker stack + Caddy + backend + Redis)

## Active Infrastructure
- Supabase full self-hosted Docker stack (`infra/supabase/`)
  - Kong, Auth, PostgREST, Realtime, Storage, Postgres, Studio, Functions, Analytics, Vector, Supavisor
- Caddy (TLS/reverse proxy)
- Redis
- Docker Compose (Supabase base + Reqstly overlay)
- GitHub Actions (staging-first deploy flow)

## Project Structure (Current)
```text
reqstly/
├── AGENTS.md
├── docs/
│   └── PLAN.md
├── infra/
│   ├── supabase/
│   ├── docker-compose.dev.yml
│   ├── docker-compose.yml
│   └── proxy/caddy/
├── scripts/
│   ├── up-dev.sh
│   ├── up-prod.sh
│   ├── smoke-check.sh
│   └── generate-supabase-dev-keys.sh
├── .env.example
└── .env.local
```

## Local Development
### Prerequisites
- Docker with Compose plugin
- `curl`
- Docker VM resources for full Supabase stack: at least `4GB` RAM and `2` CPU

### Start Dev Stack
```bash
./scripts/up-dev.sh
```

If Docker resources are lower than the minimum, `up-dev.sh` will fail fast with a Colima fix command.
You can override thresholds with `MIN_DOCKER_MEMORY_GB` and `MIN_DOCKER_CPUS`.
`up-dev.sh` now waits for service health checks and explicitly verifies the backend container health before returning.

Optional script flags:
- `COMPOSE_BUILD=1 ./scripts/up-dev.sh` to force image rebuild
- `COMPOSE_WAIT_TIMEOUT=1200 ./scripts/up-dev.sh` to increase compose wait timeout
- `BACKEND_HEALTH_TIMEOUT=1200 ./scripts/up-dev.sh` to increase explicit backend health gate timeout

Backend dev uses a bind-mounted `backend/` source tree and persistent Docker volumes (`cargo` registry/git and `target`) to cache Rust compilation between restarts.
First run can still take several minutes while caches warm up.

### Generate Dev Supabase Keys (No CLI)
Set a strong `JWT_SECRET` in `.env.local`, then run:
```bash
./scripts/generate-supabase-dev-keys.sh
```

This updates:
- `ANON_KEY`
- `SERVICE_ROLE_KEY`

### Health Checks
```bash
curl -kfsS https://api.localhost/health
curl -kfsS https://supabase.localhost/health
```

### Logging
Backend logging is `tracing`-based and observability-ready.

Key env vars:
- `LOGGING__LEVEL` (EnvFilter syntax)
- `LOGGING__FORMAT` (`json` | `pretty` | `compact`)
- `LOGGING__SERVICE_NAME`
- `LOGGING__ENVIRONMENT`

### Stop Stack
```bash
docker compose --env-file .env.local -f infra/supabase/docker-compose.yml -f infra/docker-compose.dev.yml down
```

## Production Stack (Manual)
Use `.env` for production values, then:
```bash
./scripts/up-prod.sh
```

## CI/CD
Deployment flow is staging-first:
1. Build/Test
2. Build and push backend runtime image (GHCR)
3. Deploy staging (pull image on VPS, no Rust build on server)
4. Run smoke checks
5. Deploy production (pull same image tag)

Production deployment must not bypass staging validation.

## Tooling Conventions
- Prefer `bun` over `npm`
- Prefer `bunx` over `npx`
- Only use `npm`/`npx` when explicitly requested or if `bun`/`bunx` is blocked

## Documentation
- Rewrite execution plan: [docs/PLAN.md](docs/PLAN.md)
- Agent/repo rules: [AGENTS.md](AGENTS.md)

## License
MIT
