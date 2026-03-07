# Phase 5 Execution Checklist (Re-Baseline)

This is the canonical Phase 5 migration checklist for embedded Axum auth.

## Locked Baseline

- No Supabase auth in baseline runtime.
- No authentik in baseline runtime.
- Baseline auth methods: email/password + passkeys.
- OIDC/Entra runtime deferred to Phase D.
- `app.app_users` is the only business identity FK target.
- Hard cutover reset strategy remains active.

## Canonical Checklist IDs

| Status | ID | Track | Task | Acceptance Criteria |
|---|---|---|---|---|
| [ ] | P5-RB-01 | Re-Baseline | Re-baseline docs from authentik plan to embedded auth plan | PLAN/improvements/frontend/README/infra/AGENTS aligned with embedded-auth scope |
| [ ] | P5-DB-01 | Database | Add auth tables (`app_users`, password identities, passkey credentials, challenges, auth events, oidc identities, sessions) and hardening state tables (`user_auth_security`, `auth_rate_limit_buckets`, `csrf_tokens`, `ws_token_issuances`) | Migrations apply from empty DB; required indexes/constraints present; revocation/session/rate-limit primitives exist |
| [ ] | P5-DB-02 | Database | Repoint business FKs to `app_users` | No business FK remains against `auth.users` or provider tables |
| [ ] | P5-DB-03 | Database | Remove Supabase schema/runtime coupling and RLS/publication assumptions | No migration/query requires `auth.uid()` or Supabase realtime artifacts |
| [ ] | P5-AUTH-01 | Backend Auth | Add `tower-sessions` middleware and current-user extraction | Protected handlers resolve current user from session (with defined bearer compatibility path) |
| [ ] | P5-AUTH-02 | Backend Auth | Implement password signup/login/logout/me | Endpoints behave per envelope contract; generic credential mismatch errors |
| [ ] | P5-AUTH-03 | Backend Auth | Implement passkey register/login start+finish | One-time expiring challenge lifecycle + credential persistence validated |
| [ ] | P5-AUTH-04 | Backend Auth | Add ws dual auth mode and `/api/v1/auth/ws-token` | `/ws` works via session cookie and ws bearer token compatibility |
| [ ] | P5-FE-01 | Frontend | Remove `supabase-js` and Supabase auth endpoint usage | Frontend has no auth dependency on Supabase SDK or `/auth/v1/*` |
| [ ] | P5-FE-02 | Frontend | Rewire login/signup/profile passkey UX to backend auth endpoints | Email/password/passkey flows work with Reqstly-owned contracts |
| [ ] | P5-INFRA-01 | Infrastructure | Remove authentik assumptions from compose/env docs and smoke checks | Infra docs/scripts define embedded-auth stack only |
| [ ] | P5-INFRA-02 | Infrastructure | Keep script orchestration unchanged (`up-dev`, `reset-dev-db`, `smoke-check`) | Existing script entrypoints remain canonical and documented |
| [ ] | P5-CLEAN-01 | Cleanup | Delete Supabase/authentik auth leftovers in code/docs/scripts | No active auth docs/scripts reference deprecated providers |
| [ ] | P5-CLEAN-02 | Cleanup | Full consistency pass across Phase 5 docs | No contradictory architecture/auth statements remain |

## Baseline Public Interfaces

### Canonical identity table

- `app.app_users` is the canonical application identity table.
- All business ownership/participant/audit relationships target `app.app_users(id)`.

### Baseline auth endpoints

1. `POST /api/v1/auth/signup`
2. `POST /api/v1/auth/login/password`
3. `POST /api/v1/auth/logout`
4. `GET /api/v1/me`
5. `POST /api/v1/auth/passkeys/register/start`
6. `POST /api/v1/auth/passkeys/register/finish`
7. `POST /api/v1/auth/passkeys/login/start`
8. `POST /api/v1/auth/passkeys/login/finish`
9. `POST /api/v1/auth/ws-token`

## Acceptance Scenarios

1. Password signup/login create and restore session correctly.
2. Passkey registration/login works with one-time challenge consumption.
3. Invalid issuer/audience/signature/expiry paths return consistent `401` envelopes.
4. Business endpoints continue functioning with `app_users` identity.
5. WS auth works via cookie and via minted ws bearer token.
6. CI/smoke checks pass without Supabase/authentik auth dependencies.

## Deferred Scope (Explicit)

- OIDC/Entra runtime integration (Phase D).
- Entra tenant configuration and group sync.
- Email verification enforcement.
