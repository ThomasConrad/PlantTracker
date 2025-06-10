-- Add photo_ids column to tracking_entries table
-- This allows linking multiple photos to tracking entries
ALTER TABLE tracking_entries ADD COLUMN photo_ids TEXT;