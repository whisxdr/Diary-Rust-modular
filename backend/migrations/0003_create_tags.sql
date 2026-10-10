-- Tags are per account. Case-insensitive uniqueness, enforced with a generated column
-- so the repository never has to normalize the name itself.
CREATE TABLE tags (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name       TEXT        NOT NULL,
    color      TEXT,
    normalized TEXT GENERATED ALWAYS AS (lower(name)) STORED,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX tags_user_name_unique ON tags (user_id, normalized);