#!/usr/bin/env python3
import json
import os
import ssl
import time
import urllib.error
import urllib.request
from http.cookiejar import CookieJar
from pathlib import Path
from urllib.parse import ParseResult, urlparse, urlunparse


def build_opener():
    cookie_jar = CookieJar()
    handlers = [urllib.request.HTTPCookieProcessor(cookie_jar)]
    if os.getenv("SMOKE_INSECURE_TLS") == "true":
        handlers.append(urllib.request.HTTPSHandler(context=ssl._create_unverified_context()))
    return urllib.request.build_opener(*handlers)


def request_json(opener, method: str, url: str, payload=None, headers=None):
    body = None
    req_headers = {"content-type": "application/json"}
    if headers:
        req_headers.update(headers)
    if payload is not None:
        body = json.dumps(payload).encode("utf-8")

    req = urllib.request.Request(url, data=body, method=method, headers=req_headers)
    with opener.open(req) as resp:
        raw = resp.read().decode("utf-8")
        return resp.status, json.loads(raw) if raw else {}


def request_json_with_localhost_fallback(opener, method: str, url: str, payload=None, headers=None):
    try:
        return request_json(opener, method, url, payload=payload, headers=headers)
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
        return request_json(opener, method, fallback_url, payload=payload, headers=fallback_headers)


def load_env_file() -> None:
    candidates = []
    explicit = os.getenv("ENV_FILE")
    if explicit:
        candidates.append(Path(explicit))
    candidates.extend([Path(".env.local"), Path(".env"), Path(".env.example")])

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
    api_url = os.getenv("API_URL", "https://api.localhost").rstrip("/")
    email = os.getenv("E2E_TEST_EMAIL", f"reqstly-e2e-{int(time.time())}@example.com")
    password = os.getenv("E2E_TEST_PASSWORD", "Passw0rd-Reqstly!2026")
    opener = build_opener()

    signup_status, signup_payload = request_json_with_localhost_fallback(
        opener,
        "POST",
        f"{api_url}/api/v1/auth/signup",
        payload={"email": email, "password": password, "display_name": "E2E User"},
    )
    if signup_status not in (200, 201):
        raise RuntimeError(f"signup failed: {signup_status} {signup_payload}")

    me_status, me_payload = request_json_with_localhost_fallback(
        opener, "GET", f"{api_url}/api/v1/me"
    )
    assert me_status == 200, f"/me failed after signup: {me_status} {me_payload}"

    logout_status, _ = request_json_with_localhost_fallback(
        opener, "POST", f"{api_url}/api/v1/auth/logout"
    )
    assert logout_status in (200, 204), f"/logout failed: {logout_status}"

    login_status, login_payload = request_json_with_localhost_fallback(
        opener,
        "POST",
        f"{api_url}/api/v1/auth/login/password",
        payload={"email": email, "password": password},
    )
    assert login_status == 200, f"password login failed: {login_status} {login_payload}"

    create_status, create_payload = request_json_with_localhost_fallback(
        opener,
        "POST",
        f"{api_url}/api/v1/requests",
        payload={
            "title": "Session E2E request",
            "description": "created via backend session flow",
            "category": "IT",
            "priority": "medium",
        },
    )
    assert create_status == 201, f"create failed: {create_status} {create_payload}"

    request_id = create_payload["data"]["id"]
    list_status, list_payload = request_json_with_localhost_fallback(
        opener, "GET", f"{api_url}/api/v1/requests"
    )
    assert list_status == 200, f"list failed: {list_status} {list_payload}"
    assert any(item["id"] == request_id for item in list_payload["data"]), "created request missing from list"

    print("Session-backed E2E check passed")


if __name__ == "__main__":
    main()
