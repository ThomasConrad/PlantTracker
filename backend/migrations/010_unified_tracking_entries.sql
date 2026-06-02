-- Migration 010: Unify tracking entries
-- Remove entry_type discriminator; an entry can now combine:
-- - Multiple care task completions (care_task_ids JSON array)
-- - Multiple measurements (measurements JSON array of {metric_id, value})
-- - Notes (text)
-- - Photos (photo_ids JSON array)
-- All optional, any combination valid.

-- Rebuild tracking_entries table
CREATE TABLE tracking_entries_new (
    id TEXT PRIMARY KEY NOT NULL,
    plant_id TEXT NOT NULL REFERENCES plants(id) ON DELETE CASCADE,
    timestamp TEXT NOT NULL,
    care_task_ids TEXT,           -- JSON array of care task UUIDs completed in this entry
    measurements TEXT,            -- JSON array of {metricId: string, value: any}
    notes TEXT,
    photo_ids TEXT,               -- JSON array of photo UUIDs
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- Migrate existing data
INSERT INTO tracking_entries_new (id, plant_id, timestamp, care_task_ids, measurements, notes, photo_ids, created_at, updated_at)
SELECT
    id,
    plant_id,
    timestamp,
    CASE
        WHEN care_task_id IS NOT NULL THEN json_array(care_task_id)
        ELSE NULL
    END,
    CASE
        WHEN metric_id IS NOT NULL AND value IS NOT NULL THEN json_array(json_object('metricId', metric_id, 'value', json(value)))
        ELSE NULL
    END,
    notes,
    photo_ids,
    created_at,
    updated_at
FROM tracking_entries;

DROP TABLE tracking_entries;
ALTER TABLE tracking_entries_new RENAME TO tracking_entries;

-- Recreate indexes
CREATE INDEX idx_tracking_entries_plant_id ON tracking_entries(plant_id);
CREATE INDEX idx_tracking_entries_timestamp ON tracking_entries(timestamp);
