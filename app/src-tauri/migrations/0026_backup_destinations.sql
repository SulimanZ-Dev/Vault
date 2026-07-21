-- Schema v26: Backup destinations and safe unlock flow
-- Adds support for external disk, network folder destinations
-- and Windows Credential Manager / DPAPI integration

CREATE TABLE IF NOT EXISTS backup_destinations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    label TEXT NOT NULL,
    destination_type TEXT NOT NULL CHECK(destination_type IN ('local', 'external_disk', 'network_folder', 'usb')),
    path TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 0,
    auto_mount_command TEXT,
    requires_auth INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_success_at TEXT,
    last_failure TEXT,
    UNIQUE(path)
);

-- Safe credential storage for backup unlock (uses DPAPI on Windows)
CREATE TABLE IF NOT EXISTS secure_credentials (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    credential_key TEXT NOT NULL UNIQUE,
    credential_type TEXT NOT NULL CHECK(credential_type IN ('backup_password', 'export_password', 'pin_hash')),
    encrypted_value BLOB NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Track temporary file cleanup state
CREATE TABLE IF NOT EXISTS temp_cleanup_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    operation TEXT NOT NULL,
    temp_path TEXT NOT NULL,
    cleaned_up INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    cleaned_at TEXT
);

-- Track backup discovery on external destinations
CREATE TABLE IF NOT EXISTS discovered_backups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_destination_id INTEGER REFERENCES backup_destinations(id),
    backup_root TEXT NOT NULL,
    manifest_sha256 TEXT,
    discovered_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_verified_at TEXT,
    is_valid INTEGER NOT NULL DEFAULT 0,
    UNIQUE(source_destination_id, backup_root)
);

PRAGMA user_version = 26;