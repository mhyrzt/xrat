CREATE TABLE xray_profiles (
    config_id INTEGER PRIMARY KEY REFERENCES configs (id) ON DELETE CASCADE,
    digest TEXT NOT NULL UNIQUE CHECK (length(digest) = 64),
    byte_length BIGINT NOT NULL CHECK (byte_length > 0)
);
