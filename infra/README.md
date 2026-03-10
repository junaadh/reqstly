# Reqstly Infrastructure (Phase 5 Baseline)

## Target Direction

Infrastructure baseline is:

- Reqstly backend + frontend
- PostgreSQL for app data and session store
- Embedded backend auth (no authentik in baseline runtime)
- Caddy reverse proxy (single edge instance)
- Redis
- Observability stack (`infra/observability/`)

Supabase infrastructure artifacts are no longer part of active runtime.

Entra/OIDC runtime is deferred to Phase D.

## Compose Model

Production compose uses profile-based service activation:

- core services: always enabled (`db`, `redis`, `migrate`, `backend`, `frontend`)
- edge services: `profiles: ["edge"]` (`caddy`)
- observability services: `profiles: ["obs"]`

On VPS:

- prod project: `COMPOSE_PROJECT_NAME=reqstly` with `COMPOSE_PROFILES=edge,obs`
- dev project: `COMPOSE_PROJECT_NAME=reqstly-dev` with no profiles (core only)
- shared network for cross-project routing: `EDGE_NETWORK_NAME=reqstly-edge`

This allows prod and dev to run simultaneously without container/volume collisions.

Use existing script entrypoints for orchestration:

- `./scripts/setup-dev.sh`
- `./scripts/up-dev.sh`
- `./scripts/reset-dev-db.sh`
- `./scripts/smoke-check.sh`
- `./scripts/up-prod.sh`

Direct `docker compose` is for debugging/teardown only.

## Services (Baseline)

- `backend`
- `frontend`
- `db` (Postgres)
- `migrate` (one-shot SQLx migrations container)
- `caddy`
- `redis`
- `prometheus`
- `loki`
- `promtail`
- `grafana`
- `postgres-exporter`
- `redis-exporter`

## Environment Variables

Canonical env definitions live in:

- `.env.example`
- `.env.local.example`

### Backend/auth runtime

- `DATABASE__URL`
- `SERVER__PORT`
- `SERVER__BASE_URL`
- `CORS__ALLOWED_ORIGIN`
- `AUTH__WS_TOKEN_SECRET`
- `AUTH__WS_TOKEN_ISSUER`
- `AUTH__SESSION_COOKIE_NAME`
- `AUTH__SESSION_IDLE_MINUTES`
- `AUTH__SESSION_SECURE`
- `AUTH__WEBAUTHN_RP_ID`
- `AUTH__WEBAUTHN_RP_ORIGIN`
- `AUTH__WEBAUTHN_RP_NAME`
- `POSTGRES_PASSWORD_ENCODED` (URL-encoded `POSTGRES_PASSWORD` for DSN envs)

### Caddy TLS runtime (production)

- `CADDY_TLS_CERT_DIR` (host path mounted into caddy as `/certs`)
- `CADDY_TLS_CERT_FILE` (container path, default `/certs/reqstly.pem`)
- `CADDY_TLS_KEY_FILE` (container path, default `/certs/reqstly.key`)
- `APP_DOMAIN` and `API_DOMAIN` are passed into the caddy container and used by Caddyfile env placeholders.
- `DEV_APP_DOMAIN` and `DEV_API_DOMAIN` are routed by the same edge caddy to the dev stack over `reqstly-edge`.

### Logging/telemetry

- `LOGGING__LEVEL`
- `LOGGING__FORMAT`
- `LOGGING__SERVICE_NAME`
- `LOGGING__ENVIRONMENT`

### Observability

- `PROMETHEUS_PORT`
- `LOKI_PORT`
- `GRAFANA_PORT`
- `GRAFANA_ADMIN_USER`
- `GRAFANA_ADMIN_PASSWORD`

## Local Flow

1. Bootstrap local env and certs:

```bash
./scripts/setup-dev.sh
```

2. Start stack:

```bash
./scripts/up-dev.sh
```

`migrate` is expected to exit `0` after applying migrations.

3. Reset DB when needed:

```bash
./scripts/reset-dev-db.sh
```

4. Run smoke checks:

```bash
./scripts/smoke-check.sh
```

## Operational Rules

- App business logic must only use `app.app_users` as identity root.
- Do not bind business logic to provider-internal identity tables.
- Keep same-site browser session path as default auth path.
- Keep CI smoke checks green before promoting changes to production.
- Keep `/ws` routed to backend on app domains (do not proxy websocket path to frontend).

## TLS and Domains (Cloudflare)

- If Cloudflare SSL mode is `Full (strict)`, origin cert SANs must cover both app and API hostnames.
- `*.reqstly.com` covers `dev.reqstly.com` but does not cover `api.dev.reqstly.com`.
- For dev subdomains, include either:
  - explicit `api.dev.reqstly.com`, or
  - wildcard `*.dev.reqstly.com`.
- If cert/key files are updated on host, recreate caddy to reload files/config:

```bash
cd ~/reqstly-dev
docker compose --env-file .env -f infra/docker-compose.yml up -d --force-recreate caddy
```
