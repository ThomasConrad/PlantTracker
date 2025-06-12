-- Initial database schema for Planty
-- Compatible with both SQLite and PostgreSQL

-- Users table
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    salt TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Plants table
CREATE TABLE plants (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    genus TEXT NOT NULL,
    watering_interval_days INTEGER NOT NULL,
    fertilizing_interval_days INTEGER NOT NULL,
    last_watered TEXT,
    last_fertilized TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Custom metrics table
CREATE TABLE custom_metrics (
    id TEXT PRIMARY KEY,
    plant_id TEXT NOT NULL,
    name TEXT NOT NULL,
    unit TEXT NOT NULL,
    data_type TEXT NOT NULL CHECK (data_type IN ('number', 'text', 'boolean')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (plant_id) REFERENCES plants(id) ON DELETE CASCADE
);

-- Photos table
CREATE TABLE photos (
    id TEXT PRIMARY KEY,
    plant_id TEXT NOT NULL,
    filename TEXT NOT NULL,
    original_filename TEXT NOT NULL,
    size INTEGER NOT NULL,
    content_type TEXT NOT NULL,
    data BLOB NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (plant_id) REFERENCES plants(id) ON DELETE CASCADE
);

-- Tracking entries table
CREATE TABLE tracking_entries (
    id TEXT PRIMARY KEY,
    plant_id TEXT NOT NULL,
    metric_id TEXT,
    entry_type TEXT NOT NULL CHECK (entry_type IN ('watering', 'fertilizing', 'measurement')),
    timestamp TEXT NOT NULL,
    value TEXT, -- JSON stored as TEXT for SQLite compatibility
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (plant_id) REFERENCES plants(id) ON DELETE CASCADE,
    FOREIGN KEY (metric_id) REFERENCES custom_metrics(id) ON DELETE SET NULL
);

-- Indexes for better performance
CREATE INDEX idx_plants_user_id ON plants(user_id);
CREATE INDEX idx_custom_metrics_plant_id ON custom_metrics(plant_id);
CREATE INDEX idx_photos_plant_id ON photos(plant_id);
CREATE INDEX idx_tracking_entries_plant_id ON tracking_entries(plant_id);
CREATE INDEX idx_tracking_entries_timestamp ON tracking_entries(timestamp);
CREATE INDEX idx_users_email ON users(email);