-- Add reasoning column to health scores for AI-generated explanations
ALTER TABLE plant_health_scores ADD COLUMN reasoning TEXT;
