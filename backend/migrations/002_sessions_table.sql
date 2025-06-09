-- Sessions table for persistent session storage
-- Compatible with tower-sessions-sqlx-store

CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    data BLOB NOT NULL,
    expiry_date INTEGER NOT NULL
);

-- Index for efficient cleanup of expired sessions
CREATE INDEX idx_sessions_expiry ON sessions(expiry_date);