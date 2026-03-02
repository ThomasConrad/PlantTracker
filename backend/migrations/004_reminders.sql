-- User reminder preferences and delivery tracking.
CREATE TABLE IF NOT EXISTS user_reminder_preferences (
    user_id TEXT PRIMARY KEY NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    reminder_time TEXT NOT NULL DEFAULT '09:00',
    timezone TEXT NOT NULL DEFAULT 'UTC',
    browser_notifications_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS reminder_deliveries (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    plant_id TEXT NOT NULL,
    reminder_type TEXT NOT NULL CHECK (reminder_type IN ('watering', 'fertilizing')),
    due_date TEXT NOT NULL,
    due_at TEXT NOT NULL,
    sent_at TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (plant_id) REFERENCES plants(id) ON DELETE CASCADE,
    UNIQUE(user_id, plant_id, reminder_type, due_date)
);

CREATE INDEX IF NOT EXISTS idx_reminder_deliveries_user_due
    ON reminder_deliveries(user_id, due_date);

CREATE INDEX IF NOT EXISTS idx_reminder_deliveries_unsent
    ON reminder_deliveries(user_id, sent_at);
