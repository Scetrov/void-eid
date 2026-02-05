-- Notes/Comments table
CREATE TABLE IF NOT EXISTS notes (
    id TEXT PRIMARY KEY,
    target_user_id INTEGER NOT NULL,
    author_id INTEGER NOT NULL,
    tribe TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(target_user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY(author_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Index for efficient queries
CREATE INDEX IF NOT EXISTS idx_notes_target_user_id ON notes(target_user_id);
CREATE INDEX IF NOT EXISTS idx_notes_tribe ON notes(tribe);
CREATE INDEX IF NOT EXISTS idx_notes_author_id ON notes(author_id);
