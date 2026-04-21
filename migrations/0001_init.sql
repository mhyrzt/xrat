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
    is_active INTEGER NOT NULL DEFAULT 0 CHECK (is_active IN (0, 1)),
    is_enabled INTEGER NOT NULL DEFAULT 1 CHECK (is_enabled IN (0, 1)),
    is_deleted INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0, 1)),
    is_selected INTEGER NOT NULL DEFAULT 0 CHECK (is_selected IN (0, 1)),
    imported_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TEXT,
    FOREIGN KEY (subscription_id) REFERENCES subscriptions(id)
);

CREATE INDEX idx_configs_is_active ON configs(is_active);
CREATE INDEX idx_configs_is_enabled ON configs(is_enabled);
CREATE INDEX idx_configs_subscription_deleted ON configs(subscription_id, is_deleted);
