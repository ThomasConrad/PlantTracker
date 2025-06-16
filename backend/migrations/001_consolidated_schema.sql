-- Consolidated Database Schema for PlantTracker
-- This migration consolidates all previous migrations into a single file
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

-- Sessions table for persistent session storage
-- Compatible with tower-sessions-sqlx-store
CREATE TABLE tower_sessions (
    id TEXT PRIMARY KEY,
    data BLOB NOT NULL,
    expiry_date INTEGER NOT NULL
);

-- Google OAuth tokens table for calendar integration
CREATE TABLE google_oauth_tokens (
    user_id TEXT NOT NULL PRIMARY KEY,
    access_token TEXT NOT NULL,
    refresh_token TEXT,
    expires_at TIMESTAMP,
    scope TEXT NOT NULL,
    token_type TEXT NOT NULL DEFAULT 'Bearer',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

-- Plants table with schedule customization
CREATE TABLE plants (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    genus TEXT NOT NULL,
    watering_interval_days INTEGER, -- Nullable - can be disabled
    fertilizing_interval_days INTEGER, -- Nullable - can be disabled
    watering_amount REAL, -- Amount for watering
    watering_unit TEXT, -- Unit for watering amount
    watering_notes TEXT, -- Instructions for watering
    fertilizing_amount REAL, -- Amount for fertilizing
    fertilizing_unit TEXT, -- Unit for fertilizing amount  
    fertilizing_notes TEXT, -- Instructions for fertilizing
    last_watered TEXT,
    last_fertilized TEXT,
    preview_id TEXT, -- Reference to photo for plant preview
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CHECK (watering_amount IS NULL OR watering_amount > 0),
    CHECK (fertilizing_amount IS NULL OR fertilizing_amount > 0),
    CHECK (watering_interval_days IS NULL OR watering_interval_days > 0),
    CHECK (fertilizing_interval_days IS NULL OR fertilizing_interval_days > 0)
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

-- Photos table with AVIF support
CREATE TABLE photos (
    id TEXT PRIMARY KEY,
    plant_id TEXT NOT NULL,
    filename TEXT NOT NULL,
    original_filename TEXT NOT NULL,
    size INTEGER NOT NULL,
    content_type TEXT NOT NULL,
    data BLOB NOT NULL,
    width INTEGER, -- Image width
    height INTEGER, -- Image height
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (plant_id) REFERENCES plants(id) ON DELETE CASCADE
);

-- Tracking entries table with enhanced support
-- Supports watering amounts, fertilizer notes, and photo entries
CREATE TABLE tracking_entries (
    id TEXT PRIMARY KEY,
    plant_id TEXT NOT NULL,
    metric_id TEXT,
    entry_type TEXT NOT NULL CHECK (entry_type IN ('watering', 'fertilizing', 'measurement', 'note', 'photo')),
    timestamp TEXT NOT NULL,
    value TEXT, -- JSON stored as TEXT for SQLite compatibility
                -- For watering: {"amount": 250, "unit": "ml"}
                -- For fertilizing: {"type": "liquid", "dilution": "1:10"}
    notes TEXT, -- Free-form text for additional context
    photo_ids TEXT, -- JSON array of photo UUIDs
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (plant_id) REFERENCES plants(id) ON DELETE CASCADE,
    FOREIGN KEY (metric_id) REFERENCES custom_metrics(id) ON DELETE SET NULL
);

-- Indexes for performance optimization
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_tower_sessions_expiry ON tower_sessions(expiry_date);
CREATE INDEX idx_google_oauth_tokens_user_id ON google_oauth_tokens(user_id);
CREATE INDEX idx_google_oauth_tokens_expires_at ON google_oauth_tokens(expires_at);
CREATE INDEX idx_plants_user_id ON plants(user_id);
CREATE INDEX idx_plants_preview_id ON plants(preview_id);
CREATE INDEX idx_custom_metrics_plant_id ON custom_metrics(plant_id);
CREATE INDEX idx_photos_plant_id ON photos(plant_id);
CREATE INDEX idx_tracking_entries_plant_id ON tracking_entries(plant_id);
CREATE INDEX idx_tracking_entries_timestamp ON tracking_entries(timestamp);
CREATE INDEX idx_tracking_entries_entry_type ON tracking_entries(entry_type);
CREATE INDEX idx_tracking_entries_timestamp_type ON tracking_entries(timestamp, entry_type); 