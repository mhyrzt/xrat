CREATE TABLE events (
    id BIGSERIAL PRIMARY KEY,
    level TEXT NOT NULL,
    source TEXT NOT NULL,
    kind TEXT NOT NULL,
    config_id BIGINT,
    session_id BIGINT,
    message TEXT NOT NULL,
    detail TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP::TEXT
);

CREATE INDEX idx_events_created_at ON events (created_at DESC);
CREATE INDEX idx_events_source ON events (source);
CREATE INDEX idx_events_level ON events (level);
