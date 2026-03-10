# Reqstly

Reqstly is a request-management platform with an embedded-auth Rust backend, SvelteKit frontend, and self-hosted infrastructure.

## Stack

- Backend: Rust, Axum, SQLx, PostgreSQL
- Frontend: SvelteKit, Bun, Tailwind, shadcn-svelte
- Auth: backend-owned auth (`password-auth`, `webauthn-rs`, `tower-sessions`)
- Realtime: Axum WebSocket (`/ws`)
- Infra: Docker Compose, Caddy, Redis, Prometheus, Loki, Promtail, Grafana

## Identity and Auth Model

- `app.app_users` is the canonical identity table.
- Business foreign keys point only to `app.app_users.id`.
- Password + passkey auth is first-party and handled in backend `/api/v1/auth/*` routes.
- Browser auth is cookie/session-first.
- WebSocket supports session and ws bearer-token compatibility (`/api/v1/auth/ws-token`).

## Repository Layout

```text
reqstly/
├── backend/
├── frontend/
├── docs/
├── infra/
│   ├── docker-compose.dev.yml
│   ├── docker-compose.yml
│   ├── observability/
│   └── proxy/caddy/
├── scripts/
├── AGENTS.md
├── .env.example
├── .env.local.example
└── .env.local
```

## Local Development

Prerequisites:

- Docker with Compose plugin
- `bun`
- `curl`
- `openssl`

Setup and run:

```bash
./scripts/setup-dev.sh
./scripts/up-dev.sh
```

Useful commands:

- `./scripts/reset-dev-db.sh`
- `./scripts/test-backend.sh`
- `./scripts/smoke-check.sh`

Backend checks:

```bash
cd backend
cargo fmt -- --check
cargo check
cargo test --lib
```

Frontend checks:

```bash
cd frontend
bun install --frozen-lockfile
bun run check
bun run build
```

## Production Deploy

- Use `.env` values for target environment.
- Place Cloudflare origin cert/key on host:
  - `~/certs/reqstly.pem`
  - `~/certs/reqstly.key`
- Restrict permissions (`chmod 600`).
- Ensure certificate SANs cover both `APP_DOMAIN` and `API_DOMAIN`.
  - Example: `*.reqstly.com` does not cover `api.dev.reqstly.com`.
  - Include `api.dev.reqstly.com` or `*.dev.reqstly.com` for dev API subdomains.

Start/update stack:

```bash
./scripts/up-prod.sh
```

## CI/CD

- Push `dev` => CI + deploy to dev environment
- Push `master` => CI + deploy to production environment
- Smoke checks run in CI
- Dev and prod run as isolated compose projects on the same VPS (`reqstly-dev` and `reqstly`) with a shared edge caddy route layer.

## Documentation

- Plan and gates: [docs/PLAN.md](docs/PLAN.md)
- Improvements and checklist: [docs/improvements.md](docs/improvements.md)
- Frontend/API map: [docs/frontend_functionality.md](docs/frontend_functionality.md)
- Infra runbook: [infra/README.md](infra/README.md)
- Agent conventions: [AGENTS.md](AGENTS.md)

## License

MIT
