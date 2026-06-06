-- User-facing stable short refs for configs and subscriptions.
-- The column is added nullable, backfilled with a random 12-char hex value, and
-- a UNIQUE index is created. The application layer always sets a ref on insert.
-- md5(random()::text) avoids depending on the pgcrypto extension.
ALTER TABLE configs ADD COLUMN ref TEXT;
ALTER TABLE subscriptions ADD COLUMN ref TEXT;

UPDATE configs SET ref = substr(md5(random()::TEXT || id::TEXT), 1, 12)
WHERE ref IS NULL;
UPDATE subscriptions SET ref = substr(md5(random()::TEXT || id::TEXT), 1, 12)
WHERE ref IS NULL;

CREATE UNIQUE INDEX idx_configs_ref ON configs (ref);
CREATE UNIQUE INDEX idx_subscriptions_ref ON subscriptions (ref);
