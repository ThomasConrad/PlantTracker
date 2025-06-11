-- Migration: Add Google OAuth tokens table
-- Description: Store Google OAuth2 tokens for calendar integration

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

-- Index for efficient lookups by user_id
CREATE INDEX idx_google_oauth_tokens_user_id ON google_oauth_tokens(user_id);

-- Index for finding tokens that need refresh (expires_at)
CREATE INDEX idx_google_oauth_tokens_expires_at ON google_oauth_tokens(expires_at);