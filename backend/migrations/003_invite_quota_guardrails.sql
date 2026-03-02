-- Remove legacy unlimited invite quotas so all user creation remains bounded.
UPDATE users
SET max_invites = 50,
    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
WHERE max_invites IS NULL;

-- Ensure existing invite codes always have a positive finite use limit.
UPDATE invite_codes
SET max_uses = 1,
    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
WHERE max_uses < 1;
