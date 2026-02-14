-- Table to store hashed identifiers of deleted accounts to prevent circumvention
CREATE TABLE IF NOT EXISTS identity_hashes (
    hash TEXT PRIMARY KEY,
    type TEXT NOT NULL, -- 'DISCORD' or 'WALLET'
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_identity_hashes_type ON identity_hashes(type);
