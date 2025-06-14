-- Migration 009: Enhance tracking entries for watering amounts and fertilizer notes
-- The existing table structure already supports this through the flexible 'value' JSON field
-- and 'notes' text field. This migration adds comments and indexes for better performance.

-- Add comments to document the enhanced usage patterns:
-- For watering entries:
--   - value: can store {"amount": 250, "unit": "ml"} or {"amount": 1, "unit": "cup"}
--   - notes: can store additional context like "morning watering" or "soil was very dry"
--
-- For fertilizing entries:
--   - value: can store {"type": "liquid", "dilution": "1:10"} or {"type": "granular", "amount": "1 tbsp"}
--   - notes: can store fertilizer brand, specific instructions, or observations
--
-- For other entries:
--   - value: flexible JSON for measurements, boolean values, or structured data
--   - notes: free-form text for observations, reminders, or additional context

-- The existing structure is already perfect for these use cases.
-- No schema changes needed - just enhanced usage patterns.

-- Add an index to optimize filtering by entry_type for better performance when
-- querying specific types of entries (watering, fertilizing, etc.)
CREATE INDEX IF NOT EXISTS idx_tracking_entries_entry_type ON tracking_entries(entry_type);

-- Add an index on timestamp + entry_type for calendar views
CREATE INDEX IF NOT EXISTS idx_tracking_entries_timestamp_type ON tracking_entries(timestamp, entry_type); 