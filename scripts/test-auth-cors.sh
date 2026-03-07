#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE_FILE="${COMPOSE_FILE:-${ROOT_DIR}/infra/docker-compose.dev.yml}"

if [[ -n "${ENV_FILE:-}" ]]; then
  SELECTED_ENV_FILE="${ENV_FILE}"
elif [[ -f "${ROOT_DIR}/.env.local" ]]; then
  SELECTED_ENV_FILE="${ROOT_DIR}/.env.local"
elif [[ -f "${ROOT_DIR}/.env" ]]; then
  SELECTED_ENV_FILE="${ROOT_DIR}/.env"
elif [[ -f "${ROOT_DIR}/.env.example" ]]; then
  SELECTED_ENV_FILE="${ROOT_DIR}/.env.example"
else
  echo "No env file found. Set ENV_FILE or create .env.local/.env/.env.example."
  exit 1
fi

set -a
# shellcheck disable=SC1090
source "${SELECTED_ENV_FILE}"
set +a

AUTH_CORS_BASE_URL_VALUE="${AUTH_CORS_BASE_URL:-http://127.0.0.1:${BACKEND_PORT:-3000}}"
AUTH_CORS_ORIGIN_VALUE="${AUTH_CORS_ORIGIN:-${APP_URL:-https://localhost}}"
AUTH_CORS_WAIT_TIMEOUT_VALUE="${AUTH_CORS_WAIT_TIMEOUT:-300}"

compose_cmd=(
  docker compose
  --env-file "${SELECTED_ENV_FILE}"
  -f "${COMPOSE_FILE}"
)

cleanup() {
  if [[ "${AUTH_CORS_CLEANUP:-false}" != "true" ]]; then
    return
  fi

  "${compose_cmd[@]}" down --remove-orphans >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "Starting backend stack for CORS regression check..."
"${compose_cmd[@]}" up -d --remove-orphans --wait --wait-timeout "${AUTH_CORS_WAIT_TIMEOUT_VALUE}" db migrate backend >/dev/null

request_headers() {
  local method="$1"
  local path="$2"
  local ac_request_method="$3"

  curl -sS \
    --max-time "${AUTH_CORS_CURL_TIMEOUT:-5}" \
    -D - \
    -o /dev/null \
    -X "${method}" \
    "${AUTH_CORS_BASE_URL_VALUE}${path}" \
    -H "Origin: ${AUTH_CORS_ORIGIN_VALUE}" \
    -H "Access-Control-Request-Method: ${ac_request_method}" \
    -H "Access-Control-Request-Headers: authorization,content-type,x-request-id"
}

wait_for_backend() {
  local deadline="$((SECONDS + AUTH_CORS_WAIT_TIMEOUT_VALUE))"

  while (( SECONDS < deadline )); do
    local status_code
    status_code="$(
      request_headers OPTIONS "/api/v1/me" "GET" 2>/dev/null \
        | awk 'toupper($1) ~ /^HTTP\// { print $2; exit }'
    )"

    if [[ "${status_code}" =~ ^[0-9]{3}$ ]]; then
      return 0
    fi

    sleep 2
  done

  echo "Backend did not become ready within ${AUTH_CORS_WAIT_TIMEOUT_VALUE}s."
  "${compose_cmd[@]}" ps || true
  "${compose_cmd[@]}" logs --tail 120 backend migrate db || true
  exit 1
}

extract_header() {
  local headers="$1"
  local key="$2"

  printf '%s\n' "${headers}" \
    | tr -d '\r' \
    | awk -F': ' -v target="${key}" 'tolower($1) == tolower(target) { print $2; exit }'
}

assert_cors_headers() {
  local endpoint="$1"
  local ac_request_method="$2"

  local headers
  headers="$(request_headers OPTIONS "${endpoint}" "${ac_request_method}")"

  local allow_origin
  allow_origin="$(extract_header "${headers}" "Access-Control-Allow-Origin")"
  local allow_credentials
  allow_credentials="$(extract_header "${headers}" "Access-Control-Allow-Credentials")"

  if [[ -z "${allow_origin}" ]]; then
    echo "Missing Access-Control-Allow-Origin for ${endpoint}"
    echo "${headers}"
    exit 1
  fi

  if [[ "${allow_origin}" == "*" ]]; then
    echo "Invalid wildcard Access-Control-Allow-Origin for ${endpoint}"
    echo "${headers}"
    exit 1
  fi

  if [[ "${allow_origin}" != "${AUTH_CORS_ORIGIN_VALUE}" ]]; then
    echo "Unexpected Access-Control-Allow-Origin for ${endpoint}: ${allow_origin}"
    echo "Expected: ${AUTH_CORS_ORIGIN_VALUE}"
    echo "${headers}"
    exit 1
  fi

  local allow_credentials_normalized
  allow_credentials_normalized="$(printf '%s' "${allow_credentials}" | tr '[:upper:]' '[:lower:]')"

  if [[ "${allow_credentials_normalized}" != "true" ]]; then
    echo "Access-Control-Allow-Credentials must be true for ${endpoint}"
    echo "${headers}"
    exit 1
  fi

  echo "CORS headers valid for ${endpoint}"
}

wait_for_backend

assert_cors_headers "/api/v1/auth/login/password" "POST"
assert_cors_headers "/api/v1/auth/passkeys/login/start" "POST"
assert_cors_headers "/api/v1/me" "GET"

echo "Auth CORS regression checks passed"
