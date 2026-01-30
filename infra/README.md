# Reqstly Infrastructure

This directory contains all infrastructure configuration for running Reqstly locally using Docker Compose.

## Directory Structure

```
infra/
├── proxy/
│   └── caddy/              # Reverse proxy configuration (Caddyfile - Phase 5)
│
├── observability/
│   ├── grafana/            # Dashboards and visualization
│   │   └── provisioning/   # Auto-provisioned datasources and dashboards
│   ├── prometheus/         # Metrics collection
│   │   └── prometheus.yml  # Scrape configurations
│   ├── loki/               # Log aggregation
│   │   └── loki.yml        # Log retention and storage
│   └── promtail/           # Log shipping agent
│       └── promtail.yml    # Docker log scraping
│
├── docker-compose.yml      # Main orchestration file
└── README.md               # This file
```

## Services

### Application Services

| Service | Port | Description |
|---------|------|-------------|
| **Backend** | 3000 | Rust API server (Axum) |
| **Frontend** | 5173 | Vite + TypeScript + React dev server |
| **PostgreSQL** | 5432 | Database (PostgreSQL 16) |

### Observability Services

| Service | Port | Description |
|---------|------|-------------|
| **Prometheus** | 9090 | Metrics collection and storage |
| **Grafana** | 3001 | Dashboards and visualization |
| **Loki** | 3100 | Log aggregation (9080 for gRPC) |
| **Promtail** | - | Log shipper (no exposed port) |

### Proxy Services (Phase 5)

| Service | Port | Description |
|---------|------|-------------|
| **Caddy** | 80, 443 | Reverse proxy with auto-TLS |

## Networks

Docker Compose creates three isolated networks:

- **reqstly-public** - Frontend and backend (external access)
- **reqstly-internal** - Backend and database (isolated)
- **reqstly-monitoring** - Monitoring stack (isolated)

## Quick Start

### Prerequisites

- Docker and Docker Compose installed
- `.env.local` file configured in project root

### Start All Services

```bash
# From project root
docker-compose -f infra/docker-compose.yml up -d
```

### View Logs

```bash
# All services
docker-compose -f infra/docker-compose.yml logs -f

# Specific service
docker-compose -f infra/docker-compose.yml logs -f backend
```

### Stop Services

```bash
docker-compose -f infra/docker-compose.yml down
```

### Stop and Remove Volumes

⚠️ **Warning**: This deletes all data

```bash
docker-compose -f infra/docker-compose.yml down -v
```

## Access Points

Once services are running:

- **Frontend**: http://localhost:5173
- **Backend API**: http://localhost:3000
- **Backend Health**: http://localhost:3000/health
- **Backend Metrics**: http://localhost:3000/metrics
- **Grafana**: http://localhost:3001 (admin/admin)
- **Prometheus**: http://localhost:9090

## Configuration

### Environment Variables

Copy the example environment file:

```bash
cp .env.example .env.local
```

Edit `.env.local` to configure:
- Database credentials
- JWT secrets
- Azure AD credentials (Phase 3)
- Passkey/WebAuthn settings (Phase 3)

### Prometheus Configuration

Edit `observability/prometheus/prometheus.yml` to:
- Add scrape targets
- Adjust retention period
- Configure alerting rules

### Grafana Dashboards

Add custom dashboards to:
```
observability/grafana/provisioning/dashboards/
```

They will be auto-loaded on startup.

## Data Persistence

Volumes persist data across container restarts:

- `postgres-data` - Database files
- `prometheus-data` - Metrics (15-day retention)
- `grafana-data` - Dashboards and settings
- `loki-data` - Logs (30-day retention)
- `cargo-cache` - Rust build cache

## Troubleshooting

### Database Connection Issues

```bash
# Check PostgreSQL is running
docker-compose -f infra/docker-compose.yml logs postgres

# Verify database connectivity
docker-compose -f infra/docker-compose.yml exec backend \
  psql $DATABASE_URL -c "SELECT 1"
```

### Services Not Starting

```bash
# Check service health
docker-compose -f infra/docker-compose.yml ps

# View detailed logs
docker-compose -f infra/docker-compose.yml logs --tail=100
```

### Reset Everything

```bash
# Stop all services
docker-compose -f infra/docker-compose.yml down

# Remove volumes (deletes data!)
docker volume rm reqstly_postgres-data \
  reqstly_prometheus-data \
  reqstly_grafana-data \
  reqstly_loki-data

# Restart
docker-compose -f infra/docker-compose.yml up -d
```

## Development Workflow

### Backend Development

Backend runs with hot reload. Changes to `backend/src/` trigger recompilation.

```bash
# View backend logs
docker-compose -f infra/docker-compose.yml logs -f backend
```

### Frontend Development

Frontend runs with HMR (Hot Module Replacement).

```bash
# View frontend logs
docker-compose -f infra/docker-compose.yml logs -f frontend
```

### Database Migrations

Migrations run automatically on backend startup. To manually run:

```bash
docker-compose -f infra/docker-compose.yml exec backend \
  sqlx migrate run
```

## Monitoring

### Metrics

Prometheus scrapes metrics every 15 seconds from:
- Backend (`/metrics` endpoint)
- Prometheus (self-monitoring)
- Grafana
- Loki

View in Grafana: http://localhost:3001

### Logs

Loki aggregates logs from all containers. View in Grafana:
1. Open Grafana
2. Go to Explore
3. Select Loki datasource
4. Query by label: `{service="backend"}`

### Health Checks

- **Backend**: `curl http://localhost:3000/health`
- **Database**: PostgreSQL healthcheck in docker-compose
- **Frontend**: Check browser console

## Production Considerations

For production deployment (Phase 5):

1. **Use Docker Swarm** instead of Docker Compose
2. **Enable Caddy** for TLS termination
3. **Use Docker secrets** for sensitive data
4. **Configure backup scripts** for PostgreSQL
5. **Set up log rotation** for Loki
6. **Configure alerting** in Prometheus
7. **Update CORS settings** for production domain

## Related Documentation

- [Main README](../README.md)
- [CLAUDE.md](../CLAUDE.md) - Architecture overview
- [PLAN.md](../docs/PLAN.md) - Implementation plan
- [Backend README](../backend/README.md)
- [Frontend README](../frontend/README.md)
