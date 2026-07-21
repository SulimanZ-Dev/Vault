CREATE TABLE folders (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  vault_id INTEGER NOT NULL DEFAULT 1,
  parent_id INTEGER REFERENCES folders(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  color TEXT NOT NULL DEFAULT '#7fd3a6',
  is_pinned INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(vault_id, parent_id, name)
);

CREATE TABLE document_folders (
  document_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
  folder_id INTEGER NOT NULL REFERENCES folders(id) ON DELETE CASCADE,
  PRIMARY KEY(document_id, folder_id)
);

CREATE TABLE categories (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  vault_id INTEGER NOT NULL DEFAULT 1,
  name TEXT NOT NULL,
  color TEXT NOT NULL DEFAULT '#84aef5',
  description TEXT NOT NULL DEFAULT '',
  is_pinned INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(vault_id, name)
);

CREATE TABLE document_categories (
  document_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
  category_id INTEGER NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
  PRIMARY KEY(document_id, category_id)
);

CREATE TABLE document_notes (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  document_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
  body TEXT NOT NULL,
  kind TEXT NOT NULL DEFAULT 'note',
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE note_versions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  note_id INTEGER NOT NULL REFERENCES document_notes(id) ON DELETE CASCADE,
  version_no INTEGER NOT NULL,
  body TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(note_id, version_no)
);

CREATE TABLE search_history (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  vault_id INTEGER NOT NULL DEFAULT 1,
  query TEXT NOT NULL,
  result_count INTEGER NOT NULL DEFAULT 0,
  executed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  pinned INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_document_folders_folder ON document_folders(folder_id);
CREATE INDEX idx_document_categories_category ON document_categories(category_id);
CREATE INDEX idx_document_notes_document ON document_notes(document_id, updated_at DESC);
CREATE INDEX idx_search_history_recent ON search_history(vault_id, executed_at DESC);
