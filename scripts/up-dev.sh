#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BASE_COMPOSE_FILE="${BASE_COMPOSE_FILE:-${ROOT_DIR}/infra/supabase/docker-compose.yml}"
OVERLAY_COMPOSE_FILE="${OVERLAY_COMPOSE_FILE:-${ROOT_DIR}/infra/docker-compose.dev.yml}"

if [[ -n "${ENV_FILE:-}" ]]; then
  SELECTED_ENV_FILE="${ENV_FILE}"
elif [[ -f "${ROOT_DIR}/.env.local" ]]; then
  SELECTED_ENV_FILE="${ROOT_DIR}/.env.local"
elif [[ -f "${ROOT_DIR}/.env.example" ]]; then
  SELECTED_ENV_FILE="${ROOT_DIR}/.env.example"
else
  echo "No env file found. Set ENV_FILE or create .env.local/.env.example."
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

MIN_DOCKER_MEMORY_GB="${MIN_DOCKER_MEMORY_GB:-4}"
MIN_DOCKER_CPUS="${MIN_DOCKER_CPUS:-2}"

if ! docker info >/dev/null 2>&1; then
  echo "Docker daemon is not reachable. Start Docker/Colima and retry."
  exit 1
fi

DOCKER_MEM_BYTES="$(docker info --format '{{.MemTotal}}' 2>/dev/null || echo 0)"
DOCKER_CPU_COUNT="$(docker info --format '{{.NCPU}}' 2>/dev/null || echo 0)"

if [[ "${DOCKER_MEM_BYTES}" =~ ^[0-9]+$ ]] && [[ "${DOCKER_CPU_COUNT}" =~ ^[0-9]+$ ]]; then
  REQUIRED_MEM_BYTES=$((MIN_DOCKER_MEMORY_GB * 1000 * 1000 * 1000))
  if (( DOCKER_MEM_BYTES < REQUIRED_MEM_BYTES || DOCKER_CPU_COUNT < MIN_DOCKER_CPUS )); then
    CURRENT_MEM_GB="$(awk -v mem="${DOCKER_MEM_BYTES}" 'BEGIN { printf "%.1f", mem/1024/1024/1024 }')"
    echo "Docker VM resources are too low for full Supabase stack."
    echo "Detected: ${CURRENT_MEM_GB}GB RAM, ${DOCKER_CPU_COUNT} CPU(s)"
    echo "Required: >= ${MIN_DOCKER_MEMORY_GB}GB RAM, >= ${MIN_DOCKER_CPUS} CPU(s)"
    echo "Suggested fix (Colima): colima stop && colima start --cpu ${MIN_DOCKER_CPUS} --memory ${MIN_DOCKER_MEMORY_GB} --disk 100"
    exit 1
  fi
fi

echo "Starting dev stack"
echo "- base compose: ${BASE_COMPOSE_FILE}"
echo "- overlay compose: ${OVERLAY_COMPOSE_FILE}"
echo "- env: ${SELECTED_ENV_FILE}"

COMPOSE_WAIT_TIMEOUT="${COMPOSE_WAIT_TIMEOUT:-900}"
COMPOSE_BUILD="${COMPOSE_BUILD:-0}"
BACKEND_HEALTH_TIMEOUT="${BACKEND_HEALTH_TIMEOUT:-900}"
BACKEND_HEALTH_INTERVAL="${BACKEND_HEALTH_INTERVAL:-2}"
BACKEND_HEALTH_CONTAINER="${BACKEND_HEALTH_CONTAINER:-backend}"
BACKEND_HEALTH_CHECK="${BACKEND_HEALTH_CHECK:-auto}"

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

should_wait_backend="true"
case "${BACKEND_HEALTH_CHECK}" in
  true)
    should_wait_backend="true"
    ;;
  false)
    should_wait_backend="false"
    ;;
  auto)
    if (( $# > 0 )); then
      should_wait_backend="false"
      for service in "$@"; do
        if [[ "${service}" == "${BACKEND_HEALTH_CONTAINER}" ]]; then
          should_wait_backend="true"
          break
        fi
      done
    fi
    ;;
  *)
    echo "Invalid BACKEND_HEALTH_CHECK value: ${BACKEND_HEALTH_CHECK} (expected: auto|true|false)"
    exit 1
    ;;
esac

if [[ "${should_wait_backend}" != "true" ]]; then
  echo "Skipping backend health wait (service selection does not include ${BACKEND_HEALTH_CONTAINER})."
  exit 0
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
