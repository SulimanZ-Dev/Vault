CREATE TABLE IF NOT EXISTS analysis_preferences (
 vault_id INTEGER PRIMARY KEY DEFAULT 1,
 mode TEXT NOT NULL DEFAULT 'manual' CHECK(mode IN('none','manual','folder','category','entities','all','future','all_future')),
 auto_ocr INTEGER NOT NULL DEFAULT 1, auto_classify INTEGER NOT NULL DEFAULT 1,
 auto_claims INTEGER NOT NULL DEFAULT 1, auto_relations INTEGER NOT NULL DEFAULT 1,
 include_locked INTEGER NOT NULL DEFAULT 0, updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
INSERT OR IGNORE INTO analysis_preferences(vault_id) VALUES(1);
CREATE TABLE IF NOT EXISTS analysis_exclusions (
 id INTEGER PRIMARY KEY AUTOINCREMENT, vault_id INTEGER NOT NULL DEFAULT 1,
 scope_type TEXT NOT NULL CHECK(scope_type IN('document','folder','category','document_type')),
 scope_value TEXT NOT NULL, reason TEXT NOT NULL DEFAULT '', created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
 UNIQUE(vault_id,scope_type,scope_value)
);
