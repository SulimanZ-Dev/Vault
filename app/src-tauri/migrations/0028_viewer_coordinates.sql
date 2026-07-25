-- Normalized viewer coordinates are fractions of the unrotated page:
-- x/y/width/height are in the inclusive range 0.0..1.0.
ALTER TABLE document_annotations ADD COLUMN document_version_id INTEGER REFERENCES document_versions(id) ON DELETE CASCADE;
ALTER TABLE document_annotations ADD COLUMN x REAL;
ALTER TABLE document_annotations ADD COLUMN y REAL;
ALTER TABLE document_annotations ADD COLUMN width REAL;
ALTER TABLE document_annotations ADD COLUMN height REAL;
ALTER TABLE document_annotations ADD COLUMN text_start INTEGER;
ALTER TABLE document_annotations ADD COLUMN text_end INTEGER;
ALTER TABLE document_annotations ADD COLUMN claim_id INTEGER REFERENCES claims(id) ON DELETE SET NULL;
ALTER TABLE document_annotations ADD COLUMN rule_result TEXT NOT NULL DEFAULT '';

CREATE INDEX IF NOT EXISTS idx_document_annotations_version_page
ON document_annotations(document_version_id, page_no);

CREATE INDEX IF NOT EXISTS idx_document_annotations_claim
ON document_annotations(claim_id);
