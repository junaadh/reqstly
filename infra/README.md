# Reqstly Infrastructure (Phase 5 Baseline)

## Target Direction

Infrastructure baseline is:

- Reqstly backend + frontend
- PostgreSQL for app data and session store
- Embedded backend auth (no authentik in baseline runtime)
- Caddy reverse proxy
- Redis
- Observability stack (`infra/observability/`)

Entra/OIDC runtime is deferred to Phase D.

## Compose Model

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
- Keep staging smoke checks mandatory before production promotion.
