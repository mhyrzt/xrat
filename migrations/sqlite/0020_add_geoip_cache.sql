CREATE TABLE geoip_cache (
    host TEXT PRIMARY KEY,
    ip TEXT,
    country TEXT,
    location TEXT,
    asn TEXT,
    resolved_at INTEGER NOT NULL
);
