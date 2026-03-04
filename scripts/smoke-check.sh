#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [[ -n "${ENV_FILE:-}" ]]; then
  SELECTED_ENV_FILE="${ENV_FILE}"
elif [[ -f "${ROOT_DIR}/.env.local" ]]; then
  SELECTED_ENV_FILE="${ROOT_DIR}/.env.local"
elif [[ -f "${ROOT_DIR}/.env" ]]; then
  SELECTED_ENV_FILE="${ROOT_DIR}/.env"
elif [[ -f "${ROOT_DIR}/.env.example" ]]; then
  SELECTED_ENV_FILE="${ROOT_DIR}/.env.example"
else
  SELECTED_ENV_FILE=""
fi

if [[ -n "${SELECTED_ENV_FILE}" ]]; then
  set -a
  source "${SELECTED_ENV_FILE}"
  set +a
fi

SUPABASE_PUBLIC_URL_VALUE="${SUPABASE_PUBLIC_URL:-${SUPABASE_URL:-}}"

if [[ -z "${APP_URL:-}" || -z "${API_URL:-}" || -z "${SUPABASE_PUBLIC_URL_VALUE}" ]]; then
  echo "APP_URL, API_URL, and SUPABASE_PUBLIC_URL must be set"
  exit 1
fi

curl_args=(-fsS)
if [[ "${SMOKE_INSECURE_TLS:-false}" == "true" ]]; then
  curl_args+=(-k)
fi

check_url() {
  local url="$1"
  local hostport host port scheme

  if curl "${curl_args[@]}" "${url}" >/dev/null; then
    return 0
  fi

  if [[ "${SMOKE_LOCALHOST_RESOLVE_FALLBACK:-true}" != "true" ]]; then
    return 1
  fi

  scheme="${url%%://*}"
  hostport="$(echo "${url}" | sed -E 's|^[a-zA-Z]+://([^/]+).*|\1|')"
  host="${hostport%%:*}"
  port="${hostport##*:}"

  if [[ "${hostport}" == "${host}" ]]; then
    if [[ "${scheme}" == "https" ]]; then
      port="443"
    else
      port="80"
    fi
  fi

  if [[ "${host}" == "localhost" || "${host}" == *.localhost ]]; then
    curl "${curl_args[@]}" --resolve "${host}:${port}:127.0.0.1" "${url}" >/dev/null
    return 0
  fi

  return 1
}

check_url "${APP_URL}/health"
check_url "${API_URL}/health"
check_url "${SUPABASE_PUBLIC_URL_VALUE}/health"

echo "Smoke checks passed"
