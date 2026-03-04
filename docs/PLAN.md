# Reqstly Rewrite Plan (Big-Bang, Refined)

## 1) Mission and Constraints

### Mission
Rebuild Reqstly in strict sequence with a production-first delivery model:
1. Backend first
2. Backend hardening and test depth
3. Frontend rewrite (SvelteKit)
4. Observability stack last

### Non-Negotiable Constraints
- Supabase Auth (`auth.users`) is the identity source of truth.
- App data is in Supabase Postgres (`app.*` schema).
- Infra remains Docker Compose + Caddy + Redis + VPS deploy via GitHub Actions.
- No legacy compatibility layer unless explicitly requested.
- No frontend work before backend phase gate passes.
- No observability stack before frontend phase gate passes.

### Current Reality (March 3, 2026)
- Backend reintroduced and running in dev compose.
- Frontend intentionally removed.
- Observability services intentionally removed.
- Supabase is now sourced from the official full docker bundle in `infra/supabase/`.
- Compose scripts available: `scripts/up-dev.sh`, `scripts/up-prod.sh`, `scripts/smoke-check.sh`.

## 2) Phase Status Snapshot (March 5, 2026)

- Phase 0 (Infra Foundation): **complete**
- Phase 1 (Backend Core): **complete**
- Phase 2 (Backend Hardening): **complete**
- Phase 3 (Frontend Rewrite): **not started**
- Phase 4 (Observability): **not started**

### Phase 1 completion checklist
- [x] Confirm Supabase-issued token E2E from host shell (`./scripts/token-e2e-check.py`)
- [x] OpenAPI parity check (`./scripts/openapi-parity-check.py`)
- [x] CI hard gates enabled (`fmt`, `clippy`, `openapi parity`, backend tests + coverage)
- [x] Contract tests and performance sanity checks in integration suite

### Phase 2 completion checklist
- [x] Failure-path tests for auth, validation, DB constraints, and ownership
- [x] Pagination/sorting behavior tests
- [x] Performance sanity checks for bounds and indexes
- [x] Backend test harness stable locally with repeatable runs (including DB bootstrap path)
- [x] CI backend checks are defined and enforced in repository workflows; GitHub branch protection remains an external admin setting

## 3) Architecture Decision

### Decision: Backend architecture pattern

### Context
- Team size: small.
- Product scope: medium and growing (auth, ticket lifecycle, audit, future realtime, future storage).
- Need strong testability and low coupling to framework/infrastructure details.

### Options Considered
1. Layered monolith.
2. Clean/Hexagonal modular monolith.

### Decision
Use a **modular monolith** with **Clean/Hexagonal boundaries**:
- Domain: entities, rules, invariants.
- Application: use-cases and orchestration.
- Adapters: HTTP handlers, SQLx repositories, auth/JWT integration.
- Framework/Infra: Axum, Postgres, Supabase Auth, Redis, Caddy.

### Why
- Keeps complexity appropriate for current team size.
- Preserves testability and swap-ability for adapters.
- Avoids premature microservice fragmentation.

### Consequences
- Positive: clear dependency flow inward, easier unit/integration testing, stable API contracts.
- Trade-off: requires discipline in module boundaries and review checklists.

## 4) Target Backend Structure (Phase 1-2 target)

```text
backend/
  src/
    main.rs
    config.rs
    error.rs
    response.rs
    api/
      mod.rs
      handlers/
      dto/
    app/
      use_cases/
      services/
    domain/
      request/
      audit/
      errors/
    infra/
      db/
      auth/
      cache/
  migrations/
  openapi/
```

Rules:
- `domain` has zero framework imports.
- `app` depends on `domain` only.
- `api` maps HTTP <-> app DTOs.
- `infra` implements adapter ports.

## 5) API and Data Design Baseline

### API contract
- Version prefix: `/api/v1`.
- Uniform success/error envelope across handlers.
- Health endpoints:
  - Infra/Caddy: `/health` on app domain.
  - Backend API: `/api/v1/health` (and direct `api.localhost/health` route currently proxied).

### Auth model
- JWT issued by Supabase Auth (GoTrue).
- `sub` maps to `auth.users.id`.
- No app-owned users table duplication.

### DB model (current direction)
- `app.requests` with FK to `auth.users` (`owner_user_id`, optional `assignee_user_id`).
- `app.request_audit_logs` with FK to `auth.users` (`actor_user_id`).
- RLS enabled for ownership-based access.
- Realtime publication includes request and audit tables.

## 6) Environment and Deployment Model

### Environments
- `dev`: local compose, self-signed local TLS via Caddy.
- `staging`: VPS deploy from CI, smoke tests required.
- `production`: promoted only after staging success.

### Config policy
- Canonical env vars in `.env.example`.
- Local overrides in `.env.local`.
- No duplicate aliases for same meaning.
- Keep `SMOKE_INSECURE_TLS=true` only for local self-signed cert workflows.

### Container policy
- Pin image versions (avoid floating `latest`).
- Multi-stage builds where possible.
- Health checks for critical services.
- Keep runtime services minimal per phase scope.
- For local full Supabase stack, require Docker VM capacity of at least 7GB RAM and 4 CPU (8GB RAM recommended).

## 7) CI/CD Target Workflow (Staging-first, Mandatory)

### Trigger model
- PR to `master`: run CI checks only.
- Push to `master`: CI + deploy pipeline.

### Required CI stages
1. `backend-format`: `cargo fmt --check`.
2. `backend-lint`: `cargo clippy -- -D warnings`.
3. `backend-test-unit`: fast unit tests.
4. `backend-test-integration`: DB-backed integration tests.
5. `backend-contract`: OpenAPI/schema envelope assertions.
6. `frontend-check` (Phase 3 onward): bun install/check/build.

### Required CD stages
1. Deploy staging.
2. Run staging smoke checks.
3. Manual approval gate.
4. Deploy production.
5. Run production smoke checks.

### Rollback policy
- If staging smoke fails: stop promotion.
- If production smoke fails: rollback to previous known good image/commit.
- Keep rollback command documented in runbook.

## 8) Phase Plan with Exit Gates

### Phase 0: Infra Foundation (complete)
Scope:
- Compose scripts and env normalization.
- Caddy routing for app/api/supabase domains.
- Official full Supabase stack (base compose) + Reqstly overlay services (backend/migrate/caddy/redis).

Exit gate:
- `./scripts/up-dev.sh` succeeds from clean volumes.
- `./scripts/smoke-check.sh` succeeds with local env.
- Dev compose services healthy and deterministic restart behavior.

### Phase 1: Backend Core (Supabase-dependent, complete)
Scope:
- Implement request lifecycle APIs and `/api/v1/me`.
- Enforce envelope consistency and validation rules.
- Keep migrations deterministic from empty DB.
- Maintain OpenAPI draft current with implementation.

Exit gate:
- Core endpoints functional against Supabase-issued token.
- Migrations apply cleanly on fresh DB.
- API docs and implementation match.

### Phase 2: Backend Hardening (complete)
Scope:
- Add unit + integration + contract test layers.
- Add failure-path tests (auth, validation, DB constraints, ownership).
- Add pagination and sorting behavior tests.
- Add performance sanity checks (bounded queries, index usage checks where relevant).

Exit gate:
- CI backend checks green on every PR.
- Integration tests stable and repeatable.
- Error schema and status codes consistent across all endpoints.

### Phase 3: Frontend Rewrite (SvelteKit)
Scope:
- Recreate frontend with SvelteKit SSR.
- Use Supabase Auth client flows.
- Consume backend `/api/v1` exclusively.
- Implement required routes: `/login`, `/`, `/requests`, `/requests/new`, `/requests/[id]`, `/profile`.

Tooling rule:
- Use `bun`/`bunx` for install/build/check/test flows.

Exit gate:
- End-to-end request lifecycle from UI works reliably.
- Session persistence stable across reload/navigation.
- Frontend checks green in CI.

### Phase 4: Observability (final)
Scope:
- Reintroduce Prometheus, Loki, Promtail, Grafana.
- Dashboard minimums:
  - API request rate, p95 latency, error rate.
  - Auth success/failure rates.
  - DB availability/connection saturation.
- Actionable alert rules and runbook links.

Exit gate:
- Observability stack healthy in compose and staging.
- Alerts validated through controlled failure simulation.

## 9) Quality Gates and SLO-style Targets

### Reliability targets
- Staging smoke success rate: 100% before production promotion.
- Production deployment requires successful post-deploy smoke.

### API quality targets
- All endpoints use uniform envelope.
- No unbounded list endpoint.
- Ownership access enforced at app layer and RLS layer.

### Performance guardrails
- Pagination default and max limits enforced.
- Query patterns backed by indexes matching filters/sorts.

## 10) Risk Register and Mitigations

1. Supabase image/tag drift breaks startup.
- Mitigation: pinned image tags and explicit upgrade procedure.

2. DB init/migration race conditions.
- Mitigation: dedicated `migrate` service + health-gated dependencies.

3. TLS trust failures in local smoke checks.
- Mitigation: local-only `SMOKE_INSECURE_TLS=true` support.

4. Drift between OpenAPI and implementation.
- Mitigation: contract checks in CI.

5. Phase bleed (frontend/observability started early).
- Mitigation: enforce phase gates in this plan and PR review checklist.

6. Full Supabase stack instability on low-resource local Docker VM (analytics OOM kills).
- Mitigation: enforce minimum local Docker resources (>=7GB RAM / 4 CPU, recommend 8GB RAM) in dev startup scripts and onboarding docs.

## 11) Execution Checklists (from architecture + DevOps reviews)

Architecture Design Progress:
- [x] Step 1: Understand requirements and constraints
- [x] Step 2: Assess project size and team capabilities
- [x] Step 3: Select architecture pattern
- [x] Step 4: Define directory structure
- [x] Step 5: Document trade-offs and decision
- [x] Step 6: Validate against decision framework

DevOps Setup Progress:
- [x] Step 1: Containerize application (Dockerfile)
- [~] Step 2: Set up CI/CD pipeline (staging-first exists, hardening pending)
- [x] Step 3: Define deployment strategy
- [ ] Step 4: Configure monitoring & alerting
- [x] Step 5: Set up environment management
- [ ] Step 6: Document runbooks
- [x] Step 7: Validate against anti-patterns checklist

## 12) Immediate Next Actions (ordered)

1. Enforce required CI checks in GitHub branch protection for PR hard-gating.
2. Add explicit rollback job/procedure in deploy workflow runbook.
3. Define Phase 3 frontend API consumption contract tests before SvelteKit build-out.
4. Start frontend only after backend hardening gate is green.
