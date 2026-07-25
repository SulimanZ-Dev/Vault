ALTER TABLE jobs ADD COLUMN payload_json TEXT NOT NULL DEFAULT '{}';
ALTER TABLE jobs ADD COLUMN checkpoint_json TEXT NOT NULL DEFAULT '{}';
ALTER TABLE jobs ADD COLUMN cancel_requested INTEGER NOT NULL DEFAULT 0;
ALTER TABLE jobs ADD COLUMN next_run_at TEXT;
ALTER TABLE jobs ADD COLUMN depends_on_job_id INTEGER REFERENCES jobs(id);
ALTER TABLE jobs ADD COLUMN max_attempts INTEGER NOT NULL DEFAULT 3;

ALTER TABLE automation_rules ADD COLUMN logic_operator TEXT NOT NULL DEFAULT 'AND'
  CHECK(logic_operator IN ('AND','OR','NOT'));
ALTER TABLE automation_rules ADD COLUMN conditions_json TEXT NOT NULL DEFAULT '[]';
ALTER TABLE automation_rules ADD COLUMN actions_json TEXT NOT NULL DEFAULT '[]';

CREATE TABLE dirty_documents (
  document_id INTEGER PRIMARY KEY REFERENCES documents(id) ON DELETE CASCADE,
  reason TEXT NOT NULL,
  dependency_kind TEXT NOT NULL DEFAULT 'document',
  marked_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE batch_runs (
  id INTEGER PRIMARY KEY,
  action_type TEXT NOT NULL,
  action_value TEXT NOT NULL DEFAULT '',
  document_ids_json TEXT NOT NULL,
  affected_count INTEGER NOT NULL DEFAULT 0,
  status TEXT NOT NULL CHECK(status IN ('completed','failed')),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_jobs_scheduler ON jobs(status, next_run_at, priority, id);
CREATE INDEX idx_jobs_dependency ON jobs(depends_on_job_id);

PRAGMA user_version = 29;
