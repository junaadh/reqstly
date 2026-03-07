-- Phase 5 DB hardening refinement aligned to docs/improvements.md.
-- Adds durable primitives for session security state, passkey lifecycle
-- controls, CSRF token tracking, ws-token audit/revocation hooks, and
-- distributed auth rate-limit state.

CREATE TABLE IF NOT EXISTS app.user_auth_security (
  user_id UUID PRIMARY KEY REFERENCES app.app_users(id) ON DELETE CASCADE,
  session_version INTEGER NOT NULL DEFAULT 1 CHECK (session_version > 0),
  require_reauth BOOLEAN NOT NULL DEFAULT FALSE,
  password_login_disabled BOOLEAN NOT NULL DEFAULT FALSE,
  passkey_login_disabled BOOLEAN NOT NULL DEFAULT FALSE,
  locked_until TIMESTAMPTZ,
  last_authn_at TIMESTAMPTZ,
  last_password_login_at TIMESTAMPTZ,
  last_passkey_login_at TIMESTAMPTZ,
  risk_score SMALLINT NOT NULL DEFAULT 0 CHECK (risk_score BETWEEN 0 AND 100),
  compromised_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CHECK (last_authn_at IS NULL OR last_authn_at >= created_at),
  CHECK (
    last_password_login_at IS NULL
    OR last_password_login_at >= created_at
  ),
  CHECK (
    last_passkey_login_at IS NULL
    OR last_passkey_login_at >= created_at
  ),
  CHECK (
    compromised_at IS NULL
    OR compromised_at >= created_at
  )
);

CREATE INDEX IF NOT EXISTS idx_user_auth_security_locked_until
ON app.user_auth_security (locked_until)
WHERE locked_until IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_user_auth_security_risk_score
ON app.user_auth_security (risk_score DESC)
WHERE risk_score > 0;

CREATE OR REPLACE FUNCTION app.ensure_user_auth_security()
RETURNS TRIGGER AS $$
BEGIN
  INSERT INTO app.user_auth_security (user_id)
  VALUES (NEW.id)
  ON CONFLICT (user_id) DO NOTHING;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS app_users_ensure_auth_security ON app.app_users;
CREATE TRIGGER app_users_ensure_auth_security
AFTER INSERT ON app.app_users
FOR EACH ROW
EXECUTE FUNCTION app.ensure_user_auth_security();

INSERT INTO app.user_auth_security (user_id)
SELECT users.id
FROM app.app_users users
ON CONFLICT (user_id) DO NOTHING;

DROP TRIGGER IF EXISTS user_auth_security_set_updated_at ON app.user_auth_security;
CREATE TRIGGER user_auth_security_set_updated_at
BEFORE UPDATE ON app.user_auth_security
FOR EACH ROW
EXECUTE FUNCTION app.set_updated_at();

ALTER TABLE app.user_passkey_credentials
  ADD COLUMN IF NOT EXISTS revoked_at TIMESTAMPTZ,
  ADD COLUMN IF NOT EXISTS revoked_reason TEXT;

DO $$
BEGIN
  ALTER TABLE app.user_passkey_credentials
    ADD CONSTRAINT chk_user_passkey_revocation_state
    CHECK (
      (revoked_at IS NULL AND revoked_reason IS NULL)
      OR revoked_at IS NOT NULL
    );
EXCEPTION
  WHEN duplicate_object THEN NULL;
END;
$$;

DO $$
BEGIN
  ALTER TABLE app.user_passkey_credentials
    ADD CONSTRAINT chk_user_passkey_revoked_reason_length
    CHECK (
      revoked_reason IS NULL
      OR (
        char_length(btrim(revoked_reason)) > 0
        AND char_length(revoked_reason) <= 255
      )
    );
EXCEPTION
  WHEN duplicate_object THEN NULL;
END;
$$;

DO $$
BEGIN
  ALTER TABLE app.user_passkey_credentials
    ADD CONSTRAINT chk_user_passkey_revoked_at_order
    CHECK (
      revoked_at IS NULL
      OR revoked_at >= created_at
    );
EXCEPTION
  WHEN duplicate_object THEN NULL;
END;
$$;

CREATE INDEX IF NOT EXISTS idx_user_passkey_credentials_user_active_created
ON app.user_passkey_credentials (user_id, created_at DESC)
WHERE revoked_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_user_passkey_credentials_user_revoked
ON app.user_passkey_credentials (user_id, revoked_at DESC)
WHERE revoked_at IS NOT NULL;

CREATE OR REPLACE VIEW app.user_passkey_stats AS
SELECT
  credentials.user_id,
  COUNT(*) FILTER (WHERE credentials.revoked_at IS NULL) AS active_passkey_count,
  MIN(credentials.created_at) FILTER (WHERE credentials.revoked_at IS NULL) AS first_registered_at,
  MIN(credentials.first_used_at) FILTER (WHERE credentials.revoked_at IS NULL) AS first_used_at,
  MAX(credentials.last_used_at) FILTER (WHERE credentials.revoked_at IS NULL) AS last_used_at
FROM app.user_passkey_credentials credentials
GROUP BY credentials.user_id;

CREATE TABLE IF NOT EXISTS app.auth_rate_limit_buckets (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  scope TEXT NOT NULL,
  key_hash BYTEA NOT NULL,
  window_started_at TIMESTAMPTZ NOT NULL,
  window_seconds INTEGER NOT NULL
    CHECK (window_seconds BETWEEN 1 AND 86400),
  attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count >= 0),
  blocked_until TIMESTAMPTZ,
  last_attempt_at TIMESTAMPTZ,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
    CHECK (jsonb_typeof(metadata) = 'object'),
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE (scope, key_hash, window_started_at, window_seconds),
  CHECK (
    char_length(btrim(scope)) > 0
    AND char_length(scope) <= 120
  ),
  CHECK (octet_length(key_hash) >= 16),
  CHECK (blocked_until IS NULL OR blocked_until >= window_started_at),
  CHECK (last_attempt_at IS NULL OR last_attempt_at >= window_started_at)
);

CREATE INDEX IF NOT EXISTS idx_auth_rate_limit_buckets_lookup
ON app.auth_rate_limit_buckets (scope, key_hash, window_started_at DESC);

CREATE INDEX IF NOT EXISTS idx_auth_rate_limit_buckets_blocked_until
ON app.auth_rate_limit_buckets (blocked_until)
WHERE blocked_until IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_auth_rate_limit_buckets_window_started_at
ON app.auth_rate_limit_buckets (window_started_at DESC);

CREATE OR REPLACE FUNCTION app.normalize_auth_rate_limit_bucket()
RETURNS TRIGGER AS $$
BEGIN
  NEW.scope := lower(btrim(NEW.scope));
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS auth_rate_limit_buckets_normalize_values ON app.auth_rate_limit_buckets;
CREATE TRIGGER auth_rate_limit_buckets_normalize_values
BEFORE INSERT OR UPDATE OF scope ON app.auth_rate_limit_buckets
FOR EACH ROW
EXECUTE FUNCTION app.normalize_auth_rate_limit_bucket();

DROP TRIGGER IF EXISTS auth_rate_limit_buckets_set_updated_at ON app.auth_rate_limit_buckets;
CREATE TRIGGER auth_rate_limit_buckets_set_updated_at
BEFORE UPDATE ON app.auth_rate_limit_buckets
FOR EACH ROW
EXECUTE FUNCTION app.set_updated_at();

CREATE TABLE IF NOT EXISTS app.csrf_tokens (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  session_id TEXT NOT NULL,
  user_id UUID REFERENCES app.app_users(id) ON DELETE CASCADE,
  purpose TEXT NOT NULL DEFAULT 'session',
  token_hash BYTEA NOT NULL UNIQUE,
  issued_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  expires_at TIMESTAMPTZ NOT NULL,
  consumed_at TIMESTAMPTZ,
  rotated_from_token_id UUID REFERENCES app.csrf_tokens(id) ON DELETE SET NULL,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
    CHECK (jsonb_typeof(metadata) = 'object'),
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CHECK (
    char_length(btrim(session_id)) > 0
    AND char_length(session_id) <= 255
  ),
  CHECK (
    char_length(btrim(purpose)) > 0
    AND char_length(purpose) <= 64
  ),
  CHECK (octet_length(token_hash) >= 16),
  CHECK (expires_at > issued_at),
  CHECK (consumed_at IS NULL OR consumed_at >= issued_at)
);

CREATE INDEX IF NOT EXISTS idx_csrf_tokens_session_purpose_expires
ON app.csrf_tokens (session_id, purpose, expires_at DESC);

CREATE INDEX IF NOT EXISTS idx_csrf_tokens_user_created
ON app.csrf_tokens (user_id, created_at DESC)
WHERE user_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_csrf_tokens_active_expiry
ON app.csrf_tokens (expires_at)
WHERE consumed_at IS NULL;

CREATE OR REPLACE FUNCTION app.normalize_csrf_token()
RETURNS TRIGGER AS $$
BEGIN
  NEW.session_id := btrim(NEW.session_id);
  NEW.purpose := lower(btrim(NEW.purpose));
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS csrf_tokens_normalize_values ON app.csrf_tokens;
CREATE TRIGGER csrf_tokens_normalize_values
BEFORE INSERT OR UPDATE OF session_id, purpose ON app.csrf_tokens
FOR EACH ROW
EXECUTE FUNCTION app.normalize_csrf_token();

DROP TRIGGER IF EXISTS csrf_tokens_set_updated_at ON app.csrf_tokens;
CREATE TRIGGER csrf_tokens_set_updated_at
BEFORE UPDATE ON app.csrf_tokens
FOR EACH ROW
EXECUTE FUNCTION app.set_updated_at();

CREATE TABLE IF NOT EXISTS app.ws_token_issuances (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  user_id UUID NOT NULL REFERENCES app.app_users(id) ON DELETE CASCADE,
  token_fingerprint BYTEA NOT NULL UNIQUE,
  issued_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  expires_at TIMESTAMPTZ NOT NULL,
  revoked_at TIMESTAMPTZ,
  last_used_at TIMESTAMPTZ,
  ip_address INET,
  user_agent TEXT,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
    CHECK (jsonb_typeof(metadata) = 'object'),
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CHECK (octet_length(token_fingerprint) >= 16),
  CHECK (expires_at > issued_at),
  CHECK (revoked_at IS NULL OR revoked_at >= issued_at),
  CHECK (last_used_at IS NULL OR last_used_at >= issued_at),
  CHECK (user_agent IS NULL OR char_length(user_agent) <= 1024)
);

CREATE INDEX IF NOT EXISTS idx_ws_token_issuances_user_issued
ON app.ws_token_issuances (user_id, issued_at DESC);

CREATE INDEX IF NOT EXISTS idx_ws_token_issuances_expires
ON app.ws_token_issuances (expires_at DESC);

CREATE INDEX IF NOT EXISTS idx_ws_token_issuances_revoked
ON app.ws_token_issuances (revoked_at DESC)
WHERE revoked_at IS NOT NULL;

CREATE OR REPLACE FUNCTION app.normalize_ws_token_issuance()
RETURNS TRIGGER AS $$
BEGIN
  IF NEW.user_agent IS NOT NULL THEN
    NEW.user_agent := NULLIF(btrim(NEW.user_agent), '');
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS ws_token_issuances_normalize_values ON app.ws_token_issuances;
CREATE TRIGGER ws_token_issuances_normalize_values
BEFORE INSERT OR UPDATE OF user_agent ON app.ws_token_issuances
FOR EACH ROW
EXECUTE FUNCTION app.normalize_ws_token_issuance();

DROP TRIGGER IF EXISTS ws_token_issuances_set_updated_at ON app.ws_token_issuances;
CREATE TRIGGER ws_token_issuances_set_updated_at
BEFORE UPDATE ON app.ws_token_issuances
FOR EACH ROW
EXECUTE FUNCTION app.set_updated_at();
