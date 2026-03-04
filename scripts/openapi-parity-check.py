#!/usr/bin/env python3
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OPENAPI_FILE = ROOT / "backend" / "openapi" / "openapi.yaml"


EXPECTED_IMPLEMENTATION_ROUTES = {
    ("GET", "/health"),
    ("GET", "/api/v1/health"),
    ("GET", "/api/v1/me"),
    ("GET", "/api/v1/meta/enums"),
    ("GET", "/api/v1/requests"),
    ("POST", "/api/v1/requests"),
    ("GET", "/api/v1/requests/{id}"),
    ("PATCH", "/api/v1/requests/{id}"),
    ("DELETE", "/api/v1/requests/{id}"),
    ("GET", "/api/v1/requests/{id}/audit"),
}


def parse_openapi_routes(path: Path) -> set[tuple[str, str]]:
    lines = path.read_text(encoding="utf-8").splitlines()
    in_paths = False
    current_path = None
    routes: set[tuple[str, str]] = set()

    for raw in lines:
        if not in_paths:
            if raw.strip() == "paths:":
                in_paths = True
            continue

        if re.match(r"^[^ ]", raw):
            break

        path_match = re.match(r"^\s{2}(/[^:]*):\s*$", raw)
        if path_match:
            current_path = path_match.group(1)
            continue

        method_match = re.match(r"^\s{4}(get|post|patch|delete):\s*$", raw)
        if method_match and current_path:
            routes.add((method_match.group(1).upper(), current_path))

    return routes


def main() -> int:
    if not OPENAPI_FILE.exists():
        print(f"OpenAPI file not found: {OPENAPI_FILE}", file=sys.stderr)
        return 1

    documented_routes = parse_openapi_routes(OPENAPI_FILE)

    missing_from_openapi = EXPECTED_IMPLEMENTATION_ROUTES - documented_routes
    undocumented_in_impl_manifest = documented_routes - EXPECTED_IMPLEMENTATION_ROUTES

    if missing_from_openapi:
        print("Missing OpenAPI routes:")
        for method, path in sorted(missing_from_openapi):
            print(f"  - {method} {path}")

    if undocumented_in_impl_manifest:
        print("Extra OpenAPI routes not in implementation route manifest:")
        for method, path in sorted(undocumented_in_impl_manifest):
            print(f"  - {method} {path}")

    if missing_from_openapi or undocumented_in_impl_manifest:
        return 1

    print("OpenAPI parity check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
