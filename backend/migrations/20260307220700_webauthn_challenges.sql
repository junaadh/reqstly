CREATE TABLE IF NOT EXISTS app.webauthn_challenges (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  user_id UUID REFERENCES app.app_users(id) ON DELETE CASCADE,
  flow_type TEXT NOT NULL CHECK (flow_type IN ('register', 'authenticate')),
  challenge_blob JSONB NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  consumed_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CHECK (jsonb_typeof(challenge_blob) = 'object'),
  CHECK (expires_at > created_at),
  CHECK (consumed_at IS NULL OR consumed_at >= created_at)
);

CREATE INDEX IF NOT EXISTS idx_webauthn_challenges_flow_expires
ON app.webauthn_challenges (flow_type, expires_at);

CREATE INDEX IF NOT EXISTS idx_webauthn_challenges_user_flow_created
ON app.webauthn_challenges (user_id, flow_type, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_webauthn_challenges_active_expiry
ON app.webauthn_challenges (flow_type, expires_at)
WHERE consumed_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_webauthn_challenges_created_at
ON app.webauthn_challenges (created_at DESC);
