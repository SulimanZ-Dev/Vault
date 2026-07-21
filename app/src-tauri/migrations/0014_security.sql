CREATE TABLE IF NOT EXISTS security_settings (
  vault_id INTEGER PRIMARY KEY REFERENCES vaults(id) ON DELETE CASCADE,
  mode TEXT NOT NULL DEFAULT 'comfortable',
  pin_salt TEXT,
  pin_hash TEXT,
  auto_lock_minutes INTEGER NOT NULL DEFAULT 15,
  mask_sensitive INTEGER NOT NULL DEFAULT 1 CHECK(mask_sensitive IN (0,1)),
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT OR IGNORE INTO security_settings(vault_id) VALUES(1);

ALTER TABLE documents ADD COLUMN is_locked INTEGER NOT NULL DEFAULT 0 CHECK(is_locked IN (0,1));
ALTER TABLE documents ADD COLUMN sensitivity TEXT NOT NULL DEFAULT 'normal';

CREATE TABLE IF NOT EXISTS security_events (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  event_type TEXT NOT NULL,
  target_document_id INTEGER REFERENCES documents(id) ON DELETE SET NULL,
  success INTEGER NOT NULL CHECK(success IN (0,1)),
  safe_summary TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_documents_locked ON documents(is_locked, sensitivity);
CREATE INDEX IF NOT EXISTS idx_security_events_created ON security_events(created_at DESC);
