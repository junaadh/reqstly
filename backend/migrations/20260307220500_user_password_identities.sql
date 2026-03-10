CREATE TABLE IF NOT EXISTS app.user_password_identities (
  user_id UUID PRIMARY KEY REFERENCES app.app_users(id) ON DELETE CASCADE,
  email TEXT NOT NULL,
  password_hash TEXT NOT NULL,
  failed_attempts INTEGER NOT NULL DEFAULT 0,
  last_failed_at TIMESTAMPTZ,
  locked_until TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  last_password_change_at TIMESTAMPTZ,
  CHECK (
    email = btrim(email)
    AND char_length(email) <= 320
    AND position('@' IN email) > 1
  ),
  CHECK (char_length(btrim(password_hash)) > 0),
  CHECK (failed_attempts >= 0),
  CHECK (
    locked_until IS NULL
    OR last_failed_at IS NULL
    OR locked_until >= last_failed_at
  )
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_user_password_identities_email_ci_unique
ON app.user_password_identities (lower(email));

CREATE INDEX IF NOT EXISTS idx_user_password_identities_locked_until
ON app.user_password_identities (locked_until)
WHERE locked_until IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_user_password_identities_failed_attempts
ON app.user_password_identities (failed_attempts)
WHERE failed_attempts > 0;

CREATE OR REPLACE FUNCTION app.normalize_password_identity()
RETURNS TRIGGER AS $$
DECLARE
  user_email TEXT;
  user_deleted_at TIMESTAMPTZ;
BEGIN
  NEW.email := app.normalize_email(NEW.email);

  SELECT email, deleted_at
  INTO user_email, user_deleted_at
  FROM app.app_users
  WHERE id = NEW.user_id
  FOR UPDATE;

  IF NOT FOUND THEN
    RAISE EXCEPTION 'app user % not found for password identity', NEW.user_id;
  END IF;

  IF user_deleted_at IS NOT NULL THEN
    RAISE EXCEPTION 'cannot attach password identity to deleted user %', NEW.user_id;
  END IF;

  IF user_email IS NULL THEN
    UPDATE app.app_users
    SET email = NEW.email
    WHERE id = NEW.user_id;
  ELSIF lower(user_email) <> NEW.email THEN
    RAISE EXCEPTION 'password identity email must match app user email';
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS user_password_identities_normalize_values ON app.user_password_identities;
CREATE TRIGGER user_password_identities_normalize_values
BEFORE INSERT OR UPDATE OF email ON app.user_password_identities
FOR EACH ROW
EXECUTE FUNCTION app.normalize_password_identity();

DROP TRIGGER IF EXISTS user_password_identities_set_updated_at ON app.user_password_identities;
CREATE TRIGGER user_password_identities_set_updated_at
BEFORE UPDATE ON app.user_password_identities
FOR EACH ROW
EXECUTE FUNCTION app.set_updated_at();
