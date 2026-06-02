-- AI Plant Coaching System

CREATE TABLE coach_conversations (
    id TEXT PRIMARY KEY,
    plant_id TEXT NOT NULL REFERENCES plants(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE coach_messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL REFERENCES coach_conversations(id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL,
    image_url TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE coach_suggestions (
    id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL REFERENCES coach_messages(id) ON DELETE CASCADE,
    plant_id TEXT NOT NULL REFERENCES plants(id) ON DELETE CASCADE,
    suggestion_type TEXT NOT NULL CHECK(suggestion_type IN ('schedule_change', 'new_task', 'care_action', 'photo_request')),
    payload TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending', 'accepted', 'dismissed')),
    applied_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Indexes
CREATE INDEX idx_coach_conversations_plant_user ON coach_conversations(plant_id, user_id);
CREATE INDEX idx_coach_conversations_user ON coach_conversations(user_id);
CREATE INDEX idx_coach_messages_conversation_created ON coach_messages(conversation_id, created_at);
CREATE INDEX idx_coach_suggestions_plant_status ON coach_suggestions(plant_id, status);
CREATE INDEX idx_coach_suggestions_message ON coach_suggestions(message_id);
