-- Add deleted_at column for soft delete
ALTER TABLE wallets ADD COLUMN deleted_at DATETIME;

-- Index for searching active/deleted wallets
CREATE INDEX idx_wallets_deleted_at ON wallets(deleted_at);
