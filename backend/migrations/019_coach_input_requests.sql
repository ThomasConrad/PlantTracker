-- Add input_requests column to coach messages for interactive widget state.
-- Stores JSON array of input request templates attached to assistant messages.
-- NULL means no input requests (most messages).

ALTER TABLE coach_messages ADD COLUMN input_requests TEXT;
