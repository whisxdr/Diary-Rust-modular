-- Entry/tag association. Owned by the entries feature.
ALTER TABLE tags ADD CONSTRAINT tags_id_user_id_unique UNIQUE (id, user_id);

CREATE TABLE entry_tags (
    entry_id UUID NOT NULL REFERENCES entries (id) ON DELETE CASCADE,
    tag_id   UUID NOT NULL,
    user_id  UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    PRIMARY KEY (entry_id, tag_id),
    -- Composite key keeps a tag assignment inside a single account.
    FOREIGN KEY (tag_id, user_id) REFERENCES tags (id, user_id) ON DELETE CASCADE
);

CREATE INDEX entry_tags_tag_id_idx ON entry_tags (tag_id);