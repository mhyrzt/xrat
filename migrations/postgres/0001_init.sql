CREATE TABLE subscriptions (
    id BIGSERIAL PRIMARY KEY,
    source_url TEXT,
    source_kind TEXT NOT NULL CHECK (source_kind IN ('url', 'file', 'raw_text')),
    name TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP::TEXT,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP::TEXT
);

CREATE TABLE configs (
    id BIGSERIAL PRIMARY KEY,
    subscription_id BIGINT,
    dedup_key TEXT NOT NULL UNIQUE,
    protocol TEXT NOT NULL,
    address TEXT NOT NULL,
    port BIGINT NOT NULL,
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
    is_active BIGINT NOT NULL DEFAULT 0 CHECK (is_active IN (0, 1)),
    is_enabled BIGINT NOT NULL DEFAULT 1 CHECK (is_enabled IN (0, 1)),
    is_selected BIGINT NOT NULL DEFAULT 0 CHECK (is_selected IN (0, 1)),
    imported_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP::TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP::TEXT,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP::TEXT,
    FOREIGN KEY (subscription_id) REFERENCES subscriptions(id)
);

CREATE TABLE connection_tests (
    id BIGSERIAL PRIMARY KEY,
    config_id BIGINT NOT NULL,
    icmp_ok BIGINT CHECK (icmp_ok IN (0, 1)),
    icmp_ms BIGINT,
    tcp_ok BIGINT CHECK (tcp_ok IN (0, 1)),
    tcp_ms BIGINT,
    real_delay_ok BIGINT CHECK (real_delay_ok IN (0, 1)),
    real_delay_ms BIGINT,
    failure_kind TEXT CHECK (
        failure_kind IN ('dns', 'timeout', 'refused', 'unreachable', 'permission_denied',
                         'tls', 'auth', 'process', 'proxy', 'unknown')
    ),
    failure_reason TEXT,
    tested_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP::TEXT,
    FOREIGN KEY (config_id) REFERENCES configs(id)
);

CREATE TABLE runtime_sessions (
    id BIGSERIAL PRIMARY KEY,
    config_id BIGINT,
    status TEXT NOT NULL CHECK (status IN ('starting', 'running', 'stopping', 'stopped', 'failed')),
    mixed_port BIGINT,
    process_id BIGINT,
    started_at TEXT,
    stopped_at TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP::TEXT,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP::TEXT,
    FOREIGN KEY (config_id) REFERENCES configs(id)
);

CREATE INDEX idx_configs_is_active ON configs(is_active);
CREATE INDEX idx_configs_is_enabled ON configs(is_enabled);
CREATE INDEX idx_configs_subscription_id ON configs(subscription_id);
CREATE INDEX idx_connection_tests_config_id ON connection_tests(config_id);
CREATE INDEX idx_connection_tests_tested_at ON connection_tests(tested_at);
CREATE INDEX idx_runtime_sessions_config_id ON runtime_sessions(config_id);
CREATE INDEX idx_runtime_sessions_status ON runtime_sessions(status);
