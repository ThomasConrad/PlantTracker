-- Persisted calendar subscription token for strict feed authentication
ALTER TABLE users ADD COLUMN calendar_token TEXT;
ALTER TABLE users ADD COLUMN calendar_token_updated_at TEXT;

CREATE INDEX idx_users_calendar_token ON users(calendar_token);
