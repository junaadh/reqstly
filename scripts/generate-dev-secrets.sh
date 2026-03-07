#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ENV_FILE="${1:-${ENV_FILE:-${ROOT_DIR}/.env.local}}"

if [[ ! -f "${ENV_FILE}" ]]; then
  echo "Env file not found: ${ENV_FILE}"
  echo "Usage: $0 [path-to-env-file]"
  exit 1
fi

python3 - "${ENV_FILE}" <<'PY'
import secrets
import re
import sys
from pathlib import Path


def upsert(lines: list[str], key: str, value: str) -> list[str]:
    prefix = f"{key}="
    replaced = False
    output: list[str] = []

    for line in lines:
        if line.startswith(prefix):
            output.append(f"{prefix}{value}")
            replaced = True
        else:
            output.append(line)

    if not replaced:
        output.append(f"{prefix}{value}")

    return output


env_path = Path(sys.argv[1]).resolve()
lines = env_path.read_text(encoding="utf-8").splitlines()

generated = {
    "POSTGRES_PASSWORD": secrets.token_urlsafe(24),
    "AUTH__WS_TOKEN_SECRET": secrets.token_urlsafe(48),
    "GRAFANA_ADMIN_PASSWORD": secrets.token_urlsafe(20),
}

updated = lines
for key, value in generated.items():
    updated = upsert(updated, key, value)

env_path.write_text("\n".join(updated) + "\n", encoding="utf-8")

print(f"Updated {env_path}")
for key in generated:
    preview = generated[key][:16] + "..."
    print(f"{key}={preview}")
PY
