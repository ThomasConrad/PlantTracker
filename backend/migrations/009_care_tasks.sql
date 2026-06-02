-- Full unification: replace hardcoded watering/fertilizing with flexible care_tasks system.
-- No users exist, so we can safely restructure all tables.

-- 1. Create care_tasks table
CREATE TABLE care_tasks (
    id TEXT PRIMARY KEY,
    plant_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,              -- "Water", "Fertilize", "Prune", "Repot", "Rotate", etc.
    icon TEXT,                       -- emoji or icon identifier (e.g. "💧", "🌱", "✂️")
    color TEXT,                      -- hex color for UI display (e.g. "#3B82F6")
    interval_days INTEGER,           -- NULL = one-off or manual-only (no recurring schedule)
    amount REAL,                     -- optional dosage amount
    unit TEXT,                       -- optional unit for amount (e.g. "ml", "g", "L")
    notes TEXT,                      -- freeform care instructions
    last_performed TEXT,             -- ISO 8601 timestamp of last completion
    sort_order INTEGER NOT NULL DEFAULT 0,
    archived_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (plant_id) REFERENCES plants(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_care_tasks_plant_id ON care_tasks(plant_id);
CREATE INDEX idx_care_tasks_user_id ON care_tasks(user_id);

-- 2. Rebuild plants table without watering/fertilizing columns
CREATE TABLE plants_new (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    genus TEXT NOT NULL,
    preview_id TEXT,
    archived_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

INSERT INTO plants_new (id, user_id, name, genus, preview_id, archived_at, created_at, updated_at)
SELECT id, user_id, name, genus, preview_id, archived_at, created_at, updated_at FROM plants;

DROP TABLE plants;
ALTER TABLE plants_new RENAME TO plants;
CREATE INDEX idx_plants_user_id ON plants(user_id);

-- 3. Rebuild tracking_entries with care_task_id, unified entry_type
--    entry_type: 'care' (references care_task_id), 'measurement', 'note', 'photo'
CREATE TABLE tracking_entries_new (
    id TEXT PRIMARY KEY,
    plant_id TEXT NOT NULL,
    care_task_id TEXT,               -- required when entry_type = 'care'
    entry_type TEXT NOT NULL CHECK (entry_type IN ('care', 'measurement', 'note', 'photo')),
    timestamp TEXT NOT NULL,
    value TEXT,                      -- JSON payload (amount, custom metric value, etc.)
    notes TEXT,
    metric_id TEXT,                  -- for 'measurement' entries
    photo_ids TEXT,                  -- JSON array of photo UUIDs
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (plant_id) REFERENCES plants(id) ON DELETE CASCADE,
    FOREIGN KEY (care_task_id) REFERENCES care_tasks(id) ON DELETE SET NULL,
    FOREIGN KEY (metric_id) REFERENCES custom_metrics(id) ON DELETE SET NULL
);

DROP TABLE tracking_entries;
ALTER TABLE tracking_entries_new RENAME TO tracking_entries;
CREATE INDEX idx_tracking_entries_plant_id ON tracking_entries(plant_id);
CREATE INDEX idx_tracking_entries_care_task_id ON tracking_entries(care_task_id);
CREATE INDEX idx_tracking_entries_timestamp ON tracking_entries(timestamp);

-- 4. Rebuild reminder_deliveries to reference care_task_id instead of reminder_type
CREATE TABLE reminder_deliveries_new (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    plant_id TEXT NOT NULL,
    care_task_id TEXT NOT NULL,
    due_date TEXT NOT NULL,
    due_at TEXT NOT NULL,
    sent_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (plant_id) REFERENCES plants(id) ON DELETE CASCADE,
    FOREIGN KEY (care_task_id) REFERENCES care_tasks(id) ON DELETE CASCADE,
    UNIQUE(user_id, plant_id, care_task_id, due_date)
);

DROP TABLE reminder_deliveries;
ALTER TABLE reminder_deliveries_new RENAME TO reminder_deliveries;
CREATE INDEX idx_reminder_deliveries_user_id ON reminder_deliveries(user_id);
