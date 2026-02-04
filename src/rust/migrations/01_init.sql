-- Users table
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
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
    user_id TEXT NOT NULL,
    address TEXT NOT NULL,
    verified_at DATETIME NOT NULL,
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- User-Tribe junction table (many-to-many relationship)
CREATE TABLE IF NOT EXISTS user_tribes (
    user_id TEXT NOT NULL,
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
    actor_id TEXT NOT NULL,
    target_id TEXT,
    details TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(actor_id) REFERENCES users(id)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_user_tribes_user_id ON user_tribes(user_id);
CREATE INDEX IF NOT EXISTS idx_user_tribes_tribe ON user_tribes(tribe);
CREATE INDEX IF NOT EXISTS idx_wallets_user_id ON wallets(user_id);
CREATE INDEX IF NOT EXISTS idx_wallets_address ON wallets(address);
CREATE INDEX IF NOT EXISTS idx_audit_logs_actor_id ON audit_logs(actor_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_target_id ON audit_logs(target_id);
