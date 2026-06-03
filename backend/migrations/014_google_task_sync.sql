-- Track synced Google Tasks to prevent duplicates and enable two-way sync.
-- Each row maps a Planty care_task occurrence (by due date) to a Google Task ID.

CREATE TABLE IF NOT EXISTS google_task_sync (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    user_id TEXT NOT NULL,
    plant_id TEXT NOT NULL,
    care_task_id TEXT NOT NULL,
    google_task_id TEXT NOT NULL,
    google_task_list_id TEXT NOT NULL,
    due_date TEXT NOT NULL,          -- ISO date (YYYY-MM-DD) for which this task was created
    status TEXT NOT NULL DEFAULT 'needsAction', -- 'needsAction' or 'completed'
    synced_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT,               -- When it was marked complete (from Google)
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (plant_id) REFERENCES plants(id) ON DELETE CASCADE,
    FOREIGN KEY (care_task_id) REFERENCES care_tasks(id) ON DELETE CASCADE
);

-- Index for checking if a task+date combo already exists
CREATE UNIQUE INDEX IF NOT EXISTS idx_google_task_sync_unique
    ON google_task_sync(user_id, care_task_id, due_date);

-- Index for looking up by google task ID (for two-way sync)
CREATE INDEX IF NOT EXISTS idx_google_task_sync_google_id
    ON google_task_sync(google_task_id);

-- Index for finding incomplete tasks to poll
CREATE INDEX IF NOT EXISTS idx_google_task_sync_status
    ON google_task_sync(user_id, status) WHERE status = 'needsAction';
