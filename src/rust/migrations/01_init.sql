CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    discord_id TEXT NOT NULL,
    username TEXT NOT NULL,
    discriminator TEXT NOT NULL,
    avatar TEXT
);
CREATE TABLE IF NOT EXISTS wallets (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    address TEXT NOT NULL,
    verified_at DATETIME NOT NULL,
    FOREIGN KEY(user_id) REFERENCES users(id)
);
