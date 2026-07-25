-- Schema v27: Deep domain models for identity, vehicles, employment, warranties
-- Adds MRZ parsing, vehicle/service records, employment details, warranty tracking

-- Identity documents with MRZ fields
CREATE TABLE IF NOT EXISTS identity_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    identity_type TEXT NOT NULL CHECK(identity_type IN ('passport', 'national_id', 'drivers_license', 'residence_permit', 'other')),
    document_number TEXT,
    mrz_line1 TEXT,
    mrz_line2 TEXT,
    surname TEXT,
    given_names TEXT,
    nationality TEXT,
    birth_date TEXT,
    gender TEXT,
    issuing_country TEXT,
    issue_date TEXT,
    expiry_date TEXT,
    personal_number TEXT,
    is_primary INTEGER NOT NULL DEFAULT 0,
    is_locked INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'expired', 'cancelled', 'replaced', 'future')),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(document_id)
);

-- Vehicle records
CREATE TABLE IF NOT EXISTS vehicle_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    registration_number TEXT,
    vin TEXT,
    make TEXT,
    model TEXT,
    model_year INTEGER,
    owner_name TEXT,
    insurance_company TEXT,
    insurance_number TEXT,
    inspection_date TEXT,
    tax_status TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(document_id)
);

-- Service history
CREATE TABLE IF NOT EXISTS service_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    vehicle_record_id INTEGER NOT NULL REFERENCES vehicle_records(id) ON DELETE CASCADE,
    document_id INTEGER REFERENCES documents(id),
    service_date TEXT NOT NULL,
    mileage INTEGER,
    workshop_name TEXT,
    workshop_alias TEXT,
    service_category TEXT,
    actions_performed TEXT,
    next_recommended_service TEXT,
    cost INTEGER,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(vehicle_record_id, service_date, workshop_name)
);

-- Employment records
CREATE TABLE IF NOT EXISTS employment_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    employer_name TEXT NOT NULL,
    employer_alias TEXT,
    organization_number TEXT,
    position TEXT,
    employment_type TEXT CHECK(employment_type IN ('permanent', 'temporary', 'probation', 'contract', 'self_employed', 'other')),
    start_date TEXT,
    end_date TEXT,
    employment_rate REAL,
    workplace TEXT,
    collective_agreement TEXT,
    is_primary INTEGER NOT NULL DEFAULT 0,
    is_locked INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'paused', 'ended', 'future')),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(document_id)
);

-- Salary history (separate per employment)
CREATE TABLE IF NOT EXISTS salary_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employment_record_id INTEGER NOT NULL REFERENCES employment_records(id) ON DELETE CASCADE,
    document_id INTEGER REFERENCES documents(id),
    salary_type TEXT NOT NULL CHECK(salary_type IN ('hourly', 'monthly', 'annual', 'other')),
    amount REAL NOT NULL,
    currency TEXT NOT NULL DEFAULT 'SEK',
    effective_from TEXT NOT NULL,
    effective_to TEXT,
    is_current INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(employment_record_id, effective_from)
);

-- Supplementary agreements (ändringsavtal/tilläggsavtal)
CREATE TABLE IF NOT EXISTS supplementary_agreements (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employment_record_id INTEGER NOT NULL REFERENCES employment_records(id) ON DELETE CASCADE,
    document_id INTEGER REFERENCES documents(id),
    agreement_date TEXT NOT NULL,
    changes_json TEXT NOT NULL,
    supersedes_previous INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(employment_record_id, agreement_date)
);

-- Warranty and product records
CREATE TABLE IF NOT EXISTS warranty_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    product_name TEXT NOT NULL,
    model TEXT,
    serial_number TEXT,
    purchase_date TEXT,
    store_name TEXT,
    amount INTEGER,
    warranty_period_days INTEGER,
    calculated_end_date TEXT,
    manual_end_date TEXT,
    complaint_info TEXT,
    status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'expiring_soon', 'expired', 'unknown', 'conflict')),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(document_id)
);

-- Conflict resolution decisions
CREATE TABLE IF NOT EXISTS conflict_decisions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    conflict_id INTEGER NOT NULL REFERENCES conflicts(id) ON DELETE CASCADE,
    chosen_source TEXT NOT NULL,
    chosen_value TEXT,
    decision_type TEXT NOT NULL CHECK(decision_type IN ('source_a', 'source_b', 'both', 'neither', 'unresolved')),
    user_note TEXT,
    is_locked INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(conflict_id)
);

PRAGMA user_version = 27;