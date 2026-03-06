# Observability Runbook

This runbook maps directly to Prometheus alerts defined in `infra/observability/prometheus/alerts.yml`.

## BackendUnavailable
1. Verify the backend container is healthy: `docker ps --filter name=backend`
2. Check `/metrics` locally from the Prometheus container network: `docker exec prometheus wget -qO- http://backend:3000/metrics | head`
3. Review backend logs for startup failures: `docker logs --tail 200 backend`
4. If backend is healthy but scrape still fails, reload Prometheus targets from Grafana Explore or `http://127.0.0.1:${PROMETHEUS_PORT:-9090}/targets`.

## BackendHighErrorRate
1. Open Grafana dashboard `Reqstly Observability Overview` and inspect `API Error Rate`.
2. Correlate with Loki logs panel for recent `ERROR` records from `backend`.
3. Filter backend logs by status and route to isolate one failing endpoint.
4. Roll back the latest backend image if a recent deploy introduced regression.

## BackendP95LatencyHigh
1. Check `API p95 Latency` and `API Request Rate` panels together for load spikes.
2. Confirm DB saturation and availability panels to rule out downstream latency.
3. Inspect slow request logs in backend output (`http.request` spans with higher latency).
4. If tied to a deployment, roll back and open a perf issue with offending route/query.

## PostgresUnavailable
1. Confirm `db` service health: `docker ps --filter name=db`
2. Inspect DB logs: `docker logs --tail 200 db`
3. Validate credentials/env in active env file (`POSTGRES_PASSWORD`, `POSTGRES_DB`, `POSTGRES_PORT`).
4. Validate exporter connectivity with `docker logs --tail 200 postgres-exporter`.

## PostgresConnectionSaturation
1. Check current connection saturation panel and request rate trends.
2. Inspect backend for connection leaks or long transactions.
3. Temporarily scale down noisy traffic and/or increase pool/concurrency guardrails.
4. If sustained, tune `POOLER_*` and application query patterns before increasing `max_connections`.

## RedisUnavailable
1. Confirm `redis` and `redis-exporter` containers are running.
2. Check Redis ping from exporter context: `docker exec redis-exporter wget -qO- http://localhost:9121/metrics | head`
3. Inspect Redis logs for persistence or startup failures.
4. Restart `redis` and `redis-exporter` if exporter lost connection after Redis restart.
