#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ENV_TEMPLATE_FILE="${ENV_TEMPLATE_FILE:-${ROOT_DIR}/.env.local.example}"
ENV_FILE="${ENV_FILE:-${ROOT_DIR}/.env.local}"
FORCE="${FORCE:-0}"
ROTATE_KEYS="${ROTATE_KEYS:-0}"

if [[ ! -f "${ENV_TEMPLATE_FILE}" ]]; then
  echo "Env template file not found: ${ENV_TEMPLATE_FILE}"
  exit 1
fi

if ! command -v openssl >/dev/null 2>&1; then
  echo "openssl is required but not found in PATH."
  exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 is required but not found in PATH."
  exit 1
fi

if [[ ! -x "${ROOT_DIR}/scripts/generate-supabase-dev-keys.sh" ]]; then
  echo "Missing executable helper: ${ROOT_DIR}/scripts/generate-supabase-dev-keys.sh"
  exit 1
fi

if [[ ! -x "${ROOT_DIR}/scripts/generate-dev-certs.sh" ]]; then
  echo "Missing executable helper: ${ROOT_DIR}/scripts/generate-dev-certs.sh"
  exit 1
fi

if [[ -f "${ENV_FILE}" && "${FORCE}" != "1" ]]; then
  echo "Using existing env file: ${ENV_FILE}"
  echo "Set FORCE=1 to regenerate it from ${ENV_TEMPLATE_FILE}."
else
  mkdir -p "$(dirname "${ENV_FILE}")"
  cp "${ENV_TEMPLATE_FILE}" "${ENV_FILE}"
  echo "Created env file: ${ENV_FILE}"
  ROTATE_KEYS=1
fi

if [[ "${ROTATE_KEYS}" == "1" ]]; then
  echo "Generating Supabase development secrets..."
  "${ROOT_DIR}/scripts/generate-supabase-dev-keys.sh" "${ENV_FILE}"
else
  echo "Skipping secret rotation (set ROTATE_KEYS=1 to rotate)."
fi

echo "Generating local TLS certificates for Caddy..."
"${ROOT_DIR}/scripts/generate-dev-certs.sh"

cat <<EOF

Setup complete.

Generated/updated:
- Env file: ${ENV_FILE}
- TLS certs: ${ROOT_DIR}/infra/proxy/caddy/certs/dev

Recommended next steps:
1. Trust local root CA (macOS, one-time):
   sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain "${ROOT_DIR}/infra/proxy/caddy/certs/dev/reqstly-dev-rootCA.pem"
2. Start Docker VM (example for Colima):
   colima start --cpu 4 --memory 8 --disk 100
3. Start the development stack:
   ./scripts/up-dev.sh
4. Run smoke checks:
   set -a; source .env.local; set +a
   ./scripts/smoke-check.sh

Notes:
- Keep existing .env.local and rotate only secrets:
  ROTATE_KEYS=1 ./scripts/setup-dev.sh
- Recreate .env.local from template:
  FORCE=1 ./scripts/setup-dev.sh
EOF
