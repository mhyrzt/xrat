CREATE TABLE cf_scan_results (
    id BIGSERIAL PRIMARY KEY,
    ip TEXT NOT NULL UNIQUE,
    latency_ms BIGINT,
    download_mbps DOUBLE PRECISION,
    upload_mbps DOUBLE PRECISION,
    error TEXT,
    last_scanned_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_cf_scan_results_last_scanned_at ON cf_scan_results (last_scanned_at DESC);
CREATE INDEX idx_cf_scan_results_error ON cf_scan_results (error);
CREATE INDEX idx_cf_scan_results_latency ON cf_scan_results (latency_ms);
