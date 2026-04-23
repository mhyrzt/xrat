CREATE TABLE subscriptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_url TEXT,
    source_kind TEXT NOT NULL CHECK (source_kind IN ('url', 'file', 'raw_text')),
    name TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    subscription_id INTEGER,
    dedup_key TEXT NOT NULL UNIQUE,
    protocol TEXT NOT NULL,
    address TEXT NOT NULL,
    port INTEGER NOT NULL,
    username TEXT,
    uuid TEXT,
    password TEXT,
    method TEXT,
    network TEXT NOT NULL,
    tls TEXT,
    sni TEXT,
    host TEXT,
    path TEXT,
    name TEXT,
    raw_config TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 0 CHECK (is_active IN (0, 1)),
    is_enabled INTEGER NOT NULL DEFAULT 1 CHECK (is_enabled IN (0, 1)),
    is_selected INTEGER NOT NULL DEFAULT 0 CHECK (is_selected IN (0, 1)),
    imported_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (subscription_id) REFERENCES subscriptions(id)
);

CREATE TABLE connection_tests (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    config_id INTEGER NOT NULL,
    tcp_ok INTEGER CHECK (tcp_ok IN (0, 1)),
    tcp_ms INTEGER,
    real_delay_ok INTEGER CHECK (real_delay_ok IN (0, 1)),
    real_delay_ms INTEGER,
    failure_kind TEXT CHECK (
        failure_kind IN ('dns', 'timeout', 'refused', 'tls', 'auth', 'process', 'unknown')
    ),
    failure_reason TEXT,
    tested_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (config_id) REFERENCES configs(id)
);

CREATE TABLE runtime_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    config_id INTEGER,
    status TEXT NOT NULL CHECK (status IN ('starting', 'running', 'stopping', 'stopped', 'failed')),
    mixed_port INTEGER,
    process_id INTEGER,
    started_at TEXT,
    stopped_at TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (config_id) REFERENCES configs(id)
);

CREATE INDEX idx_configs_is_active ON configs(is_active);
CREATE INDEX idx_configs_is_enabled ON configs(is_enabled);
CREATE INDEX idx_configs_subscription_id ON configs(subscription_id);
CREATE INDEX idx_connection_tests_config_id ON connection_tests(config_id);
CREATE INDEX idx_connection_tests_tested_at ON connection_tests(tested_at);
CREATE INDEX idx_runtime_sessions_config_id ON runtime_sessions(config_id);
CREATE INDEX idx_runtime_sessions_status ON runtime_sessions(status);
