-- User-facing stable short refs for configs and subscriptions.
-- SQLite cannot add a NOT NULL UNIQUE column via ALTER, so the column is added
-- nullable, backfilled with random 12-char hex, and a UNIQUE index is created.
-- The application layer always sets a ref on insert.
ALTER TABLE configs ADD COLUMN ref TEXT;
ALTER TABLE subscriptions ADD COLUMN ref TEXT;

UPDATE configs SET ref = lower(hex(randomblob(6))) WHERE ref IS NULL;
UPDATE subscriptions SET ref = lower(hex(randomblob(6))) WHERE ref IS NULL;

CREATE UNIQUE INDEX idx_configs_ref ON configs (ref);
CREATE UNIQUE INDEX idx_subscriptions_ref ON subscriptions (ref);
