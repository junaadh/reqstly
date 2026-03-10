CREATE TABLE IF NOT EXISTS app.auth_events (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  user_id UUID REFERENCES app.app_users(id) ON DELETE SET NULL,
  event_type TEXT NOT NULL,
  success BOOLEAN NOT NULL,
  ip_address INET,
  user_agent TEXT,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CHECK (char_length(btrim(event_type)) > 0 AND char_length(event_type) <= 120),
  CHECK (user_agent IS NULL OR char_length(user_agent) <= 1024),
  CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS idx_auth_events_user_created
ON app.auth_events (user_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_auth_events_type_created
ON app.auth_events (event_type, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_auth_events_type_success_created
ON app.auth_events (event_type, success, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_auth_events_failed_created
ON app.auth_events (created_at DESC)
WHERE success = FALSE;

CREATE OR REPLACE FUNCTION app.normalize_auth_event()
RETURNS TRIGGER AS $$
BEGIN
  NEW.event_type := btrim(NEW.event_type);
  IF NEW.user_agent IS NOT NULL THEN
    NEW.user_agent := NULLIF(btrim(NEW.user_agent), '');
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS auth_events_normalize_values ON app.auth_events;
CREATE TRIGGER auth_events_normalize_values
BEFORE INSERT OR UPDATE OF event_type, user_agent ON app.auth_events
FOR EACH ROW
EXECUTE FUNCTION app.normalize_auth_event();
