CREATE TABLE IF NOT EXISTS import_sessions (
  id INTEGER PRIMARY KEY,
  vault_id INTEGER NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
  method TEXT NOT NULL,
  source_label TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'running',
  scanned_count INTEGER NOT NULL DEFAULT 0,
  imported_count INTEGER NOT NULL DEFAULT 0,
  duplicate_count INTEGER NOT NULL DEFAULT 0,
  failed_count INTEGER NOT NULL DEFAULT 0,
  started_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  completed_at TEXT
);

CREATE TABLE IF NOT EXISTS import_session_items (
  id INTEGER PRIMARY KEY,
  import_session_id INTEGER NOT NULL REFERENCES import_sessions(id) ON DELETE CASCADE,
  source_name TEXT NOT NULL,
  document_id INTEGER REFERENCES documents(id) ON DELETE SET NULL,
  status TEXT NOT NULL,
  safe_error TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_import_sessions_started ON import_sessions(started_at DESC);
CREATE INDEX IF NOT EXISTS idx_import_items_session ON import_session_items(import_session_id, id);
