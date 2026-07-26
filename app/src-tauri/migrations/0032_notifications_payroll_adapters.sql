CREATE TABLE IF NOT EXISTS payroll_records (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  employment_record_id INTEGER REFERENCES employment_records(id) ON DELETE SET NULL,
  document_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
  period_label TEXT,
  payment_date TEXT,
  gross_amount REAL,
  net_amount REAL,
  currency TEXT NOT NULL DEFAULT 'SEK',
  fields_json TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(document_id)
);

CREATE TABLE IF NOT EXISTS local_notifications (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  notification_key TEXT NOT NULL UNIQUE,
  severity TEXT NOT NULL CHECK(severity IN ('info','warning','urgent')),
  category TEXT NOT NULL,
  title TEXT NOT NULL,
  body TEXT NOT NULL,
  document_id INTEGER REFERENCES documents(id) ON DELETE CASCADE,
  due_date TEXT,
  read_at TEXT,
  dismissed_at TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_local_notifications_state
ON local_notifications(dismissed_at, read_at, due_date);

CREATE TABLE IF NOT EXISTS plugin_adapter_artifacts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  plugin_id TEXT NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
  adapter_kind TEXT NOT NULL,
  artifact_key TEXT NOT NULL,
  label TEXT NOT NULL,
  value_json TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(plugin_id, adapter_kind, artifact_key)
);

PRAGMA user_version = 32;
