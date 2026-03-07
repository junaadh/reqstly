CREATE TABLE IF NOT EXISTS app.app_users (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  email TEXT,
  display_name TEXT NOT NULL DEFAULT 'user',
  email_verified BOOLEAN NOT NULL DEFAULT FALSE,
  is_active BOOLEAN NOT NULL DEFAULT TRUE,
  is_sso_user BOOLEAN NOT NULL DEFAULT FALSE,
  is_anonymous BOOLEAN NOT NULL DEFAULT FALSE,
  deleted_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  last_login_at TIMESTAMPTZ,
  CHECK (char_length(btrim(display_name)) > 0),
  CHECK (char_length(display_name) <= 120),
  CHECK (
    email IS NULL
    OR (
      email = btrim(email)
      AND char_length(email) <= 320
      AND position('@' IN email) > 1
    )
  ),
  CHECK (deleted_at IS NULL OR is_active = FALSE)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_app_users_email_ci_unique
ON app.app_users (lower(email))
WHERE email IS NOT NULL
  AND deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_app_users_active_created_at
ON app.app_users (created_at DESC)
WHERE is_active = TRUE
  AND deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_app_users_deleted_at
ON app.app_users (deleted_at)
WHERE deleted_at IS NOT NULL;

CREATE OR REPLACE FUNCTION app.normalize_app_user()
RETURNS TRIGGER AS $$
BEGIN
  NEW.email := app.normalize_email(NEW.email);
  NEW.display_name := btrim(NEW.display_name);

  IF NEW.deleted_at IS NOT NULL THEN
    NEW.is_active := FALSE;
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS app_users_normalize_values ON app.app_users;
CREATE TRIGGER app_users_normalize_values
BEFORE INSERT OR UPDATE OF email, display_name, deleted_at, is_active ON app.app_users
FOR EACH ROW
EXECUTE FUNCTION app.normalize_app_user();

DROP TRIGGER IF EXISTS app_users_set_updated_at ON app.app_users;
CREATE TRIGGER app_users_set_updated_at
BEFORE UPDATE ON app.app_users
FOR EACH ROW
EXECUTE FUNCTION app.set_updated_at();
