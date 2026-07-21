CREATE TABLE graph_relations (
 id INTEGER PRIMARY KEY AUTOINCREMENT, vault_id INTEGER NOT NULL DEFAULT 1,
 source_type TEXT NOT NULL, source_id INTEGER NOT NULL, target_type TEXT NOT NULL, target_id INTEGER NOT NULL,
 relation_type TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'approved' CHECK(status IN('suggested','approved','rejected','historical')),
 valid_from TEXT, valid_to TEXT, rule_explanation TEXT NOT NULL DEFAULT '',
 created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP, updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
 UNIQUE(vault_id,source_type,source_id,target_type,target_id,relation_type)
);
CREATE TABLE calendar_events (
 id INTEGER PRIMARY KEY AUTOINCREMENT, vault_id INTEGER NOT NULL DEFAULT 1,
 document_id INTEGER REFERENCES documents(id) ON DELETE CASCADE, title TEXT NOT NULL, event_date TEXT NOT NULL,
 event_type TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'active' CHECK(status IN('active','completed','cancelled')),
 source_kind TEXT NOT NULL DEFAULT 'manual', note TEXT NOT NULL DEFAULT '',
 created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP, updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_graph_relation_source ON graph_relations(source_type,source_id);
CREATE INDEX idx_graph_relation_target ON graph_relations(target_type,target_id);
CREATE INDEX idx_calendar_event_date ON calendar_events(vault_id,event_date);
