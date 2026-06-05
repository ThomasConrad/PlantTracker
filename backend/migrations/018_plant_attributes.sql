-- Plant attributes: extensible key-value metadata about a plant's needs/characteristics.
-- Examples: light requirement, humidity, temperature range, soil type, toxicity, growth rate, etc.
-- The AI can create arbitrary attributes during identification or via coach suggestions.

CREATE TABLE IF NOT EXISTS plant_attributes (
    id TEXT PRIMARY KEY NOT NULL,
    plant_id TEXT NOT NULL REFERENCES plants(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- Normalized key for deduplication (e.g., "light", "humidity", "temperature")
    key TEXT NOT NULL,
    -- Display label (e.g., "Light Requirement", "Humidity", "Temperature Range")
    label TEXT NOT NULL,
    -- The actual value (e.g., "Bright indirect light", "60-80%", "18-25°C")
    value TEXT NOT NULL,
    -- Optional emoji icon for display
    icon TEXT,
    -- Category for grouping (e.g., "environment", "soil", "growth", "toxicity")
    category TEXT,
    -- Where this attribute came from
    source TEXT NOT NULL DEFAULT 'user' CHECK (source IN ('identification', 'coach', 'user')),
    -- Display ordering
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    -- Each plant can only have one attribute per key
    UNIQUE(plant_id, key)
);

CREATE INDEX IF NOT EXISTS idx_plant_attributes_plant_id ON plant_attributes(plant_id);
CREATE INDEX IF NOT EXISTS idx_plant_attributes_user_id ON plant_attributes(user_id);
