-- Create tribes table
CREATE TABLE IF NOT EXISTS tribes (
    name TEXT PRIMARY KEY,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    description TEXT
);

-- Backfill from user_tribes
INSERT OR IGNORE INTO tribes (name)
SELECT DISTINCT tribe FROM user_tribes;
