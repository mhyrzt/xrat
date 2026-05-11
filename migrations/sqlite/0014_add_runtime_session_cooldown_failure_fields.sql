ALTER TABLE runtime_sessions
    ADD COLUMN cooldown_until TEXT;

ALTER TABLE runtime_sessions
    ADD COLUMN last_failed_at TEXT;

ALTER TABLE runtime_sessions
    ADD COLUMN last_failed_reason_code TEXT;
