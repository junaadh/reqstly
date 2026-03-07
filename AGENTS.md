# AGENTS.md

Repository rules and working conventions for coding agents.

## Project Direction

Reqstly is in a big-bang rewrite.

Current strategic target:

- Keep modular monolith architecture.
- Keep `/api/v1` API surface.
- Run embedded auth inside backend (Axum + Rust), not an external auth server.
- Baseline auth methods: email/password + passkeys.
- Keep OIDC/Entra deferred to Phase D.
- Keep canonical identity in `app.app_users`.

## Current Active Infra

- Caddy (TLS/reverse proxy)
- Redis
- Prometheus + Loki + Promtail + Grafana
- Docker Compose
- VPS deploy via GitHub Actions

Source of truth for planning:
- `docs/PLAN.md`

## Priority Rules

1. Follow `docs/PLAN.md` phase order.
2. Keep observability baseline healthy unless explicitly directed otherwise.
3. Do not reintroduce Supabase/authentik auth dependencies.
4. Do not bind business logic to provider-internal identity tables.

## Package Manager Policy

Use `bun` and `bunx` by default.

- `bun install` instead of `npm install`
- `bun add` instead of `npm install <pkg>`
- `bun run <script>` instead of `npm run <script>`
- `bunx <tool>` instead of `npx <tool>`

Use `npm`/`npx` only if explicitly requested or blocked.

## Runtime and Infra Commands

Preferred commands:

- Dev up: `./scripts/up-dev.sh`
- Prod up: `./scripts/up-prod.sh`
- Dev DB reset: `./scripts/reset-dev-db.sh`
- Smoke checks: `./scripts/smoke-check.sh`

## Backend Standards

- Framework: Rust + Axum
- DB access: SQLx
- API prefix: `/api/v1`
- Auth baseline:
  - `password-auth` for passwords
  - `webauthn-rs` for passkeys
  - `tower-sessions` for first-party cookie sessions
- Identity mapping: business FKs and ownership resolve through `app.app_users`
- Response shape: consistent envelope for success and errors
- Migrations required for schema changes

Minimum backend gate:

- Backend compiles
- Migrations apply from empty DB
- Core endpoints implemented and tested
- CI backend checks green

## CI/CD Rules

Deployment flow must remain:

1. Build/test
2. Deploy staging
3. Run staging smoke checks
4. Deploy production

Never bypass staging validation.

## Documentation Rules

When architecture or phase-order changes:

- Update `docs/PLAN.md`
- Update this `AGENTS.md`
- Update phase checklist docs if scope/acceptance changed

## Safety and Scope

- Prefer minimal, reversible changes.
- Keep env vars canonical; avoid duplicated aliases.
- Do not add out-of-scope services without explicit request.
- If uncertain on phase sequencing, defer to `docs/PLAN.md`.
