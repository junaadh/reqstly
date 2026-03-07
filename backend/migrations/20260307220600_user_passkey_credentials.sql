CREATE TABLE IF NOT EXISTS app.user_passkey_credentials (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  user_id UUID NOT NULL REFERENCES app.app_users(id) ON DELETE CASCADE,
  credential_id BYTEA NOT NULL UNIQUE,
  public_key BYTEA,
  credential_json JSONB NOT NULL,
  sign_count BIGINT NOT NULL DEFAULT 0,
  transports TEXT[],
  aaguid UUID,
  nickname TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  first_used_at TIMESTAMPTZ,
  last_used_at TIMESTAMPTZ,
  CHECK (octet_length(credential_id) > 0),
  CHECK (jsonb_typeof(credential_json) = 'object'),
  CHECK (sign_count >= 0),
  CHECK (nickname IS NULL OR char_length(nickname) <= 120),
  CHECK (first_used_at IS NULL OR first_used_at >= created_at),
  CHECK (last_used_at IS NULL OR last_used_at >= created_at),
  CHECK (
    first_used_at IS NULL
    OR last_used_at IS NULL
    OR first_used_at <= last_used_at
  )
);

CREATE INDEX IF NOT EXISTS idx_user_passkey_credentials_user_created
ON app.user_passkey_credentials (user_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_user_passkey_credentials_user_first_used
ON app.user_passkey_credentials (user_id, first_used_at ASC)
WHERE first_used_at IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_user_passkey_credentials_user_last_used
ON app.user_passkey_credentials (user_id, last_used_at DESC)
WHERE last_used_at IS NOT NULL;

CREATE OR REPLACE FUNCTION app.normalize_passkey_credential()
RETURNS TRIGGER AS $$
BEGIN
  IF NEW.nickname IS NOT NULL THEN
    NEW.nickname := NULLIF(btrim(NEW.nickname), '');
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS user_passkey_credentials_normalize_values ON app.user_passkey_credentials;
CREATE TRIGGER user_passkey_credentials_normalize_values
BEFORE INSERT OR UPDATE OF nickname ON app.user_passkey_credentials
FOR EACH ROW
EXECUTE FUNCTION app.normalize_passkey_credential();

DROP TRIGGER IF EXISTS user_passkey_credentials_set_updated_at ON app.user_passkey_credentials;
CREATE TRIGGER user_passkey_credentials_set_updated_at
BEFORE UPDATE ON app.user_passkey_credentials
FOR EACH ROW
EXECUTE FUNCTION app.set_updated_at();
