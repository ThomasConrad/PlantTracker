-- Plant memories: AI-accumulated and user-correctable facts about each plant
-- These provide persistent context for the coach across conversations
CREATE TABLE plant_memories (
    id TEXT PRIMARY KEY NOT NULL,
    plant_id TEXT NOT NULL REFERENCES plants(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- What kind of fact this is (enables structured display + filtering)
    fact_type TEXT NOT NULL CHECK (fact_type IN (
        'location',           -- "on kitchen windowsill, south-facing"
        'light',              -- "bright indirect, ~6h/day"
        'soil',               -- "well-draining cactus mix"
        'pot',                -- "6 inch terracotta with drainage"
        'watering_preference',-- "likes to dry out between waterings"
        'temperature',        -- "kept at 20-22°C, away from drafts"
        'humidity',           -- "near humidifier, ~60% RH"
        'growth_habit',       -- "tends to lean toward light"
        'symptom_pattern',    -- "leaves droop when thirsty"
        'pest_history',       -- "had spider mites in March 2025"
        'fertilizer_preference', -- "responds well to diluted seaweed"
        'propagation',        -- "propagated from mother plant in 2024"
        'acquisition',        -- "bought from local nursery, May 2024"
        'species_note',       -- corrections or details about species/cultivar
        'general'             -- catch-all for other observations
    )),

    -- The actual fact content
    content TEXT NOT NULL,

    -- How confident we are (0.0–1.0), AI-extracted facts start lower
    confidence REAL NOT NULL DEFAULT 0.7,

    -- Who created this fact
    source TEXT NOT NULL CHECK (source IN ('coach', 'user')) DEFAULT 'coach',

    -- Which coach message ID extracted this (NULL for user-added)
    source_message_id TEXT,

    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_plant_memories_plant_user ON plant_memories(plant_id, user_id);
CREATE INDEX idx_plant_memories_fact_type ON plant_memories(plant_id, fact_type);

-- Plant health scores: daily snapshots for gamification groundwork
-- Score derived from: care adherence, coach sentiment, overdue status
CREATE TABLE plant_health_scores (
    id TEXT PRIMARY KEY NOT NULL,
    plant_id TEXT NOT NULL REFERENCES plants(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- 0.0 to 1.0, displayed as 0-5 hearts (rounded to nearest half)
    score REAL NOT NULL,

    -- Components that make up the score (stored for debugging/transparency)
    care_adherence REAL,   -- 0-1: % of scheduled tasks completed on time recently
    overdue_penalty REAL,  -- 0-1: penalty for currently overdue tasks
    coach_sentiment REAL,  -- 0-1: last coach assessment of plant health

    -- When this score was computed
    scored_at TEXT NOT NULL,

    created_at TEXT NOT NULL
);

CREATE INDEX idx_health_scores_plant_date ON plant_health_scores(plant_id, scored_at);
CREATE UNIQUE INDEX idx_health_scores_plant_day ON plant_health_scores(plant_id, scored_at);
