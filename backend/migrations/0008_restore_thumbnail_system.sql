-- Add migration script here

-- Restore thumbnail system with AVIF processing
-- This migration restores the thumbnail system but with all images as AVIF

-- Add thumbnail reference back to plants table
-- The thumbnail_id will simply reference an existing photo ID
ALTER TABLE plants ADD COLUMN thumbnail_id TEXT;

-- Create index for thumbnail lookup
CREATE INDEX idx_plants_thumbnail_id ON plants(thumbnail_id);

-- Simple design: photos are just photos, plant thumbnails reference existing photos
-- All photos are stored as AVIF format in the data column
-- A plant's thumbnail is just a reference to whichever photo should be displayed as the thumbnail
