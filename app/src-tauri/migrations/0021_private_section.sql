ALTER TABLE documents ADD COLUMN is_private INTEGER NOT NULL DEFAULT 0 CHECK(is_private IN (0,1));
ALTER TABLE documents ADD COLUMN is_hidden INTEGER NOT NULL DEFAULT 0 CHECK(is_hidden IN (0,1));
ALTER TABLE files ADD COLUMN encrypted_at_rest INTEGER NOT NULL DEFAULT 0 CHECK(encrypted_at_rest IN (0,1));
ALTER TABLE files ADD COLUMN encrypted_original_name TEXT;

CREATE INDEX IF NOT EXISTS idx_documents_private ON documents(is_private,is_hidden,trashed_at);
