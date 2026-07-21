CREATE TABLE IF NOT EXISTS entities (
  id INTEGER PRIMARY KEY,
  vault_id INTEGER NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
  entity_type TEXT NOT NULL,
  display_name TEXT NOT NULL,
  normalized_name TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(vault_id, entity_type, normalized_name)
);

CREATE TABLE IF NOT EXISTS document_entities (
  document_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
  entity_id INTEGER NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
  role TEXT NOT NULL DEFAULT 'mentioned',
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY(document_id, entity_id, role)
);

CREATE INDEX IF NOT EXISTS idx_document_entities_entity_id
ON document_entities(entity_id);
