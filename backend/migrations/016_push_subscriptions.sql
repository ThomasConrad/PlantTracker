-- Web push notification subscriptions
-- Each row represents a browser/device that has opted in to push notifications
CREATE TABLE push_subscriptions (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- The push service endpoint URL (unique per subscription)
    endpoint TEXT NOT NULL UNIQUE,

    -- Encryption keys from PushSubscription.getKey()
    p256dh TEXT NOT NULL,
    auth TEXT NOT NULL,

    -- User agent / device label for display in settings
    user_agent TEXT,

    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_push_subscriptions_user ON push_subscriptions(user_id);
