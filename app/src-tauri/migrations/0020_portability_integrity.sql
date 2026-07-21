CREATE TABLE IF NOT EXISTS integrity_scans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    checked_files INTEGER NOT NULL DEFAULT 0,
    intact_files INTEGER NOT NULL DEFAULT 0,
    missing_files INTEGER NOT NULL DEFAULT 0,
    changed_files INTEGER NOT NULL DEFAULT 0,
    duplicate_groups INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS duplicate_decisions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    primary_document_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    secondary_document_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    decision TEXT NOT NULL CHECK(decision IN ('keep_both','primary_selected','not_duplicate')),
    note TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(primary_document_id, secondary_document_id)
);
