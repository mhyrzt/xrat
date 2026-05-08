ALTER TABLE connection_tests ADD COLUMN connect_ms BIGINT;
ALTER TABLE connection_tests ADD COLUMN ttfb_ms BIGINT;
ALTER TABLE connection_tests ADD COLUMN http_status BIGINT;
ALTER TABLE connection_tests ADD COLUMN endpoint_ip TEXT;
ALTER TABLE connection_tests ADD COLUMN endpoint_location TEXT;

CREATE INDEX idx_connection_tests_http_status ON connection_tests (http_status);
