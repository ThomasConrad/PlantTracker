-- Push notification preference columns
-- Each controls whether a category of push notification is sent to the user's device(s).

ALTER TABLE user_reminder_preferences ADD COLUMN push_health_alerts BOOLEAN NOT NULL DEFAULT 1;
ALTER TABLE user_reminder_preferences ADD COLUMN push_daily_summary BOOLEAN NOT NULL DEFAULT 1;
ALTER TABLE user_reminder_preferences ADD COLUMN push_coach_suggestions BOOLEAN NOT NULL DEFAULT 1;
ALTER TABLE user_reminder_preferences ADD COLUMN push_reminders BOOLEAN NOT NULL DEFAULT 1;
