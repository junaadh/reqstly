# Reqstly Rewrite Plan (Re-Baselined Phase 5)

## 1) Mission and Phase Order

Reqstly remains a big-bang rewrite with strict phase sequencing:
1. Backend core
2. Backend hardening + tests
3. Frontend rewrite
4. Observability baseline
5. Phase 5 auth/data-platform refactor

## 2) Phase Snapshot (March 7, 2026)

- Phase 0: complete
- Phase 1: complete
- Phase 2: complete
- Phase 3: complete
- Phase 4: complete
- Phase 5: baseline complete (embedded auth)

## 3) Phase 5 Re-Baseline Summary

Phase 5 is now an embedded-auth migration inside the existing Rust/Axum backend.

- Supabase auth: out of baseline scope.
- authentik: out of baseline scope.
- Entra/OIDC runtime: deferred to Phase D.
- Baseline auth methods: email/password + passkeys.
- Canonical identity: `app.app_users` for all business FKs.
- Session model: first-party cookie sessions via `tower-sessions` + SQLx store.
- WebSocket auth target: cookie-first with bearer compatibility via `/api/v1/auth/ws-token`.

## 4) Locked Decisions

1. Auth baseline includes only email/password + passkeys.
2. Entra/OIDC is deferred to Phase D.
3. Session model uses `tower-sessions` with persistent SQLx-backed store in non-test environments.
4. WebSocket auth supports both session cookie and bearer token compatibility.
5. Hard cutover reset strategy remains in force for Phase 5.
6. `app.app_users` is canonical identity for all business FKs.

## 5) Architecture Plan (Modular Monolith)

- Keep current modular monolith and `/api/v1` API surface.
- Add dedicated auth module:
  - `backend/src/auth/{mod.rs,types.rs,errors.rs,repo.rs,password.rs,passkey.rs,session.rs,user_map.rs,middleware.rs,routes.rs,service.rs,rate_limit.rs}`
- Responsibilities:
  - `service.rs`: orchestration
  - `repo.rs`: SQLx queries
  - `routes.rs`: Axum handlers
  - `middleware.rs`: current-user/session extraction
- Business handlers consume app-level identity (`app_user_id`) only.
- Business handlers must not access password hashes or WebAuthn internals.

## 6) Database Plan

### New tables

1. `app.app_users`
2. `app.user_password_identities`
3. `app.user_passkey_credentials`
4. `app.webauthn_challenges`
5. `app.user_oidc_identities` (schema-ready only in baseline)
6. `app.auth_events`
7. Session table from `tower-sessions-sqlx-store` migration

### FK repoint targets

1. `app.requests.owner_user_id -> app.app_users(id)`
2. `app.requests.assignee_user_id -> app.app_users(id)`
3. `app.request_participants.user_id -> app.app_users(id)`
4. `app.request_audit_logs.actor_user_id -> app.app_users(id)`
5. `app.user_preferences.user_id -> app.app_users(id)`

### Index/constraint baseline

- Case-insensitive unique email index on `app.app_users(lower(email)) WHERE email IS NOT NULL`
- Case-insensitive unique login email index on `app.user_password_identities(lower(email))`
- Unique `credential_id` index for passkeys
- Challenge lookup/expiry indexes:
  - `(flow_type, expires_at)`
  - `(user_id, flow_type, created_at DESC)`

## 7) Public API Contracts (Phase 5 Baseline)

### Auth endpoints

1. `POST /api/v1/auth/signup`
2. `POST /api/v1/auth/login/password`
3. `GET /api/v1/auth/csrf`
4. `POST /api/v1/auth/logout`
5. `POST /api/v1/auth/sessions/revoke`
6. `POST /api/v1/auth/ws-token`
7. `GET /api/v1/auth/passkeys`
8. `POST /api/v1/auth/passkeys/register/start`
9. `POST /api/v1/auth/passkeys/register/finish`
10. `POST /api/v1/auth/passkeys/signup/start`
11. `POST /api/v1/auth/passkeys/signup/finish`
12. `POST /api/v1/auth/passkeys/login/start`
13. `POST /api/v1/auth/passkeys/login/finish`
14. `GET /api/v1/me`

### WebSocket contract

- Endpoint remains `/ws`.
- Auth acceptance:
  - session cookie
  - `Authorization: Bearer <token>`
  - `?token=<token>` for compatibility
- Bearer tokens must be minted by `/api/v1/auth/ws-token` and match active issuance records.
- Phase 5 frontend target is cookie-first path.

## 8) Security Baseline

1. Password hashing: `password-auth::generate_hash`, verification: `password-auth::verify_password`.
2. Generic login failure for invalid credentials.
3. Passkey challenge TTL: 5 minutes, one-time consumption required.
4. Session rotation on successful signup/login/passkey login.
5. Cookie defaults: `HttpOnly`, `Secure` (non-local), `SameSite=Lax`, explicit inactivity expiry.
6. CSRF controls for session-mutating browser endpoints (origin checks + CSRF token path).
7. Rate-limit hook points on signup/login/passkey start+finish.
8. Auth audit events for success/failure/logout/passkey enrollment.
9. No plaintext password logging or storage.
10. Preserve consistent `401/403/422/429` response-envelope behavior.

## 9) Migration Outcomes

1. Base migrations were rewritten for hard cutover reset and no longer depend on `auth.users`, `auth.uid()`, or Supabase realtime assumptions.
2. Supabase compatibility bootstrap was removed from integration tests.
3. Auth tables + session store migrations are part of the main migration chain.
4. SQLx queries previously coupled to `auth.users` now resolve identity via `app.app_users`.
5. Business handler auth resolution uses session/current-user extraction instead of Supabase JWT assumptions.
6. Script entrypoints remain canonical:
   - `./scripts/up-dev.sh`
   - `./scripts/reset-dev-db.sh`
   - `./scripts/smoke-check.sh`

## 10) Implementation Phases

### Phase A: Schema + Password + Session

Status: complete.

### Phase B: Passkeys

Status: complete.

### Phase C: Compatibility + Cleanup

Status: complete.
Implemented outcomes:
1. ws token mint endpoint + dual-mode ws auth.
2. Supabase/authentik auth runtime removal from backend/frontend paths.
3. CSRF/origin + rate-limit + auth-security hardening enabled.
4. OIDC extension points kept inert and compile-ready.

### Phase D: Optional OIDC Later

1. Add openidconnect runtime wiring and callback handlers.
2. Activate `user_oidc_identities` linking rules.
3. Add Entra-specific docs/tests only when feature is enabled.

## 11) Test and Acceptance Scenarios

1. Migration-from-empty succeeds without Supabase/authentik schema dependencies.
2. Password signup creates `app_users` + `user_password_identities` + session.
3. Password login validates success/failure and generic errors.
4. `/api/v1/me` resolves session user and enforces inactive-user behavior.
5. Passkey register flow requires authenticated user and one-time challenges.
6. Passkey login flow validates challenge/origin/rp/counter and creates session.
7. WS auth works with session cookie and minted ws bearer token.
8. Business endpoints enforce ownership/participant rules with `app_users.id`.
9. Rate-limit + CSRF checks return correct envelope/status.
10. Auth audit events are written for success/failure paths.
11. Frontend auth works without `supabase-js` and without `/auth/v1/*`.

## 12) Risks and Mitigations

1. Hidden Supabase coupling in tests/scripts.
- Mitigation: grep checks + migration-from-empty + smoke checks.

2. Session rollout regressions in browser flows.
- Mitigation: `/api/v1/me` bootstrap tests + CSRF/origin integration tests.

3. Passkey replay/state handling bugs.
- Mitigation: one-time challenge consumption, expiry, and state-owner checks.

## 13) Completion Status

Phase 5 baseline criteria are met in-repo:

1. Checklist IDs in `docs/improvements.md` are completed for baseline scope.
2. Supabase/authentik auth runtime dependencies are removed from active path.
3. `app.app_users` is canonical identity and business FK target.
4. Password + passkey flows are implemented and tested.
5. Baseline scripts/smoke flow run without deprecated auth dependencies.

Phase D (OIDC/Entra runtime) remains intentionally deferred.

## 14) Context7-Validated References

- `webauthn-rs` start/finish flow and server-side state persistence requirements:
  - https://docs.rs/webauthn-rs/latest/webauthn_rs/index.html
  - https://docs.rs/webauthn-rs/latest/webauthn_rs/prelude/index.html
- `tower-sessions` Axum middleware + SQLx store usage pattern:
  - https://github.com/maxcountryman/tower-sessions/blob/main/README.md
  - https://github.com/maxcountryman/tower-sessions/blob/main/CHANGELOG.md
- `password-auth` hash/verify behavior:
  - https://github.com/rustcrypto/password-hashes/blob/master/password-auth/README.md
