CREATE TABLE connection_test_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    kind TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE connection_tests
ADD COLUMN run_id INTEGER REFERENCES connection_test_runs (id);

CREATE INDEX idx_connection_test_runs_created_at ON connection_test_runs (created_at);
CREATE INDEX idx_connection_tests_run_id ON connection_tests (run_id);
