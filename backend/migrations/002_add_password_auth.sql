-- Password authentication table (separate from users for better security and flexibility)
CREATE TABLE IF NOT EXISTS passwords (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID UNIQUE NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Index for faster lookups during login
CREATE INDEX idx_passwords_user_id ON passwords(user_id);

-- Update sessions table to allow password authentication
ALTER TABLE sessions DROP CONSTRAINT IF EXISTS sessions_provider_check;
ALTER TABLE sessions ADD CONSTRAINT sessions_provider_check
    CHECK (provider IN ('azure_ad', 'passkey', 'password'));

-- Function to update updated_at timestamp for passwords
CREATE TRIGGER update_passwords_updated_at BEFORE UPDATE ON passwords
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
