ALTER TABLE plugins ADD COLUMN api_version TEXT NOT NULL DEFAULT '1.0';
ALTER TABLE plugins ADD COLUMN adapter_kind TEXT NOT NULL DEFAULT 'metadata_extractor';
ALTER TABLE plugins ADD COLUMN manifest_sha256 TEXT NOT NULL DEFAULT '';
ALTER TABLE plugins ADD COLUMN signature_status TEXT NOT NULL DEFAULT 'local_checksum';
ALTER TABLE plugins ADD COLUMN resource_limit INTEGER NOT NULL DEFAULT 10000;

CREATE TABLE IF NOT EXISTS external_import_sources (
  provider TEXT PRIMARY KEY CHECK(provider IN ('google_drive','onedrive','gmail','outlook')),
  display_name TEXT NOT NULL,
  enabled INTEGER NOT NULL DEFAULT 0 CHECK(enabled IN (0,1)),
  approved INTEGER NOT NULL DEFAULT 0 CHECK(approved IN (0,1)),
  local_staging_path TEXT,
  last_preview_at TEXT,
  last_import_at TEXT,
  last_result TEXT,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT OR IGNORE INTO external_import_sources(provider,display_name) VALUES
  ('google_drive','Google Drive'),
  ('onedrive','OneDrive'),
  ('gmail','Gmail'),
  ('outlook','Outlook');

PRAGMA user_version = 30;
