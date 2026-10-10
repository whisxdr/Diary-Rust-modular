-- Mood catalog. Seeded, never user-editable, keeps insights aggregations stable.
CREATE TABLE moods (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug       TEXT        NOT NULL UNIQUE,
    label      TEXT        NOT NULL,
    icon       TEXT        NOT NULL,
    score      SMALLINT    NOT NULL,
    position   SMALLINT    NOT NULL
);

INSERT INTO moods (slug, label, icon, score, position) VALUES
    ('radiant',  'Radiant',  'sun',    5, 1),
    ('happy',    'Happy',    'smile',  4, 2),
    ('calm',     'Calm',     'leaf',   3, 3),
    ('tired',    'Tired',    'moon',   2, 4),
    ('anxious',  'Anxious',  'wave',   2, 5),
    ('sad',      'Sad',      'cloud',  1, 6),
    ('angry',    'Angry',    'flame',  1, 7);

-- Diary entries. One entry per user per calendar day keeps the journal timeline simple.
CREATE TABLE entries (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    entry_date DATE        NOT NULL,
    title      TEXT        NOT NULL DEFAULT '',
    content    TEXT        NOT NULL DEFAULT '',
    word_count INTEGER     NOT NULL DEFAULT 0,
    mood_id    UUID        REFERENCES moods (id) ON DELETE SET NULL,
    status     TEXT        NOT NULL DEFAULT 'published',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT entries_status_valid CHECK (status IN ('draft', 'published'))
);

CREATE UNIQUE INDEX entries_user_date_unique ON entries (user_id, entry_date);
CREATE INDEX entries_user_updated_idx ON entries (user_id, updated_at DESC);
CREATE INDEX entries_user_mood_idx ON entries (user_id, mood_id);

-- Full text search over the entry body, owned by the entries feature.
ALTER TABLE entries ADD COLUMN search_vector tsvector
    GENERATED ALWAYS AS (
        setweight(to_tsvector('simple', coalesce(title, '')), 'A') ||
        setweight(to_tsvector('simple', coalesce(content, '')), 'B')
    ) STORED;

CREATE INDEX entries_search_idx ON entries USING GIN (search_vector);