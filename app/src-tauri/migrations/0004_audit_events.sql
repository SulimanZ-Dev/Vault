CREATE TABLE IF NOT EXISTS audit_events (
  id INTEGER PRIMARY KEY,
  vault_id INTEGER NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
  actor_kind TEXT NOT NULL DEFAULT 'local_app',
  event_type TEXT NOT NULL,
  target_type TEXT NOT NULL,
  target_id INTEGER,
  safe_summary TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_audit_events_created_at
ON audit_events(created_at DESC);
