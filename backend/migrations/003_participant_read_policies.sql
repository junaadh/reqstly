DROP POLICY IF EXISTS requests_select_own ON app.requests;
DROP POLICY IF EXISTS requests_select_participant ON app.requests;
CREATE POLICY requests_select_participant ON app.requests
FOR SELECT
USING (
  EXISTS (
    SELECT 1
    FROM app.request_participants participants
    WHERE participants.request_id = app.requests.id
      AND participants.user_id = auth.uid()
  )
);

DROP POLICY IF EXISTS request_audit_logs_select_own ON app.request_audit_logs;
DROP POLICY IF EXISTS request_audit_logs_select_participant ON app.request_audit_logs;
CREATE POLICY request_audit_logs_select_participant ON app.request_audit_logs
FOR SELECT
USING (
  EXISTS (
    SELECT 1
    FROM app.request_participants participants
    WHERE participants.request_id = app.request_audit_logs.request_id
      AND participants.user_id = auth.uid()
  )
);

