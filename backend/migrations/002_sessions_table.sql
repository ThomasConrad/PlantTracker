-- Sessions table for persistent session storage
-- Compatible with tower-sessions-sqlx-store

CREATE TABLE tower_sessions (
    id TEXT PRIMARY KEY,
    data BLOB NOT NULL,
    expiry_date INTEGER NOT NULL
);

-- Index for efficient cleanup of expired sessions
CREATE INDEX idx_tower_sessions_expiry ON tower_sessions(expiry_date);