#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE_FILE="${COMPOSE_FILE:-${ROOT_DIR}/infra/docker-compose.yml}"
EXTRA_COMPOSE_FILE="${EXTRA_COMPOSE_FILE:-}"

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

if [[ ! -f "${COMPOSE_FILE}" ]]; then
  echo "Compose file not found: ${COMPOSE_FILE}"
  exit 1
fi

if [[ -n "${EXTRA_COMPOSE_FILE}" && ! -f "${EXTRA_COMPOSE_FILE}" ]]; then
  echo "Extra compose file not found: ${EXTRA_COMPOSE_FILE}"
  exit 1
fi

echo "Starting prod stack"
echo "- compose: ${COMPOSE_FILE}"
if [[ -n "${EXTRA_COMPOSE_FILE}" ]]; then
  echo "- extra compose: ${EXTRA_COMPOSE_FILE}"
fi
echo "- env: ${SELECTED_ENV_FILE}"

COMPOSE_WAIT_TIMEOUT="${COMPOSE_WAIT_TIMEOUT:-600}"
COMPOSE_BUILD="${COMPOSE_BUILD:-1}"
BACKEND_HEALTH_TIMEOUT="${BACKEND_HEALTH_TIMEOUT:-600}"
BACKEND_HEALTH_INTERVAL="${BACKEND_HEALTH_INTERVAL:-2}"
BACKEND_HEALTH_CONTAINER="${BACKEND_HEALTH_CONTAINER:-backend}"

compose_cmd=(
  docker compose
  --env-file "${SELECTED_ENV_FILE}"
  -f "${COMPOSE_FILE}"
)

if [[ -n "${EXTRA_COMPOSE_FILE}" ]]; then
  compose_cmd+=(-f "${EXTRA_COMPOSE_FILE}")
fi

compose_up_args=(up -d --remove-orphans --wait --wait-timeout "${COMPOSE_WAIT_TIMEOUT}")
if [[ "${COMPOSE_BUILD}" == "1" ]]; then
  compose_up_args+=(--build)
fi

"${compose_cmd[@]}" "${compose_up_args[@]}" "$@"

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
