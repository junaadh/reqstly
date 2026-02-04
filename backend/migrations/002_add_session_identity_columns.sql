-- Add missing session identity columns if they don't exist
ALTER TABLE sessions
    ADD COLUMN IF NOT EXISTS external_identity_id UUID REFERENCES external_identities(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS provider VARCHAR(50) NOT NULL DEFAULT 'azure_ad' CHECK (provider IN ('azure_ad', 'passkey'));

CREATE INDEX IF NOT EXISTS idx_sessions_external_identity_id
    ON sessions(external_identity_id)
    WHERE external_identity_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_sessions_provider ON sessions(provider);
