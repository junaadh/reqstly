#!/usr/bin/env python3
import json
import os
import ssl
import time
import urllib.error
import urllib.request
from pathlib import Path
from urllib.parse import ParseResult, urlparse, urlunparse


def request_json(method: str, url: str, payload=None, headers=None):
    body = None
    req_headers = {"content-type": "application/json"}
    if headers:
        req_headers.update(headers)
    if payload is not None:
        body = json.dumps(payload).encode("utf-8")

    req = urllib.request.Request(url, data=body, method=method, headers=req_headers)
    ctx = ssl._create_unverified_context() if os.getenv("SMOKE_INSECURE_TLS") == "true" else None
    with urllib.request.urlopen(req, context=ctx) as resp:
        raw = resp.read().decode("utf-8")
        return resp.status, json.loads(raw) if raw else {}


def request_json_with_localhost_fallback(method: str, url: str, payload=None, headers=None):
    try:
        return request_json(method, url, payload=payload, headers=headers)
    except urllib.error.HTTPError:
        raise
    except urllib.error.URLError:
        if os.getenv("SMOKE_LOCALHOST_RESOLVE_FALLBACK", "true") != "true":
            raise

        parsed = urlparse(url)
        host = parsed.hostname
        if not host or not host.endswith(".localhost"):
            raise

        fallback_host = os.getenv("LOCALHOST_FALLBACK_HOST", "127.0.0.1")
        port = parsed.port
        if port is None:
            port = 443 if parsed.scheme == "https" else 80

        fallback_netloc = f"{fallback_host}:{port}"
        fallback_url = urlunparse(
            ParseResult(
                scheme=parsed.scheme,
                netloc=fallback_netloc,
                path=parsed.path,
                params=parsed.params,
                query=parsed.query,
                fragment=parsed.fragment,
            )
        )

        fallback_headers = dict(headers or {})
        fallback_headers["Host"] = host if parsed.port is None else f"{host}:{parsed.port}"
        return request_json(method, fallback_url, payload=payload, headers=fallback_headers)


def load_env_file() -> None:
    candidates = []
    explicit = os.getenv("ENV_FILE")
    if explicit:
        candidates.append(Path(explicit))
    candidates.extend(
        [
            Path(".env.local"),
            Path(".env"),
            Path(".env.example"),
        ]
    )

    env_file = next((path for path in candidates if path.exists()), None)
    if env_file is None:
        return

    for raw in env_file.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        os.environ.setdefault(key, value)


def main():
    load_env_file()
    api_url = os.getenv("API_URL", "https://api.localhost")
    supabase_url = os.getenv("SUPABASE_PUBLIC_URL") or os.getenv("SUPABASE_URL") or "https://supabase.localhost"
    anon_key = os.getenv("ANON_KEY") or os.getenv("SUPABASE_ANON_KEY")
    email = os.getenv("E2E_TEST_EMAIL", f"reqstly-e2e-{int(time.time())}@example.com")
    password = os.getenv("E2E_TEST_PASSWORD", "Passw0rd-Reqstly!")
    auth_headers = {"apikey": anon_key} if anon_key else None

    signup_url = f"{supabase_url}/auth/v1/signup"
    token = None

    try:
        _, signup_payload = request_json_with_localhost_fallback(
            "POST",
            signup_url,
            payload={"email": email, "password": password},
            headers=auth_headers,
        )
        token = signup_payload.get("access_token")
    except urllib.error.HTTPError as err:
        # User may already exist; fall back to password grant.
        if err.code not in (400, 422):
            raise

    if not token:
        grant_url = f"{supabase_url}/auth/v1/token?grant_type=password"
        _, grant_payload = request_json_with_localhost_fallback(
            "POST",
            grant_url,
            payload={"email": email, "password": password},
            headers=auth_headers,
        )
        token = grant_payload.get("access_token")

    if not token:
        raise RuntimeError("Could not obtain Supabase access token")

    auth_headers = {"Authorization": f"Bearer {token}"}

    me_status, me_payload = request_json_with_localhost_fallback(
        "GET", f"{api_url}/api/v1/me", headers=auth_headers
    )
    assert me_status == 200, f"/me failed: {me_status} {me_payload}"

    create_status, create_payload = request_json_with_localhost_fallback(
        "POST",
        f"{api_url}/api/v1/requests",
        payload={
            "title": "Token E2E request",
            "description": "created via Supabase token flow",
            "category": "IT",
            "priority": "medium",
        },
        headers=auth_headers,
    )
    assert create_status == 201, f"create failed: {create_status} {create_payload}"

    request_id = create_payload["data"]["id"]
    list_status, list_payload = request_json_with_localhost_fallback(
        "GET", f"{api_url}/api/v1/requests", headers=auth_headers
    )
    assert list_status == 200, f"list failed: {list_status} {list_payload}"
    assert any(item["id"] == request_id for item in list_payload["data"]), "created request missing from list"

    print("Token-backed E2E check passed")


if __name__ == "__main__":
    main()
