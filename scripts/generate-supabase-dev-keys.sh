#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ENV_FILE="${1:-${ENV_FILE:-${ROOT_DIR}/.env.local}}"
SUPABASE_KEYGEN_SCRIPT="${ROOT_DIR}/infra/supabase/utils/generate-keys.sh"

if [[ ! -f "${ENV_FILE}" ]]; then
  echo "Env file not found: ${ENV_FILE}"
  echo "Usage: $0 [path-to-env-file]"
  exit 1
fi

if [[ ! -f "${SUPABASE_KEYGEN_SCRIPT}" ]]; then
  echo "Supabase key generator not found: ${SUPABASE_KEYGEN_SCRIPT}"
  exit 1
fi

KEY_OUTPUT="$(sh "${SUPABASE_KEYGEN_SCRIPT}")"
export KEY_OUTPUT

python3 - "${ENV_FILE}" <<'PY'
import os
import re
import sys
from pathlib import Path


def parse_env(path: Path) -> tuple[dict, list[str]]:
    values: dict[str, str] = {}
    lines = path.read_text(encoding="utf-8").splitlines()
    pattern = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)=(.*)$")
    for raw in lines:
        stripped = raw.strip()
        if not stripped or stripped.startswith("#"):
            continue
        match = pattern.match(raw)
        if not match:
            continue
        key, value = match.group(1), match.group(2)
        values[key] = value
    return values, lines


def upsert(lines: list[str], key: str, value: str) -> list[str]:
    prefix = f"{key}="
    replaced = False
    out: list[str] = []
    for line in lines:
        if line.startswith(prefix):
            out.append(f"{prefix}{value}")
            replaced = True
        else:
            out.append(line)
    if not replaced:
        out.append(f"{prefix}{value}")
    return out


def parse_generated_key_values(output: str) -> dict[str, str]:
    generated: dict[str, str] = {}
    pattern = re.compile(r"^([A-Z0-9_]+)=(.+)$")
    for raw in output.splitlines():
        match = pattern.match(raw.strip())
        if match:
            generated[match.group(1)] = match.group(2)
    return generated


env_path = Path(sys.argv[1]).resolve()
_, env_lines = parse_env(env_path)
generated = parse_generated_key_values(os.environ.get("KEY_OUTPUT", ""))
required_keys = [
    "JWT_SECRET",
    "ANON_KEY",
    "SERVICE_ROLE_KEY",
    "SECRET_KEY_BASE",
    "VAULT_ENC_KEY",
    "PG_META_CRYPTO_KEY",
    "LOGFLARE_PUBLIC_ACCESS_TOKEN",
    "LOGFLARE_PRIVATE_ACCESS_TOKEN",
    "S3_PROTOCOL_ACCESS_KEY_ID",
    "S3_PROTOCOL_ACCESS_KEY_SECRET",
    "MINIO_ROOT_PASSWORD",
    "POSTGRES_PASSWORD",
    "DASHBOARD_PASSWORD",
]
missing = [key for key in required_keys if key not in generated]
if missing:
    print(
        "Supabase key generator did not return required keys: "
        + ", ".join(missing),
        file=sys.stderr,
    )
    sys.exit(1)

updated = env_lines
for key in required_keys:
    updated = upsert(updated, key, generated[key])
env_path.write_text("\n".join(updated) + "\n", encoding="utf-8")

print(f"Updated {env_path}")
for key in ("JWT_SECRET", "ANON_KEY", "SERVICE_ROLE_KEY", "POSTGRES_PASSWORD"):
    value = generated[key]
    preview = value[:24] + "..." if len(value) > 24 else value
    print(f"{key}={preview}")
PY
