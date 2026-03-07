CREATE TABLE IF NOT EXISTS app.user_oidc_identities (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  user_id UUID NOT NULL REFERENCES app.app_users(id) ON DELETE CASCADE,
  provider TEXT NOT NULL,
  subject TEXT NOT NULL,
  email TEXT,
  claims JSONB,
  identity_data JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  last_login_at TIMESTAMPTZ,
  UNIQUE (provider, subject),
  CHECK (char_length(btrim(provider)) > 0),
  CHECK (char_length(btrim(subject)) > 0),
  CHECK (
    email IS NULL
    OR (
      email = btrim(email)
      AND char_length(email) <= 320
      AND position('@' IN email) > 1
    )
  ),
  CHECK (claims IS NULL OR jsonb_typeof(claims) = 'object'),
  CHECK (jsonb_typeof(identity_data) = 'object')
);

CREATE INDEX IF NOT EXISTS idx_user_oidc_identities_user_id
ON app.user_oidc_identities (user_id);

CREATE INDEX IF NOT EXISTS idx_user_oidc_identities_user_provider
ON app.user_oidc_identities (user_id, provider);

CREATE INDEX IF NOT EXISTS idx_user_oidc_identities_provider_email_ci
ON app.user_oidc_identities (provider, lower(email))
WHERE email IS NOT NULL;

CREATE OR REPLACE FUNCTION app.normalize_user_oidc_identity()
RETURNS TRIGGER AS $$
BEGIN
  NEW.provider := lower(btrim(NEW.provider));
  NEW.subject := btrim(NEW.subject);
  NEW.email := app.normalize_email(NEW.email);
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS user_oidc_identities_normalize_values ON app.user_oidc_identities;
CREATE TRIGGER user_oidc_identities_normalize_values
BEFORE INSERT OR UPDATE OF provider, subject, email ON app.user_oidc_identities
FOR EACH ROW
EXECUTE FUNCTION app.normalize_user_oidc_identity();

DROP TRIGGER IF EXISTS user_oidc_identities_set_updated_at ON app.user_oidc_identities;
CREATE TRIGGER user_oidc_identities_set_updated_at
BEFORE UPDATE ON app.user_oidc_identities
FOR EACH ROW
EXECUTE FUNCTION app.set_updated_at();
