CREATE TABLE vault_notes (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  vault_id INTEGER NOT NULL DEFAULT 1 REFERENCES vaults(id) ON DELETE CASCADE,
  title TEXT NOT NULL,
  body TEXT NOT NULL,
  kind TEXT NOT NULL DEFAULT 'note',
  target_type TEXT NOT NULL DEFAULT 'vault',
  target_id INTEGER,
  tags_json TEXT NOT NULL DEFAULT '[]',
  code_words_json TEXT NOT NULL DEFAULT '[]',
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE vault_note_versions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  note_id INTEGER NOT NULL REFERENCES vault_notes(id) ON DELETE CASCADE,
  version_no INTEGER NOT NULL,
  title TEXT NOT NULL,
  body TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(note_id, version_no)
);

CREATE TABLE note_templates (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  vault_id INTEGER NOT NULL DEFAULT 1 REFERENCES vaults(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  body TEXT NOT NULL,
  UNIQUE(vault_id, name)
);

CREATE INDEX idx_vault_notes_target ON vault_notes(vault_id, target_type, target_id);
CREATE INDEX idx_vault_notes_updated ON vault_notes(vault_id, updated_at DESC);
INSERT OR IGNORE INTO note_templates(vault_id,name,body) VALUES
  (1,'Mötesanteckning','# Möte\n\n- Datum: \n- Deltagare: \n\n## Beslut\n\n- [ ] Uppföljning'),
  (1,'Dokumentgranskning','# Granskning\n\n## Observationer\n\n## Att göra\n\n- [ ] Kontrollera källa');
