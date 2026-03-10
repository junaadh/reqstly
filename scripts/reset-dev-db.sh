#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE_FILE="${COMPOSE_FILE:-${ROOT_DIR}/infra/docker-compose.dev.yml}"
COMPOSE_WAIT_TIMEOUT="${COMPOSE_WAIT_TIMEOUT:-900}"
START_BACKEND="${START_BACKEND:-1}"
START_FRONTEND="${START_FRONTEND:-0}"

usage() {
  cat <<'EOF'
Reset local dev database state and rerun migrations.

Usage:
  ./scripts/reset-dev-db.sh [-y|--yes]

Options:
  -y, --yes     Skip confirmation prompt.

Environment overrides:
  ENV_FILE               Env file to use (default: .env.local, then .env.example)
  COMPOSE_FILE           Compose file path (default: infra/docker-compose.dev.yml)
  COMPOSE_WAIT_TIMEOUT   Docker compose wait timeout in seconds (default: 900)
  START_BACKEND          1 to restart backend after migration, 0 to skip (default: 1)
  START_FRONTEND         1 to restart frontend+caddy after migration, 0 to skip (default: 0)
EOF
}

AUTO_CONFIRM=0
while (( $# > 0 )); do
  case "$1" in
    -y|--yes)
      AUTO_CONFIRM=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1"
      usage
      exit 1
      ;;
  esac
done

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

if ! docker info >/dev/null 2>&1; then
  echo "Docker daemon is not reachable. Start Docker/Colima and retry."
  exit 1
fi

set -a
# shellcheck disable=SC1090
source "${SELECTED_ENV_FILE}"
set +a

required_vars=(
  COMPOSE_PROJECT_NAME
  POSTGRES_PASSWORD
  POSTGRES_DB
)

missing=()
for var_name in "${required_vars[@]}"; do
  if [[ -z "${!var_name:-}" ]]; then
    missing+=("${var_name}")
  fi
done

if (( ${#missing[@]} > 0 )); then
  echo "Missing required env vars in ${SELECTED_ENV_FILE}:"
  for var_name in "${missing[@]}"; do
    echo "  - ${var_name}"
  done
  exit 1
fi

confirm() {
  if [[ "${AUTO_CONFIRM}" == "1" ]]; then
    return 0
  fi

  printf "This will DROP local DB data and rerun migrations. Continue? (y/N) "
  read -r reply
  case "${reply}" in
    [Yy]|[Yy][Ee][Ss]) ;;
    *)
      echo "Canceled."
      exit 1
      ;;
  esac
}

compose_cmd=(
  docker compose
  --env-file "${SELECTED_ENV_FILE}"
  -f "${COMPOSE_FILE}"
)

services_to_reset=(
  frontend
  caddy
  backend
  migrate
  postgres-exporter
  db
)

required_services=(
  db
)

db_volumes=(
  "${COMPOSE_PROJECT_NAME}_db-data"
)

echo "Resetting local dev DB"
echo "- env: ${SELECTED_ENV_FILE}"
echo "- project: ${COMPOSE_PROJECT_NAME}"
echo "- compose: ${COMPOSE_FILE}"
confirm

echo "Stopping related services..."
"${compose_cmd[@]}" stop "${services_to_reset[@]}" >/dev/null 2>&1 || true

echo "Removing related containers..."
"${compose_cmd[@]}" rm -f "${services_to_reset[@]}" >/dev/null 2>&1 || true

echo "Removing database volumes..."
for vol in "${db_volumes[@]}"; do
  if docker volume inspect "${vol}" >/dev/null 2>&1; then
    docker volume rm "${vol}" >/dev/null
    echo "  - removed ${vol}"
  else
    echo "  - not found ${vol} (already removed)"
  fi
done

echo "Starting database and migrations..."
echo "Starting database..."
"${compose_cmd[@]}" up -d --remove-orphans --wait --wait-timeout "${COMPOSE_WAIT_TIMEOUT}" "${required_services[@]}"

echo "Running migrations..."
set +e
"${compose_cmd[@]}" up --no-deps --abort-on-container-exit --exit-code-from migrate migrate
migrate_exit_code=$?
set -e

if (( migrate_exit_code != 0 )); then
  echo "Migration failed with exit code ${migrate_exit_code}."
  "${compose_cmd[@]}" logs --tail 200 migrate || true
  exit "${migrate_exit_code}"
fi

if [[ "${START_BACKEND}" == "1" ]]; then
  echo "Starting backend..."
  "${compose_cmd[@]}" up -d --wait --wait-timeout "${COMPOSE_WAIT_TIMEOUT}" backend
fi

if [[ "${START_FRONTEND}" == "1" ]]; then
  echo "Starting frontend and caddy..."
  "${compose_cmd[@]}" up -d --wait --wait-timeout "${COMPOSE_WAIT_TIMEOUT}" frontend caddy
fi

echo "Database reset complete."
