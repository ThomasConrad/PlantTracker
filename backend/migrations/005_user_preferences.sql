-- Add user-level locale and unit preferences.
ALTER TABLE users ADD COLUMN first_day_of_week TEXT NOT NULL DEFAULT 'sunday' CHECK (first_day_of_week IN ('sunday', 'monday'));
ALTER TABLE users ADD COLUMN preferred_units TEXT NOT NULL DEFAULT 'metric' CHECK (preferred_units IN ('metric', 'imperial'));
