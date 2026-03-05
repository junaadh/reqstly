#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BASE_COMPOSE_FILE="${BASE_COMPOSE_FILE:-${ROOT_DIR}/infra/supabase/docker-compose.yml}"
OVERLAY_COMPOSE_FILE="${OVERLAY_COMPOSE_FILE:-${ROOT_DIR}/infra/docker-compose.dev.yml}"
COMPOSE_WAIT_TIMEOUT="${COMPOSE_WAIT_TIMEOUT:-900}"
START_BACKEND="${START_BACKEND:-1}"
START_OPTIONAL_SUPABASE="${START_OPTIONAL_SUPABASE:-1}"
OPTIONAL_SUPABASE_RETRIES="${OPTIONAL_SUPABASE_RETRIES:-3}"

usage() {
  cat <<'EOF'
Reset local dev database state and restart Supabase services with dev config.

Usage:
  ./scripts/reset-dev-db.sh [-y|--yes]

Options:
  -y, --yes     Skip confirmation prompt.

Environment overrides:
  ENV_FILE               Env file to use (default: .env.local, then .env.example)
  COMPOSE_WAIT_TIMEOUT   docker compose wait timeout in seconds (default: 900)
  START_BACKEND          1 to restart backend after migration, 0 to skip (default: 1)
  START_OPTIONAL_SUPABASE 1 to ensure full Supabase stack is started (default: 1)
  OPTIONAL_SUPABASE_RETRIES retries for optional service wait failures (default: 3)
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

if [[ ! -f "${BASE_COMPOSE_FILE}" ]]; then
  echo "Base compose file not found: ${BASE_COMPOSE_FILE}"
  exit 1
fi

if [[ ! -f "${OVERLAY_COMPOSE_FILE}" ]]; then
  echo "Overlay compose file not found: ${OVERLAY_COMPOSE_FILE}"
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

# Force dev-safe defaults for auth-related config if omitted.
: "${SUPABASE_PUBLIC_URL:=https://supabase.localhost}"
: "${API_EXTERNAL_URL:=${SUPABASE_PUBLIC_URL}}"
: "${SITE_URL:=https://localhost}"
: "${MFA_ENABLED:=true}"
: "${MFA_WEBAUTHN_ENABLED:=true}"
: "${MFA_WEBAUTHN_ENROLL_ENABLED:=true}"
: "${MFA_WEBAUTHN_VERIFY_ENABLED:=true}"
: "${MFA_WEB_AUTHN_ENABLED:=${MFA_WEBAUTHN_ENABLED}}"
: "${MFA_WEB_AUTHN_ENROLL_ENABLED:=${MFA_WEBAUTHN_ENROLL_ENABLED}}"
: "${MFA_WEB_AUTHN_VERIFY_ENABLED:=${MFA_WEBAUTHN_VERIFY_ENABLED}}"

required_vars=(
  COMPOSE_PROJECT_NAME
  POSTGRES_PASSWORD
  POSTGRES_HOST
  POSTGRES_DB
  POSTGRES_PORT
  JWT_SECRET
  ANON_KEY
  SERVICE_ROLE_KEY
  SUPABASE_PUBLIC_URL
  API_EXTERNAL_URL
  SITE_URL
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

  printf "This will DROP local DB data and restart Supabase dev services. Continue? (y/N) "
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
  -f "${BASE_COMPOSE_FILE}"
  -f "${OVERLAY_COMPOSE_FILE}"
)

services_to_reset=(
  backend
  migrate
  studio
  kong
  auth
  rest
  realtime
  storage
  imgproxy
  meta
  functions
  analytics
  supavisor
  db
  vector
)

required_supabase_services=(
  vector
  db
  analytics
  auth
  rest
  kong
)

optional_supabase_services=(
  realtime
  storage
  imgproxy
  meta
  functions
  supavisor
  studio
)

db_volumes=(
  "${COMPOSE_PROJECT_NAME}_supabase-db-data"
  "${COMPOSE_PROJECT_NAME}_db-config"
)

echo "Resetting local dev DB"
echo "- env: ${SELECTED_ENV_FILE}"
echo "- project: ${COMPOSE_PROJECT_NAME}"
echo "- base compose: ${BASE_COMPOSE_FILE}"
echo "- overlay compose: ${OVERLAY_COMPOSE_FILE}"
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

echo "Starting required Supabase services..."
"${compose_cmd[@]}" up -d --remove-orphans --wait --wait-timeout "${COMPOSE_WAIT_TIMEOUT}" "${required_supabase_services[@]}"

echo "Running backend migrations..."
"${compose_cmd[@]}" run --rm migrate

if [[ "${START_BACKEND}" == "1" ]]; then
  echo "Starting backend..."
  "${compose_cmd[@]}" up -d --wait --wait-timeout "${COMPOSE_WAIT_TIMEOUT}" backend
fi

if [[ "${START_OPTIONAL_SUPABASE}" == "1" ]]; then
  echo "Starting optional Supabase services..."
  attempt=1
  while true; do
    if "${compose_cmd[@]}" up -d --wait --wait-timeout "${COMPOSE_WAIT_TIMEOUT}" "${optional_supabase_services[@]}"; then
      break
    fi

    if (( attempt >= OPTIONAL_SUPABASE_RETRIES )); then
      echo "Optional Supabase services failed to become healthy after ${OPTIONAL_SUPABASE_RETRIES} attempts."
      "${compose_cmd[@]}" ps || true
      exit 1
    fi

    attempt=$((attempt + 1))
    echo "Optional services not healthy yet, retrying (${attempt}/${OPTIONAL_SUPABASE_RETRIES})..."
    sleep 5
  done
fi

echo "Database reset complete."
echo "Supabase URL: ${SUPABASE_PUBLIC_URL}"
