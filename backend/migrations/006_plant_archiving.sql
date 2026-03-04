ALTER TABLE plants ADD COLUMN archived_at TEXT;

CREATE INDEX idx_plants_archived_at ON plants(archived_at);
