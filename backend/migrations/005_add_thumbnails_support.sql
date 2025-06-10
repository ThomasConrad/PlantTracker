-- Add thumbnail support to photos and plants tables

-- Add thumbnail fields to photos table
ALTER TABLE photos ADD COLUMN thumbnail_data BLOB;
ALTER TABLE photos ADD COLUMN thumbnail_width INTEGER;
ALTER TABLE photos ADD COLUMN thumbnail_height INTEGER;

-- Add thumbnail reference to plants table
ALTER TABLE plants ADD COLUMN thumbnail_id TEXT;

-- Add foreign key constraint (if supported by the database)
-- For SQLite, this will be handled in the application layer
-- ALTER TABLE plants ADD CONSTRAINT fk_plants_thumbnail_id FOREIGN KEY (thumbnail_id) REFERENCES photos(id) ON DELETE SET NULL;

-- Index for thumbnail lookup
CREATE INDEX idx_plants_thumbnail_id ON plants(thumbnail_id);