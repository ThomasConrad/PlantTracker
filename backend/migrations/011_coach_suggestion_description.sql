-- Add description column to coach_suggestions
ALTER TABLE coach_suggestions ADD COLUMN description TEXT NOT NULL DEFAULT '';
