-- Attachments hold metadata only; bytes live on disk under the storage root.
CREATE TABLE attachments (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id       UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    entry_id      UUID        REFERENCES entries (id) ON DELETE CASCADE,
    original_name TEXT        NOT NULL,
    stored_path   TEXT        NOT NULL,
    content_type  TEXT        NOT NULL,
    size_bytes    BIGINT      NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

ALTER TABLE attachments ADD CONSTRAINT attachments_id_user_id_unique UNIQUE (id, user_id);

CREATE INDEX attachments_user_id_idx ON attachments (user_id, created_at DESC);
CREATE INDEX attachments_entry_id_idx ON attachments (entry_id);