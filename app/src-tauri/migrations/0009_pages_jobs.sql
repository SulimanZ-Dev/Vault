CREATE TABLE IF NOT EXISTS document_pages (
  id INTEGER PRIMARY KEY,
  document_version_id INTEGER NOT NULL REFERENCES document_versions(id) ON DELETE CASCADE,
  page_no INTEGER NOT NULL,
  source_kind TEXT NOT NULL,
  text_quality TEXT NOT NULL DEFAULT 'unknown',
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(document_version_id, page_no)
);

CREATE TABLE IF NOT EXISTS text_spans (
  id INTEGER PRIMARY KEY,
  document_page_id INTEGER NOT NULL REFERENCES document_pages(id) ON DELETE CASCADE,
  source_kind TEXT NOT NULL,
  text TEXT NOT NULL,
  confidence REAL,
  language TEXT NOT NULL DEFAULT 'swe',
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE VIRTUAL TABLE IF NOT EXISTS span_fts USING fts5(
  text_span_id UNINDEXED,
  document_id UNINDEXED,
  page_no UNINDEXED,
  text
);

ALTER TABLE jobs ADD COLUMN progress_current INTEGER NOT NULL DEFAULT 0;
ALTER TABLE jobs ADD COLUMN progress_total INTEGER NOT NULL DEFAULT 0;
ALTER TABLE jobs ADD COLUMN pause_requested INTEGER NOT NULL DEFAULT 0;
ALTER TABLE jobs ADD COLUMN result_summary TEXT;

CREATE INDEX IF NOT EXISTS idx_document_pages_version ON document_pages(document_version_id, page_no);
CREATE INDEX IF NOT EXISTS idx_text_spans_page ON text_spans(document_page_id);
