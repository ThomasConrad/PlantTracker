-- Add user roles and invite creation limits

-- Add role and invite management columns to users table
ALTER TABLE users ADD COLUMN role TEXT NOT NULL DEFAULT 'user' CHECK (role IN ('admin', 'moderator', 'user'));
ALTER TABLE users ADD COLUMN can_create_invites BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE users ADD COLUMN max_invites INTEGER DEFAULT NULL; -- NULL means unlimited (for admins)
ALTER TABLE users ADD COLUMN invites_created INTEGER NOT NULL DEFAULT 0;
ALTER TABLE users ADD COLUMN invites_remaining INTEGER DEFAULT NULL; -- Computed field for convenience

-- Create a function to update invites_remaining when invites_created or max_invites changes
-- Note: SQLite doesn't have stored procedures, so we'll manage this in application code

-- Update existing invite_codes to track usage better
ALTER TABLE invite_codes ADD COLUMN users_registered TEXT; -- JSON array of user IDs who registered with this code

-- Create admin_settings table for global configuration
CREATE TABLE admin_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Insert default admin settings
INSERT INTO admin_settings (key, value, description) VALUES 
    ('max_total_users', '1000', 'Maximum total users allowed in the system'),
    ('default_user_invite_limit', '5', 'Default number of invites new users can create'),
    ('registration_enabled', 'true', 'Whether new user registration is enabled');

-- Create user_invite_usage table to track who used whose invites
CREATE TABLE user_invite_usage (
    id TEXT PRIMARY KEY,
    invite_code_id TEXT NOT NULL,
    creator_user_id TEXT, -- Who created the invite
    registered_user_id TEXT NOT NULL, -- Who used the invite to register
    registered_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (invite_code_id) REFERENCES invite_codes(id) ON DELETE CASCADE,
    FOREIGN KEY (creator_user_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (registered_user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Indexes for the new tables and columns
CREATE INDEX idx_users_role ON users(role);
CREATE INDEX idx_users_can_create_invites ON users(can_create_invites);
CREATE INDEX idx_users_invites_created ON users(invites_created);
CREATE INDEX idx_user_invite_usage_creator ON user_invite_usage(creator_user_id);
CREATE INDEX idx_user_invite_usage_registered_user ON user_invite_usage(registered_user_id);
CREATE INDEX idx_user_invite_usage_registered_at ON user_invite_usage(registered_at);
