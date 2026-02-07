# Reqstly

A learning project to build an internal request management system. This is a personal project for exploring Docker Swarm, Rust backend development, React frontend, and modern authentication patterns.

## What is Reqstly?

A simple ticketing system where teams can:
- Submit requests (IT, Ops, Admin, HR)
- Track status (Open → In Progress → Resolved)
- View audit history
- Login via Azure AD or Passkeys

This is **not** a production work project - it's a personal learning project to gain experience with:
- Docker Swarm orchestration on a single VPS
- Rust backend with Axum framework
- React frontend with TypeScript and Vite
- PostgreSQL with sqlx (compile-time checked queries)
- Azure AD OIDC integration
- WebAuthn Passkeys implementation
- Full observability stack (Prometheus, Loki, Grafana)

## Architecture

```
Single VPS (Docker Swarm)
├── Caddy (reverse proxy, auto-TLS)
├── Frontend (Vite + TypeScript + React)
├── Backend (Rust, 2 replicas)
├── PostgreSQL
└── Monitoring (Prometheus, Loki, Grafana)
```

## Tech Stack

- **Orchestration**: Docker Swarm (single VPS deployment)
- **Reverse Proxy**: Caddy 2.7 (auto-TLS)
- **Frontend**: Vite + TypeScript + React
- **Backend**: Rust with Axum framework
- **Database**: PostgreSQL 16 with sqlx
- **Auth**: Azure AD SSO + Passkeys (WebAuthn)
- **Observability**: Prometheus + Loki + Grafana
- **CI/CD**: GitHub Actions

## Getting Started

### Prerequisites

- Docker and Docker Swarm
- Domain name pointing to your VPS
- Azure AD tenant (for SSO)
- VPS: 2 vCPUs, 4GB RAM, 80GB SSD

### Quick Start

1. Clone and configure:
   ```bash
   git clone https://github.com/yourusername/reqstly.git
   cd reqstly
   cp .env.example .env
   # Edit .env with your settings
   ```

2. Setup VPS:
   ```bash
   ./scripts/setup-vps.sh user@your-vps-ip
   ```

3. Deploy:
   ```bash
   ./scripts/deploy.sh user@your-vps-ip
   ```

4. Access at `https://your-domain.com`

### Local Development

**Backend (Rust)**
```bash
cd backend
cargo run
```

**Frontend (React)**
```bash
cd frontend
bun install
bun run dev
```

**All Services (Docker)**
```bash
docker-compose -f infra/docker-compose.yml up --build
```

## What I'm Learning

This project helps me explore:

1. **DevOps Patterns**: Docker Swarm, overlay networks, secrets management, rolling updates
2. **Backend Development**: Rust, Axum, async programming, database patterns
3. **Frontend Development**: React, TypeScript, modern tooling
4. **Authentication**: OIDC integration, WebAuthn, session management
5. **Observability**: Metrics, structured logging, dashboards, alerting
6. **Security**: Network isolation, container hardening, secure session handling

## Project Structure

```
reqstly/
├── backend/          # Rust API server
├── frontend/         # React TypeScript frontend
├── infra/            # Docker configs, monitoring
├── scripts/          # Deployment automation
└── docs/             # Planning and documentation
```

## Documentation

- [Implementation Plan](docs/PLAN.md) - 14-day development roadmap

## Status

This is a work in progress. See [docs/PLAN.md](docs/PLAN.md) for current implementation status.

## License

MIT
