CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE SCHEMA IF NOT EXISTS app;

CREATE OR REPLACE FUNCTION app.set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION app.normalize_email(raw_email TEXT)
RETURNS TEXT AS $$
DECLARE
  normalized TEXT;
BEGIN
  IF raw_email IS NULL THEN
    RETURN NULL;
  END IF;

  normalized := lower(btrim(raw_email));
  IF normalized = '' THEN
    RETURN NULL;
  END IF;

  RETURN normalized;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

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
