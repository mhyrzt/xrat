CREATE TABLE events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    level TEXT NOT NULL,
    source TEXT NOT NULL,
    kind TEXT NOT NULL,
    config_id INTEGER,
    session_id INTEGER,
    message TEXT NOT NULL,
    detail TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_events_created_at ON events (created_at DESC);
CREATE INDEX idx_events_source ON events (source);
CREATE INDEX idx_events_level ON events (level);
