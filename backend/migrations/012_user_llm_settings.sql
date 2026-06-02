-- Add per-user LLM/AI coach settings
-- Users can configure their own provider (OpenAI-compatible base URL + API key + model)
ALTER TABLE users ADD COLUMN llm_base_url TEXT;
ALTER TABLE users ADD COLUMN llm_api_key TEXT;
ALTER TABLE users ADD COLUMN llm_model TEXT;
