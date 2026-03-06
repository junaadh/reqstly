# Reqstly Frontend (SvelteKit)

This directory contains the Phase 3 frontend rewrite for Reqstly.

## Stack
- SvelteKit + TypeScript
- Bun package manager/runtime
- Tailwind CSS
- shadcn-svelte UI primitives

## Local Development

From repository root:
```bash
./scripts/setup-dev.sh
./scripts/up-dev.sh
```

The frontend is served through Caddy at:
- `https://localhost`

Backend API goes through:
- `https://api.localhost`

Supabase gateway goes through:
- `https://supabase.localhost`

## Common Commands

Inside `frontend/`:
```bash
bun install --frozen-lockfile
bun run dev
bun run check
bun run build
```

## Notes
- In docker dev mode, frontend source and `node_modules` are mounted for auto-reload.
- Auth supports email/password, Microsoft OAuth, and passkey flows (with backend bridge endpoints for local behavior).
