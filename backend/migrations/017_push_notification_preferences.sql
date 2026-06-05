-- Push notification preference columns
-- Each controls whether a category of push notification is sent to the user's device(s).
-- Uses CREATE TABLE + INSERT + DROP + RENAME pattern for idempotency in SQLite
-- (ALTER TABLE ADD COLUMN does not support IF NOT EXISTS).

CREATE TABLE IF NOT EXISTS user_reminder_preferences_new (
    user_id TEXT PRIMARY KEY NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    reminder_time TEXT NOT NULL DEFAULT '09:00',
    timezone TEXT NOT NULL DEFAULT 'UTC',
    browser_notifications_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    push_health_alerts BOOLEAN NOT NULL DEFAULT 1,
    push_daily_summary BOOLEAN NOT NULL DEFAULT 1,
    push_coach_suggestions BOOLEAN NOT NULL DEFAULT 1,
    push_reminders BOOLEAN NOT NULL DEFAULT 1,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Copy existing rows (only base columns; new push_* columns get defaults)
INSERT OR IGNORE INTO user_reminder_preferences_new (
    user_id, enabled, reminder_time, timezone, browser_notifications_enabled,
    created_at, updated_at
)
SELECT
    user_id, enabled, reminder_time, timezone, browser_notifications_enabled,
    created_at, updated_at
FROM user_reminder_preferences;

DROP TABLE user_reminder_preferences;

ALTER TABLE user_reminder_preferences_new RENAME TO user_reminder_preferences;
