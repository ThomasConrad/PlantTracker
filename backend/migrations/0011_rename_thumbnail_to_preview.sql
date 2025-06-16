-- Rename thumbnail_id column to preview_id in plants table
ALTER TABLE plants RENAME COLUMN thumbnail_id TO preview_id;

-- Update index if it exists
DROP INDEX IF EXISTS idx_plants_thumbnail_id;
CREATE INDEX IF NOT EXISTS idx_plants_preview_id ON plants(preview_id);
