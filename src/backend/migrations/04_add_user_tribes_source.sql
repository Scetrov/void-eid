-- Add source column to user_tribes
ALTER TABLE user_tribes ADD COLUMN source TEXT NOT NULL DEFAULT 'MANUAL';
