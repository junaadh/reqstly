# Reqstly Infrastructure

Infrastructure is split into:
- `infra/supabase/` (official Supabase self-host Docker bundle, vendored)
- Reqstly overlays:
  - `infra/docker-compose.dev.yml`
  - `infra/docker-compose.yml`
- Observability configs and Grafana provisioning:
  - `infra/observability/`
- Caddy reverse proxy configs in `infra/proxy/caddy/`

## Compose Model

Run with two compose files:
- Base: `infra/supabase/docker-compose.yml`
- Overlay: `infra/docker-compose.dev.yml` (dev) or `infra/docker-compose.yml` (prod)

Helper scripts:
- Dev up: `./scripts/up-dev.sh`
- Prod up: `./scripts/up-prod.sh`
- Smoke checks: `./scripts/smoke-check.sh`

## Services

Supabase base services include:
- `kong`, `auth`, `rest`, `realtime`, `storage`, `imgproxy`, `meta`
- `db`, `supavisor`, `studio`, `functions`, `analytics`, `vector`

Reqstly overlay services include:
- `backend`
- `migrate`
- `caddy`
- `redis`
- `prometheus`
- `loki`
- `promtail`
- `grafana`
- `postgres-exporter`
- `redis-exporter`

## Environment

Canonical env vars are in:
- `.env.example` (template)
- `.env.local` (local dev)

Key canonical Supabase vars:
- `POSTGRES_PASSWORD`
- `JWT_SECRET`
- `ANON_KEY`
- `SERVICE_ROLE_KEY`
- `SUPABASE_PUBLIC_URL`
- `API_EXTERNAL_URL`
- `POSTGRES_HOST`, `POSTGRES_DB`, `POSTGRES_PORT`

Key observability vars:
- `PROMETHEUS_PORT`
- `LOKI_PORT`
- `GRAFANA_PORT`
- `GRAFANA_ADMIN_USER`
- `GRAFANA_ADMIN_PASSWORD`

## Local Flow

1. Bootstrap local env + certs:
```bash
./scripts/setup-dev.sh
```

2. Start stack:
```bash
./scripts/up-dev.sh
```

3. Run smoke checks:
```bash
set -a; source .env.local; set +a
./scripts/smoke-check.sh
```

4. Open observability UIs:
- Grafana: `http://127.0.0.1:${GRAFANA_PORT:-3001}`
- Prometheus: `http://127.0.0.1:${PROMETHEUS_PORT:-9090}`

5. Stop stack:
```bash
docker compose --env-file .env.local \
  -f infra/supabase/docker-compose.yml \
  -f infra/docker-compose.dev.yml \
  down
```

## Notes

- `api.localhost` routes to Reqstly backend through Caddy.
- `supabase.localhost` routes to Supabase `kong` through Caddy.
- Backend uses `DATABASE_URL` semantics for `sqlx` migration and runtime DB access.
- `scripts/setup-dev.sh` is the default local bootstrap entrypoint. It creates `.env.local` and local TLS certs.
- Grafana dashboard provisioning lives under `infra/observability/grafana/`.
- Prometheus alert rules reference [docs/OBSERVABILITY_RUNBOOK.md](../docs/OBSERVABILITY_RUNBOOK.md).
