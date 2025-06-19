-- Add invite system and waitlist tables

-- Invite codes table for controlling registration
CREATE TABLE invite_codes (
    id TEXT PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    created_by TEXT, -- Admin user who created the invite (nullable for system-generated)
    used_by TEXT, -- User who used the invite (nullable until used)
    max_uses INTEGER NOT NULL DEFAULT 1, -- How many times this code can be used
    current_uses INTEGER NOT NULL DEFAULT 0, -- How many times it has been used
    expires_at TEXT, -- When the invite expires (nullable for no expiry)
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (used_by) REFERENCES users(id) ON DELETE SET NULL,
    CHECK (current_uses <= max_uses),
    CHECK (max_uses > 0)
);

-- Waitlist table for users waiting for access
CREATE TABLE waitlist (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    name TEXT,
    message TEXT, -- Optional message from the user
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'invited', 'registered')),
    invited_at TEXT, -- When they were sent an invite
    invite_code TEXT, -- The invite code that was sent to them
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (invite_code) REFERENCES invite_codes(code) ON DELETE SET NULL
);

-- Indexes for performance
CREATE INDEX idx_invite_codes_code ON invite_codes(code);
CREATE INDEX idx_invite_codes_created_by ON invite_codes(created_by);
CREATE INDEX idx_invite_codes_used_by ON invite_codes(used_by);
CREATE INDEX idx_invite_codes_is_active ON invite_codes(is_active);
CREATE INDEX idx_invite_codes_expires_at ON invite_codes(expires_at);
CREATE INDEX idx_waitlist_email ON waitlist(email);
CREATE INDEX idx_waitlist_status ON waitlist(status);
CREATE INDEX idx_waitlist_created_at ON waitlist(created_at);
