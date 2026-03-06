# Reqstly Improvements Before Phase 3 Close

## Purpose
This document captures high-impact improvements and additional features to finish Phase 3 cleanly and reduce carry-over risk into Phase 4.

Scope assumptions:
- Backend, frontend, DB, API, security, and deployment hardening are in scope.
- Observability stack rollout itself remains Phase 4, but Phase-3-ready hooks are included.

## Current Snapshot
- Backend `/api/v1` + websocket `/ws` are implemented.
- Participant visibility model exists via `app.request_participants`.
- Frontend routes and realtime patching are in place.
- Passkey + email/password + Microsoft flows are integrated.
- CI currently enforces backend checks; frontend checks are still missing.
- Root onboarding docs are now aligned with current frontend status and local setup flow.

## Priority 0: Must Complete Before Declaring Phase 3 Done

| ID | Area | Improvement | Why | Acceptance Criteria |
|---|---|---|---|---|
| P0-01 | CI/CD | Add frontend gates to CI (`bun install`, `bun run check`, `bun run build`) | Prevent frontend regressions on PRs and branch pushes | CI fails on broken Svelte/TypeScript/build; green baseline on `rewrite/sveltekit` |
| P0-02 | Docs | Update `README.md` to reflect actual state (frontend present, phase status current) | Avoid operational confusion and incorrect onboarding | README matches `docs/PLAN.md` and active compose commands |
| P0-03 | Auth | Stabilize session-expired flow and refresh behavior end-to-end | Current redirect/refresh path has been fragile under CORS/session drift | `?reason=session-expired` recovery works without manual storage cleanup across Safari/Chromium |
| P0-04 | Security | Add regression test for auth CORS headers (`/auth/v1/token`, `/auth/v1/user`) | Prevent wildcard CORS regressions on credentialed requests | Automated test asserts explicit origin + `allow-credentials=true` |
| P0-05 | API | Implement `PATCH /api/v1/me` for profile persistence | Profile Save currently depends on auth metadata update path only | Display name update goes through backend API and is reflected in `/api/v1/me` |
| P0-06 | API | Implement `/api/v1/preferences` GET/PATCH | Settings are currently browser-local only | Preferences persist server-side and hydrate on all devices |
| P0-07 | API+DB | Add server-side request search (`q`) to `GET /api/v1/requests` | Current search is page-local in UI; misses non-loaded records | Query returns correct cross-page matches with filters/sort/pagination |
| P0-08 | Realtime | Add deterministic resync contract on reconnect | Current fallback invalidate loop is coarse | Reconnect triggers targeted resync for active view with no stale list/detail mismatch |
| P0-09 | Realtime+Tests | Add integration tests for assign/delete/status patch fanout | Collaboration correctness is critical and historically error-prone | Tests prove owner/assignee/participant visibility and live update behavior |
| P0-10 | UX/A11Y | Resolve remaining accessibility gaps in auth/list/detail screens | Phase close should include baseline usability quality | Keyboard nav and ARIA checks pass for comboboxes, dialogs, and action menus |

## Priority 1: High-Value Improvements to Pull Into Late Phase 3

| ID | Area | Improvement | Why | Acceptance Criteria |
|---|---|---|---|---|
| P1-01 | DB Design | Add trigram search indexes for request text search (title/description) | Keeps `q` search fast as dataset grows | Search query plan uses index; p95 search latency within target |
| P1-02 | DB Design | Revisit participant source model (`owner/assignee/actor`) to preserve multi-source semantics | Current single `source` value can lose provenance | Model supports user being owner+actor+assignee without losing data fidelity |
| P1-03 | API Design | Add cursor pagination option for large request lists | Offset pagination degrades at high offsets | Cursor mode available and documented; UI can opt-in |
| P1-04 | Rust/API | Standardize error codes and field-level detail consistency across all handlers | Cleaner client behavior and easier test assertions | Error envelope parity test covers all main endpoint families |
| P1-05 | Security | Introduce auth endpoint rate limits (login/passkey/otp) | Reduces brute-force and abuse risk | Configurable per-route limits with clear 429 responses |
| P1-06 | Security | Add CSRF strategy for state-changing form actions when cookie auth is present | Reduces cross-site submission risk | CSRF token or equivalent protection on POST/PATCH/DELETE form flows |
| P1-07 | Frontend | Replace native `<select>` controls with accessible custom select where needed | Better interaction consistency and design cohesion | Keyboard and screen reader behavior matches design system baseline |
| P1-08 | Frontend | Add optimistic UI state + rollback for request updates | Better perceived performance and fewer jumpy transitions | Edit and assignment changes feel instant and reconcile correctly on failure |
| P1-09 | Performance | Add request-list virtualization trigger for large pages | Avoid UI jank on dense datasets | Smooth scroll and stable interaction on 500+ rows |
| P1-10 | DevOps | Extend smoke checks to include auth + realtime scenario | Current smoke covers service health only | Smoke validates login, request create, realtime patch receive, delete |

## Priority 2: Additional Product Features Worth Starting

| ID | Area | Feature | Value | Acceptance Criteria |
|---|---|---|---|---|
| P2-01 | Requests | Request comments thread per ticket | Collaboration context beyond audit diffs | CRUD comments with participant visibility and realtime append |
| P2-02 | Requests | Attachment support (metadata first, file transport later) | Improves operational usefulness | Upload/list/delete metadata path and permission checks |
| P2-03 | Workflow | SLA fields (`due_at`, breach indicator, aging views) | Better prioritization and accountability | SLA badges + overdue filters on dashboard/list |
| P2-04 | Dashboard | Saved views/filters per user | Faster repeated workflows | User can save, rename, and reuse list filter presets |
| P2-05 | Notifications | In-app notification center for assignments/status changes | Reduces missed updates | Notification feed with read/unread state and deep links |
| P2-06 | Team | Domain workspace management and member directory API | Better assignee discovery and policy control | Directory endpoint + role-aware assignment controls |

## Execution Order Recommendation

1. Ship all `P0` items first and re-run full CI + smoke.
2. Pull `P1-01`, `P1-04`, `P1-05`, `P1-10` next for stability.
3. Pull the remaining `P1` items based on capacity.
4. Start `P2` only after Phase 3 exit gate is formally met.

## Phase 3 Exit Definition (Proposed)

Phase 3 can be marked complete when all are true:

1. `P0-01` through `P0-10` are done and merged.
2. CI is green on backend and frontend checks.
3. Staging deploy + smoke pass on the same commit.
4. `docs/frontend_functionality.md`, `docs/PLAN.md`, and `README.md` are consistent.
5. Realtime collaboration scenarios pass automated integration tests.
