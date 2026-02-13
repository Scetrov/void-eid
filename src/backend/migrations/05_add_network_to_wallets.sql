-- Add network column to wallets table
ALTER TABLE wallets ADD COLUMN network TEXT NOT NULL DEFAULT 'mainnet';
