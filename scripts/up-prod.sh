#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BASE_COMPOSE_FILE="${BASE_COMPOSE_FILE:-${ROOT_DIR}/infra/supabase/docker-compose.yml}"
OVERLAY_COMPOSE_FILE="${OVERLAY_COMPOSE_FILE:-${ROOT_DIR}/infra/docker-compose.yml}"

if [[ -n "${ENV_FILE:-}" ]]; then
  SELECTED_ENV_FILE="${ENV_FILE}"
elif [[ -f "${ROOT_DIR}/.env" ]]; then
  SELECTED_ENV_FILE="${ROOT_DIR}/.env"
elif [[ -f "${ROOT_DIR}/.env.example" ]]; then
  SELECTED_ENV_FILE="${ROOT_DIR}/.env.example"
else
  echo "No env file found. Set ENV_FILE or create .env/.env.example."
  exit 1
fi

if [[ ! -f "${BASE_COMPOSE_FILE}" ]]; then
  echo "Base compose file not found: ${BASE_COMPOSE_FILE}"
  exit 1
fi

if [[ ! -f "${OVERLAY_COMPOSE_FILE}" ]]; then
  echo "Overlay compose file not found: ${OVERLAY_COMPOSE_FILE}"
  exit 1
fi

echo "Starting prod stack"
echo "- base compose: ${BASE_COMPOSE_FILE}"
echo "- overlay compose: ${OVERLAY_COMPOSE_FILE}"
echo "- env: ${SELECTED_ENV_FILE}"

COMPOSE_WAIT_TIMEOUT="${COMPOSE_WAIT_TIMEOUT:-600}"
COMPOSE_BUILD="${COMPOSE_BUILD:-1}"
BACKEND_HEALTH_TIMEOUT="${BACKEND_HEALTH_TIMEOUT:-600}"
BACKEND_HEALTH_INTERVAL="${BACKEND_HEALTH_INTERVAL:-2}"
BACKEND_HEALTH_CONTAINER="${BACKEND_HEALTH_CONTAINER:-backend}"

if [[ "${COMPOSE_BUILD}" == "1" ]]; then
  docker compose \
    --env-file "${SELECTED_ENV_FILE}" \
    -f "${BASE_COMPOSE_FILE}" \
    -f "${OVERLAY_COMPOSE_FILE}" \
    up -d --build --remove-orphans --wait --wait-timeout "${COMPOSE_WAIT_TIMEOUT}" "$@"
else
  docker compose \
    --env-file "${SELECTED_ENV_FILE}" \
    -f "${BASE_COMPOSE_FILE}" \
    -f "${OVERLAY_COMPOSE_FILE}" \
    up -d --remove-orphans --wait --wait-timeout "${COMPOSE_WAIT_TIMEOUT}" "$@"
fi

echo "Waiting for backend health (${BACKEND_HEALTH_CONTAINER})..."
elapsed=0
while (( elapsed < BACKEND_HEALTH_TIMEOUT )); do
  health_status="$(docker inspect --format '{{if .State.Health}}{{.State.Health.Status}}{{else}}none{{end}}' "${BACKEND_HEALTH_CONTAINER}" 2>/dev/null || true)"
  if [[ "${health_status}" == "healthy" ]]; then
    echo "Backend is healthy."
    exit 0
  fi

  sleep "${BACKEND_HEALTH_INTERVAL}"
  elapsed=$((elapsed + BACKEND_HEALTH_INTERVAL))
done

echo "Backend did not become healthy within ${BACKEND_HEALTH_TIMEOUT}s."
docker logs --tail 200 "${BACKEND_HEALTH_CONTAINER}" || true
exit 1
