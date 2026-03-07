# Reqstly

Reqstly is in a big-bang rewrite with strict phase ordering.

## Rewrite Order

1. Backend core
2. Backend hardening and tests
3. Frontend rewrite (SvelteKit)
4. Observability
5. Auth/data platform refactor (Phase 5, active)

## Phase Snapshot (March 7, 2026)

- Phase 0: complete
- Phase 1: complete
- Phase 2: complete
- Phase 3: complete
- Phase 4: complete
- Phase 5: active (embedded auth re-baseline)

## Target Stack (Phase 5 Baseline)

- Backend: Rust + Axum + SQLx
- Frontend: SvelteKit + Bun + Tailwind + shadcn-svelte
- Database: PostgreSQL (`app.*` schema)
- Auth (embedded in backend):
  - `password-auth` for password hashing/verification
  - `webauthn-rs` for passkeys
  - `tower-sessions` + SQLx store for first-party sessions
- Realtime: Axum WebSocket (`/ws`) with cookie-first auth + ws bearer compatibility
- Infra: Caddy, Redis, Observability stack, Docker Compose

## Identity Model

- `app.app_users` is canonical app identity.
- Business FKs point only to `app.app_users.id`.
- No business coupling to Supabase/authentik/internal provider tables.

## Transition Note

Phase 5 is in progress. Some legacy Supabase assets remain in-repo until cleanup checklist items complete.

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

Canonical entrypoints (kept unchanged during migration):

- `./scripts/setup-dev.sh`
- `./scripts/up-dev.sh`
- `./scripts/reset-dev-db.sh`
- `./scripts/test-backend.sh`
- `./scripts/smoke-check.sh`
- `./scripts/up-prod.sh`

### Prerequisites

- Docker with Compose plugin
- `curl`
- `openssl`
- `bun`

### Start Stack

```bash
./scripts/up-dev.sh
```

### Backend Checks

```bash
cd backend
cargo fmt -- --check
cargo check
cargo test --lib
```

### Frontend Checks

```bash
cd frontend
bun install --frozen-lockfile
bun run check
bun run build
```

### Smoke Checks

```bash
./scripts/smoke-check.sh
```

## Production

Use `.env` production values, then:

```bash
./scripts/up-prod.sh
```

## CI/CD

Deployment remains staging-first and mandatory:

1. Build/test
2. Deploy staging
3. Run staging smoke checks
4. Deploy production

## Tooling Conventions

- Prefer `bun` over `npm`
- Prefer `bunx` over `npx`

## Documentation

- Plan and gates: [docs/PLAN.md](docs/PLAN.md)
- Phase 5 checklist: [docs/improvements.md](docs/improvements.md)
- Frontend/API map: [docs/frontend_functionality.md](docs/frontend_functionality.md)
- Infra runbook: [infra/README.md](infra/README.md)
- Agent rules: [AGENTS.md](AGENTS.md)

## License

MIT
