# AGENTS.md

This file defines repository rules and working conventions for coding agents.

## Project Description
Reqstly is being rebuilt as a big-bang rewrite.

Current goal:
- Rebuild backend first (depends on Supabase DB + Supabase Auth)
- Harden backend tests
- Build frontend after backend is stable
- Integrate observability stack
- Execute Phase 5 improvements backlog after observability gate

Current active infrastructure:
- Supabase full self-hosted stack from `infra/supabase/`
- Caddy (TLS/reverse proxy)
- Redis
- Prometheus + Loki + Promtail + Grafana
- Docker Compose (Supabase base + Reqstly overlays)
- VPS deploy via GitHub Actions

Source of planning truth:
- `docs/PLAN.md`

## Current Repo State
- `backend/` is reintroduced and is the active implementation focus.
- `frontend/` is reintroduced (SvelteKit) and active.
- `infra/supabase/` is vendored from upstream for full-stack self-hosted Supabase.
- `infra/observability/` is reintroduced and wired through compose overlays.

Do not reintroduce legacy codepaths unless explicitly requested.

## Priority Rules
1. Follow `docs/PLAN.md` phase order strictly.
2. Do not start frontend work before backend gates pass.
3. Keep observability baseline healthy; avoid removing Prometheus/Loki/Grafana integration without explicit request.
4. Keep big-bang direction; no legacy compatibility layer unless requested.

## Package Manager Policy (Important)
Use `bun` and `bunx` by default.

Required conventions:
- Use `bun install` instead of `npm install`.
- Use `bun add` instead of `npm install <pkg>`.
- Use `bun run <script>` instead of `npm run <script>`.
- Use `bunx <tool>` instead of `npx <tool>`.

Do not use `npm` or `npx` unless the user explicitly asks for it or there is a proven blocker with `bun`/`bunx`.

## Runtime and Infra Commands
Preferred stack commands:
- Dev up: `./scripts/up-dev.sh`
- Prod up: `./scripts/up-prod.sh`
- Smoke checks: `./scripts/smoke-check.sh`

If calling Docker Compose directly, use:
- Dev: `infra/supabase/docker-compose.yml` + `infra/docker-compose.dev.yml` with `.env.local`
- Prod: `infra/supabase/docker-compose.yml` + `infra/docker-compose.yml` with `.env` (or `.env.example` fallback)

## Backend Standards (when backend is reintroduced)
- Framework: Rust + Axum
- DB access: sqlx
- API prefix: `/api/v1`
- Auth: Supabase JWT validation
- Response shape: consistent envelope for success and errors
- Migrations: required for schema changes

Minimum backend quality gate before frontend:
- Backend compiles and runs in compose
- Migrations apply cleanly from empty DB
- Core endpoints implemented and tested
- CI backend checks green

## CI/CD Rules
Deployment flow must stay:
1. Build/test
2. Deploy staging
3. Run staging smoke checks
4. Deploy production

Never bypass staging validation for production deployment.

## Documentation Rules
When making significant architecture or phase-order changes:
- Update `docs/PLAN.md`
- Update this `AGENTS.md` if repo rules changed

## Safety and Scope
- Prefer minimal, reversible changes per step.
- Keep env vars canonical and avoid duplicated aliases.
- Do not add services not in current phase scope.
- If uncertain about phase ordering, defer to `docs/PLAN.md`.
