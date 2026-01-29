# Reqstly - Project Plan

**Vision**: Internal request management system on a single VPS using Docker Swarm with modern authentication

**Timeline**: 14 days  
**Deployment**: Single VPS with Docker Swarm  
**Authentication**: Azure AD SSO + Passkeys (WebAuthn)

---

## Executive Summary

### What is Reqstly?
A lightweight internal ticketing system where teams can:
- Submit requests (IT, Ops, Admin, HR)
- Track status (Open → In Progress → Resolved)
- View audit history
- Login via Azure AD or Passkeys

### Why This Stack?
- **Single VPS**: Cost-effective, realistic for small/medium teams
- **Docker Swarm**: Production orchestration without K8s overhead
- **Hybrid Auth**: Enterprise SSO + modern passwordless experience
- **Full Observability**: Metrics, logs, dashboards from day one

### Target VPS Specs
- **Provider**: DigitalOcean, Hetzner, Vultr (~$8/month)
- **Resources**: 2 vCPUs, 4GB RAM, 80GB SSD
- **OS**: Ubuntu 24.04 LTS

---

## Architecture Overview

```
┌─────────────────────────────────────────────┐
│  VPS (Single Server - Docker Swarm)         │
│                                             │
│  Internet → Caddy (TLS) → Overlay Networks  │
│                 │                           │
│                 ├─► Frontend (Vite + TypeScript + React Compiler)        │
│                 ├─► Backend (Rust)          │
│                 ├─► PostgreSQL              │
│                 └─► Grafana (Monitoring)    │
│                                             │
│  Prometheus ← Metrics ← All Services        │
│  Loki ← Logs ← All Services                 │
└─────────────────────────────────────────────┘
```

### Container Allocation
| Service | Replicas | Purpose |
|---------|----------|---------|
| Caddy | 1 | Reverse proxy, auto-TLS, metrics |
| Frontend | 1 | Vite + TypeScript + React Compiler |
| Backend | 2 | Rust API (load balanced) |
| PostgreSQL | 1 | Primary database |
| Prometheus | 1 | Metrics collection |
| Loki | 1 | Log aggregation |
| Promtail | 1 | Log shipping |
| Grafana | 1 | Visualization |

---

## Technology Stack

| Layer | Technology | Why |
|-------|-----------|-----|
| **Orchestration** | Docker Swarm | Built-in, no K8s complexity |
| **Reverse Proxy** | Caddy 2.7 | Auto-HTTPS, simple config |
| **Frontend** | Vite + TypeScript + React Compiler | Modern, type-safe SPA with optimizations |
| **Backend** | Rust (Axum) | Fast, safe, low memory |
| **Database** | PostgreSQL 16 | ACID, proven |
| **ORM** | sqlx | Async, compile-time checks |
| **Auth** | Azure AD + Passkeys | SSO + passwordless |
| **Metrics** | Prometheus | Industry standard |
| **Logs** | Loki + Promtail | Lightweight aggregation |
| **Dashboards** | Grafana | Unified observability |
| **CI/CD** | GitHub Actions | Free, integrated |

---

## Phase Breakdown

### Phase 1: Foundation (Days 1-3)
**Goal**: Secure VPS with Docker Swarm ready

**Deliverables**:
- [ ] VPS provisioned and hardened (UFW, fail2ban, SSH keys)
- [ ] Docker + Docker Swarm initialized
- [ ] Overlay networks created (public, internal, monitoring)
- [ ] Docker secrets initialized
- [ ] GitHub repository with clean structure
- [ ] Database schema designed (users, requests, passkeys, sessions, audit_logs)
- [ ] `.env.example` and `.gitignore` configured

**Key Files**:
- `scripts/setup-vps.sh` - VPS hardening automation
- `backend/migrations/001_initial_schema.sql` - Database DDL
- `.env.example` - All required environment variables

---

### Phase 2: Core Application (Days 4-6)
**Goal**: Working API and basic frontend

**Deliverables**:
- [ ] Rust backend with Axum framework
- [ ] Database models (User, Request, Session, PasskeyCredential)
- [ ] CRUD handlers for requests
- [ ] Health check endpoint
- [ ] Vite Typescript React frontend scaffolded
- [ ] Basic UI components (login, request list, create form)
- [ ] Dockerfiles for backend and frontend
- [ ] Docker Compose stack definition

**Key Files**:
- `backend/Cargo.toml` - Rust dependencies
- `backend/src/main.rs` - API server
- `backend/src/models/*.rs` - Data models
- `backend/src/handlers/*.rs` - HTTP handlers
- `frontend/package.json` - Node dependencies
- `frontend/src/App.tsx` - Main Vite Typescript React app
- `infra/docker-compose.yml` - Swarm stack definition

---

### Phase 3: Authentication (Days 7-8)
**Goal**: Both auth methods working

**Deliverables**:
- [ ] Azure AD OIDC integration
- [ ] JWT token validation
- [ ] Passkey registration flow (WebAuthn)
- [ ] Passkey authentication flow
- [ ] Session management (httpOnly cookies)
- [ ] Auth middleware for protected routes
- [ ] Frontend auth context and hooks

**Key Files**:
- `backend/src/auth/azure.rs` - Azure AD integration
- `backend/src/auth/passkey.rs` - WebAuthn implementation
- `backend/src/auth/session.rs` - Session management
- `frontend/src/auth/AuthContext.tsx` - Auth state management
- `frontend/src/auth/usePasskey.ts` - Passkey hooks

**Testing Checklist**:
- [ ] Can login with Azure AD
- [ ] Can register a passkey after Azure login
- [ ] Can login with passkey on subsequent visits
- [ ] Sessions expire correctly
- [ ] Logout works for both methods

---

### Phase 4: Observability (Days 9-10)
**Goal**: Full visibility into system health

**Deliverables**:
- [ ] Prometheus scraping all services
- [ ] Structured JSON logging (backend)
- [ ] Loki collecting all logs
- [ ] Grafana dashboards:
  - Service overview (requests/sec, errors, latency)
  - Infrastructure (CPU, RAM, disk)
  - Business metrics (requests by category, resolution time)
- [ ] Basic alerts configured
- [ ] Caddy metrics endpoint exposed

**Key Files**:
- `infra/prometheus/prometheus.yml` - Scrape configs
- `infra/prometheus/alerts.yml` - Alert rules
- `infra/loki/loki.yml` - Log retention config
- `infra/promtail/promtail.yml` - Log collection
- `infra/grafana/provisioning/` - Auto-provisioning
- `infra/grafana/dashboards/*.json` - Dashboard definitions
- `backend/src/metrics.rs` - Prometheus metrics

**Metrics to Track**:
- HTTP request rate, duration, status codes
- Database connection pool stats
- Active sessions
- Request creation rate
- Auth success/failure rates

---

### Phase 5: Deployment & Automation (Days 11-12)
**Goal**: One-command deployment with rollback

**Deliverables**:
- [ ] Caddyfile for reverse proxy (auto-HTTPS)
- [ ] Docker Swarm stack tested locally
- [ ] Deployment script with health checks
- [ ] Rollback script
- [ ] Backup script for PostgreSQL
- [ ] GitHub Actions CI/CD pipeline:
  - Linting (Rust clippy, ESLint)
  - Testing (unit tests)
  - Security scanning (cargo audit, Trivy)
  - Build and push Docker images
  - Deploy to VPS (on main branch)
- [ ] Secrets management (Docker secrets)

**Key Files**:
- `infra/caddy/Caddyfile` - Reverse proxy config
- `scripts/deploy.sh` - Deployment automation
- `scripts/rollback.sh` - Emergency rollback
- `scripts/backup.sh` - Database backups
- `.github/workflows/ci.yml` - CI pipeline
- `.github/workflows/deploy.yml` - CD pipeline

**Deployment Flow**:
1. Developer pushes to `main`
2. GitHub Actions: lint → test → build → push images
3. SSH to VPS
4. Run `docker stack deploy`
5. Health check passes → success
6. Health check fails → automatic rollback

---

### Phase 6: Documentation & Polish (Days 13-14)
**Goal**: Interview-ready project

**Deliverables**:
- [ ] README.md with:
  - Project overview
  - Architecture diagram
  - Quick start guide
  - Deployment instructions
- [ ] ARCHITECTURE.md - Deep dive on design decisions
- [ ] RUNBOOK.md - Operational procedures:
  - How to deploy
  - How to rollback
  - How to backup/restore
  - How to scale
  - Common issues and fixes
- [ ] ADRs (Architecture Decision Records):
  - Why single-VPS Swarm?
  - Why Caddy over Nginx?
  - Why hybrid auth (Azure + Passkeys)?
  - Why Rust backend?
- [ ] Demo script for interviews
- [ ] Code cleanup (remove TODOs, add comments)
- [ ] Security audit checklist

**Key Files**:
- `README.md` - Main documentation
- `docs/ARCHITECTURE.md` - Technical deep dive
- `docs/RUNBOOK.md` - Operations manual
- `docs/ADR/*.md` - Decision records
- `scripts/demo.sh` - Interview demo

---

## Success Criteria

### Technical Validation
- [ ] All containers healthy in Swarm
- [ ] Can login with Azure AD
- [ ] Can register and use passkey
- [ ] Can create/update/view requests
- [ ] Audit logs capture all changes
- [ ] Metrics visible in Grafana
- [ ] Logs searchable in Loki
- [ ] TLS certificates auto-renew
- [ ] Health checks trigger container restart
- [ ] Rolling updates work without downtime
- [ ] Backup/restore tested

### Security Checklist
- [ ] No root login via SSH
- [ ] Firewall enabled (only 22, 80, 443)
- [ ] Fail2ban protecting SSH
- [ ] Docker secrets for sensitive data
- [ ] Sessions use httpOnly cookies
- [ ] HTTPS enforced (HSTS headers)
- [ ] WebAuthn credentials stored securely
- [ ] Database network is isolated
- [ ] No secrets in git history

### Performance Targets
- [ ] API response time p95 < 200ms
- [ ] Frontend first paint < 2s
- [ ] Can handle 100 concurrent users
- [ ] Memory usage stable over 24h

### Documentation Quality
- [ ] Non-technical person can understand README
- [ ] Another engineer can deploy from docs
- [ ] All design decisions explained
- [ ] Interview demo takes < 5 minutes

---

## Interview Preparation

### Demo Flow (5 minutes)
1. **Architecture Overview** (30s)
   - Show diagram: single VPS, Docker Swarm, all components
   
2. **Authentication Demo** (1.5 min)
   - Login with Azure AD
   - Register a passkey
   - Logout and login with passkey
   
3. **Core Functionality** (1 min)
   - Create a request
   - Show in database (via Grafana query or direct SQL)
   - Update status
   - View audit log
   
4. **Observability** (1.5 min)
   - Open Grafana dashboard
   - Show live metrics (request rate, latency)
   - Search logs in Loki
   - Show alert configuration
   
5. **Deployment** (30s)
   - Trigger GitHub Actions workflow
   - Show rolling update in progress
   - Health check passes

### Key Talking Points

**Why Single VPS?**
> "I wanted to demonstrate production patterns without cloud complexity. Docker Swarm provides orchestration, networking, secrets, and health checks—all the essentials—while staying cost-effective. This mirrors real-world constraints where not every team has Kubernetes."

**Why Hybrid Auth?**
> "Azure AD provides enterprise SSO, but passkeys eliminate password fatigue for returning users. WebAuthn is cryptographically secure, phishing-resistant, and the future of authentication. Supporting both shows I can integrate legacy and modern patterns."

**Why Caddy?**
> "Caddy automates TLS—no certbot cron jobs, no manual renewal. It embodies DevOps philosophy: eliminate toil through automation. The configuration is also 80% smaller than Nginx, reducing complexity and failure modes."

**Why Rust?**
> "Rust provides memory safety without garbage collection overhead. For a resource-constrained VPS, every MB counts. The type system also catches bugs at compile time, reducing production issues."

**How Would You Scale This?**
> "Vertical: Upgrade VPS to 16GB RAM. Horizontal: Add Swarm nodes, scale backend replicas, add PostgreSQL read replicas. Eventually: Move to managed K8s, add Redis for sessions, CDN for frontend assets. But start simple, scale when metrics prove you need it."

**Disaster Recovery?**
> "Daily automated backups to S3-compatible storage. All infrastructure is in code (Docker Compose, Caddyfile). RTO: 30 minutes to rebuild from scratch. RPO: 24 hours (daily backups). For critical production, I'd add WAL archiving for point-in-time recovery."

### Questions to Expect

**Q: Why not Kubernetes?**
> "K8s is powerful but overkill here. Swarm provides 80% of the orchestration features with 20% of the complexity. For a single-server deployment, Swarm's simplicity is a feature, not a limitation."

**Q: What about database replication?**
> "I kept PostgreSQL single-instance to focus on the DevOps patterns around it—backups, monitoring, connection pooling. In production, I'd add streaming replication with PgBouncer for connection pooling and automatic failover."

**Q: Security concerns with VPS?**
> "Defense in depth: hardened SSH, firewall, fail2ban, network isolation, secrets management, audit logging, HTTPS everywhere, automated security updates. The attack surface is minimal—only ports 80/443 exposed, everything else firewalled."

**Q: How do you monitor costs?**
> "VPS is fixed cost ($45/month). For scaling, I'd track cost-per-request using Prometheus metrics. If traffic grows, compare VPS scaling vs cloud migration economics. Sometimes staying on VPS is cheaper than equivalent cloud services."

---

## Repository Structure

```
reqstly/
├── backend/
│   ├── src/
│   │   ├── main.rs
│   │   ├── config.rs
│   │   ├── db.rs
│   │   ├── metrics.rs
│   │   ├── auth/
│   │   │   ├── mod.rs
│   │   │   ├── azure.rs
│   │   │   ├── passkey.rs
│   │   │   └── session.rs
│   │   ├── handlers/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs
│   │   │   ├── requests.rs
│   │   │   └── health.rs
│   │   └── models/
│   │       ├── mod.rs
│   │       ├── user.rs
│   │       ├── request.rs
│   │       └── passkey.rs
│   ├── migrations/
│   │   └── 001_initial_schema.sql
│   ├── Cargo.toml
│   └── Dockerfile
│
├── frontend/
│   ├── src/
│   │   ├── App.tsx
│   │   ├── components/
│   │   ├── pages/
│   │   ├── auth/
│   │   └── api/
│   ├── package.json
│   └── Dockerfile
│
├── infra/
│   ├── caddy/
│   │   └── Caddyfile
│   ├── prometheus/
│   │   ├── prometheus.yml
│   │   └── alerts.yml
│   ├── loki/
│   │   └── loki.yml
│   ├── promtail/
│   │   └── promtail.yml
│   ├── grafana/
│   │   ├── provisioning/
│   │   └── dashboards/
│   ├── docker-compose.yml
│   └── docker-compose.monitoring.yml
│
├── scripts/
│   ├── setup-vps.sh
│   ├── deploy.sh
│   ├── rollback.sh
│   └── backup.sh
│
├── docs/
│   ├── ARCHITECTURE.md
│   ├── RUNBOOK.md
│   └── ADR/
│
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── deploy.yml
│
├── .env.example
├── .gitignore
├── README.md
└── PLAN.md
```

---

## Risk Mitigation

### Technical Risks

**Risk**: VPS runs out of memory
- **Mitigation**: Container resource limits, Prometheus alerts, swap space
- **Contingency**: Vertical scaling (upgrade VPS plan)

**Risk**: Database corruption
- **Mitigation**: Daily backups, WAL archiving, transaction logs
- **Contingency**: Restore from backup (documented in runbook)

**Risk**: TLS certificate renewal fails
- **Mitigation**: Caddy automatic renewal, monitoring cert expiry
- **Contingency**: Manual Let's Encrypt renewal procedure

**Risk**: Swarm manager node fails
- **Mitigation**: Automated backups, health monitoring
- **Contingency**: Rebuild from infra code + restore DB backup

### Timeline Risks

**Risk**: Authentication takes longer than 2 days
- **Mitigation**: Start with Azure AD only, add passkeys later
- **Buffer**: Can reduce observability scope if needed

**Risk**: Unfamiliar with WebAuthn
- **Mitigation**: Use `webauthn-rs` crate (batteries included)
- **Fallback**: Document passkey integration as "future work"

---

## Post-Launch Roadmap

### Phase 7: Enhancements (Optional)
- [ ] Email notifications for request updates
- [ ] Request comments/discussion
- [ ] File attachments
- [ ] Request templates
- [ ] SLA tracking
- [ ] Mobile app (React Native)

### Phase 8: Advanced Features (Optional)
- [ ] Multi-tenancy support
- [ ] Advanced analytics dashboard
- [ ] Slack/Teams integration
- [ ] API rate limiting
- [ ] Elasticsearch for full-text search
- [ ] PostgreSQL read replicas

---

## Time Estimates

| Phase | Days | Hours | Focus |
|-------|------|-------|-------|
| Phase 1 | 3 | 24 | VPS setup, database schema |
| Phase 2 | 3 | 24 | Core app (backend + frontend) |
| Phase 3 | 2 | 16 | Authentication (Azure + Passkeys) |
| Phase 4 | 2 | 16 | Observability (metrics + logs) |
| Phase 5 | 2 | 16 | Deployment automation |
| Phase 6 | 2 | 16 | Documentation + demo prep |
| **Total** | **14** | **112** | |

**Daily commitment**: 8 hours/day average

---

## Final Checklist

### Before Interview
- [ ] Can deploy entire stack in < 5 minutes
- [ ] Can demo all features in < 5 minutes
- [ ] Can explain every technical decision
- [ ] Have answers to common scaling questions
- [ ] Screenshots/videos of system working
- [ ] Clean git history (no "fix typo" commits)
- [ ] All secrets removed from repo
- [ ] README has compelling intro paragraph

### Portfolio Presentation
- [ ] Add to personal website/portfolio
- [ ] Write blog post about hybrid auth implementation
- [ ] Share on LinkedIn with key learnings
- [ ] Include in CV under "Personal Projects"
- [ ] Prepare 2-minute elevator pitch
