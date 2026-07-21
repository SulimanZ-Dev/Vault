CREATE TABLE IF NOT EXISTS code_words (
  id INTEGER PRIMARY KEY,
  vault_id INTEGER NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
  word TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(vault_id, word)
);

CREATE INDEX IF NOT EXISTS idx_documents_review_queue
ON documents(inbox_status, document_type, document_date);

CREATE INDEX IF NOT EXISTS idx_claims_review_status
ON claims(status, claim_type);
