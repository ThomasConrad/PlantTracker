-- Remove thumbnail system and convert to single AVIF image storage
-- This migration removes the dual image system (original + thumbnail) 
-- in favor of a single optimized AVIF image

-- Remove thumbnail-specific columns from photos table
ALTER TABLE photos DROP COLUMN thumbnail_data;
ALTER TABLE photos DROP COLUMN thumbnail_width; 
ALTER TABLE photos DROP COLUMN thumbnail_height;

-- Remove thumbnail reference from plants table
DROP INDEX IF EXISTS idx_plants_thumbnail_id;
ALTER TABLE plants DROP COLUMN thumbnail_id;

-- Add new columns for AVIF-based system
ALTER TABLE photos ADD COLUMN width INTEGER;
ALTER TABLE photos ADD COLUMN height INTEGER;

-- Note: content_type will be updated to 'image/avif' for all new uploads
-- The data column will now store the optimized AVIF image instead of original + thumbnail