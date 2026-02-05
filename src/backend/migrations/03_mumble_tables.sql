-- Mumble Accounts table
CREATE TABLE IF NOT EXISTS mumble_accounts (
    user_id INTEGER PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Index for username lookups
CREATE INDEX IF NOT EXISTS idx_mumble_accounts_username ON mumble_accounts(username);
