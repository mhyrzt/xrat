ALTER TABLE configs ADD COLUMN is_deleted INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0, 1));
ALTER TABLE configs ADD COLUMN deleted_at TEXT;
