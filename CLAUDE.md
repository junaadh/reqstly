# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Reqstly** is an internal request management system designed for single-VPS deployment using Docker Swarm. It provides a lightweight ticketing system where teams can submit requests (IT, Ops, Admin, HR), track status changes, view audit history, and authenticate via Azure AD or Passkeys (WebAuthn).

### Current State

This project is in the **planning phase**. The comprehensive implementation plan is documented in `docs/PLAN.md`. No code has been implemented yet - this is a greenfield project ready for development following the 14-day implementation roadmap.

## Architecture

### Technology Stack

- **Orchestration**: Docker Swarm (single VPS deployment)
- **Reverse Proxy**: Caddy 2.7 (auto-TLS, simple config)
- **Frontend**: Vite + TypeScript + React Compiler
- **Backend**: Rust with Axum framework
- **Database**: PostgreSQL 16 with sqlx ORM (async, compile-time checked queries)
- **Authentication**: Azure AD SSO + Passkeys (WebAuthn)
- **Observability**: Prometheus (metrics) + Grafana (dashboards) + Loki + Promtail (logs)
- **CI/CD**: GitHub Actions

### System Architecture

```
Internet → Caddy (TLS) → Overlay Networks
                           ├─► Frontend (Vite + TypeScript + React Compiler)
                           ├─► Backend (Rust, 2 replicas)
                           ├─► PostgreSQL
                           └─► Grafana (Monitoring)

Prometheus ← Metrics ← All Services
Loki ← Logs ← All Services
```

### Container Allocation

- Caddy: 1 replica (reverse proxy, auto-TLS)
- Frontend: 1 replica (Vite + TypeScript + React Compiler)
- Backend: 2 replicas (Rust API, load balanced)
- PostgreSQL: 1 replica (primary database)
- Monitoring stack: Prometheus, Loki, Promtail, Grafana (1 replica each)

## Repository Structure

```
reqstly/
├── backend/          # Rust API server
│   ├── src/
│   │   ├── main.rs          # API server entry point
│   │   ├── config.rs        # Configuration management
│   │   ├── db.rs            # Database connection pool
│   │   ├── metrics.rs       # Prometheus metrics
│   │   ├── auth/            # Authentication module
│   │   │   ├── azure.rs     # Azure AD OIDC integration
│   │   │   ├── passkey.rs   # WebAuthn implementation
│   │   │   └── session.rs   # Session management
│   │   ├── handlers/        # HTTP request handlers
│   │   │   ├── auth.rs
│   │   │   ├── requests.rs
│   │   │   └── health.rs
│   │   └── models/          # Data models
│   │       ├── user.rs
│   │       ├── request.rs
│   │       └── passkey.rs
│   ├── migrations/
│   │   └── 001_initial_schema.sql
│   ├── Cargo.toml
│   └── Dockerfile
│
├── frontend/         # Vite + TypeScript + React Compiler
│   ├── src/
│   │   ├── App.tsx           # Main React app
│   │   ├── components/       # UI components
│   │   ├── pages/            # Page components
│   │   ├── auth/             # Auth context and hooks
│   │   └── api/              # API client
│   ├── package.json
│   └── Dockerfile
│
├── infra/            # Infrastructure configuration
│   ├── proxy/
│   │   └── caddy/
│   │       └── Caddyfile         # Reverse proxy config (Phase 5)
│   ├── observability/
│   │   ├── grafana/
│   │   │   └── provisioning/     # Auto-provisioned datasources/dashboards
│   │   ├── prometheus/
│   │   │   └── prometheus.yml    # Scrape configs
│   │   ├── loki/
│   │   │   └── loki.yml          # Log retention
│   │   └── promtail/
│   │       └── promtail.yml      # Log collection
│   ├── docker-compose.yml
│   └── README.md                # Infrastructure documentation
│
├── scripts/          # Automation scripts
│   ├── setup-vps.sh         # VPS hardening and setup
│   ├── deploy.sh            # Deployment with health checks
│   ├── rollback.sh          # Emergency rollback
│   └── backup.sh            # PostgreSQL backups
│
├── docs/
│   ├── PLAN.md              # Comprehensive 14-day implementation plan
│   ├── ARCHITECTURE.md      # Technical deep dive (to be created)
│   ├── RUNBOOK.md           # Operations manual (to be created)
│   └── ADR/                 # Architecture Decision Records (to be created)
│
├── .github/
│   └── workflows/
│       ├── ci.yml           # Lint, test, security scan
│       └── deploy.yml       # CD pipeline
│
├── .env.example      # Environment variable template
├── .gitignore
└── README.md         # To be created
```

## Common Development Commands

### Backend (Rust)

```bash
# Build backend
cd backend && cargo build

# Run backend locally (development)
cd backend && cargo run

# Run tests
cd backend && cargo test

# Run specific test
cd backend && cargo test test_name

# Lint with clippy
cd backend && cargo clippy

# Check for security vulnerabilities
cd backend && cargo audit
```

### Frontend (Vite Typescript React)

```bash
# Install dependencies
cd frontend && bun install

# Run development server
cd frontend && bun run dev

# Build for production
cd frontend && bun run build

# Run linter
cd frontend && bun run lint

# Run tests
cd frontend && bun test
```

### Docker Development

```bash
# Build and start all services (development)
docker-compose -f infra/docker-compose.yml up --build

# Build and start monitoring stack
docker-compose -f infra/docker-compose.monitoring.yml up --build

# View logs
docker-compose -f infra/docker-compose.yml logs -f backend

# Stop all services
docker-compose -f infra/docker-compose.yml down
```

### Deployment

```bash
# Deploy to VPS (from local machine)
./scripts/deploy.sh

# Rollback deployment
./scripts/rollback.sh

# Backup PostgreSQL database
./scripts/backup.sh
```

## Key Design Decisions

### Why Single VPS with Docker Swarm?
Demonstrates production DevOps patterns without cloud complexity. Docker Swarm provides orchestration, networking, secrets, and health checks while remaining cost-effective. Mirrors real-world constraints where not every team needs Kubernetes.

### Why Hybrid Authentication (Azure AD + Passkeys)?
- **Azure AD**: Enterprise SSO integration
- **Passkeys**: Passwordless authentication, phishing-resistant, modern WebAuthn
- Supporting both demonstrates ability to integrate legacy (OIDC) and modern (WebAuthn) auth patterns

### Why Caddy Instead of Nginx?
Caddy automates TLS certificate renewal (no certbot cron jobs), has simpler configuration (80% less than Nginx), and embodies DevOps automation philosophy. Eliminates manual certificate management toil.

### Why Rust Backend?
Memory safety without garbage collection overhead. For resource-constrained VPS, every MB counts. Type system catches bugs at compile time, reducing production issues. The `axum` framework provides ergonomic async HTTP handling.

### Why sqlx Over ORM?
sqlx provides async database access with compile-time query verification. Catches SQL errors at build time, not runtime. Lower overhead than heavy ORMs while maintaining type safety.

## Database Schema

### Core Tables

- **users**: User profiles, Azure AD subject mapping
- **requests**: Ticket data (title, description, category, status, priority)
- **passkey_credentials**: WebAuthn credential storage
- **sessions**: Session tokens for authenticated users
- **audit_logs**: Complete audit trail of all state changes

### Key Relationships

- Users have many requests
- Users have many passkey credentials
- Users have many sessions
- Requests have many audit log entries

## Authentication Flow

### Azure AD Flow
1. User clicks "Login with Azure AD"
2. Backend generates OIDC authorization URL
3. User authenticates with Azure AD
4. Azure redirects to backend callback with code
5. Backend exchanges code for JWT
6. Backend validates JWT and creates session
7. Session cookie set (httpOnly, secure)

### Passkey Flow
1. User enters email/username
2. Browser calls WebAuthn `get()` assertion
3. Passkey hardware authenticates user
4. Assertion sent to backend
5. Backend verifies signature against stored credential
6. Session created and cookie set

### Passkey Registration
1. User must first login via Azure AD
2. User opts to register passkey
3. Browser calls WebAuthn `create()` ceremony
4. Credential public key sent to backend
5. Backend stores credential linked to user

## Observability Strategy

### Metrics (Prometheus)
- HTTP request rate, duration, status codes
- Database connection pool stats
- Active sessions count
- Request creation rate by category
- Authentication success/failure rates
- Container resource usage (CPU, memory)

### Logging (Loki)
- Structured JSON logs from all services
- Correlation IDs for request tracing
- Log levels: ERROR, WARN, INFO, DEBUG
- Retention: 30 days default

### Dashboards (Grafana)
- Service Overview: Request rate, error rate, latency percentiles
- Infrastructure: CPU, RAM, disk usage by container
- Business Metrics: Requests by category, resolution time
- Authentication: Login methods, success rates

## Security Considerations

### Network Isolation
- Three overlay networks: public, internal, monitoring
- Database only accessible on internal network
- Monitoring stack isolated on monitoring network

### Secrets Management
- Use Docker secrets for sensitive data
- Never commit secrets to git
- `.env.example` shows required variables without values

### Session Security
- httpOnly cookies prevent XSS access
- Secure flag ensures HTTPS-only transmission
- SameSite flag prevents CSRF
- Session expiration with automatic cleanup

### Database Security
- Prepared statements via sqlx (SQL injection protection)
- Database user limited to minimum required privileges
- Network isolation prevents external access

## Performance Targets

- API response time p95 < 200ms
- Frontend first paint < 2s
- Support 100 concurrent users
- Memory usage stable over 24h (no leaks)

## Scaling Strategy

### Vertical Scaling (Current Phase)
- Upgrade VPS resources (2 vCPUs → 4 vCPUs, 4GB → 16GB RAM)
- Add swap space for memory pressure handling

### Horizontal Scaling (Future)
- Add Swarm nodes for multi-server deployment
- Scale backend replicas based on load
- Add PostgreSQL read replicas for read-heavy workloads
- Add Redis for session storage (shared across instances)

### Migration Path to Cloud
- Managed Kubernetes (EKS/GKE/AKS)
- Managed PostgreSQL (RDS/Cloud SQL)
- CDN for frontend assets (CloudFront/Cloudflare)
- Load balancer (ALB/Cloud Load Balancing)

## Implementation Phases

Refer to `docs/PLAN.md` for detailed 14-day implementation roadmap:

1. **Phase 1 (Days 1-3)**: Foundation - VPS setup, Docker Swarm, database schema
2. **Phase 2 (Days 4-6)**: Core Application - Rust backend, React frontend
3. **Phase 3 (Days 7-8)**: Authentication - Azure AD + Passkeys
4. **Phase 4 (Days 9-10)**: Observability - Metrics, logs, dashboards
5. **Phase 5 (Days 11-12)**: Deployment & Automation - CI/CD, scripts
6. **Phase 6 (Days 13-14)**: Documentation & Polish - README, runbooks, ADRs

## Testing Strategy

- **Unit Tests**: Rust backend model and handler logic
- **Integration Tests**: API endpoint testing
- **E2E Tests**: Critical user flows (login, create request, update status)
- **Security Scanning**: `cargo audit` for dependencies, Trivy for containers
- **Load Testing**: Verify 100 concurrent user target

## Common Workflows

### Adding a New API Endpoint
1. Define handler in `backend/src/handlers/`
2. Add route in `backend/src/main.rs`
3. Create SQL queries in `backend/src/models/` if needed
4. Add corresponding frontend API client in `frontend/src/api/`
5. Build UI components in `frontend/src/components/`
6. Add Prometheus metrics in `backend/src/metrics.rs`
7. Test endpoint locally, then in Docker Compose

### Database Schema Migration
1. Create new migration file in `backend/migrations/`
2. Write SQL DDL changes
3. Test migration locally
4. Update Rust models in `backend/src/models/`
5. Document breaking changes in RUNBOOK.md

### Adding Metrics
1. Define metric in `backend/src/metrics.rs`
2. Increment/expose metric in handlers
3. Add scrape target to `infra/prometheus/prometheus.yml`
4. Create Grafana dashboard panel in `infra/grafana/dashboards/`

## Troubleshooting

### Container Won't Start
```bash
# Check logs
docker service logs reqstly_backend

# Check service state
docker service ps reqstly_backend

# Restart service
docker service update --force reqstly_backend
```

### Database Connection Failed
- Verify PostgreSQL container is running
- Check overlay network connectivity
- Verify DATABASE_URL in environment
- Check connection pool limits

### High Memory Usage
```bash
# Check container stats
docker stats

# Restart service to clear memory
docker service update --force reqstly_backend

# Adjust container memory limits in docker-compose.yml
```

## Important Notes

- This is a greenfield project - follow the plan in `docs/PLAN.md`
- Always implement observability (metrics, logs) alongside features
- Security first: validate input, use prepared statements, isolate networks
- Test authentication flows thoroughly before moving to next phase
- Document architectural decisions in `docs/ADR/`
- Keep deployment simple and automated
- Focus on completion of each phase before starting next
- Target VPS: 2 vCPUs, 4GB RAM, 80GB SSD (resource-constrained)
