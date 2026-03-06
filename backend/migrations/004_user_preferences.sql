CREATE TABLE IF NOT EXISTS app.user_preferences (
  user_id UUID PRIMARY KEY REFERENCES auth.users(id) ON DELETE CASCADE,
  email_digest BOOLEAN NOT NULL DEFAULT TRUE,
  browser_alerts BOOLEAN NOT NULL DEFAULT TRUE,
  default_page_size INTEGER NOT NULL DEFAULT 20 CHECK (default_page_size IN (10, 20, 50, 100)),
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

DROP TRIGGER IF EXISTS user_preferences_set_updated_at ON app.user_preferences;
CREATE TRIGGER user_preferences_set_updated_at
BEFORE UPDATE ON app.user_preferences
FOR EACH ROW
EXECUTE FUNCTION app.set_updated_at();

ALTER TABLE app.user_preferences ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS user_preferences_select_own ON app.user_preferences;
CREATE POLICY user_preferences_select_own ON app.user_preferences
FOR SELECT
USING (auth.uid() = user_id);

DROP POLICY IF EXISTS user_preferences_insert_own ON app.user_preferences;
CREATE POLICY user_preferences_insert_own ON app.user_preferences
FOR INSERT
WITH CHECK (auth.uid() = user_id);

DROP POLICY IF EXISTS user_preferences_update_own ON app.user_preferences;
CREATE POLICY user_preferences_update_own ON app.user_preferences
FOR UPDATE
USING (auth.uid() = user_id)
WITH CHECK (auth.uid() = user_id);
