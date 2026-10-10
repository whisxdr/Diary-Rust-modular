-- Import history. A row is created before the archive is parsed and updated once the
-- transaction commits, so an interrupted import is visible to the user and can be resumed.
CREATE TABLE import_jobs (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    filename    TEXT        NOT NULL,
    status      TEXT        NOT NULL,
    summary     JSONB       NOT NULL DEFAULT '{}'::jsonb,
    -- The validated archive is kept so a commit can run in a later request.
    archive     JSONB       NOT NULL DEFAULT '{}'::jsonb,
    error       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    CONSTRAINT import_jobs_status_valid CHECK (status IN ('pending', 'completed', 'failed'))
);

CREATE INDEX import_jobs_user_id_idx ON import_jobs (user_id, created_at DESC);