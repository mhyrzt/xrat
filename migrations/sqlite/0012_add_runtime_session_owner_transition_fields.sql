ALTER TABLE runtime_sessions
ADD COLUMN owner_kind TEXT;

ALTER TABLE runtime_sessions
ADD COLUMN owner_instance_id TEXT;

ALTER TABLE runtime_sessions
ADD COLUMN last_transition_reason_code TEXT;

ALTER TABLE runtime_sessions
ADD COLUMN last_transition_reason_detail TEXT;
