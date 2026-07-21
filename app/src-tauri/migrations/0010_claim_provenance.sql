ALTER TABLE claims ADD COLUMN source_span_id INTEGER REFERENCES text_spans(id) ON DELETE SET NULL;
ALTER TABLE claims ADD COLUMN effective_from TEXT;
ALTER TABLE claims ADD COLUMN effective_to TEXT;
ALTER TABLE claims ADD COLUMN actuality_status TEXT NOT NULL DEFAULT 'uncertain';
ALTER TABLE claims ADD COLUMN actuality_explanation TEXT NOT NULL DEFAULT 'Ingen deterministisk aktualitetsregel kunde tillämpas.';

CREATE INDEX IF NOT EXISTS idx_claims_source_span_id ON claims(source_span_id);
CREATE INDEX IF NOT EXISTS idx_claims_actuality ON claims(actuality_status, effective_from, effective_to);
