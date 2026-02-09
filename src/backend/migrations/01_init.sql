-- Users table
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY,
    discord_id TEXT NOT NULL,
    username TEXT NOT NULL,
    discriminator TEXT NOT NULL,
    avatar TEXT,
    is_admin BOOLEAN DEFAULT FALSE,
    last_login_at DATETIME
);

-- Wallets table
CREATE TABLE IF NOT EXISTS wallets (
    id TEXT PRIMARY KEY,
    user_id INTEGER NOT NULL,
    address TEXT NOT NULL,
    verified_at DATETIME NOT NULL,
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- User-Tribe junction table (many-to-many relationship)
CREATE TABLE IF NOT EXISTS user_tribes (
    user_id INTEGER NOT NULL,
    tribe TEXT NOT NULL,
    wallet_id TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    is_admin BOOLEAN DEFAULT FALSE,
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY(wallet_id) REFERENCES wallets(id) ON DELETE SET NULL,
    UNIQUE(user_id, tribe)
);

-- Audit logs table
CREATE TABLE IF NOT EXISTS audit_logs (
    id TEXT PRIMARY KEY,
    action TEXT NOT NULL,
    actor_id INTEGER NOT NULL,
    target_id INTEGER,
    details TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(actor_id) REFERENCES users(id)
);

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

-- Mumble Accounts table
CREATE TABLE IF NOT EXISTS mumble_accounts (
    user_id INTEGER PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_user_tribes_user_id ON user_tribes(user_id);
CREATE INDEX IF NOT EXISTS idx_user_tribes_tribe ON user_tribes(tribe);
CREATE INDEX IF NOT EXISTS idx_wallets_user_id ON wallets(user_id);
-- Unique index on address (replaces original non-unique index from earlier migration)
CREATE UNIQUE INDEX IF NOT EXISTS idx_wallets_address_unique ON wallets(address);

CREATE INDEX IF NOT EXISTS idx_audit_logs_actor_id ON audit_logs(actor_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_target_id ON audit_logs(target_id);

CREATE INDEX IF NOT EXISTS idx_notes_target_user_id ON notes(target_user_id);
CREATE INDEX IF NOT EXISTS idx_notes_tribe ON notes(tribe);
CREATE INDEX IF NOT EXISTS idx_notes_author_id ON notes(author_id);

CREATE INDEX IF NOT EXISTS idx_mumble_accounts_username ON mumble_accounts(username);
