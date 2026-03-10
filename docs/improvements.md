# Phase 5 Execution Checklist (Embedded Auth Baseline)

This checklist tracks the Phase 5 re-baseline from Supabase/authentik auth to embedded Axum auth.

Status as of March 7, 2026: baseline complete and parity achieved for core auth/business flows.

## Locked Baseline

- No Supabase auth runtime dependency.
- No authentik runtime dependency.
- Baseline auth methods: email/password + passkeys.
- OIDC/Entra runtime deferred to Phase D.
- `app.app_users` is the only business identity FK target.
- Hard cutover reset strategy used for migration.

## Canonical Checklist IDs

| Status | ID | Track | Task | Acceptance Criteria |
|---|---|---|---|---|
| [x] | P5-RB-01 | Re-Baseline | Re-baseline docs from authentik plan to embedded auth plan | PLAN/improvements/frontend/README/infra/AGENTS aligned with embedded-auth scope |
| [x] | P5-DB-01 | Database | Add auth tables (`app_users`, password identities, passkey credentials, challenges, auth events, oidc identities, sessions) and hardening state tables (`user_auth_security`, `auth_rate_limit_buckets`, `csrf_tokens`, `ws_token_issuances`) | Migrations apply from empty DB; required indexes/constraints present; revocation/session/rate-limit primitives exist |
| [x] | P5-DB-02 | Database | Repoint business FKs to `app_users` | No business FK remains against `auth.users` or provider tables |
| [x] | P5-DB-03 | Database | Remove Supabase schema/runtime coupling and RLS/publication assumptions | No migration/query requires `auth.uid()` or Supabase realtime artifacts |
| [x] | P5-AUTH-01 | Backend Auth | Add `tower-sessions` middleware and current-user extraction | Protected handlers resolve current user from session with ws bearer compatibility |
| [x] | P5-AUTH-02 | Backend Auth | Implement password signup/login/logout/me | Endpoints behave per envelope contract; generic credential mismatch errors |
| [x] | P5-AUTH-03 | Backend Auth | Implement passkey register/login start+finish | One-time expiring challenge lifecycle + credential persistence validated |
| [x] | P5-AUTH-04 | Backend Auth | Add ws dual auth mode and `/api/v1/auth/ws-token` | `/ws` works via session cookie and ws bearer token compatibility |
| [x] | P5-FE-01 | Frontend | Remove `supabase-js` and Supabase auth endpoint usage | Frontend has no auth dependency on Supabase SDK or `/auth/v1/*` |
| [x] | P5-FE-02 | Frontend | Rewire login/signup/profile passkey UX to backend auth endpoints | Email/password/passkey flows work with Reqstly-owned contracts |
| [x] | P5-INFRA-01 | Infrastructure | Remove authentik assumptions from compose/env docs and smoke checks | Infra docs/scripts define embedded-auth stack only |
| [x] | P5-INFRA-02 | Infrastructure | Keep script orchestration unchanged (`up-dev`, `reset-dev-db`, `smoke-check`) | Existing script entrypoints remain canonical and documented |
| [x] | P5-CLEAN-01 | Cleanup | Delete Supabase/authentik auth leftovers in code/docs/scripts | No active auth docs/scripts reference deprecated providers |
| [x] | P5-CLEAN-02 | Cleanup | Full consistency pass across Phase 5 docs | No contradictory architecture/auth statements remain |

## Baseline Public Interfaces

### Canonical identity

- `app.app_users` is canonical application identity.
- All business ownership/participant/audit relationships target `app.app_users(id)`.

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

## Deferred Scope (Explicit)

- OIDC/Entra runtime integration (Phase D).
- Entra tenant configuration and group sync.
- Email verification enforcement.
