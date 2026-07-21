ALTER TABLE documents ADD COLUMN is_favorite INTEGER NOT NULL DEFAULT 0;
CREATE INDEX IF NOT EXISTS idx_documents_favorite ON documents(vault_id,is_favorite,updated_at DESC);
CREATE TABLE IF NOT EXISTS collections (
 id INTEGER PRIMARY KEY AUTOINCREMENT, vault_id INTEGER NOT NULL DEFAULT 1,
 name TEXT NOT NULL, description TEXT NOT NULL DEFAULT '', color TEXT NOT NULL DEFAULT '#7fd3a6',
 is_pinned INTEGER NOT NULL DEFAULT 0, created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
 updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP, UNIQUE(vault_id,name)
);
CREATE TABLE IF NOT EXISTS collection_documents (
 collection_id INTEGER NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
 document_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
 added_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP, PRIMARY KEY(collection_id,document_id)
);
CREATE INDEX IF NOT EXISTS idx_collection_documents_document ON collection_documents(document_id);
