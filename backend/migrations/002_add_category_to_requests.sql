-- Add category column to requests for older databases
ALTER TABLE requests
    ADD COLUMN IF NOT EXISTS category VARCHAR(50) NOT NULL DEFAULT 'IT';

DO $$
BEGIN
    ALTER TABLE requests
        ADD CONSTRAINT requests_category_check
        CHECK (category IN ('IT', 'Ops', 'Admin', 'HR'));
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

CREATE INDEX IF NOT EXISTS idx_requests_category ON requests(category);
