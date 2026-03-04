CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE SCHEMA IF NOT EXISTS app;

CREATE OR REPLACE FUNCTION app.set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION app.sync_request_status_timestamps()
RETURNS TRIGGER AS $$
BEGIN
  IF NEW.status = 'resolved' AND (TG_OP = 'INSERT' OR OLD.status IS DISTINCT FROM 'resolved') THEN
    NEW.resolved_at = COALESCE(NEW.resolved_at, NOW());
  ELSIF NEW.status <> 'resolved' THEN
    NEW.resolved_at = NULL;
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE IF NOT EXISTS app.requests (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  owner_user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
  title VARCHAR(255) NOT NULL CHECK (length(btrim(title)) > 0),
  description TEXT CHECK (description IS NULL OR length(description) <= 5000),
  category VARCHAR(20) NOT NULL CHECK (category IN ('IT', 'Ops', 'Admin', 'HR')),
  status VARCHAR(20) NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'in_progress', 'resolved')),
  priority VARCHAR(20) NOT NULL DEFAULT 'medium' CHECK (priority IN ('low', 'medium', 'high')),
  assignee_user_id UUID REFERENCES auth.users(id) ON DELETE SET NULL,
  resolved_at TIMESTAMPTZ,
  due_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CHECK (
    (status = 'resolved' AND resolved_at IS NOT NULL)
    OR (status <> 'resolved' AND resolved_at IS NULL)
  )
);

CREATE TABLE IF NOT EXISTS app.request_audit_logs (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  request_id UUID NOT NULL REFERENCES app.requests(id) ON DELETE CASCADE,
  actor_user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
  action VARCHAR(20) NOT NULL CHECK (action IN ('created', 'updated', 'deleted', 'status_changed')),
  old_value JSONB NOT NULL DEFAULT '{}'::jsonb,
  new_value JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_requests_owner_user_id ON app.requests(owner_user_id);
CREATE INDEX IF NOT EXISTS idx_requests_owner_user_created_at
ON app.requests(owner_user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_requests_owner_user_updated_at
ON app.requests(owner_user_id, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_requests_owner_user_status_created_at
ON app.requests(owner_user_id, status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_requests_owner_user_category_created_at
ON app.requests(owner_user_id, category, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_requests_owner_user_priority_created_at
ON app.requests(owner_user_id, priority, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_requests_assignee_status_updated_at
ON app.requests(assignee_user_id, status, updated_at DESC)
WHERE assignee_user_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_requests_status ON app.requests(status);
CREATE INDEX IF NOT EXISTS idx_requests_category ON app.requests(category);
CREATE INDEX IF NOT EXISTS idx_requests_priority ON app.requests(priority);
CREATE INDEX IF NOT EXISTS idx_requests_created_at ON app.requests(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_request_audit_logs_request_id_created_at
ON app.request_audit_logs(request_id, created_at DESC);

DROP TRIGGER IF EXISTS requests_set_updated_at ON app.requests;
CREATE TRIGGER requests_set_updated_at
BEFORE UPDATE ON app.requests
FOR EACH ROW
EXECUTE FUNCTION app.set_updated_at();

DROP TRIGGER IF EXISTS requests_sync_status_timestamps ON app.requests;
CREATE TRIGGER requests_sync_status_timestamps
BEFORE INSERT OR UPDATE OF status, resolved_at ON app.requests
FOR EACH ROW
EXECUTE FUNCTION app.sync_request_status_timestamps();

ALTER TABLE app.requests ENABLE ROW LEVEL SECURITY;
ALTER TABLE app.request_audit_logs ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS requests_select_own ON app.requests;
CREATE POLICY requests_select_own ON app.requests
FOR SELECT
USING (auth.uid() = owner_user_id);

DROP POLICY IF EXISTS requests_insert_own ON app.requests;
CREATE POLICY requests_insert_own ON app.requests
FOR INSERT
WITH CHECK (auth.uid() = owner_user_id);

DROP POLICY IF EXISTS requests_update_own ON app.requests;
CREATE POLICY requests_update_own ON app.requests
FOR UPDATE
USING (auth.uid() = owner_user_id)
WITH CHECK (auth.uid() = owner_user_id);

DROP POLICY IF EXISTS requests_delete_own ON app.requests;
CREATE POLICY requests_delete_own ON app.requests
FOR DELETE
USING (auth.uid() = owner_user_id);

DROP POLICY IF EXISTS request_audit_logs_select_own ON app.request_audit_logs;
CREATE POLICY request_audit_logs_select_own ON app.request_audit_logs
FOR SELECT
USING (
  EXISTS (
    SELECT 1
    FROM app.requests req
    WHERE req.id = request_audit_logs.request_id
      AND req.owner_user_id = auth.uid()
  )
);

DO $$
BEGIN
  IF EXISTS (
    SELECT 1
    FROM pg_publication
    WHERE pubname = 'supabase_realtime'
  ) THEN
    IF NOT EXISTS (
      SELECT 1
      FROM pg_publication_tables
      WHERE pubname = 'supabase_realtime'
        AND schemaname = 'app'
        AND tablename = 'requests'
    ) THEN
      ALTER PUBLICATION supabase_realtime ADD TABLE app.requests;
    END IF;

    IF NOT EXISTS (
      SELECT 1
      FROM pg_publication_tables
      WHERE pubname = 'supabase_realtime'
        AND schemaname = 'app'
        AND tablename = 'request_audit_logs'
    ) THEN
      ALTER PUBLICATION supabase_realtime ADD TABLE app.request_audit_logs;
    END IF;
  END IF;
END;
$$;
