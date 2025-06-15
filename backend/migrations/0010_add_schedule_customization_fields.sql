-- Migration 0010: Add schedule customization fields to plants table
-- This migration adds optional amount and instruction note fields for watering and fertilizing schedules,
-- and allows schedules to be disabled by setting interval to NULL

-- SQLite doesn't support ALTER COLUMN to change nullability or ADD CONSTRAINT with CHECK on existing table
-- We need to recreate the table with the new structure

-- Create new plants table with updated schema
CREATE TABLE plants_new (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    genus TEXT NOT NULL,
    watering_interval_days INTEGER, -- Now nullable
    fertilizing_interval_days INTEGER, -- Now nullable
    watering_amount REAL, -- New field for watering amount
    watering_unit TEXT, -- New field for watering unit
    watering_notes TEXT, -- New field for watering instructions
    fertilizing_amount REAL, -- New field for fertilizing amount
    fertilizing_unit TEXT, -- New field for fertilizing unit  
    fertilizing_notes TEXT, -- New field for fertilizing instructions
    last_watered TEXT,
    last_fertilized TEXT,
    thumbnail_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CHECK (watering_amount IS NULL OR watering_amount > 0),
    CHECK (fertilizing_amount IS NULL OR fertilizing_amount > 0),
    CHECK (watering_interval_days IS NULL OR watering_interval_days > 0),
    CHECK (fertilizing_interval_days IS NULL OR fertilizing_interval_days > 0)
);

-- Copy data from old table to new table
INSERT INTO plants_new (
    id, user_id, name, genus, watering_interval_days, fertilizing_interval_days,
    last_watered, last_fertilized, thumbnail_id, created_at, updated_at
)
SELECT 
    id, user_id, name, genus, watering_interval_days, fertilizing_interval_days,
    last_watered, last_fertilized, thumbnail_id, created_at, updated_at
FROM plants;

-- Drop old table and rename new table
DROP TABLE plants;
ALTER TABLE plants_new RENAME TO plants;

-- Recreate indexes
CREATE INDEX idx_plants_user_id ON plants(user_id);
