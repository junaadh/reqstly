CREATE TABLE IF NOT EXISTS app.requests (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  owner_user_id UUID NOT NULL REFERENCES app.app_users(id) ON DELETE CASCADE,
  title VARCHAR(255) NOT NULL CHECK (char_length(btrim(title)) > 0),
  description TEXT CHECK (description IS NULL OR char_length(description) <= 5000),
  category VARCHAR(20) NOT NULL CHECK (category IN ('IT', 'Ops', 'Admin', 'HR')),
  status VARCHAR(20) NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'in_progress', 'resolved')),
  priority VARCHAR(20) NOT NULL DEFAULT 'medium' CHECK (priority IN ('low', 'medium', 'high')),
  assignee_user_id UUID REFERENCES app.app_users(id) ON DELETE SET NULL,
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
  actor_user_id UUID NOT NULL REFERENCES app.app_users(id) ON DELETE CASCADE,
  action VARCHAR(20) NOT NULL CHECK (action IN ('created', 'updated', 'deleted', 'status_changed')),
  old_value JSONB NOT NULL DEFAULT '{}'::jsonb CHECK (jsonb_typeof(old_value) = 'object'),
  new_value JSONB NOT NULL DEFAULT '{}'::jsonb CHECK (jsonb_typeof(new_value) = 'object'),
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
CREATE INDEX IF NOT EXISTS idx_request_audit_logs_actor_created_at
ON app.request_audit_logs(actor_user_id, created_at DESC);

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
