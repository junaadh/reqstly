#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BACKEND_DIR="${ROOT_DIR}/backend"
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

if [[ "${BACKEND_BOOTSTRAP_DB:-true}" == "true" ]]; then
  "${ROOT_DIR}/scripts/up-dev.sh" db >/dev/null
fi

TEST_DATABASE_HOST="${TEST_DATABASE_HOST:-127.0.0.1}"
TEST_DATABASE_PORT="${TEST_DATABASE_PORT:-54323}"
TEST_DATABASE_NAME="${TEST_DATABASE_NAME:-${POSTGRES_DB:-postgres}}"
TEST_DATABASE_PASSWORD="${TEST_DATABASE_PASSWORD:-${POSTGRES_PASSWORD:-super-secret-and-long-postgres-password}}"

export TEST_DATABASE_ADMIN_URL="${TEST_DATABASE_ADMIN_URL:-postgres://postgres:${TEST_DATABASE_PASSWORD}@${TEST_DATABASE_HOST}:${TEST_DATABASE_PORT}/${TEST_DATABASE_NAME}}"

cd "${BACKEND_DIR}"

cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets -- --test-threads=1

if [[ "${BACKEND_COVERAGE:-true}" == "true" ]]; then
  if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
    echo "cargo-llvm-cov is required for coverage checks. Install with: cargo install cargo-llvm-cov"
    exit 1
  fi

  cargo llvm-cov \
    --all-targets \
    --workspace \
    --fail-under-lines "${BACKEND_MIN_COVERAGE:-60}" \
    -- --test-threads=1
fi

echo "Backend test harness passed"
