-- Update entry_type constraint to include 'note' type
-- SQLite doesn't support altering constraints directly, so we need to recreate the table

-- Create new table with updated constraint
CREATE TABLE tracking_entries_new (
    id TEXT PRIMARY KEY,
    plant_id TEXT NOT NULL,
    metric_id TEXT,
    entry_type TEXT NOT NULL CHECK (entry_type IN ('watering', 'fertilizing', 'measurement', 'note')),
    timestamp TEXT NOT NULL,
    value TEXT, -- JSON stored as TEXT for SQLite compatibility
    notes TEXT,
    photo_ids TEXT, -- JSON array of photo UUIDs
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (plant_id) REFERENCES plants(id) ON DELETE CASCADE,
    FOREIGN KEY (metric_id) REFERENCES custom_metrics(id) ON DELETE SET NULL
);

-- Copy data from old table
INSERT INTO tracking_entries_new (id, plant_id, metric_id, entry_type, timestamp, value, notes, photo_ids, created_at, updated_at)
SELECT id, plant_id, metric_id, entry_type, timestamp, value, notes, photo_ids, created_at, updated_at
FROM tracking_entries;

-- Drop old table
DROP TABLE tracking_entries;

-- Rename new table
ALTER TABLE tracking_entries_new RENAME TO tracking_entries;

-- Recreate indexes
CREATE INDEX idx_tracking_entries_plant_id ON tracking_entries(plant_id);
CREATE INDEX idx_tracking_entries_timestamp ON tracking_entries(timestamp);