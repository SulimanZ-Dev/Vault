ALTER TABLE documents ADD COLUMN document_type TEXT NOT NULL DEFAULT 'Oklassificerat';
ALTER TABLE documents ADD COLUMN document_date TEXT;
ALTER TABLE documents ADD COLUMN source_label TEXT NOT NULL DEFAULT 'Lokal import';
ALTER TABLE documents ADD COLUMN match_explanation TEXT NOT NULL DEFAULT 'Ingen sökträff vald';
