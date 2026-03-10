#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE_FILE="${COMPOSE_FILE:-${ROOT_DIR}/infra/docker-compose.dev.yml}"
EXTRA_COMPOSE_FILE="${EXTRA_COMPOSE_FILE:-}"

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

if [[ ! -f "${COMPOSE_FILE}" ]]; then
  echo "Compose file not found: ${COMPOSE_FILE}"
  exit 1
fi

if [[ -n "${EXTRA_COMPOSE_FILE}" && ! -f "${EXTRA_COMPOSE_FILE}" ]]; then
  echo "Extra compose file not found: ${EXTRA_COMPOSE_FILE}"
  exit 1
fi

MIN_DOCKER_MEMORY_GB="${MIN_DOCKER_MEMORY_GB:-2}"
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
    echo "Docker VM resources are too low for the development stack."
    echo "Detected: ${CURRENT_MEM_GB}GB RAM, ${DOCKER_CPU_COUNT} CPU(s)"
    echo "Required: >= ${MIN_DOCKER_MEMORY_GB}GB RAM, >= ${MIN_DOCKER_CPUS} CPU(s)"
    echo "Suggested fix (Colima): colima stop && colima start"
    echo "Optional sizing override: colima start --cpu <cores> --memory <gb> --disk 100"
    exit 1
  fi
fi

echo "Starting dev stack"
echo "- compose: ${COMPOSE_FILE}"
if [[ -n "${EXTRA_COMPOSE_FILE}" ]]; then
  echo "- extra compose: ${EXTRA_COMPOSE_FILE}"
fi
echo "- env: ${SELECTED_ENV_FILE}"

COMPOSE_WAIT_TIMEOUT="${COMPOSE_WAIT_TIMEOUT:-900}"
COMPOSE_BUILD="${COMPOSE_BUILD:-0}"
COMPOSE_NO_DEPS="${COMPOSE_NO_DEPS:-0}"
COMPOSE_FORCE_RECREATE="${COMPOSE_FORCE_RECREATE:-0}"
STACK_HEALTH_TIMEOUT="${STACK_HEALTH_TIMEOUT:-300}"
STACK_HEALTH_INTERVAL="${STACK_HEALTH_INTERVAL:-2}"
HEALTHCHECK_EXCLUDED_SERVICES="${HEALTHCHECK_EXCLUDED_SERVICES:-migrate}"
BACKEND_HEALTH_TIMEOUT="${BACKEND_HEALTH_TIMEOUT:-900}"
BACKEND_HEALTH_INTERVAL="${BACKEND_HEALTH_INTERVAL:-2}"
BACKEND_HEALTH_CONTAINER="${BACKEND_HEALTH_CONTAINER:-backend}"
BACKEND_HEALTH_CHECK="${BACKEND_HEALTH_CHECK:-auto}"

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
if [[ "${COMPOSE_FORCE_RECREATE}" == "1" ]]; then
  compose_up_args+=(--force-recreate)
fi
if [[ "${COMPOSE_NO_DEPS}" == "1" ]] && (( $# > 0 )); then
  compose_up_args+=(--no-deps)
fi

"${compose_cmd[@]}" "${compose_up_args[@]}" "$@"

IFS=',' read -r -a excluded_services <<< "${HEALTHCHECK_EXCLUDED_SERVICES}"

is_excluded_service() {
  local service="$1"
  local excluded trimmed

  for excluded in "${excluded_services[@]}"; do
    trimmed="${excluded#"${excluded%%[![:space:]]*}"}"
    trimmed="${trimmed%"${trimmed##*[![:space:]]}"}"
    if [[ -n "${trimmed}" && "${service}" == "${trimmed}" ]]; then
      return 0
    fi
  done

  return 1
}

get_unhealthy_services() {
  local service services container_id status health
  services="$("${compose_cmd[@]}" config --services)"

  for service in ${services}; do
    if is_excluded_service "${service}"; then
      continue
    fi

    container_id="$("${compose_cmd[@]}" ps -q "${service}" | head -n1)"
    if [[ -z "${container_id}" ]]; then
      echo "${service}"
      continue
    fi

    status="$(docker inspect --format '{{.State.Status}}' "${container_id}" 2>/dev/null || echo unknown)"
    health="$(docker inspect --format '{{if .State.Health}}{{.State.Health.Status}}{{else}}none{{end}}' "${container_id}" 2>/dev/null || echo unknown)"

    if [[ "${status}" != "running" ]]; then
      echo "${service}"
      continue
    fi

    if [[ "${health}" != "none" && "${health}" != "healthy" ]]; then
      echo "${service}"
      continue
    fi
  done
}

read_unhealthy_services() {
  unhealthy_services=()
  while IFS= read -r service; do
    if [[ -n "${service}" ]]; then
      unhealthy_services+=("${service}")
    fi
  done < <(get_unhealthy_services)
}

if [[ "${COMPOSE_FORCE_RECREATE}" == "1" ]] && (( $# > 0 )); then
  echo "Force-recreate requested for selected services; reconciling full stack health."
  elapsed=0
  unhealthy_services=()

  while (( elapsed < STACK_HEALTH_TIMEOUT )); do
    read_unhealthy_services

    if (( ${#unhealthy_services[@]} == 0 )); then
      echo "All stack services are healthy."
      break
    fi

    echo "Unhealthy services detected: ${unhealthy_services[*]}"
    for service in "${unhealthy_services[@]}"; do
      container_id="$("${compose_cmd[@]}" ps -q "${service}" | head -n1)"
      if [[ -n "${container_id}" ]]; then
        health_status="$(docker inspect --format '{{if .State.Health}}{{.State.Health.Status}}{{else}}none{{end}}' "${container_id}" 2>/dev/null || echo unknown)"
        if [[ "${health_status}" == "starting" ]]; then
          echo "Service ${service} is still starting; waiting instead of restarting."
          continue
        fi
      fi

      echo "Restarting unhealthy service: ${service}"
      "${compose_cmd[@]}" up -d --no-deps --force-recreate "${service}"
    done

    sleep "${STACK_HEALTH_INTERVAL}"
    elapsed=$((elapsed + STACK_HEALTH_INTERVAL))
  done

  if (( ${#unhealthy_services[@]} > 0 )); then
    echo "Some services are still unhealthy after ${STACK_HEALTH_TIMEOUT}s."
    "${compose_cmd[@]}" ps
    for service in "${unhealthy_services[@]}"; do
      container_id="$("${compose_cmd[@]}" ps -q "${service}" | head -n1)"
      if [[ -n "${container_id}" ]]; then
        echo "--- ${service} logs ---"
        docker logs --tail 120 "${container_id}" || true
      fi
    done
    exit 1
  fi
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
