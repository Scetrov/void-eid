CREATE UNIQUE INDEX IF NOT EXISTS idx_wallets_address_unique ON wallets(address);
DROP INDEX IF EXISTS idx_wallets_address;
