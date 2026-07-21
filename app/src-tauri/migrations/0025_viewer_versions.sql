ALTER TABLE document_versions ADD COLUMN comment TEXT NOT NULL DEFAULT '';
ALTER TABLE document_versions ADD COLUMN origin TEXT NOT NULL DEFAULT 'import';

CREATE TABLE document_annotations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    page_no INTEGER NOT NULL DEFAULT 1,
    annotation_type TEXT NOT NULL CHECK(annotation_type IN ('bookmark','note','highlight')),
    selected_text TEXT NOT NULL DEFAULT '',
    body TEXT NOT NULL DEFAULT '',
    color TEXT NOT NULL DEFAULT '#ffe08a',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_document_annotations_document ON document_annotations(document_id, page_no);
