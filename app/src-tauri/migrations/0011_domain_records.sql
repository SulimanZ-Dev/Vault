CREATE TABLE IF NOT EXISTS domain_records (
  id INTEGER PRIMARY KEY,
  vault_id INTEGER NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
  document_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
  domain_type TEXT NOT NULL,
  record_type TEXT NOT NULL,
  subject TEXT NOT NULL,
  effective_from TEXT,
  effective_to TEXT,
  actuality_status TEXT NOT NULL,
  actuality_explanation TEXT NOT NULL,
  fields_json TEXT NOT NULL DEFAULT '{}',
  source_page_no INTEGER,
  extraction_method TEXT NOT NULL,
  manually_locked INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(document_id, domain_type, record_type)
);

CREATE INDEX IF NOT EXISTS idx_domain_records_domain ON domain_records(domain_type, actuality_status);
CREATE INDEX IF NOT EXISTS idx_domain_records_period ON domain_records(effective_from, effective_to);
