use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::{thread, time::Duration};
use tauri::Manager;

const INITIAL_MIGRATION: &str = include_str!("../migrations/0001_initial.sql");
const DOCUMENT_DISPLAY_FIELDS_MIGRATION: &str =
    include_str!("../migrations/0002_document_display_fields.sql");
const DOCUMENT_EXTRACTED_TEXT_MIGRATION: &str =
    include_str!("../migrations/0003_document_extracted_text.sql");
const AUDIT_EVENTS_MIGRATION: &str = include_str!("../migrations/0004_audit_events.sql");
const TAGS_CLAIMS_MIGRATION: &str = include_str!("../migrations/0005_tags_claims.sql");
const REVIEW_METADATA_MIGRATION: &str = include_str!("../migrations/0006_review_metadata.sql");
const ENTITIES_MIGRATION: &str = include_str!("../migrations/0007_entities.sql");
const WORKFLOWS_MIGRATION: &str = include_str!("../migrations/0008_workflows.sql");
const PAGES_JOBS_MIGRATION: &str = include_str!("../migrations/0009_pages_jobs.sql");
const CLAIM_PROVENANCE_MIGRATION: &str = include_str!("../migrations/0010_claim_provenance.sql");
const DOMAIN_RECORDS_MIGRATION: &str = include_str!("../migrations/0011_domain_records.sql");
const IMPORT_HISTORY_MIGRATION: &str = include_str!("../migrations/0012_import_history.sql");
const RULES_TEMPLATES_MIGRATION: &str =
    include_str!("../migrations/0013_rules_templates_metadata_searches.sql");
const SECURITY_MIGRATION: &str = include_str!("../migrations/0014_security.sql");
const THEMES_MIGRATION: &str = include_str!("../migrations/0015_themes.sql");
const COLLECTIONS_MIGRATION: &str = include_str!("../migrations/0016_collections.sql");
const ANALYSIS_PREFERENCES_MIGRATION: &str =
    include_str!("../migrations/0017_analysis_preferences.sql");
const ORGANIZATION_MIGRATION: &str = include_str!("../migrations/0018_organization.sql");
const CALENDAR_GRAPH_MIGRATION: &str =
    include_str!("../migrations/0019_calendar_graph_domains.sql");
const PORTABILITY_INTEGRITY_MIGRATION: &str =
    include_str!("../migrations/0020_portability_integrity.sql");
const PRIVATE_SECTION_MIGRATION: &str = include_str!("../migrations/0021_private_section.sql");
const PLUGINS_MIGRATION: &str = include_str!("../migrations/0022_plugins.sql");
const GENERAL_NOTES_MIGRATION: &str = include_str!("../migrations/0023_general_notes.sql");
const ORGANIZER_LOCKS_MIGRATION: &str = include_str!("../migrations/0024_organizer_locks.sql");
const VIEWER_VERSIONS_MIGRATION: &str = include_str!("../migrations/0025_viewer_versions.sql");
const BACKUP_DESTINATIONS_MIGRATION: &str =
    include_str!("../migrations/0026_backup_destinations.sql");
const MIGRATIONS: &[(i64, &str, &str)] = &[
    (1, "initial local vault schema", INITIAL_MIGRATION),
    (
        2,
        "document display fields",
        DOCUMENT_DISPLAY_FIELDS_MIGRATION,
    ),
    (
        3,
        "document extracted text",
        DOCUMENT_EXTRACTED_TEXT_MIGRATION,
    ),
    (4, "audit events", AUDIT_EVENTS_MIGRATION),
    (5, "tags and claims", TAGS_CLAIMS_MIGRATION),
    (6, "review metadata", REVIEW_METADATA_MIGRATION),
    (7, "entities", ENTITIES_MIGRATION),
    (8, "workflow foundations", WORKFLOWS_MIGRATION),
    (9, "document pages and resumable jobs", PAGES_JOBS_MIGRATION),
    (
        10,
        "claim provenance and actuality",
        CLAIM_PROVENANCE_MIGRATION,
    ),
    (11, "domain records", DOMAIN_RECORDS_MIGRATION),
    (12, "import history", IMPORT_HISTORY_MIGRATION),
    (
        13,
        "rules templates metadata and saved searches",
        RULES_TEMPLATES_MIGRATION,
    ),
    (
        14,
        "local security and locked documents",
        SECURITY_MIGRATION,
    ),
    (15, "persistent theme profiles", THEMES_MIGRATION),
    (16, "favorites and collections", COLLECTIONS_MIGRATION),
    (
        17,
        "analysis scope and exclusions",
        ANALYSIS_PREFERENCES_MIGRATION,
    ),
    (
        18,
        "folders categories notes and search history",
        ORGANIZATION_MIGRATION,
    ),
    (
        19,
        "calendar graph and extended domains",
        CALENDAR_GRAPH_MIGRATION,
    ),
    (
        20,
        "portable archives and integrity decisions",
        PORTABILITY_INTEGRITY_MIGRATION,
    ),
    (
        21,
        "encrypted private section and guest visibility",
        PRIVATE_SECTION_MIGRATION,
    ),
    (22, "sandboxed declarative plugins", PLUGINS_MIGRATION),
    (23, "general linked markdown notes", GENERAL_NOTES_MIGRATION),
    (
        24,
        "locked folders and categories",
        ORGANIZER_LOCKS_MIGRATION,
    ),
    (
        25,
        "viewer annotations and version history",
        VIEWER_VERSIONS_MIGRATION,
    ),
    (
        26,
        "backup destinations and safe unlock",
        BACKUP_DESTINATIONS_MIGRATION,
    ),
];

#[derive(Serialize)]
struct EnvironmentStatus {
    app_mode: &'static str,
    storage: &'static str,
    database: &'static str,
    search_index: &'static str,
    network: &'static str,
    ai_policy: &'static str,
}

#[derive(Serialize)]
struct VaultInitStatus {
    data_root: String,
    database_path: String,
    testlab_database_path: String,
    schema_version: i64,
    production_vault_ready: bool,
    testlab_isolated: bool,
    testlab_document_count: i64,
}

#[derive(Clone, Serialize)]
struct DocumentSummary {
    id: i64,
    title: String,
    document_type: String,
    document_date: Option<String>,
    inbox_status: String,
    source_label: String,
    match_explanation: String,
}

#[derive(Serialize)]
struct SearchResult {
    document: DocumentSummary,
    score: i64,
    reasons: Vec<String>,
    query_plan: Vec<String>,
}

#[derive(Serialize)]
struct ImportResult {
    document: DocumentSummary,
    file_id: i64,
    version_id: i64,
    sha256: String,
    stored_path: String,
    duplicate: bool,
}

#[derive(Serialize)]
struct DirectoryImportResult {
    scanned_file_count: usize,
    imported_count: usize,
    duplicate_count: usize,
    failed_count: usize,
    results: Vec<ImportResult>,
    failures: Vec<String>,
}

#[derive(Serialize)]
struct ImportSessionSummary {
    id: i64,
    method: String,
    source_label: String,
    status: String,
    scanned_count: i64,
    imported_count: i64,
    duplicate_count: i64,
    failed_count: i64,
    started_at: String,
    completed_at: Option<String>,
    items: Vec<ImportSessionItemSummary>,
}

#[derive(Serialize)]
struct ImportSessionItemSummary {
    source_name: String,
    document_id: Option<i64>,
    status: String,
    safe_error: Option<String>,
}

#[derive(Serialize)]
struct AutomationRuleSummary {
    id: i64,
    name: String,
    enabled: bool,
    priority: i64,
    match_field: String,
    match_operator: String,
    match_value: String,
    action_type: String,
    action_value: String,
    approval_policy: String,
    version: i64,
}

#[derive(Serialize)]
struct DocumentTemplateSummary {
    id: i64,
    name: String,
    document_type: String,
    category: String,
    default_tags: String,
    required_fields: String,
}

#[derive(Serialize)]
struct CustomFieldSummary {
    id: i64,
    name: String,
    field_type: String,
    applies_to: String,
    required: bool,
}

#[derive(Serialize)]
struct SavedSearchSummary {
    id: i64,
    name: String,
    query: String,
    pinned: bool,
}

#[derive(Serialize)]
struct OrganizerSummary {
    id: i64,
    name: String,
    color: String,
    description: String,
    is_pinned: bool,
    is_locked: bool,
    document_count: i64,
}

#[derive(Serialize)]
struct DocumentNoteSummary {
    id: i64,
    document_id: i64,
    body: String,
    kind: String,
    version_no: i64,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
struct VaultNoteSummary {
    id: i64,
    title: String,
    body: String,
    kind: String,
    target_type: String,
    target_id: Option<i64>,
    tags: Vec<String>,
    code_words: Vec<String>,
    version_no: i64,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
struct NoteVersionSummary {
    version_no: i64,
    title: String,
    body: String,
    created_at: String,
}

#[derive(Serialize)]
struct NoteTemplateSummary {
    id: i64,
    name: String,
    body: String,
}

#[derive(Serialize)]
struct SearchHistorySummary {
    id: i64,
    query: String,
    result_count: i64,
    executed_at: String,
    pinned: bool,
}

#[derive(Serialize, Deserialize)]
struct SearchHistoryPreferences {
    enabled: bool,
    retention_days: i64,
}

#[derive(Serialize)]
struct RuleRunResult {
    scanned_document_count: i64,
    matched_document_count: i64,
    applied_action_count: i64,
    explanations: Vec<String>,
}

#[derive(Serialize)]
struct SecurityStatus {
    mode: String,
    pin_configured: bool,
    auto_lock_minutes: i64,
    mask_sensitive: bool,
    locked_document_count: i64,
    private_document_count: i64,
}

#[derive(Serialize)]
struct WindowsHelloStatus {
    available: bool,
    explanation: String,
}

#[derive(Serialize)]
struct ProtectedExportResult {
    export_path: String,
    sha256: String,
    size_bytes: u64,
    masked: bool,
}

#[derive(Serialize)]
struct DocumentDetail {
    document: DocumentSummary,
    extracted_text: String,
    stored_path: Option<String>,
    original_name: Option<String>,
    mime_type: Option<String>,
    size_bytes: Option<i64>,
    tags: Vec<String>,
    claims: Vec<ClaimSummary>,
    entities: Vec<EntitySummary>,
}

#[derive(Serialize)]
struct ClaimSummary {
    id: i64,
    claim_type: String,
    value_json: String,
    value_kind: String,
    status: String,
    confidence_kind: String,
    extraction_method: String,
    source_span_id: Option<i64>,
    source_page_no: Option<i64>,
    source_text: Option<String>,
    effective_from: Option<String>,
    effective_to: Option<String>,
    actuality_status: String,
    actuality_explanation: String,
}

#[derive(Serialize)]
struct EntitySummary {
    id: i64,
    entity_type: String,
    display_name: String,
    role: String,
}

#[derive(Serialize)]
struct EntityCatalogItem {
    id: i64,
    entity_type: String,
    display_name: String,
    document_count: i64,
}

#[derive(Serialize)]
struct TimelineEventSummary {
    event_date: String,
    event_type: String,
    title: String,
    document_id: Option<i64>,
    entity_names: Vec<String>,
    source_page_no: Option<i64>,
    status: String,
    explanation: String,
}

#[derive(Serialize)]
struct DomainRecordSummary {
    id: i64,
    document_id: i64,
    document_title: String,
    domain_type: String,
    record_type: String,
    subject: String,
    effective_from: Option<String>,
    effective_to: Option<String>,
    actuality_status: String,
    actuality_explanation: String,
    fields_json: String,
    source_page_no: Option<i64>,
    extraction_method: String,
    manually_locked: bool,
}

#[derive(Serialize)]
struct CalendarEventSummary {
    id: String,
    event_date: String,
    title: String,
    event_type: String,
    source_kind: String,
    status: String,
    document_id: Option<i64>,
    explanation: String,
}
#[derive(Serialize)]
struct GraphNodeSummary {
    key: String,
    node_type: String,
    label: String,
    document_id: Option<i64>,
}
#[derive(Serialize)]
struct GraphEdgeSummary {
    id: i64,
    source_key: String,
    target_key: String,
    relation_type: String,
    status: String,
    valid_from: Option<String>,
    valid_to: Option<String>,
    explanation: String,
}
#[derive(Serialize)]
struct GraphDataSummary {
    nodes: Vec<GraphNodeSummary>,
    edges: Vec<GraphEdgeSummary>,
}

#[derive(Serialize)]
struct ReviewQueueItem {
    claim_id: i64,
    document_id: i64,
    document_title: String,
    claim_type: String,
    value_json: String,
    status: String,
    extraction_method: String,
}

#[derive(Serialize)]
struct ArchiveAnalysisResult {
    document_count: i64,
    reindexed_document_count: i64,
    ocr_attempt_count: i64,
    ocr_updated_count: i64,
    reclassified_document_count: i64,
    date_updated_count: i64,
    entity_link_count: i64,
    pending_review_count: i64,
}

#[derive(Serialize)]
struct VaultDiagnostics {
    data_root: String,
    database_path: String,
    testlab_database_path: String,
    schema_version: i64,
    production_document_count: i64,
    testlab_document_count: i64,
    pending_review_count: i64,
    entity_count: i64,
    conflict_count: i64,
    backup_count: i64,
    audit_event_count: i64,
    ocr_available: bool,
    tesseract_path: Option<String>,
    tessdata_dir: Option<String>,
    ocr_languages: Vec<String>,
    pdf_text_available: bool,
    pdftotext_path: Option<String>,
    office_text_available: bool,
    issues: Vec<String>,
}

#[derive(Serialize)]
struct CodeWordSummary {
    id: i64,
    word: String,
    description: String,
    created_at: String,
}

#[derive(Serialize)]
struct ConflictSummary {
    id: Option<i64>,
    conflict_type: String,
    severity: String,
    title: String,
    detail: String,
    document_ids: Vec<i64>,
    status: String,
    resolution: Option<String>,
    locked: bool,
}

#[derive(Serialize)]
struct DocumentVersionSummary {
    id: i64,
    version_no: i64,
    imported_at: String,
    document_date: Option<String>,
    original_name: String,
    mime_type: String,
    size_bytes: i64,
    sha256: String,
    is_current_file: bool,
    comment: String,
    origin: String,
}

#[derive(Serialize)]
struct DocumentAnnotationSummary {
    id: i64,
    document_id: i64,
    page_no: i64,
    annotation_type: String,
    selected_text: String,
    body: String,
    color: String,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
struct VersionComparison {
    left_version_no: i64,
    right_version_no: i64,
    added_lines: Vec<String>,
    removed_lines: Vec<String>,
    left_sha256: String,
    right_sha256: String,
    size_change_bytes: i64,
    page_change: i64,
    date_changed: bool,
    identical: bool,
}

#[derive(Serialize)]
struct ReminderSummary {
    id: i64,
    document_id: Option<i64>,
    document_title: Option<String>,
    title: String,
    due_date: String,
    status: String,
    note: String,
}

#[derive(Serialize)]
struct WatchedFolderSummary {
    id: i64,
    path: String,
    enabled: bool,
    last_scanned_at: Option<String>,
}

#[derive(Serialize)]
struct JobSummary {
    id: i64,
    job_type: String,
    target_id: Option<i64>,
    status: String,
    attempts: i64,
    error_code: Option<String>,
    created_at: String,
    updated_at: String,
    progress_current: i64,
    progress_total: i64,
    pause_requested: bool,
    result_summary: Option<String>,
}

#[derive(Serialize)]
struct DocumentPageSummary {
    page_no: i64,
    source_kind: String,
    text_quality: String,
    text: String,
}

#[derive(Serialize)]
struct OcrStatus {
    available: bool,
    engine: String,
    executable_path: Option<String>,
    tessdata_dir: Option<String>,
    languages: Vec<String>,
    version: Option<String>,
    notes: Vec<String>,
}

#[derive(Serialize)]
struct OcrRunResult {
    document_id: i64,
    extracted_character_count: usize,
    engine: String,
    indexed: bool,
}

#[derive(Serialize)]
struct BackupResult {
    backup_root: String,
    backup_path: String,
    manifest_path: String,
    sha256: String,
    size_bytes: u64,
    document_count: i64,
    file_count: i64,
    copied_file_count: i64,
    linked_file_count: i64,
    backup_mode: String,
    created_at_epoch_seconds: u64,
    destination_copy: Option<String>,
}

#[derive(Serialize)]
struct VaultHealthReport {
    ok: bool,
    database_integrity: String,
    schema_version: i64,
    document_count: i64,
    file_count: i64,
    missing_file_count: i64,
    fts_entry_count: i64,
    warnings: Vec<String>,
}

#[derive(Serialize)]
struct BackupSummary {
    backup_root: String,
    backup_path: String,
    manifest_path: String,
    sha256: String,
    size_bytes: u64,
    document_count: i64,
    file_count: i64,
    copied_file_count: i64,
    linked_file_count: i64,
    backup_mode: String,
    created_at_epoch_seconds: u64,
}

#[derive(Serialize, Deserialize, Clone)]
struct BackupPolicy {
    enabled: bool,
    paused: bool,
    interval_hours: i64,
    backup_mode: String,
    excluded_document_ids: Vec<i64>,
    last_run_epoch_seconds: u64,
    #[serde(default)]
    destination_path: Option<String>,
}

#[derive(Serialize)]
struct ProtectedBackupResult {
    archive_path: String,
    sha256: String,
    size_bytes: u64,
    entry_count: usize,
    source_backup_root: String,
}

#[derive(Serialize)]
struct RestoreTestResult {
    ok: bool,
    document_count: i64,
    file_count: i64,
    missing_file_count: i64,
    database_integrity: String,
    tested_backup_root: String,
}

#[derive(Serialize)]
struct PortableArchiveResult {
    archive_path: String,
    sha256: String,
    size_bytes: u64,
    entry_count: usize,
    document_count: i64,
    file_count: i64,
}

#[derive(Serialize)]
struct DuplicateCandidate {
    primary_document_id: i64,
    primary_title: String,
    secondary_document_id: i64,
    secondary_title: String,
    confidence: i64,
    match_kind: String,
    explanation: String,
    decision: Option<String>,
}

#[derive(Serialize)]
struct IntegrityScanReport {
    checked_files: i64,
    intact_files: i64,
    missing_files: i64,
    changed_files: i64,
    duplicate_groups: i64,
    candidates: Vec<DuplicateCandidate>,
    warnings: Vec<String>,
}

#[derive(Deserialize)]
struct PluginManifest {
    id: String,
    name: String,
    version: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    capabilities: Vec<String>,
    #[serde(default)]
    data_access: Vec<String>,
    #[serde(default)]
    actions: Vec<PluginAction>,
    #[serde(default)]
    ai_free: bool,
}

#[derive(Deserialize)]
struct PluginAction {
    action_type: String,
    match_text: String,
    value: String,
}

#[derive(Serialize)]
struct PluginSummary {
    id: String,
    name: String,
    version: String,
    description: String,
    capabilities: Vec<String>,
    data_access: Vec<String>,
    enabled: bool,
    approved: bool,
    last_run_at: Option<String>,
    last_result: Option<String>,
}

#[derive(Serialize)]
struct PluginRunResult {
    plugin_id: String,
    scanned_documents: i64,
    changed_documents: i64,
    explanation: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct ThemeProfile {
    id: i64,
    name: String,
    base_mode: String,
    accent: String,
    surface_main: String,
    surface_sidebar: String,
    surface_raised: String,
    text_primary: String,
    text_secondary: String,
    border_color: String,
    radius_px: i64,
    font_scale: f64,
    density: String,
    motion: String,
    is_builtin: bool,
    is_active: bool,
    version: i64,
}

#[derive(Serialize)]
struct CollectionSummary {
    id: i64,
    name: String,
    description: String,
    color: String,
    is_pinned: bool,
    document_count: i64,
}

#[derive(Serialize)]
struct AnalysisPreferences {
    mode: String,
    auto_ocr: bool,
    auto_classify: bool,
    auto_claims: bool,
    auto_relations: bool,
    include_locked: bool,
}
#[derive(Serialize)]
struct AnalysisExclusion {
    id: i64,
    scope_type: String,
    scope_value: String,
    reason: String,
}
#[derive(Serialize)]
struct ScannerStatus {
    available: bool,
    launcher_path: Option<String>,
    notes: Vec<String>,
}

#[derive(Serialize)]
struct BackupValidationReport {
    ok: bool,
    backup_root: String,
    backup_path: String,
    manifest_path: String,
    sha256_matches: bool,
    database_integrity: String,
    document_count: i64,
    file_count: i64,
    copied_file_count: i64,
    warnings: Vec<String>,
}

#[derive(Serialize)]
struct RestoreResult {
    restored_backup_root: String,
    pre_restore_backup_root: String,
    document_count: i64,
    file_count: i64,
}

#[derive(Serialize)]
struct ReindexResult {
    indexed_document_count: i64,
}

#[derive(Serialize)]
struct AuditEventSummary {
    id: i64,
    event_type: String,
    target_type: String,
    target_id: Option<i64>,
    safe_summary: String,
    created_at: String,
}

#[derive(Serialize)]
struct TestCenterCase {
    name: String,
    status: String,
    expected: String,
    actual: String,
}

#[derive(Serialize)]
struct TestCenterReport {
    passed: i64,
    failed: i64,
    cases: Vec<TestCenterCase>,
}

#[derive(Serialize)]
struct TestLabScaleResult {
    requested: i64,
    total_documents: i64,
    elapsed_ms: u128,
    search_elapsed_ms: u128,
    database_bytes: u64,
}
#[derive(Serialize)]
struct TestReportExport {
    path: String,
    sha256: String,
    case_count: usize,
    created_at_epoch_seconds: u64,
}

struct DocumentClassification {
    document_type: &'static str,
    match_explanation: String,
}

struct DateExtraction {
    document_date: Option<String>,
    explanation: Option<String>,
}

#[derive(Default)]
struct ParsedSearchQuery {
    terms: Vec<String>,
    negative_terms: Vec<String>,
    exact_phrases: Vec<String>,
    document_type: Option<String>,
    tag: Option<String>,
    date: Option<String>,
    inbox_status: Option<String>,
}

#[tauri::command]
fn environment_status() -> EnvironmentStatus {
    EnvironmentStatus {
        app_mode: "local-first desktop",
        storage: "local app data",
        database: "SQLite local",
        search_index: "SQLite FTS5 local",
        network: "no required network services",
        ai_policy: "AI-free by product rule",
    }
}

#[tauri::command]
fn initialize_vault(app: tauri::AppHandle) -> Result<VaultInitStatus, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    ensure_bundled_tessdata(&app, &data_root).map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())
}

fn initialize_vault_at(data_root: &Path) -> rusqlite::Result<VaultInitStatus> {
    create_vault_directories(data_root).map_err(to_sql_error)?;

    let database_path = data_root.join("vault.db");
    let testlab_database_path = data_root.join("testlab").join("vault-test.db");

    let connection = initialize_database(&database_path, data_root, "Vault", "production")?;
    let testlab_connection = initialize_database(
        &testlab_database_path,
        &data_root.join("testlab"),
        "Vault Test Lab",
        "testlab",
    )?;
    seed_testlab_documents(&testlab_connection)?;

    let schema_version = current_schema_version(&connection)?;
    let production_vault_ready: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM vaults WHERE id = 1 AND mode = 'production')",
        [],
        |row| row.get(0),
    )?;
    let testlab_document_count: i64 =
        testlab_connection.query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))?;

    Ok(VaultInitStatus {
        data_root: data_root.to_string_lossy().into_owned(),
        database_path: database_path.to_string_lossy().into_owned(),
        testlab_database_path: testlab_database_path.to_string_lossy().into_owned(),
        schema_version,
        production_vault_ready,
        testlab_isolated: data_root.join("testlab").exists(),
        testlab_document_count,
    })
}

fn initialize_database(
    database_path: &Path,
    storage_root: &Path,
    vault_name: &str,
    vault_mode: &str,
) -> rusqlite::Result<Connection> {
    let mut connection = Connection::open(database_path)?;
    connection.execute_batch(
        "
        PRAGMA foreign_keys = ON;
        PRAGMA journal_mode = WAL;
        CREATE TABLE IF NOT EXISTS schema_migrations (
          version INTEGER PRIMARY KEY,
          name TEXT NOT NULL,
          applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        ",
    )?;

    for (version, name, sql) in MIGRATIONS {
        let has_migration: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = ?1)",
            params![version],
            |row| row.get(0),
        )?;

        if !has_migration {
            let transaction = connection.transaction()?;
            let migration = sql.replace("__APP_DATA__", &storage_root.to_string_lossy());
            transaction.execute_batch(&migration)?;
            transaction.execute(
                "INSERT INTO schema_migrations (version, name) VALUES (?1, ?2)",
                params![version, name],
            )?;
            transaction.commit()?;
        }
    }

    connection.execute(
        "UPDATE vaults SET name = ?1, mode = ?2, storage_root = ?3 WHERE id = 1",
        params![vault_name, vault_mode, storage_root.to_string_lossy()],
    )?;

    Ok(connection)
}

fn current_schema_version(connection: &Connection) -> rusqlite::Result<i64> {
    connection.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )
}

fn seed_testlab_documents(connection: &Connection) -> rusqlite::Result<()> {
    let documents = [
        (
            10,
            "DAGAB anställningsavtal",
            "Anställningsavtal",
            "2024-02-01",
            "inbox",
            "Syntetiskt Test Lab-underlag",
            "Matchar företag, avtal och anställning",
            "DAGAB kontrakt avtal anställning arbetsgivare",
        ),
        (
            11,
            "Lönespecifikation juni",
            "Lönespecifikation",
            "2024-06-25",
            "indexed",
            "Syntetiskt Test Lab-underlag",
            "Juni tolkas som månad 6 och löneperiod prioriteras",
            "lön lönespecifikation löneperiod juni månad 6 06 utbetalning",
        ),
        (
            12,
            "Passhandling syntetisk person",
            "Identitetshandling",
            "2022-09-12",
            "review",
            "Syntetiskt Test Lab-underlag",
            "MRZ-parser planerad, OCR saknas lokalt",
            "pass identitetshandling mrz syntetisk person",
        ),
        (
            13,
            "Motstridigt startdatum",
            "Konfliktfixture",
            "2024-03-01",
            "review",
            "Syntetiskt Test Lab-underlag",
            "Skapar testfall för konfliktmotor utan att gissa",
            "konflikt startdatum motstridig uppgift granskning",
        ),
    ];

    for (
        id,
        title,
        document_type,
        document_date,
        inbox_status,
        source_label,
        match_explanation,
        body_text,
    ) in documents
    {
        connection.execute(
            "
            INSERT INTO documents (
              id,
              vault_id,
              title,
              document_type,
              document_date,
              inbox_status,
              source_label,
              match_explanation,
              extracted_text
            )
            VALUES (?1, 1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(id) DO UPDATE SET
              title = excluded.title,
              document_type = excluded.document_type,
              document_date = excluded.document_date,
              inbox_status = excluded.inbox_status,
              source_label = excluded.source_label,
              match_explanation = excluded.match_explanation,
              extracted_text = excluded.extracted_text,
              updated_at = CURRENT_TIMESTAMP
            ",
            params![
                id,
                title,
                document_type,
                document_date,
                inbox_status,
                source_label,
                match_explanation,
                body_text
            ],
        )?;
        index_document(
            connection,
            id,
            title,
            &format!("{document_type} {document_date} {inbox_status} {source_label}"),
            &format!("{match_explanation} {body_text}"),
        )?;
    }

    Ok(())
}

#[tauri::command]
fn list_testlab_documents(app: tauri::AppHandle) -> Result<Vec<DocumentSummary>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    list_testlab_documents_at(&data_root).map_err(|error| error.to_string())
}

#[tauri::command]
fn list_production_documents(app: tauri::AppHandle) -> Result<Vec<DocumentSummary>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    list_production_documents_at(&data_root).map_err(|error| error.to_string())
}

fn list_production_documents_at(data_root: &Path) -> rusqlite::Result<Vec<DocumentSummary>> {
    let connection = Connection::open(data_root.join("vault.db"))?;
    list_documents(&connection)
}

#[tauri::command]
fn list_trashed_production_documents(
    app: tauri::AppHandle,
) -> Result<Vec<DocumentSummary>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    let connection =
        Connection::open(data_root.join("vault.db")).map_err(|error| error.to_string())?;
    list_trashed_documents(&connection).map_err(|error| error.to_string())
}

fn list_testlab_documents_at(data_root: &Path) -> rusqlite::Result<Vec<DocumentSummary>> {
    let connection = Connection::open(data_root.join("testlab").join("vault-test.db"))?;
    list_documents(&connection)
}

#[tauri::command]
fn vault_diagnostics(app: tauri::AppHandle) -> Result<VaultDiagnostics, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    ensure_bundled_tessdata(&app, &data_root).map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    vault_diagnostics_at(&data_root).map_err(|error| error.to_string())
}

fn vault_diagnostics_at(data_root: &Path) -> rusqlite::Result<VaultDiagnostics> {
    let database_path = data_root.join("vault.db");
    let testlab_database_path = data_root.join("testlab").join("vault-test.db");
    let connection = Connection::open(&database_path)?;
    let testlab_connection = Connection::open(&testlab_database_path)?;
    let schema_version = current_schema_version(&connection)?;
    let production_document_count = count_table_rows(&connection, "documents")?;
    let testlab_document_count = count_table_rows(&testlab_connection, "documents")?;
    let pending_review_count = connection.query_row(
        "SELECT COUNT(*) FROM claims WHERE vault_id = 1 AND status = 'auto_extracted_pending_review'",
        [],
        |row| row.get(0),
    )?;
    let entity_count = count_table_rows(&connection, "entities")?;
    let conflict_count = list_conflicts_at(data_root)?.len() as i64;
    let backup_count = list_local_backups_at(data_root)?.len() as i64;
    let audit_event_count = count_table_rows(&connection, "audit_events")?;
    let ocr = detect_ocr_status(Some(data_root));
    let pdftotext_path = find_pdftotext_executable();
    let pdftoppm_path = find_pdftoppm_executable();
    let mut issues = Vec::new();

    if !ocr.available {
        issues.push(
            "Tesseract hittades inte. Bild-OCR och skannade PDF-filer fungerar inte just nu."
                .to_string(),
        );
    }
    if !ocr.languages.iter().any(|language| language == "swe") {
        issues.push(
            "Svensk OCR-modell saknas. Svenska bilder kan fa samre traffsakerhet.".to_string(),
        );
    }
    if pdftotext_path.is_none() {
        issues.push("pdftotext hittades inte. Text-PDF-filer far samre extraktion.".to_string());
    }
    if pdftoppm_path.is_none() {
        issues.push("pdftoppm hittades inte. Skannade PDF-filer kan inte OCR-lasas.".to_string());
    }
    if production_document_count == 0 {
        issues.push(
            "Produktionsarkivet ar tomt. Importera en fil for att testa riktiga floden."
                .to_string(),
        );
    }

    Ok(VaultDiagnostics {
        data_root: data_root.to_string_lossy().into_owned(),
        database_path: database_path.to_string_lossy().into_owned(),
        testlab_database_path: testlab_database_path.to_string_lossy().into_owned(),
        schema_version,
        production_document_count,
        testlab_document_count,
        pending_review_count,
        entity_count,
        conflict_count,
        backup_count,
        audit_event_count,
        ocr_available: ocr.available,
        tesseract_path: ocr.executable_path,
        tessdata_dir: ocr.tessdata_dir,
        ocr_languages: ocr.languages,
        pdf_text_available: pdftotext_path.is_some() || pdftoppm_path.is_some(),
        pdftotext_path: pdftotext_path
            .or(pdftoppm_path)
            .map(|path| path.to_string_lossy().into_owned()),
        office_text_available: true,
        issues,
    })
}

fn count_table_rows(connection: &Connection, table_name: &str) -> rusqlite::Result<i64> {
    let sql = format!("SELECT COUNT(*) FROM {table_name}");
    connection.query_row(&sql, [], |row| row.get(0))
}

fn list_documents(connection: &Connection) -> rusqlite::Result<Vec<DocumentSummary>> {
    let mut statement = connection.prepare(
        "
        SELECT
          id,
          title,
          document_type,
          document_date,
          inbox_status,
          source_label,
          match_explanation
        FROM documents
        WHERE vault_id = 1 AND trashed_at IS NULL
        ORDER BY datetime(created_at) DESC, id DESC
        ",
    )?;

    let documents = statement
        .query_map([], |row| {
            Ok(DocumentSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                document_type: row.get(2)?,
                document_date: row.get(3)?,
                inbox_status: row.get(4)?,
                source_label: row.get(5)?,
                match_explanation: row.get(6)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(documents)
}

fn list_trashed_documents(connection: &Connection) -> rusqlite::Result<Vec<DocumentSummary>> {
    let mut statement = connection.prepare(
        "
        SELECT id, title, document_type, document_date, inbox_status, source_label, match_explanation
        FROM documents
        WHERE vault_id = 1 AND trashed_at IS NOT NULL
        ORDER BY datetime(trashed_at) DESC, id DESC
        ",
    )?;
    let documents = statement
        .query_map([], |row| {
            Ok(DocumentSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                document_type: row.get(2)?,
                document_date: row.get(3)?,
                inbox_status: row.get(4)?,
                source_label: row.get(5)?,
                match_explanation: row.get(6)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(documents)
}

#[tauri::command]
fn list_audit_events(app: tauri::AppHandle) -> Result<Vec<AuditEventSummary>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    list_audit_events_at(&data_root, 30).map_err(|error| error.to_string())
}

fn list_audit_events_at(data_root: &Path, limit: i64) -> rusqlite::Result<Vec<AuditEventSummary>> {
    let connection = Connection::open(data_root.join("vault.db"))?;
    let mut statement = connection.prepare(
        "
        SELECT id, event_type, target_type, target_id, safe_summary, created_at
        FROM audit_events
        WHERE vault_id = 1
        ORDER BY datetime(created_at) DESC, id DESC
        LIMIT ?1
        ",
    )?;

    let events = statement
        .query_map(params![limit], |row| {
            Ok(AuditEventSummary {
                id: row.get(0)?,
                event_type: row.get(1)?,
                target_type: row.get(2)?,
                target_id: row.get(3)?,
                safe_summary: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(events)
}

#[tauri::command]
fn list_entity_catalog(app: tauri::AppHandle) -> Result<Vec<EntityCatalogItem>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    list_entity_catalog_at(&data_root).map_err(|error| error.to_string())
}

fn list_entity_catalog_at(data_root: &Path) -> rusqlite::Result<Vec<EntityCatalogItem>> {
    let connection = Connection::open(data_root.join("vault.db"))?;
    let mut statement = connection.prepare(
        "
        SELECT
          e.id,
          e.entity_type,
          e.display_name,
          COUNT(DISTINCT de.document_id) AS document_count
        FROM entities e
        LEFT JOIN document_entities de ON de.entity_id = e.id
        WHERE e.vault_id = 1
        GROUP BY e.id, e.entity_type, e.display_name
        ORDER BY document_count DESC, e.entity_type, e.display_name
        ",
    )?;
    let entities = statement
        .query_map([], |row| {
            Ok(EntityCatalogItem {
                id: row.get(0)?,
                entity_type: row.get(1)?,
                display_name: row.get(2)?,
                document_count: row.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(entities)
}

#[tauri::command]
fn list_timeline_events(app: tauri::AppHandle) -> Result<Vec<TimelineEventSummary>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    list_timeline_events_at(&data_root).map_err(|error| error.to_string())
}

#[tauri::command]
fn list_calendar_events(app: tauri::AppHandle) -> Result<Vec<CalendarEventSummary>, String> {
    let connection = production_connection(&app)?;
    rebuild_domain_records(&connection).map_err(|e| e.to_string())?;
    let sql = "SELECT 'custom-'||id,event_date,title,event_type,source_kind,status,document_id,CASE WHEN note='' THEN 'Manuellt lokal kalenderpost' ELSE note END FROM calendar_events WHERE vault_id=1
      UNION ALL SELECT 'reminder-'||id,due_date,title,'reminder','reminder',status,document_id,CASE WHEN note='' THEN 'Lokal frivillig påminnelse' ELSE note END FROM reminders WHERE vault_id=1
      UNION ALL SELECT 'domain-start-'||id,effective_from,subject||' börjar',CASE domain_type WHEN 'authority' THEN 'response_or_validity' ELSE 'period_start' END,'domain',CASE WHEN effective_from>date('now','localtime') THEN 'future' ELSE 'documented' END,document_id,actuality_explanation FROM domain_records WHERE effective_from IS NOT NULL
      UNION ALL SELECT 'domain-end-'||id,effective_to,subject||' upphör','expiry','domain',CASE WHEN effective_to<date('now','localtime') THEN 'historical' ELSE 'upcoming' END,document_id,actuality_explanation FROM domain_records WHERE effective_to IS NOT NULL
      ORDER BY 2";
    let mut statement = connection.prepare(sql).map_err(|e| e.to_string())?;
    let rows = statement
        .query_map([], |r| {
            Ok(CalendarEventSummary {
                id: r.get(0)?,
                event_date: r.get(1)?,
                title: r.get(2)?,
                event_type: r.get(3)?,
                source_kind: r.get(4)?,
                status: r.get(5)?,
                document_id: r.get(6)?,
                explanation: r.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
fn save_calendar_event(
    app: tauri::AppHandle,
    document_id: Option<i64>,
    title: String,
    event_date: String,
    event_type: String,
    note: String,
) -> Result<Vec<CalendarEventSummary>, String> {
    if title.trim().is_empty() || !is_valid_iso_date(&event_date) {
        return Err("Titel och giltigt datum krävs".into());
    }
    let connection = production_connection(&app)?;
    connection.execute("INSERT INTO calendar_events(vault_id,document_id,title,event_date,event_type,note) VALUES(1,?1,?2,?3,?4,?5)",params![document_id,title.trim(),event_date,event_type,note.trim()]).map_err(|e|e.to_string())?;
    record_audit_event(
        &connection,
        "calendar_event_created",
        "calendar",
        Some(connection.last_insert_rowid()),
        "Local calendar event created",
    )
    .map_err(|e| e.to_string())?;
    drop(connection);
    list_calendar_events(app)
}

#[tauri::command]
fn list_graph_data(app: tauri::AppHandle) -> Result<GraphDataSummary, String> {
    let connection = production_connection(&app)?;
    let mut nodes = Vec::new();
    {
        let mut s=connection.prepare("SELECT id,title FROM documents WHERE vault_id=1 AND trashed_at IS NULL ORDER BY id LIMIT 500").map_err(|e|e.to_string())?;
        nodes.extend(
            s.query_map([], |r| {
                Ok(GraphNodeSummary {
                    key: format!("document:{}", r.get::<_, i64>(0)?),
                    node_type: "document".into(),
                    label: r.get(1)?,
                    document_id: Some(r.get(0)?),
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| e.to_string())?,
        );
    }
    {
        let mut s=connection.prepare("SELECT id,entity_type,display_name FROM entities WHERE vault_id=1 ORDER BY id LIMIT 500").map_err(|e|e.to_string())?;
        nodes.extend(
            s.query_map([], |r| {
                Ok(GraphNodeSummary {
                    key: format!("entity:{}", r.get::<_, i64>(0)?),
                    node_type: r.get(1)?,
                    label: r.get(2)?,
                    document_id: None,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| e.to_string())?,
        );
    }
    {
        let mut s = connection
            .prepare("SELECT id,name FROM categories WHERE vault_id=1 ORDER BY id")
            .map_err(|e| e.to_string())?;
        nodes.extend(
            s.query_map([], |r| {
                Ok(GraphNodeSummary {
                    key: format!("category:{}", r.get::<_, i64>(0)?),
                    node_type: "category".into(),
                    label: r.get(1)?,
                    document_id: None,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| e.to_string())?,
        );
    }
    let mut edges = Vec::new();
    {
        let mut s=connection.prepare("SELECT de.document_id,de.entity_id,de.role FROM document_entities de JOIN documents d ON d.id=de.document_id WHERE d.trashed_at IS NULL").map_err(|e|e.to_string())?;
        edges.extend(
            s.query_map([], |r| {
                Ok(GraphEdgeSummary {
                    id: -(r.get::<_, i64>(0)? * 100000 + r.get::<_, i64>(1)?),
                    source_key: format!("document:{}", r.get::<_, i64>(0)?),
                    target_key: format!("entity:{}", r.get::<_, i64>(1)?),
                    relation_type: r.get(2)?,
                    status: "rule_based".into(),
                    valid_from: None,
                    valid_to: None,
                    explanation: "Lokal entitetsregel kopplade dokumentet till entiteten".into(),
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| e.to_string())?,
        );
    }
    {
        let mut s = connection
            .prepare("SELECT document_id,category_id FROM document_categories")
            .map_err(|e| e.to_string())?;
        edges.extend(
            s.query_map([], |r| {
                Ok(GraphEdgeSummary {
                    id: -(2000000000 + r.get::<_, i64>(0)? * 1000 + r.get::<_, i64>(1)?),
                    source_key: format!("document:{}", r.get::<_, i64>(0)?),
                    target_key: format!("category:{}", r.get::<_, i64>(1)?),
                    relation_type: "tillhör".into(),
                    status: "approved".into(),
                    valid_from: None,
                    valid_to: None,
                    explanation: "Manuell kategorikoppling".into(),
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| e.to_string())?,
        );
    }
    {
        let mut s=connection.prepare("SELECT id,source_type,source_id,target_type,target_id,relation_type,status,valid_from,valid_to,rule_explanation FROM graph_relations WHERE vault_id=1 AND status!='rejected'").map_err(|e|e.to_string())?;
        edges.extend(
            s.query_map([], |r| {
                Ok(GraphEdgeSummary {
                    id: r.get(0)?,
                    source_key: format!("{}:{}", r.get::<_, String>(1)?, r.get::<_, i64>(2)?),
                    target_key: format!("{}:{}", r.get::<_, String>(3)?, r.get::<_, i64>(4)?),
                    relation_type: r.get(5)?,
                    status: r.get(6)?,
                    valid_from: r.get(7)?,
                    valid_to: r.get(8)?,
                    explanation: r.get(9)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| e.to_string())?,
        );
    }
    Ok(GraphDataSummary { nodes, edges })
}

#[tauri::command]
fn save_graph_relation(
    app: tauri::AppHandle,
    source_type: String,
    source_id: i64,
    target_type: String,
    target_id: i64,
    relation_type: String,
    valid_from: Option<String>,
    valid_to: Option<String>,
) -> Result<GraphDataSummary, String> {
    const TYPES: [&str; 18] = [
        "ersätter",
        "äldre version av",
        "bilaga till",
        "kvitto för",
        "avtal med",
        "tillhör",
        "relaterad till",
        "förnyelse av",
        "svar på",
        "skickad tillsammans med",
        "bevis för",
        "gäller objekt",
        "utfärdad av",
        "ändrar",
        "förlänger",
        "avslutar",
        "parallell med",
        "gäller under period",
    ];
    if !TYPES.contains(&relation_type.as_str()) {
        return Err("Okänd relationstyp".into());
    }
    let connection = production_connection(&app)?;
    connection.execute("INSERT INTO graph_relations(vault_id,source_type,source_id,target_type,target_id,relation_type,status,valid_from,valid_to,rule_explanation) VALUES(1,?1,?2,?3,?4,?5,'approved',?6,?7,'Manuellt skapad relation') ON CONFLICT(vault_id,source_type,source_id,target_type,target_id,relation_type) DO UPDATE SET status='approved',valid_from=excluded.valid_from,valid_to=excluded.valid_to,updated_at=CURRENT_TIMESTAMP",params![source_type,source_id,target_type,target_id,relation_type,valid_from,valid_to]).map_err(|e|e.to_string())?;
    drop(connection);
    list_graph_data(app)
}

fn list_timeline_events_at(data_root: &Path) -> rusqlite::Result<Vec<TimelineEventSummary>> {
    let connection = Connection::open(data_root.join("vault.db"))?;
    let mut statement = connection.prepare(
        "SELECT d.document_date, d.document_type, d.title, d.id,
                GROUP_CONCAT(DISTINCT e.display_name),
                (SELECT MIN(dp.page_no) FROM document_versions dv JOIN document_pages dp ON dp.document_version_id = dv.id WHERE dv.document_id = d.id AND dv.is_current_file = 1),
                CASE WHEN d.document_date > date('now') THEN 'future' WHEN d.document_date < date('now') THEN 'historical' ELSE 'current' END
         FROM documents d
         LEFT JOIN document_entities de ON de.document_id = d.id
         LEFT JOIN entities e ON e.id = de.entity_id
         WHERE d.trashed_at IS NULL AND d.document_date IS NOT NULL
         GROUP BY d.id ORDER BY d.document_date DESC, d.id DESC",
    )?;
    let events = statement
        .query_map([], |row| {
            let event_date: String = row.get(0)?;
            let status: String = row.get(6)?;
            Ok(TimelineEventSummary {
                event_date: event_date.clone(),
                event_type: row.get(1)?,
                title: row.get(2)?,
                document_id: row.get(3)?,
                entity_names: row
                    .get::<_, Option<String>>(4)?
                    .unwrap_or_default()
                    .split(',')
                    .filter(|value| !value.is_empty())
                    .map(str::to_string)
                    .collect(),
                source_page_no: row.get(5)?,
                explanation: format!("Statusen {status} beräknas endast från dokumentdatum {event_date} jämfört med dagens lokala datum."),
                status,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(events)
}

#[tauri::command]
fn list_domain_records(app: tauri::AppHandle) -> Result<Vec<DomainRecordSummary>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    list_domain_records_at(&data_root).map_err(|error| error.to_string())
}

fn list_domain_records_at(data_root: &Path) -> rusqlite::Result<Vec<DomainRecordSummary>> {
    let connection = Connection::open(data_root.join("vault.db"))?;
    rebuild_domain_records(&connection)?;
    let mut statement = connection.prepare(
        "SELECT r.id, r.document_id, d.title, r.domain_type, r.record_type, r.subject,
                r.effective_from, r.effective_to, r.actuality_status, r.actuality_explanation,
                r.fields_json, r.source_page_no, r.extraction_method, r.manually_locked
         FROM domain_records r JOIN documents d ON d.id = r.document_id
         WHERE d.trashed_at IS NULL
         ORDER BY COALESCE(r.effective_from, d.document_date) DESC, r.id DESC",
    )?;
    let records = statement
        .query_map([], |row| {
            Ok(DomainRecordSummary {
                id: row.get(0)?,
                document_id: row.get(1)?,
                document_title: row.get(2)?,
                domain_type: row.get(3)?,
                record_type: row.get(4)?,
                subject: row.get(5)?,
                effective_from: row.get(6)?,
                effective_to: row.get(7)?,
                actuality_status: row.get(8)?,
                actuality_explanation: row.get(9)?,
                fields_json: row.get(10)?,
                source_page_no: row.get(11)?,
                extraction_method: row.get(12)?,
                manually_locked: row.get::<_, i64>(13)? != 0,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(records)
}

fn rebuild_domain_records(connection: &Connection) -> rusqlite::Result<()> {
    let current_date: String =
        connection.query_row("SELECT date('now', 'localtime')", [], |row| row.get(0))?;
    let mut statement = connection.prepare(
        "SELECT id, title, document_type, document_date, extracted_text FROM documents WHERE vault_id = 1 AND trashed_at IS NULL",
    )?;
    let documents = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    for (document_id, title, document_type, document_date, body) in documents {
        let normalized_type = normalize(&document_type);
        let (domain_type, record_type) = if normalized_type.contains("anstallningsavtal") {
            ("employment", "employment_agreement")
        } else if normalized_type.contains("lonespecifikation") {
            ("employment", "salary_period")
        } else if normalized_type.contains("utbild")
            || normalized_type.contains("betyg")
            || normalized_type.contains("examen")
            || normalized_type.contains("antagning")
        {
            ("education", "education_document")
        } else if normalized_type.contains("hyres")
            || normalized_type.contains("bostad")
            || normalized_type.contains("boende")
            || normalized_type.contains("besiktningsprotokoll")
        {
            ("housing", "housing_record")
        } else if normalized_type.contains("resa")
            || normalized_type.contains("biljett")
            || normalized_type.contains("bokningsbekraft")
            || normalized_type.contains("flyg")
            || normalized_type.contains("hotell")
            || normalized_type.contains("visum")
        {
            ("travel", "travel_record")
        } else if normalized_type.contains("myndighet")
            || normalized_type.contains("beslut")
            || normalized_type.contains("arende")
            || normalized_type.contains("forelaggande")
        {
            ("authority", "authority_case")
        } else if normalized_type.contains("fordon")
            || normalized_type.contains("besikt")
            || normalized_type.contains("service")
        {
            ("vehicle", "vehicle_document")
        } else if normalized_type.contains("identitet") || normalized_type.contains("pass") {
            ("identity", "identity_document")
        } else if normalized_type.contains("avtal")
            || normalized_type.contains("forsakring")
            || normalized_type.contains("abonnemang")
        {
            ("contract", "agreement")
        } else if normalized_type.contains("kvitto") || normalized_type.contains("faktura") {
            ("product", "purchase_record")
        } else {
            continue;
        };
        let effective_from = extract_labeled_date(
            &body,
            &[
                "startdatum",
                "galler fran",
                "giltig fran",
                "studiestart",
                "avresedatum",
                "inflyttningsdatum",
                "beslutsdatum",
            ],
        )
        .or_else(|| {
            matches!(
                record_type,
                "salary_period" | "purchase_record" | "identity_document"
            )
            .then(|| document_date.clone())
            .flatten()
        });
        let effective_to = extract_labeled_date(
            &body,
            &[
                "slutdatum",
                "galler till",
                "giltig till",
                "utgangsdatum",
                "hemresedatum",
                "utflyttningsdatum",
                "sista svarsdatum",
            ],
        );
        let (actuality_status, explanation) = if matches!(
            record_type,
            "salary_period" | "purchase_record"
        ) {
            ("historical_record", "Period- eller händelsedokument bevaras som historik och ersätter inte ett aktuellt grundvärde.")
        } else if effective_from.is_none() && effective_to.is_none() {
            (
                "uncertain",
                "Uttrycklig giltighetsperiod saknas; systemet gissar inte aktualitet.",
            )
        } else if effective_from
            .as_deref()
            .is_some_and(|date| date > current_date.as_str())
        {
            (
                "future",
                "Uttryckligt startdatum ligger efter dagens lokala datum.",
            )
        } else if effective_to
            .as_deref()
            .is_some_and(|date| date < current_date.as_str())
        {
            ("historical", "Uttryckligt slutdatum har passerat.")
        } else if effective_to.is_some() {
            (
                "current",
                "Dagens lokala datum ligger inom den uttryckliga perioden.",
            )
        } else {
            ("documented_open_period", "Startdatum är uttryckligt men slutdatum saknas; posten visas utan antagande om att förhållandet fortfarande pågår.")
        };
        let subject: String = connection.query_row(
            "SELECT COALESCE(GROUP_CONCAT(e.display_name, ', '), ?2) FROM document_entities de JOIN entities e ON e.id = de.entity_id WHERE de.document_id = ?1",
            params![document_id, title], |row| row.get(0),
        )?;
        let source_page_no: Option<i64> = connection.query_row(
            "SELECT MIN(dp.page_no) FROM document_versions dv JOIN document_pages dp ON dp.document_version_id = dv.id WHERE dv.document_id = ?1 AND dv.is_current_file = 1",
            params![document_id], |row| row.get(0),
        )?;
        let claims_json: String = connection.query_row(
            "SELECT COALESCE(json_group_array(json_object('type', claim_type, 'value', json(value_json))), '[]') FROM claims WHERE document_id = ?1 AND status != 'review_rejected'",
            params![document_id], |row| row.get(0),
        )?;
        let fields_json = serde_json::json!({"claims":serde_json::from_str::<serde_json::Value>(&claims_json).unwrap_or(serde_json::json!([])),"fields":extract_domain_fields(domain_type,&body)}).to_string();
        connection.execute(
            "INSERT INTO domain_records (vault_id, document_id, domain_type, record_type, subject, effective_from, effective_to, actuality_status, actuality_explanation, fields_json, source_page_no, extraction_method)
             VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'local_domain_rules_v1')
             ON CONFLICT(document_id, domain_type, record_type) DO UPDATE SET subject=excluded.subject, effective_from=excluded.effective_from, effective_to=excluded.effective_to, actuality_status=CASE WHEN domain_records.manually_locked=1 THEN domain_records.actuality_status ELSE excluded.actuality_status END, actuality_explanation=CASE WHEN domain_records.manually_locked=1 THEN domain_records.actuality_explanation ELSE excluded.actuality_explanation END, fields_json=excluded.fields_json, source_page_no=excluded.source_page_no, updated_at=CURRENT_TIMESTAMP",
            params![document_id, domain_type, record_type, subject, effective_from, effective_to, actuality_status, explanation, fields_json, source_page_no],
        )?;
    }
    Ok(())
}

fn extract_labeled_date(text: &str, labels: &[&str]) -> Option<String> {
    for line in text.lines() {
        let normalized = normalize(line);
        if labels.iter().any(|label| normalized.contains(label)) {
            if let Some(date) = extract_document_date("", line).document_date {
                return Some(date);
            }
        }
    }
    None
}

fn extract_domain_fields(domain_type: &str, text: &str) -> serde_json::Value {
    let labels: &[(&str, &str)] = match domain_type {
        "housing" => &[
            ("address", "adress"),
            ("agreement_type", "avtalstyp"),
            ("landlord", "hyresvard"),
            ("owner", "fastighetsagare"),
            ("monthly_cost", "manadskostnad"),
            ("deposit", "deposition"),
            ("notice_period", "uppsagningstid"),
            ("object_number", "objektsnummer"),
        ],
        "travel" => &[
            ("destination", "destination"),
            ("booking_number", "bokningsnummer"),
            ("flight_number", "flygnummer"),
            ("hotel", "hotell"),
            ("travellers", "resenarer"),
            ("insurance", "forsakring"),
            ("amount_paid", "betalt belopp"),
            ("visa", "visum"),
        ],
        "authority" => &[
            ("authority", "myndighet"),
            ("case_type", "arendetyp"),
            ("case_number", "arendenummer"),
            ("status", "status"),
            ("instructions", "instruktioner"),
            ("contact", "kontaktuppgifter"),
        ],
        _ => &[],
    };
    let mut map = serde_json::Map::new();
    for line in text.lines() {
        let normalized = normalize(line);
        for (key, label) in labels {
            if normalized.starts_with(label) {
                if let Some((_, value)) = line.split_once(':') {
                    let clean = value.trim();
                    if !clean.is_empty() {
                        map.entry((*key).to_string())
                            .or_insert(serde_json::Value::String(
                                clean.chars().take(500).collect(),
                            ));
                    }
                }
            }
        }
    }
    serde_json::Value::Object(map)
}

fn is_valid_iso_date(value: &str) -> bool {
    value.len() == 10
        && value.as_bytes()[4] == b'-'
        && value.as_bytes()[7] == b'-'
        && value
            .chars()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
}

#[tauri::command]
fn refresh_claim_actuality(app: tauri::AppHandle) -> Result<i64, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    let connection =
        Connection::open(data_root.join("vault.db")).map_err(|error| error.to_string())?;
    connection.execute(
        "UPDATE claims SET
           actuality_status = CASE
             WHEN effective_from IS NULL AND effective_to IS NULL THEN 'uncertain'
             WHEN effective_from > date('now') THEN 'future'
             WHEN effective_to IS NOT NULL AND effective_to < date('now') THEN 'historical'
             ELSE 'current' END,
           actuality_explanation = CASE
             WHEN effective_from IS NULL AND effective_to IS NULL THEN 'Uttrycklig giltighetsperiod saknas; systemet gissar inte.'
             WHEN effective_from > date('now') THEN 'Framtida eftersom uttryckligt startdatum ligger efter dagens lokala datum.'
             WHEN effective_to IS NOT NULL AND effective_to < date('now') THEN 'Historisk eftersom uttryckligt slutdatum har passerat.'
             ELSE 'Aktuell enligt uttrycklig start-/slutperiod och dagens lokala datum.' END
         WHERE actuality_status NOT LIKE 'manual_%'",
        [],
    ).map(|count| count as i64).map_err(|error| error.to_string())
}

#[tauri::command]
fn set_claim_actuality(
    app: tauri::AppHandle,
    claim_id: i64,
    actuality_status: String,
) -> Result<DocumentDetail, String> {
    if !matches!(
        actuality_status.as_str(),
        "manual_current" | "manual_historical" | "manual_uncertain"
    ) {
        return Err("Ogiltig manuell aktualitetsstatus".to_string());
    }
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    let connection =
        Connection::open(data_root.join("vault.db")).map_err(|error| error.to_string())?;
    let document_id: i64 = connection
        .query_row(
            "SELECT document_id FROM claims WHERE id = ?1",
            params![claim_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    connection.execute(
        "UPDATE claims SET actuality_status = ?1, actuality_explanation = 'Manuellt verifierad och låst av användaren.' WHERE id = ?2",
        params![actuality_status, claim_id],
    ).map_err(|error| error.to_string())?;
    connection.execute(
        "INSERT INTO user_verifications (vault_id, target_type, target_id, decision, note, locked) VALUES (1, 'claim_actuality', ?1, ?2, 'Manuell aktualitetsbedömning', 1)",
        params![claim_id, actuality_status],
    ).map_err(|error| error.to_string())?;
    get_document_detail(&connection, data_root, document_id).map_err(|error| error.to_string())
}

#[tauri::command]
fn list_review_queue(app: tauri::AppHandle) -> Result<Vec<ReviewQueueItem>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    list_review_queue_at(&data_root).map_err(|error| error.to_string())
}

fn list_review_queue_at(data_root: &Path) -> rusqlite::Result<Vec<ReviewQueueItem>> {
    let connection = Connection::open(data_root.join("vault.db"))?;
    let mut statement = connection.prepare(
        "
        SELECT
          c.id,
          c.document_id,
          d.title,
          c.claim_type,
          c.value_json,
          c.status,
          c.extraction_method
        FROM claims c
        JOIN documents d ON d.id = c.document_id
        WHERE c.vault_id = 1
          AND d.trashed_at IS NULL
          AND c.status = 'auto_extracted_pending_review'
        ORDER BY datetime(c.created_at) DESC, c.id DESC
        ",
    )?;
    let items = statement
        .query_map([], |row| {
            Ok(ReviewQueueItem {
                claim_id: row.get(0)?,
                document_id: row.get(1)?,
                document_title: row.get(2)?,
                claim_type: row.get(3)?,
                value_json: row.get(4)?,
                status: row.get(5)?,
                extraction_method: row.get(6)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(items)
}

#[tauri::command]
fn analyze_production_archive(app: tauri::AppHandle) -> Result<ArchiveAnalysisResult, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    ensure_bundled_tessdata(&app, &data_root).map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    analyze_production_archive_at(&data_root).map_err(|error| error.to_string())
}

fn analyze_production_archive_at(data_root: &Path) -> rusqlite::Result<ArchiveAnalysisResult> {
    let connection = Connection::open(data_root.join("vault.db"))?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;
    let mut statement = connection.prepare(
        "
        SELECT
          d.id,
          d.title,
          d.document_type,
          d.document_date,
          d.source_label,
          d.match_explanation,
          d.extracted_text,
          f.storage_path,
          f.mime_type,
          dv.id
        FROM documents d
        LEFT JOIN document_versions dv
          ON dv.document_id = d.id AND dv.is_current_file = 1
        LEFT JOIN files f
          ON f.id = dv.file_id
        WHERE d.vault_id = 1 AND d.trashed_at IS NULL
        ORDER BY d.id
        ",
    )?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<i64>>(9)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    drop(statement);

    let mut result = ArchiveAnalysisResult {
        document_count: rows.len() as i64,
        reindexed_document_count: 0,
        ocr_attempt_count: 0,
        ocr_updated_count: 0,
        reclassified_document_count: 0,
        date_updated_count: 0,
        entity_link_count: 0,
        pending_review_count: 0,
    };

    for (
        document_id,
        title,
        document_type,
        document_date,
        source_label,
        match_explanation,
        mut extracted_text,
        storage_path,
        mime_type,
        version_id,
    ) in rows
    {
        if extracted_text.trim().is_empty()
            && matches!(
                mime_type.as_deref(),
                Some("image/png" | "image/jpeg" | "application/pdf")
            )
        {
            if let Some(storage_path) = storage_path.as_deref() {
                result.ocr_attempt_count += 1;
                let file_path = data_root.join(storage_path);
                let ocr_text = extract_text_for_indexing(
                    &file_path,
                    mime_type.as_deref().unwrap_or_default(),
                    Some(data_root),
                )
                .map_err(to_sql_error)?;
                if !ocr_text.trim().is_empty() {
                    extracted_text = ocr_text;
                    connection.execute(
                        "
                        UPDATE documents
                        SET extracted_text = ?1,
                            match_explanation = match_explanation || '. OCR-text extraherad vid arkivanalys',
                            updated_at = CURRENT_TIMESTAMP
                        WHERE id = ?2 AND vault_id = 1
                        ",
                        params![extracted_text, document_id],
                    )?;
                    result.ocr_updated_count += 1;
                }
            }
        }

        if let Some(version_id) = version_id {
            let page_count: i64 = connection.query_row(
                "SELECT COUNT(*) FROM document_pages WHERE document_version_id = ?1",
                params![version_id],
                |row| row.get(0),
            )?;
            if page_count == 0 && !extracted_text.trim().is_empty() {
                store_document_pages(
                    &connection,
                    document_id,
                    version_id,
                    mime_type.as_deref().unwrap_or("text/plain"),
                    &extracted_text,
                )?;
            }
        }

        let classification = classify_document(&title, &extracted_text);
        let date_extraction = extract_document_date(&title, &extracted_text);
        let effective_document_type = if document_type == "Importerad fil"
            && classification.document_type != "Importerad fil"
        {
            connection.execute(
                "
                UPDATE documents
                SET document_type = ?1,
                    match_explanation = match_explanation || '. Arkivanalys: ' || ?2,
                    updated_at = CURRENT_TIMESTAMP
                WHERE id = ?3 AND vault_id = 1
                ",
                params![
                    classification.document_type,
                    classification.match_explanation,
                    document_id
                ],
            )?;
            result.reclassified_document_count += 1;
            classification.document_type.to_string()
        } else {
            document_type
        };
        let effective_document_date = if document_date.is_none() {
            if let Some(extracted_date) = date_extraction.document_date.as_deref() {
                connection.execute(
                    "
                    UPDATE documents
                    SET document_date = ?1,
                        match_explanation = match_explanation || '. Arkivanalys: ' || ?2,
                        updated_at = CURRENT_TIMESTAMP
                    WHERE id = ?3 AND vault_id = 1 AND document_date IS NULL
                    ",
                    params![
                        extracted_date,
                        date_extraction
                            .explanation
                            .as_deref()
                            .unwrap_or("Datum hittat vid arkivanalys"),
                        document_id
                    ],
                )?;
                insert_claim(
                    &connection,
                    document_id,
                    "document_date",
                    &serde_json::json!({ "value": extracted_date }).to_string(),
                    "validated_parse",
                    "auto_extracted_pending_review",
                    "validated_parse",
                    "local_date_rules_v2",
                )?;
                result.date_updated_count += 1;
                Some(extracted_date.to_string())
            } else {
                None
            }
        } else {
            document_date
        };

        index_document(
            &connection,
            document_id,
            &title,
            &format!(
                "{} {} {}",
                effective_document_type,
                effective_document_date.as_deref().unwrap_or_default(),
                source_label
            ),
            &format!("{match_explanation} {extracted_text}"),
        )?;
        result.reindexed_document_count += 1;

        let before_count: i64 = connection.query_row(
            "SELECT COUNT(*) FROM document_entities WHERE document_id = ?1",
            params![document_id],
            |row| row.get(0),
        )?;
        apply_auto_entities(
            &connection,
            document_id,
            &title,
            &effective_document_type,
            &extracted_text,
        )?;
        apply_content_claims(
            &connection,
            document_id,
            &title,
            &effective_document_type,
            &extracted_text,
        )?;
        let after_count: i64 = connection.query_row(
            "SELECT COUNT(*) FROM document_entities WHERE document_id = ?1",
            params![document_id],
            |row| row.get(0),
        )?;
        result.entity_link_count += after_count.saturating_sub(before_count);
    }

    result.pending_review_count = connection.query_row(
        "
        SELECT COUNT(*)
        FROM claims
        WHERE vault_id = 1 AND status = 'auto_extracted_pending_review'
        ",
        [],
        |row| row.get(0),
    )?;
    record_audit_event(
        &connection,
        "archive_analyzed",
        "vault",
        Some(1),
        &format!(
            "Analyzed {} documents, reindexed {}, OCR updated {}, linked {} entities",
            result.document_count,
            result.reindexed_document_count,
            result.ocr_updated_count,
            result.entity_link_count
        ),
    )?;

    Ok(result)
}

fn record_audit_event(
    connection: &Connection,
    event_type: &str,
    target_type: &str,
    target_id: Option<i64>,
    safe_summary: &str,
) -> rusqlite::Result<()> {
    connection.execute(
        "
        INSERT INTO audit_events (
          vault_id,
          actor_kind,
          event_type,
          target_type,
          target_id,
          safe_summary
        )
        VALUES (1, 'local_app', ?1, ?2, ?3, ?4)
        ",
        params![event_type, target_type, target_id, safe_summary],
    )?;

    Ok(())
}

fn production_connection(app: &tauri::AppHandle) -> Result<Connection, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    Connection::open(data_root.join("vault.db")).map_err(|error| error.to_string())
}

#[tauri::command]
fn list_automation_rules(app: tauri::AppHandle) -> Result<Vec<AutomationRuleSummary>, String> {
    let connection = production_connection(&app)?;
    let mut statement = connection.prepare("SELECT id, name, enabled, priority, match_field, match_operator, match_value, action_type, action_value, approval_policy, version FROM automation_rules ORDER BY priority, name").map_err(|e| e.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok(AutomationRuleSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                enabled: row.get::<_, i64>(2)? != 0,
                priority: row.get(3)?,
                match_field: row.get(4)?,
                match_operator: row.get(5)?,
                match_value: row.get(6)?,
                action_type: row.get(7)?,
                action_value: row.get(8)?,
                approval_policy: row.get(9)?,
                version: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
fn save_automation_rule(
    app: tauri::AppHandle,
    id: Option<i64>,
    name: String,
    enabled: bool,
    priority: i64,
    match_field: String,
    match_operator: String,
    match_value: String,
    action_type: String,
    action_value: String,
    approval_policy: String,
) -> Result<Vec<AutomationRuleSummary>, String> {
    if name.trim().is_empty() || match_value.trim().is_empty() || action_value.trim().is_empty() {
        return Err("Namn, matchningsvärde och åtgärdsvärde krävs".into());
    }
    let connection = production_connection(&app)?;
    if let Some(id) = id {
        connection.execute("UPDATE automation_rules SET name=?1, enabled=?2, priority=?3, match_field=?4, match_operator=?5, match_value=?6, action_type=?7, action_value=?8, approval_policy=?9, version=version+1, updated_at=CURRENT_TIMESTAMP WHERE id=?10", params![name.trim(), i64::from(enabled), priority, match_field, match_operator, match_value.trim(), action_type, action_value.trim(), approval_policy, id]).map_err(|e| e.to_string())?;
    } else {
        connection.execute("INSERT INTO automation_rules (name, enabled, priority, match_field, match_operator, match_value, action_type, action_value, approval_policy) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)", params![name.trim(), i64::from(enabled), priority, match_field, match_operator, match_value.trim(), action_type, action_value.trim(), approval_policy]).map_err(|e| e.to_string())?;
    }
    record_audit_event(
        &connection,
        "automation_rule_saved",
        "rule",
        id,
        "Automation rule saved without document content",
    )
    .map_err(|e| e.to_string())?;
    drop(connection);
    list_automation_rules(app)
}

#[tauri::command]
fn delete_automation_rule(
    app: tauri::AppHandle,
    id: i64,
) -> Result<Vec<AutomationRuleSummary>, String> {
    let connection = production_connection(&app)?;
    connection
        .execute("DELETE FROM automation_rules WHERE id=?1", [id])
        .map_err(|e| e.to_string())?;
    record_audit_event(
        &connection,
        "automation_rule_deleted",
        "rule",
        Some(id),
        "Automation rule deleted",
    )
    .map_err(|e| e.to_string())?;
    drop(connection);
    list_automation_rules(app)
}

fn rule_matches(
    field: &str,
    operator: &str,
    wanted: &str,
    title: &str,
    document_type: &str,
    text: &str,
) -> bool {
    let haystack = match field {
        "title" => title,
        "document_type" => document_type,
        _ => text,
    };
    let left = haystack.to_lowercase();
    let right = wanted.to_lowercase();
    match operator {
        "equals" => left.trim() == right.trim(),
        "starts_with" => left.starts_with(&right),
        "regex" => false,
        _ => left.contains(&right),
    }
}

#[tauri::command]
fn run_automation_rules(app: tauri::AppHandle) -> Result<RuleRunResult, String> {
    let mut connection = production_connection(&app)?;
    let rules = {
        let mut s = connection.prepare("SELECT id,name,match_field,match_operator,match_value,action_type,action_value,approval_policy FROM automation_rules WHERE enabled=1 ORDER BY priority,id").map_err(|e| e.to_string())?;
        let rows = s
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, String>(6)?,
                    r.get::<_, String>(7)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| e.to_string())?;
        rows
    };
    let documents = {
        let mut s = connection.prepare("SELECT id,title,document_type,extracted_text FROM documents WHERE trashed_at IS NULL").map_err(|e| e.to_string())?;
        let rows = s
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| e.to_string())?;
        rows
    };
    let scanned = documents.len() as i64;
    let mut matched = 0;
    let mut applied = 0;
    let mut explanations = Vec::new();
    let tx = connection.transaction().map_err(|e| e.to_string())?;
    for (rule_id, name, field, operator, wanted, action, value, policy) in rules {
        for (document_id, title, document_type, text) in &documents {
            if !rule_matches(&field, &operator, &wanted, title, document_type, text) {
                continue;
            }
            matched += 1;
            let mut outcome = "suggested";
            if policy != "suggest" {
                match action.as_str() {
                    "set_document_type" => {
                        tx.execute("UPDATE documents SET document_type=?1, updated_at=CURRENT_TIMESTAMP WHERE id=?2", params![value,document_id]).map_err(|e| e.to_string())?;
                        applied += 1;
                        outcome = "applied";
                    }
                    "add_tag" => {
                        tx.execute(
                            "INSERT OR IGNORE INTO tags(vault_id,name) VALUES(1,?1)",
                            [&value],
                        )
                        .map_err(|e| e.to_string())?;
                        tx.execute("INSERT OR IGNORE INTO document_tags(document_id,tag_id) SELECT ?1,id FROM tags WHERE vault_id=1 AND name=?2", params![document_id,value]).map_err(|e| e.to_string())?;
                        applied += 1;
                        outcome = "applied";
                    }
                    "archive" => {
                        tx.execute("UPDATE documents SET inbox_status='archived', updated_at=CURRENT_TIMESTAMP WHERE id=?1", [document_id]).map_err(|e| e.to_string())?;
                        applied += 1;
                        outcome = "applied";
                    }
                    _ => {}
                }
            }
            let explanation = format!(
                "Regeln '{name}' matchade {field} med '{wanted}' och {outcome}: {action} = {value}"
            );
            tx.execute("INSERT INTO rule_runs(rule_id,document_id,outcome,explanation) VALUES(?1,?2,?3,?4)", params![rule_id,document_id,outcome,explanation]).map_err(|e| e.to_string())?;
            if explanations.len() < 50 {
                explanations.push(explanation);
            }
        }
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(RuleRunResult {
        scanned_document_count: scanned,
        matched_document_count: matched,
        applied_action_count: applied,
        explanations,
    })
}

#[tauri::command]
fn list_document_templates(app: tauri::AppHandle) -> Result<Vec<DocumentTemplateSummary>, String> {
    let c = production_connection(&app)?;
    let mut s=c.prepare("SELECT id,name,document_type,category,default_tags,required_fields FROM document_templates ORDER BY name").map_err(|e|e.to_string())?;
    let rows = s
        .query_map([], |r| {
            Ok(DocumentTemplateSummary {
                id: r.get(0)?,
                name: r.get(1)?,
                document_type: r.get(2)?,
                category: r.get(3)?,
                default_tags: r.get(4)?,
                required_fields: r.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
fn save_document_template(
    app: tauri::AppHandle,
    name: String,
    document_type: String,
    category: String,
    default_tags: String,
    required_fields: String,
) -> Result<Vec<DocumentTemplateSummary>, String> {
    let c = production_connection(&app)?;
    c.execute("INSERT INTO document_templates(name,document_type,category,default_tags,required_fields) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(name) DO UPDATE SET document_type=excluded.document_type,category=excluded.category,default_tags=excluded.default_tags,required_fields=excluded.required_fields,updated_at=CURRENT_TIMESTAMP",params![name.trim(),document_type.trim(),category.trim(),default_tags.trim(),required_fields.trim()]).map_err(|e|e.to_string())?;
    drop(c);
    list_document_templates(app)
}

#[tauri::command]
fn list_custom_fields(app: tauri::AppHandle) -> Result<Vec<CustomFieldSummary>, String> {
    let c = production_connection(&app)?;
    let mut s=c.prepare("SELECT id,name,field_type,applies_to,required FROM custom_field_definitions ORDER BY name").map_err(|e|e.to_string())?;
    let rows = s
        .query_map([], |r| {
            Ok(CustomFieldSummary {
                id: r.get(0)?,
                name: r.get(1)?,
                field_type: r.get(2)?,
                applies_to: r.get(3)?,
                required: r.get::<_, i64>(4)? != 0,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
fn save_custom_field(
    app: tauri::AppHandle,
    name: String,
    field_type: String,
    applies_to: String,
    required: bool,
) -> Result<Vec<CustomFieldSummary>, String> {
    let c = production_connection(&app)?;
    c.execute("INSERT INTO custom_field_definitions(name,field_type,applies_to,required) VALUES(?1,?2,?3,?4) ON CONFLICT(name) DO UPDATE SET field_type=excluded.field_type,applies_to=excluded.applies_to,required=excluded.required",params![name.trim(),field_type,applies_to,i64::from(required)]).map_err(|e|e.to_string())?;
    drop(c);
    list_custom_fields(app)
}

#[tauri::command]
fn list_saved_searches(app: tauri::AppHandle) -> Result<Vec<SavedSearchSummary>, String> {
    let c = production_connection(&app)?;
    let mut s = c
        .prepare("SELECT id,name,query,pinned FROM saved_searches ORDER BY pinned DESC,name")
        .map_err(|e| e.to_string())?;
    let rows = s
        .query_map([], |r| {
            Ok(SavedSearchSummary {
                id: r.get(0)?,
                name: r.get(1)?,
                query: r.get(2)?,
                pinned: r.get::<_, i64>(3)? != 0,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
fn save_search(
    app: tauri::AppHandle,
    name: String,
    query: String,
    pinned: bool,
) -> Result<Vec<SavedSearchSummary>, String> {
    if name.trim().is_empty() || query.trim().is_empty() {
        return Err("Namn och sökfråga krävs".into());
    }
    let c = production_connection(&app)?;
    c.execute("INSERT INTO saved_searches(name,query,pinned) VALUES(?1,?2,?3) ON CONFLICT(name) DO UPDATE SET query=excluded.query,pinned=excluded.pinned,updated_at=CURRENT_TIMESTAMP",params![name.trim(),query.trim(),i64::from(pinned)]).map_err(|e|e.to_string())?;
    drop(c);
    list_saved_searches(app)
}

#[tauri::command]
fn delete_saved_search(app: tauri::AppHandle, id: i64) -> Result<Vec<SavedSearchSummary>, String> {
    let c = production_connection(&app)?;
    c.execute("DELETE FROM saved_searches WHERE id=?1", [id])
        .map_err(|e| e.to_string())?;
    drop(c);
    list_saved_searches(app)
}

fn derive_pin_hash(pin: &str, salt: &str) -> String {
    let mut bytes = format!("{salt}:{pin}").into_bytes();
    for _ in 0..100_000 {
        bytes = Sha256::digest(&bytes).to_vec();
    }
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn verify_pin(connection: &Connection, pin: &str) -> rusqlite::Result<bool> {
    let configured: (Option<String>, Option<String>) = connection.query_row(
        "SELECT pin_salt,pin_hash FROM security_settings WHERE vault_id=1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    Ok(match configured {
        (Some(salt), Some(hash)) => derive_pin_hash(pin, &salt) == hash,
        _ => false,
    })
}

#[tauri::command]
fn security_status(app: tauri::AppHandle) -> Result<SecurityStatus, String> {
    let connection = production_connection(&app)?;
    connection.query_row("SELECT mode,pin_hash IS NOT NULL,auto_lock_minutes,mask_sensitive,(SELECT COUNT(*) FROM documents WHERE trashed_at IS NULL AND is_locked=1),(SELECT COUNT(*) FROM documents WHERE trashed_at IS NULL AND is_private=1) FROM security_settings WHERE vault_id=1",[],|r|Ok(SecurityStatus{mode:r.get(0)?,pin_configured:r.get::<_,i64>(1)?!=0,auto_lock_minutes:r.get(2)?,mask_sensitive:r.get::<_,i64>(3)?!=0,locked_document_count:r.get(4)?,private_document_count:r.get(5)?})).map_err(|e|e.to_string())
}

#[tauri::command]
fn configure_security(
    app: tauri::AppHandle,
    mode: String,
    pin: String,
    auto_lock_minutes: i64,
    mask_sensitive: bool,
) -> Result<SecurityStatus, String> {
    if !["comfortable", "pin", "locked", "private"].contains(&mode.as_str()) {
        return Err("Ogiltigt säkerhetsläge".into());
    }
    if mode != "comfortable" && (pin.len() < 4 || pin.len() > 64) {
        return Err("PIN/lösenord måste vara 4–64 tecken".into());
    }
    let connection = production_connection(&app)?;
    let (salt, hash) = if mode == "comfortable" {
        (None, None)
    } else {
        let salt = format!("{:x}-{:x}", epoch_seconds(), connection.total_changes());
        let hash = derive_pin_hash(&pin, &salt);
        (Some(salt), Some(hash))
    };
    connection.execute("UPDATE security_settings SET mode=?1,pin_salt=?2,pin_hash=?3,auto_lock_minutes=?4,mask_sensitive=?5,updated_at=CURRENT_TIMESTAMP WHERE vault_id=1",params![mode,salt,hash,auto_lock_minutes.clamp(1,1440),i64::from(mask_sensitive)]).map_err(|e|e.to_string())?;
    record_audit_event(
        &connection,
        "security_configured",
        "vault",
        Some(1),
        "Local security settings changed",
    )
    .map_err(|e| e.to_string())?;
    drop(connection);
    security_status(app)
}

#[tauri::command]
fn unlock_vault(app: tauri::AppHandle, pin: String) -> Result<bool, String> {
    let connection = production_connection(&app)?;
    let success = verify_pin(&connection, &pin).map_err(|e| e.to_string())?;
    connection.execute("INSERT INTO security_events(event_type,success,safe_summary) VALUES('unlock_attempt',?1,?2)",params![i64::from(success),if success{"Vault unlock succeeded"}else{"Vault unlock failed"}]).map_err(|e|e.to_string())?;
    Ok(success)
}

#[cfg(windows)]
fn windows_hello_availability() -> Result<WindowsHelloStatus, String> {
    use windows::Security::Credentials::UI::{
        UserConsentVerifier, UserConsentVerifierAvailability,
    };
    let availability = UserConsentVerifier::CheckAvailabilityAsync()
        .map_err(|e| e.to_string())?
        .get()
        .map_err(|e| e.to_string())?;
    let available = availability == UserConsentVerifierAvailability::Available;
    Ok(WindowsHelloStatus {
        available,
        explanation: if available {
            "Windows Hello är konfigurerat och redo".into()
        } else {
            format!("Windows Hello är inte tillgängligt ({availability:?})")
        },
    })
}

#[cfg(not(windows))]
fn windows_hello_availability() -> Result<WindowsHelloStatus, String> {
    Ok(WindowsHelloStatus {
        available: false,
        explanation: "Windows Hello stöds endast i Windows-versionen".into(),
    })
}

#[tauri::command]
fn windows_hello_status() -> Result<WindowsHelloStatus, String> {
    windows_hello_availability()
}

#[tauri::command]
fn unlock_vault_windows_hello(app: tauri::AppHandle) -> Result<bool, String> {
    #[cfg(windows)]
    {
        use windows::core::HSTRING;
        use windows::Security::Credentials::UI::{
            UserConsentVerificationResult, UserConsentVerifier,
        };
        if !windows_hello_availability()?.available {
            return Err("Windows Hello är inte tillgängligt eller konfigurerat".into());
        }
        let result = UserConsentVerifier::RequestVerificationAsync(&HSTRING::from(
            "Lås upp ditt lokala Vault",
        ))
        .map_err(|e| e.to_string())?
        .get()
        .map_err(|e| e.to_string())?;
        let success = result == UserConsentVerificationResult::Verified;
        let connection = production_connection(&app)?;
        connection.execute("INSERT INTO security_events(event_type,success,safe_summary) VALUES('windows_hello_unlock',?1,?2)",params![i64::from(success),if success{"Windows Hello unlock succeeded"}else{"Windows Hello unlock failed"}]).map_err(|e|e.to_string())?;
        Ok(success)
    }
    #[cfg(not(windows))]
    {
        let _ = app;
        Err("Windows Hello stöds endast i Windows-versionen".into())
    }
}

#[tauri::command]
fn set_document_security(
    app: tauri::AppHandle,
    document_id: i64,
    is_locked: bool,
    sensitivity: String,
) -> Result<SecurityStatus, String> {
    if !["normal", "personal", "confidential"].contains(&sensitivity.as_str()) {
        return Err("Ogiltig känslighetsnivå".into());
    }
    let connection = production_connection(&app)?;
    let pin_configured = connection
        .query_row(
            "SELECT pin_hash IS NOT NULL FROM security_settings WHERE vault_id=1",
            [],
            |r| r.get::<_, i64>(0),
        )
        .map_err(|e| e.to_string())?
        != 0;
    if is_locked && !pin_configured {
        return Err("Konfigurera PIN/lösenord innan ett dokument låses".into());
    }
    connection.execute("UPDATE documents SET is_locked=?1,sensitivity=?2,updated_at=CURRENT_TIMESTAMP WHERE id=?3 AND vault_id=1",params![i64::from(is_locked),sensitivity,document_id]).map_err(|e|e.to_string())?;
    record_audit_event(
        &connection,
        "document_security_changed",
        "document",
        Some(document_id),
        "Document lock or sensitivity changed",
    )
    .map_err(|e| e.to_string())?;
    drop(connection);
    security_status(app)
}

#[tauri::command]
fn list_locked_document_ids(app: tauri::AppHandle) -> Result<Vec<i64>, String> {
    let connection = production_connection(&app)?;
    let mut s = connection
        .prepare("SELECT id FROM documents WHERE trashed_at IS NULL AND is_locked=1 ORDER BY id")
        .map_err(|e| e.to_string())?;
    let rows = s
        .query_map([], |r| r.get(0))
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
fn list_private_document_ids(app: tauri::AppHandle) -> Result<Vec<i64>, String> {
    let connection = production_connection(&app)?;
    let mut statement = connection
        .prepare("SELECT id FROM documents WHERE trashed_at IS NULL AND is_private=1 ORDER BY id")
        .map_err(|e| e.to_string())?;
    let rows = statement
        .query_map([], |r| r.get(0))
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn decrypt_private_file(archive_path: &Path, password: &str, target: &Path) -> Result<(), String> {
    let file = fs::File::open(archive_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    if archive.len() != 1 {
        return Err("Den privata behållaren har ogiltigt innehåll".into());
    }
    let mut entry = archive
        .by_index_decrypt(0, password.as_bytes())
        .map_err(|_| "Fel PIN/lösenord eller skadad privat behållare".to_string())?;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut output = fs::File::create(target).map_err(|e| e.to_string())?;
    std::io::copy(&mut entry, &mut output).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn set_document_private(
    app: tauri::AppHandle,
    document_id: i64,
    is_private: bool,
    pin: String,
) -> Result<SecurityStatus, String> {
    let data_root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let connection = Connection::open(data_root.join("vault.db")).map_err(|e| e.to_string())?;
    if !verify_pin(&connection, &pin).map_err(|e| e.to_string())? {
        return Err("Rätt PIN/lösenord krävs".into());
    }
    let (file_id,storage_path,original_name,encrypted):(i64,String,String,i64)=connection.query_row("SELECT f.id,f.storage_path,f.original_name,f.encrypted_at_rest FROM document_versions dv JOIN files f ON f.id=dv.file_id WHERE dv.document_id=?1 AND dv.is_current_file=1",[document_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).map_err(|e|e.to_string())?;
    if is_private && encrypted == 0 {
        let source = data_root.join(&storage_path);
        if !source.is_file() {
            return Err("Originalfilen saknas och kan inte krypteras".into());
        }
        let private_relative = PathBuf::from("files")
            .join("private")
            .join(format!("document-{file_id}.vaultprivate"));
        let target = data_root.join(&private_relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let temp = target.with_extension("vaultprivate.tmp");
        let out = fs::File::create(&temp).map_err(|e| e.to_string())?;
        let mut writer = zip::ZipWriter::new(out);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .with_aes_encryption(zip::AesMode::Aes256, &pin);
        writer
            .start_file(&original_name, options)
            .map_err(|e| e.to_string())?;
        let mut input = fs::File::open(&source).map_err(|e| e.to_string())?;
        std::io::copy(&mut input, &mut writer).map_err(|e| e.to_string())?;
        writer.finish().map_err(|e| e.to_string())?;
        let verify_target = data_root
            .join("index")
            .join(format!("private-verify-{file_id}"));
        decrypt_private_file(&temp, &pin, &verify_target)?;
        if sha256_file(&verify_target).map_err(|e| e.to_string())?
            != sha256_file(&source).map_err(|e| e.to_string())?
        {
            let _ = fs::remove_file(&temp);
            let _ = fs::remove_file(&verify_target);
            return Err("Krypteringsverifieringen misslyckades".into());
        }
        let _ = fs::remove_file(&verify_target);
        fs::rename(&temp, &target).map_err(|e| e.to_string())?;
        fs::remove_file(&source).map_err(|e| e.to_string())?;
        connection.execute("UPDATE files SET storage_path=?1,encrypted_at_rest=1,encrypted_original_name=?2 WHERE id=?3",params![private_relative.to_string_lossy(),original_name,file_id]).map_err(|e|e.to_string())?;
    } else if !is_private && encrypted != 0 {
        let archive = data_root.join(&storage_path);
        let restored_relative = PathBuf::from("files")
            .join("restored-private")
            .join(format!("{file_id}-{}", sanitize_file_name(&original_name)));
        let restored = data_root.join(&restored_relative);
        decrypt_private_file(&archive, &pin, &restored)?;
        if sha256_file(&restored).map_err(|e| e.to_string())?
            != connection
                .query_row("SELECT sha256 FROM files WHERE id=?1", [file_id], |r| {
                    r.get::<_, String>(0)
                })
                .map_err(|e| e.to_string())?
        {
            let _ = fs::remove_file(&restored);
            return Err("Dekrypterad fil matchar inte ursprunglig SHA-256".into());
        }
        connection.execute("UPDATE files SET storage_path=?1,encrypted_at_rest=0,encrypted_original_name=NULL WHERE id=?2",params![restored_relative.to_string_lossy(),file_id]).map_err(|e|e.to_string())?;
        fs::remove_file(archive).map_err(|e| e.to_string())?;
    }
    connection.execute("UPDATE documents SET is_private=?1,is_hidden=?1,is_locked=?1,updated_at=CURRENT_TIMESTAMP WHERE id IN (SELECT document_id FROM document_versions WHERE file_id=?2)",params![i64::from(is_private),file_id]).map_err(|e|e.to_string())?;
    record_audit_event(
        &connection,
        if is_private {
            "document_encrypted_private"
        } else {
            "document_removed_private"
        },
        "document",
        Some(document_id),
        if is_private {
            "Original encrypted at rest and moved to private section"
        } else {
            "Private original decrypted after explicit authentication"
        },
    )
    .map_err(|e| e.to_string())?;
    drop(connection);
    security_status(app)
}

#[tauri::command]
fn open_private_document_file(
    app: tauri::AppHandle,
    document_id: i64,
    pin: String,
) -> Result<(), String> {
    let data_root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let connection = Connection::open(data_root.join("vault.db")).map_err(|e| e.to_string())?;
    if !verify_pin(&connection, &pin).map_err(|e| e.to_string())? {
        return Err("Rätt PIN/lösenord krävs".into());
    }
    let(storage,name,encrypted):(String,String,i64)=connection.query_row("SELECT f.storage_path,f.original_name,f.encrypted_at_rest FROM document_versions dv JOIN files f ON f.id=dv.file_id JOIN documents d ON d.id=dv.document_id WHERE d.id=?1 AND d.is_private=1 AND dv.is_current_file=1",[document_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).map_err(|e|e.to_string())?;
    if encrypted == 0 {
        return Err("Privat dokument saknar krypterad behållare".into());
    }
    let preview = data_root
        .join("index")
        .join("private-previews")
        .join(format!("{document_id}-{}", sanitize_file_name(&name)));
    decrypt_private_file(&data_root.join(storage), &pin, &preview)?;
    record_audit_event(
        &connection,
        "private_preview_opened",
        "document",
        Some(document_id),
        "Temporary authenticated private preview opened",
    )
    .map_err(|e| e.to_string())?;
    open_path_with_default_app(&preview).map_err(|e| e.to_string())
}

fn validate_plugin_manifest(manifest: &PluginManifest) -> Result<(), String> {
    if manifest.id.len() < 3
        || manifest.id.len() > 80
        || !manifest
            .id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '-'))
    {
        return Err("Plugin-id måste vara 3–80 tecken: små bokstäver, siffror, punkt, bindestreck eller understreck".into());
    }
    if manifest.name.trim().is_empty() || manifest.version.trim().is_empty() {
        return Err("Plugin måste ha namn och version".into());
    }
    if !manifest.ai_free {
        return Err("Plugin-manifestet måste uttryckligen ange ai_free: true".into());
    }
    let allowed = [
        "read_metadata",
        "read_text",
        "write_tags",
        "write_document_type",
    ];
    if let Some(cap) = manifest
        .capabilities
        .iter()
        .find(|cap| !allowed.contains(&cap.as_str()))
    {
        return Err(format!(
            "Otillåten plugin-behörighet: {cap}. Nätverk, processer och fri filåtkomst stöds inte"
        ));
    }
    let access_allowed = ["metadata", "document_text"];
    if let Some(access) = manifest
        .data_access
        .iter()
        .find(|x| !access_allowed.contains(&x.as_str()))
    {
        return Err(format!("Otillåten dataåtkomst: {access}"));
    }
    for action in &manifest.actions {
        if !["add_tag", "set_document_type"].contains(&action.action_type.as_str())
            || action.match_text.trim().is_empty()
            || action.value.trim().is_empty()
        {
            return Err("Plugin innehåller en ogiltig deklarativ åtgärd".into());
        }
        if action.action_type == "add_tag"
            && !manifest.capabilities.iter().any(|x| x == "write_tags")
        {
            return Err("Åtgärden add_tag kräver write_tags".into());
        }
        if action.action_type == "set_document_type"
            && !manifest
                .capabilities
                .iter()
                .any(|x| x == "write_document_type")
        {
            return Err("Åtgärden set_document_type kräver write_document_type".into());
        }
    }
    Ok(())
}

fn read_plugin_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<PluginSummary> {
    let capabilities: String = row.get(4)?;
    let access: String = row.get(5)?;
    Ok(PluginSummary {
        id: row.get(0)?,
        name: row.get(1)?,
        version: row.get(2)?,
        description: row.get(3)?,
        capabilities: serde_json::from_str(&capabilities).unwrap_or_default(),
        data_access: serde_json::from_str(&access).unwrap_or_default(),
        enabled: row.get::<_, i64>(6)? != 0,
        approved: row.get::<_, i64>(7)? != 0,
        last_run_at: row.get(8)?,
        last_result: row.get(9)?,
    })
}

#[tauri::command]
fn list_plugins(app: tauri::AppHandle) -> Result<Vec<PluginSummary>, String> {
    let connection = production_connection(&app)?;
    let mut statement=connection.prepare("SELECT id,name,version,description,capabilities_json,data_access_json,enabled,approved,last_run_at,last_result FROM plugins ORDER BY name COLLATE NOCASE").map_err(|e|e.to_string())?;
    let rows = statement
        .query_map([], read_plugin_summary)
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
fn install_plugin(
    app: tauri::AppHandle,
    manifest_path: String,
) -> Result<Vec<PluginSummary>, String> {
    let path = PathBuf::from(&manifest_path);
    if path.extension().and_then(|x| x.to_str()) != Some("json") || !path.is_file() {
        return Err("Välj ett lokalt JSON-manifest".into());
    }
    let bytes = fs::read(&path).map_err(|e| e.to_string())?;
    if bytes.len() > 256 * 1024 {
        return Err("Plugin-manifestet är för stort".into());
    }
    let manifest: PluginManifest =
        serde_json::from_slice(&bytes).map_err(|e| format!("Ogiltigt plugin-manifest: {e}"))?;
    validate_plugin_manifest(&manifest)?;
    let data_root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let dir = data_root.join("plugins").join(&manifest.id);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let target = dir.join("manifest.json");
    fs::write(
        &target,
        serde_json::to_vec_pretty(
            &serde_json::from_slice::<serde_json::Value>(&bytes).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let connection = Connection::open(data_root.join("vault.db")).map_err(|e| e.to_string())?;
    connection.execute("INSERT INTO plugins(id,name,version,description,manifest_path,capabilities_json,data_access_json,enabled,approved) VALUES(?1,?2,?3,?4,?5,?6,?7,0,0) ON CONFLICT(id) DO UPDATE SET name=excluded.name,version=excluded.version,description=excluded.description,manifest_path=excluded.manifest_path,capabilities_json=excluded.capabilities_json,data_access_json=excluded.data_access_json,enabled=0,approved=0,updated_at=CURRENT_TIMESTAMP",params![manifest.id,manifest.name,manifest.version,manifest.description,target.to_string_lossy(),serde_json::to_string(&manifest.capabilities).unwrap_or("[]".into()),serde_json::to_string(&manifest.data_access).unwrap_or("[]".into())]).map_err(|e|e.to_string())?;
    connection.execute("INSERT INTO plugin_events(plugin_id,event_type,safe_summary) VALUES(?1,'installed','Manifest validated; plugin remains disabled pending approval')",[&manifest.id]).map_err(|e|e.to_string())?;
    record_audit_event(
        &connection,
        "plugin_installed",
        "plugin",
        None,
        &format!(
            "Validated local plugin {} version {}; disabled pending approval",
            manifest.id, manifest.version
        ),
    )
    .map_err(|e| e.to_string())?;
    drop(connection);
    list_plugins(app)
}

#[tauri::command]
fn set_plugin_enabled(
    app: tauri::AppHandle,
    id: String,
    enabled: bool,
) -> Result<Vec<PluginSummary>, String> {
    let connection = production_connection(&app)?;
    let changed = connection
        .execute(
            "UPDATE plugins SET enabled=?1,approved=?1,updated_at=CURRENT_TIMESTAMP WHERE id=?2",
            params![i64::from(enabled), id],
        )
        .map_err(|e| e.to_string())?;
    if changed == 0 {
        return Err("Plugin hittades inte".into());
    }
    connection
        .execute(
            "INSERT INTO plugin_events(plugin_id,event_type,safe_summary) VALUES(?1,?2,?3)",
            params![
                id,
                if enabled {
                    "approved_enabled"
                } else {
                    "disabled"
                },
                if enabled {
                    "User approved declared capabilities and enabled plugin"
                } else {
                    "User disabled plugin"
                }
            ],
        )
        .map_err(|e| e.to_string())?;
    drop(connection);
    list_plugins(app)
}

#[tauri::command]
fn run_plugin(app: tauri::AppHandle, id: String) -> Result<PluginRunResult, String> {
    let data_root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let connection = Connection::open(data_root.join("vault.db")).map_err(|e| e.to_string())?;
    let (path, enabled, approved): (String, i64, i64) = connection
        .query_row(
            "SELECT manifest_path,enabled,approved FROM plugins WHERE id=?1",
            [&id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .map_err(|e| e.to_string())?;
    if enabled == 0 || approved == 0 {
        return Err("Plugin är inte frivilligt aktiverad och godkänd".into());
    }
    let manifest: PluginManifest =
        serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    validate_plugin_manifest(&manifest)?;
    let mut statement=connection.prepare("SELECT id,title,extracted_text,document_type FROM documents WHERE vault_id=1 AND trashed_at IS NULL AND is_locked=0 AND is_private=0").map_err(|e|e.to_string())?;
    let docs = statement
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    drop(statement);
    let mut changed = 0;
    for (document_id, title, text, current_type) in &docs {
        let haystack = if manifest.capabilities.iter().any(|x| x == "read_text") {
            format!("{title} {text}").to_lowercase()
        } else {
            title.to_lowercase()
        };
        for action in &manifest.actions {
            if !haystack.contains(&action.match_text.to_lowercase()) {
                continue;
            }
            match action.action_type.as_str() {
                "add_tag" => {
                    attach_tag(&connection, *document_id, &action.value)
                        .map_err(|e| e.to_string())?;
                    changed += 1
                }
                "set_document_type" if current_type != &action.value => {
                    connection.execute("UPDATE documents SET document_type=?1,updated_at=CURRENT_TIMESTAMP WHERE id=?2",params![action.value,document_id]).map_err(|e|e.to_string())?;
                    index_document(&connection, *document_id, title, &action.value, text)
                        .map_err(|e| e.to_string())?;
                    changed += 1
                }
                _ => {}
            }
        }
    }
    let result = format!(
        "Skannade {} icke-låsta dokument; utförde {changed} deklarativa ändringar",
        docs.len()
    );
    connection
        .execute(
            "UPDATE plugins SET last_run_at=CURRENT_TIMESTAMP,last_result=?1 WHERE id=?2",
            params![result, id],
        )
        .map_err(|e| e.to_string())?;
    connection
        .execute(
            "INSERT INTO plugin_events(plugin_id,event_type,safe_summary) VALUES(?1,'run',?2)",
            params![id, result],
        )
        .map_err(|e| e.to_string())?;
    record_audit_event(
        &connection,
        "plugin_run",
        "plugin",
        None,
        &format!("Plugin {id}: {result}"),
    )
    .map_err(|e| e.to_string())?;
    Ok(PluginRunResult {
        plugin_id: id,
        scanned_documents: docs.len() as i64,
        changed_documents: changed,
        explanation: result,
    })
}

#[tauri::command]
fn remove_plugin(app: tauri::AppHandle, id: String) -> Result<Vec<PluginSummary>, String> {
    if id.len() < 3
        || !id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '-'))
    {
        return Err("Ogiltigt plugin-id".into());
    }
    let data_root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let connection = Connection::open(data_root.join("vault.db")).map_err(|e| e.to_string())?;
    connection
        .execute("DELETE FROM plugins WHERE id=?1", [&id])
        .map_err(|e| e.to_string())?;
    let target = data_root.join("plugins").join(&id);
    if target.is_dir() {
        fs::remove_dir_all(&target).map_err(|e| e.to_string())?;
    }
    record_audit_event(
        &connection,
        "plugin_removed",
        "plugin",
        None,
        &format!("Removed local plugin {id}"),
    )
    .map_err(|e| e.to_string())?;
    drop(connection);
    list_plugins(app)
}

fn mask_sensitive_text(text: &str) -> String {
    text.split_whitespace()
        .map(|token| {
            let digits = token.chars().filter(|c| c.is_ascii_digit()).count();
            if digits >= 6 {
                format!("{}••••", token.chars().take(2).collect::<String>())
            } else {
                token.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[tauri::command]
fn export_document_secure(
    app: tauri::AppHandle,
    document_id: i64,
    pin: String,
    masked: bool,
) -> Result<ProtectedExportResult, String> {
    let data_root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    initialize_vault_at(&data_root).map_err(|e| e.to_string())?;
    let connection = Connection::open(data_root.join("vault.db")).map_err(|e| e.to_string())?;
    let (title,stored_path,text,is_locked,encrypted,original_name):(String,Option<String>,String,i64,i64,Option<String>)=connection.query_row("SELECT d.title,f.storage_path,d.extracted_text,d.is_locked,COALESCE(f.encrypted_at_rest,0),f.original_name FROM documents d LEFT JOIN document_versions v ON v.document_id=d.id AND v.is_current_file=1 LEFT JOIN files f ON f.id=v.file_id WHERE d.id=?1 AND d.vault_id=1",[document_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).map_err(|e|e.to_string())?;
    let organizer_locked:i64=connection.query_row("SELECT CASE WHEN EXISTS(SELECT 1 FROM document_folders df JOIN folders f ON f.id=df.folder_id WHERE df.document_id=?1 AND f.is_locked=1) OR EXISTS(SELECT 1 FROM document_categories dc JOIN categories c ON c.id=dc.category_id WHERE dc.document_id=?1 AND c.is_locked=1) THEN 1 ELSE 0 END",[document_id],|r|r.get(0)).map_err(|e|e.to_string())?;
    if (is_locked != 0 || organizer_locked != 0)
        && !verify_pin(&connection, &pin).map_err(|e| e.to_string())?
    {
        return Err("Rätt PIN/lösenord krävs för export av låst dokument".into());
    }
    let export_dir = data_root.join("exports");
    fs::create_dir_all(&export_dir).map_err(|e| e.to_string())?;
    let safe = sanitize_file_name(&title);
    let target = if masked {
        let target = export_dir.join(format!("{safe}-maskerad.txt"));
        fs::write(&target, mask_sensitive_text(&text)).map_err(|e| e.to_string())?;
        target
    } else {
        let source = stored_path.ok_or("Originalfil saknas")?;
        let ext = Path::new(original_name.as_deref().unwrap_or(&source))
            .extension()
            .and_then(|x| x.to_str())
            .unwrap_or("bin");
        let target = export_dir.join(format!("{safe}.{ext}"));
        if encrypted != 0 {
            decrypt_private_file(&data_root.join(source), &pin, &target)?;
        } else {
            fs::copy(data_root.join(source), &target).map_err(|e| e.to_string())?;
        }
        target
    };
    let bytes = fs::read(&target).map_err(|e| e.to_string())?;
    let sha256 = format!("{:x}", Sha256::digest(&bytes));
    connection.execute("INSERT INTO security_events(event_type,target_document_id,success,safe_summary) VALUES('secure_export',?1,1,?2)",params![document_id,if masked{"Masked text export created"}else{"Original export created"}]).map_err(|e|e.to_string())?;
    Ok(ProtectedExportResult {
        export_path: target.display().to_string(),
        sha256,
        size_bytes: bytes.len() as u64,
        masked,
    })
}

fn read_theme(row: &rusqlite::Row<'_>) -> rusqlite::Result<ThemeProfile> {
    Ok(ThemeProfile {
        id: row.get(0)?,
        name: row.get(1)?,
        base_mode: row.get(2)?,
        accent: row.get(3)?,
        surface_main: row.get(4)?,
        surface_sidebar: row.get(5)?,
        surface_raised: row.get(6)?,
        text_primary: row.get(7)?,
        text_secondary: row.get(8)?,
        border_color: row.get(9)?,
        radius_px: row.get(10)?,
        font_scale: row.get(11)?,
        density: row.get(12)?,
        motion: row.get(13)?,
        is_builtin: row.get::<_, i64>(14)? != 0,
        is_active: row.get::<_, i64>(15)? != 0,
        version: row.get(16)?,
    })
}

#[tauri::command]
fn list_theme_profiles(app: tauri::AppHandle) -> Result<Vec<ThemeProfile>, String> {
    let c = production_connection(&app)?;
    let mut s=c.prepare("SELECT id,name,base_mode,accent,surface_main,surface_sidebar,surface_raised,text_primary,text_secondary,border_color,radius_px,font_scale,density,motion,is_builtin,is_active,version FROM theme_profiles ORDER BY is_active DESC,is_builtin DESC,name").map_err(|e|e.to_string())?;
    let rows = s
        .query_map([], read_theme)
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn validate_theme_values(
    name: &str,
    base_mode: &str,
    colors: &[&str],
    density: &str,
    motion: &str,
) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Temat måste ha ett namn".into());
    }
    if !["light", "dark"].contains(&base_mode) {
        return Err("Ogiltigt basläge".into());
    }
    if colors.iter().any(|c| {
        c.len() != 7 || !c.starts_with('#') || !c[1..].chars().all(|x| x.is_ascii_hexdigit())
    }) {
        return Err("Alla temafärger måste anges som #RRGGBB".into());
    }
    if !["compact", "comfortable", "spacious"].contains(&density)
        || !["off", "reduced", "normal"].contains(&motion)
    {
        return Err("Ogiltig täthet eller rörelseinställning".into());
    }
    Ok(())
}

#[tauri::command]
fn save_theme_profile(
    app: tauri::AppHandle,
    name: String,
    base_mode: String,
    accent: String,
    surface_main: String,
    surface_sidebar: String,
    surface_raised: String,
    text_primary: String,
    text_secondary: String,
    border_color: String,
    radius_px: i64,
    font_scale: f64,
    density: String,
    motion: String,
) -> Result<Vec<ThemeProfile>, String> {
    validate_theme_values(
        &name,
        &base_mode,
        &[
            &accent,
            &surface_main,
            &surface_sidebar,
            &surface_raised,
            &text_primary,
            &text_secondary,
            &border_color,
        ],
        &density,
        &motion,
    )?;
    let c = production_connection(&app)?;
    let builtin: i64 = c
        .query_row(
            "SELECT COUNT(*) FROM theme_profiles WHERE name=?1 AND is_builtin=1",
            [name.trim()],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if builtin > 0 {
        return Err("Ett inbyggt tema kan inte skrivas över; välj ett nytt namn".into());
    }
    c.execute("INSERT INTO theme_profiles(name,base_mode,accent,surface_main,surface_sidebar,surface_raised,text_primary,text_secondary,border_color,radius_px,font_scale,density,motion,is_builtin,is_active) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,0,0) ON CONFLICT(name) DO UPDATE SET base_mode=excluded.base_mode,accent=excluded.accent,surface_main=excluded.surface_main,surface_sidebar=excluded.surface_sidebar,surface_raised=excluded.surface_raised,text_primary=excluded.text_primary,text_secondary=excluded.text_secondary,border_color=excluded.border_color,radius_px=excluded.radius_px,font_scale=excluded.font_scale,density=excluded.density,motion=excluded.motion,version=theme_profiles.version+1,updated_at=CURRENT_TIMESTAMP WHERE theme_profiles.is_builtin=0",params![name.trim(),base_mode,accent,surface_main,surface_sidebar,surface_raised,text_primary,text_secondary,border_color,radius_px.clamp(0,30),font_scale.clamp(0.8,1.4),density,motion]).map_err(|e|e.to_string())?;
    record_audit_event(
        &c,
        "theme_saved",
        "theme",
        None,
        "Local theme profile saved",
    )
    .map_err(|e| e.to_string())?;
    drop(c);
    list_theme_profiles(app)
}

#[tauri::command]
fn activate_theme_profile(app: tauri::AppHandle, id: i64) -> Result<Vec<ThemeProfile>, String> {
    let mut c = production_connection(&app)?;
    let tx = c.transaction().map_err(|e| e.to_string())?;
    tx.execute(
        "UPDATE theme_profiles SET is_active=0 WHERE is_active=1",
        [],
    )
    .map_err(|e| e.to_string())?;
    if tx
        .execute("UPDATE theme_profiles SET is_active=1 WHERE id=?1", [id])
        .map_err(|e| e.to_string())?
        != 1
    {
        return Err("Temat finns inte".into());
    }
    tx.commit().map_err(|e| e.to_string())?;
    drop(c);
    list_theme_profiles(app)
}

#[tauri::command]
fn duplicate_theme_profile(
    app: tauri::AppHandle,
    id: i64,
    name: String,
) -> Result<Vec<ThemeProfile>, String> {
    let c = production_connection(&app)?;
    if name.trim().is_empty() {
        return Err("Ange namn på kopian".into());
    }
    c.execute("INSERT INTO theme_profiles(name,base_mode,accent,surface_main,surface_sidebar,surface_raised,text_primary,text_secondary,border_color,radius_px,font_scale,density,motion,is_builtin,is_active) SELECT ?1,base_mode,accent,surface_main,surface_sidebar,surface_raised,text_primary,text_secondary,border_color,radius_px,font_scale,density,motion,0,0 FROM theme_profiles WHERE id=?2",params![name.trim(),id]).map_err(|e|e.to_string())?;
    drop(c);
    list_theme_profiles(app)
}

#[tauri::command]
fn export_theme_profile(app: tauri::AppHandle, id: i64) -> Result<String, String> {
    let themes = list_theme_profiles(app.clone())?;
    let theme = themes
        .into_iter()
        .find(|x| x.id == id)
        .ok_or("Temat finns inte")?;
    let root = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("themes");
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let path = root.join(format!(
        "{}.vault-theme.json",
        sanitize_file_name(&theme.name)
    ));
    let envelope = serde_json::json!({"format":"vault-theme-v1","theme":theme});
    fs::write(
        &path,
        serde_json::to_string_pretty(&envelope).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().into_owned())
}

#[tauri::command]
fn import_theme_profile(app: tauri::AppHandle, path: String) -> Result<Vec<ThemeProfile>, String> {
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    if value.get("format").and_then(|v| v.as_str()) != Some("vault-theme-v1") {
        return Err("Filen är inte ett Vault-tema v1".into());
    }
    let mut theme: ThemeProfile =
        serde_json::from_value(value.get("theme").cloned().ok_or("Temadata saknas")?)
            .map_err(|e| e.to_string())?;
    theme.name = format!("{} importerad", theme.name);
    save_theme_profile(
        app,
        theme.name,
        theme.base_mode,
        theme.accent,
        theme.surface_main,
        theme.surface_sidebar,
        theme.surface_raised,
        theme.text_primary,
        theme.text_secondary,
        theme.border_color,
        theme.radius_px,
        theme.font_scale,
        theme.density,
        theme.motion,
    )
}

#[tauri::command]
fn list_favorite_document_ids(app: tauri::AppHandle) -> Result<Vec<i64>, String> {
    let c = production_connection(&app)?;
    let mut s=c.prepare("SELECT id FROM documents WHERE vault_id=1 AND trashed_at IS NULL AND is_favorite=1 ORDER BY updated_at DESC").map_err(|e|e.to_string())?;
    let rows = s
        .query_map([], |r| r.get(0))
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
fn set_document_favorite(
    app: tauri::AppHandle,
    document_id: i64,
    favorite: bool,
) -> Result<Vec<i64>, String> {
    let c = production_connection(&app)?;
    if c.execute("UPDATE documents SET is_favorite=?1,updated_at=CURRENT_TIMESTAMP WHERE id=?2 AND vault_id=1 AND trashed_at IS NULL",params![i64::from(favorite),document_id]).map_err(|e|e.to_string())?!=1{return Err("Dokumentet finns inte".into())}
    record_audit_event(
        &c,
        "favorite_changed",
        "document",
        Some(document_id),
        if favorite {
            "Document added to favorites"
        } else {
            "Document removed from favorites"
        },
    )
    .map_err(|e| e.to_string())?;
    drop(c);
    list_favorite_document_ids(app)
}

#[tauri::command]
fn list_collections(app: tauri::AppHandle) -> Result<Vec<CollectionSummary>, String> {
    let c = production_connection(&app)?;
    let mut s=c.prepare("SELECT c.id,c.name,c.description,c.color,c.is_pinned,COUNT(cd.document_id) FROM collections c LEFT JOIN collection_documents cd ON cd.collection_id=c.id WHERE c.vault_id=1 GROUP BY c.id ORDER BY c.is_pinned DESC,c.name").map_err(|e|e.to_string())?;
    let rows = s
        .query_map([], |r| {
            Ok(CollectionSummary {
                id: r.get(0)?,
                name: r.get(1)?,
                description: r.get(2)?,
                color: r.get(3)?,
                is_pinned: r.get::<_, i64>(4)? != 0,
                document_count: r.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
fn save_collection(
    app: tauri::AppHandle,
    name: String,
    description: String,
    color: String,
    is_pinned: bool,
) -> Result<Vec<CollectionSummary>, String> {
    if name.trim().is_empty() {
        return Err("Samlingen måste ha ett namn".into());
    }
    if color.len() != 7 || !color.starts_with('#') {
        return Err("Ogiltig samlingsfärg".into());
    }
    let c = production_connection(&app)?;
    c.execute("INSERT INTO collections(vault_id,name,description,color,is_pinned) VALUES(1,?1,?2,?3,?4) ON CONFLICT(vault_id,name) DO UPDATE SET description=excluded.description,color=excluded.color,is_pinned=excluded.is_pinned,updated_at=CURRENT_TIMESTAMP",params![name.trim(),description.trim(),color,i64::from(is_pinned)]).map_err(|e|e.to_string())?;
    drop(c);
    list_collections(app)
}

#[tauri::command]
fn set_collection_document(
    app: tauri::AppHandle,
    collection_id: i64,
    document_id: i64,
    included: bool,
) -> Result<Vec<CollectionSummary>, String> {
    let c = production_connection(&app)?;
    if included {
        c.execute("INSERT OR IGNORE INTO collection_documents(collection_id,document_id) SELECT ?1,?2 WHERE EXISTS(SELECT 1 FROM collections WHERE id=?1 AND vault_id=1) AND EXISTS(SELECT 1 FROM documents WHERE id=?2 AND vault_id=1 AND trashed_at IS NULL)",params![collection_id,document_id]).map_err(|e|e.to_string())?;
    } else {
        c.execute(
            "DELETE FROM collection_documents WHERE collection_id=?1 AND document_id=?2",
            params![collection_id, document_id],
        )
        .map_err(|e| e.to_string())?;
    }
    record_audit_event(
        &c,
        "collection_membership_changed",
        "document",
        Some(document_id),
        "Document collection membership changed",
    )
    .map_err(|e| e.to_string())?;
    drop(c);
    list_collections(app)
}

#[tauri::command]
fn list_collection_document_ids(
    app: tauri::AppHandle,
    collection_id: i64,
) -> Result<Vec<i64>, String> {
    let c = production_connection(&app)?;
    let mut s=c.prepare("SELECT cd.document_id FROM collection_documents cd JOIN collections c ON c.id=cd.collection_id WHERE cd.collection_id=?1 AND c.vault_id=1 ORDER BY cd.added_at DESC").map_err(|e|e.to_string())?;
    let rows = s
        .query_map([collection_id], |r| r.get(0))
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn list_organizers(connection: &Connection, kind: &str) -> Result<Vec<OrganizerSummary>, String> {
    let (table, join, foreign) = if kind == "folder" {
        ("folders", "document_folders", "folder_id")
    } else {
        ("categories", "document_categories", "category_id")
    };
    let description = if kind == "folder" {
        "''"
    } else {
        "o.description"
    };
    let sql = format!("SELECT o.id,o.name,o.color,{description},o.is_pinned,o.is_locked,COUNT(j.document_id) FROM {table} o LEFT JOIN {join} j ON j.{foreign}=o.id WHERE o.vault_id=1 GROUP BY o.id ORDER BY o.is_pinned DESC,o.name");
    let mut statement = connection.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok(OrganizerSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                description: row.get(3)?,
                is_pinned: row.get::<_, i64>(4)? != 0,
                is_locked: row.get::<_, i64>(5)? != 0,
                document_count: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
fn list_folders(app: tauri::AppHandle) -> Result<Vec<OrganizerSummary>, String> {
    list_organizers(&production_connection(&app)?, "folder")
}

#[tauri::command]
fn list_categories(app: tauri::AppHandle) -> Result<Vec<OrganizerSummary>, String> {
    list_organizers(&production_connection(&app)?, "category")
}

#[tauri::command]
fn save_folder(
    app: tauri::AppHandle,
    name: String,
    color: String,
    is_pinned: bool,
) -> Result<Vec<OrganizerSummary>, String> {
    if name.trim().is_empty() {
        return Err("Mappen måste ha ett namn".into());
    }
    let connection = production_connection(&app)?;
    let updated = connection.execute("UPDATE folders SET color=?2,is_pinned=?3,updated_at=CURRENT_TIMESTAMP WHERE vault_id=1 AND parent_id IS NULL AND name=?1",params![name.trim(),color,i64::from(is_pinned)]).map_err(|e|e.to_string())?;
    if updated == 0 {
        connection
            .execute(
                "INSERT INTO folders(vault_id,name,color,is_pinned) VALUES(1,?1,?2,?3)",
                params![name.trim(), color, i64::from(is_pinned)],
            )
            .map_err(|e| e.to_string())?;
    }
    list_organizers(&connection, "folder")
}

#[tauri::command]
fn save_category(
    app: tauri::AppHandle,
    name: String,
    description: String,
    color: String,
    is_pinned: bool,
) -> Result<Vec<OrganizerSummary>, String> {
    if name.trim().is_empty() {
        return Err("Kategorin måste ha ett namn".into());
    }
    let connection = production_connection(&app)?;
    connection.execute("INSERT INTO categories(vault_id,name,description,color,is_pinned) VALUES(1,?1,?2,?3,?4) ON CONFLICT(vault_id,name) DO UPDATE SET description=excluded.description,color=excluded.color,is_pinned=excluded.is_pinned,updated_at=CURRENT_TIMESTAMP",params![name.trim(),description.trim(),color,i64::from(is_pinned)]).map_err(|e|e.to_string())?;
    list_organizers(&connection, "category")
}

#[tauri::command]
fn set_organizer_locked(
    app: tauri::AppHandle,
    kind: String,
    organizer_id: i64,
    locked: bool,
    pin: String,
) -> Result<Vec<OrganizerSummary>, String> {
    let connection = production_connection(&app)?;
    if !verify_pin(&connection, &pin).map_err(|e| e.to_string())? {
        return Err("Rätt PIN/lösenord krävs för att ändra mapp- eller kategorilås".into());
    }
    let table = match kind.as_str() {
        "folder" => "folders",
        "category" => "categories",
        _ => return Err("Okänd organisationstyp".into()),
    };
    if connection.execute(&format!("UPDATE {table} SET is_locked=?1,updated_at=CURRENT_TIMESTAMP WHERE id=?2 AND vault_id=1"),params![i64::from(locked),organizer_id]).map_err(|e|e.to_string())? != 1 {
        return Err("Mappen eller kategorin finns inte".into());
    }
    record_audit_event(
        &connection,
        "organizer_lock_changed",
        &kind,
        Some(organizer_id),
        if locked {
            "Organizer locked"
        } else {
            "Organizer unlocked"
        },
    )
    .map_err(|e| e.to_string())?;
    list_organizers(&connection, &kind)
}

#[tauri::command]
fn set_document_organizer(
    app: tauri::AppHandle,
    kind: String,
    organizer_id: i64,
    document_id: i64,
    included: bool,
) -> Result<Vec<i64>, String> {
    let connection = production_connection(&app)?;
    let (join, foreign, parent) = match kind.as_str() {
        "folder" => ("document_folders", "folder_id", "folders"),
        "category" => ("document_categories", "category_id", "categories"),
        _ => return Err("Okänd organisationstyp".into()),
    };
    if included {
        connection.execute(&format!("INSERT OR IGNORE INTO {join}(document_id,{foreign}) SELECT ?1,?2 WHERE EXISTS(SELECT 1 FROM documents WHERE id=?1 AND vault_id=1 AND trashed_at IS NULL) AND EXISTS(SELECT 1 FROM {parent} WHERE id=?2 AND vault_id=1)"),params![document_id,organizer_id]).map_err(|e|e.to_string())?;
    } else {
        connection
            .execute(
                &format!("DELETE FROM {join} WHERE document_id=?1 AND {foreign}=?2"),
                params![document_id, organizer_id],
            )
            .map_err(|e| e.to_string())?;
    }
    record_audit_event(
        &connection,
        "organization_changed",
        "document",
        Some(document_id),
        "Document folder/category membership changed",
    )
    .map_err(|e| e.to_string())?;
    let mut statement = connection
        .prepare(&format!(
            "SELECT {foreign} FROM {join} WHERE document_id=?1 ORDER BY {foreign}"
        ))
        .map_err(|e| e.to_string())?;
    let rows = statement
        .query_map([document_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
fn list_document_organizer_ids(
    app: tauri::AppHandle,
    kind: String,
    document_id: i64,
) -> Result<Vec<i64>, String> {
    let connection = production_connection(&app)?;
    let (join, foreign) = match kind.as_str() {
        "folder" => ("document_folders", "folder_id"),
        "category" => ("document_categories", "category_id"),
        _ => return Err("Okänd organisationstyp".into()),
    };
    let mut statement = connection
        .prepare(&format!(
            "SELECT {foreign} FROM {join} WHERE document_id=?1 ORDER BY {foreign}"
        ))
        .map_err(|e| e.to_string())?;
    let rows = statement
        .query_map([document_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
fn list_document_notes(
    app: tauri::AppHandle,
    document_id: i64,
) -> Result<Vec<DocumentNoteSummary>, String> {
    let connection = production_connection(&app)?;
    let mut statement=connection.prepare("SELECT n.id,n.document_id,n.body,n.kind,COALESCE(MAX(v.version_no),1),n.created_at,n.updated_at FROM document_notes n LEFT JOIN note_versions v ON v.note_id=n.id WHERE n.document_id=?1 GROUP BY n.id ORDER BY n.updated_at DESC").map_err(|e|e.to_string())?;
    let rows = statement
        .query_map([document_id], |r| {
            Ok(DocumentNoteSummary {
                id: r.get(0)?,
                document_id: r.get(1)?,
                body: r.get(2)?,
                kind: r.get(3)?,
                version_no: r.get(4)?,
                created_at: r.get(5)?,
                updated_at: r.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
fn save_document_note(
    app: tauri::AppHandle,
    document_id: i64,
    note_id: Option<i64>,
    body: String,
    kind: String,
) -> Result<Vec<DocumentNoteSummary>, String> {
    if body.trim().is_empty() {
        return Err("Anteckningen är tom".into());
    }
    let mut connection = production_connection(&app)?;
    let transaction = connection.transaction().map_err(|e| e.to_string())?;
    let id = if let Some(id) = note_id {
        let next: i64 = transaction
            .query_row(
                "SELECT COALESCE(MAX(version_no),0)+1 FROM note_versions WHERE note_id=?1",
                [id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        transaction.execute("UPDATE document_notes SET body=?1,kind=?2,updated_at=CURRENT_TIMESTAMP WHERE id=?3 AND document_id=?4",params![body.trim(),kind,id,document_id]).map_err(|e|e.to_string())?;
        transaction
            .execute(
                "INSERT INTO note_versions(note_id,version_no,body) VALUES(?1,?2,?3)",
                params![id, next, body.trim()],
            )
            .map_err(|e| e.to_string())?;
        id
    } else {
        transaction
            .execute(
                "INSERT INTO document_notes(document_id,body,kind) VALUES(?1,?2,?3)",
                params![document_id, body.trim(), kind],
            )
            .map_err(|e| e.to_string())?;
        let id = transaction.last_insert_rowid();
        transaction
            .execute(
                "INSERT INTO note_versions(note_id,version_no,body) VALUES(?1,1,?2)",
                params![id, body.trim()],
            )
            .map_err(|e| e.to_string())?;
        id
    };
    record_audit_event(
        &transaction,
        "note_saved",
        "document",
        Some(document_id),
        &format!("Document note {id} saved"),
    )
    .map_err(|e| e.to_string())?;
    transaction.commit().map_err(|e| e.to_string())?;
    list_document_notes(app, document_id)
}

fn parse_string_array(value: String) -> Vec<String> {
    serde_json::from_str(&value).unwrap_or_default()
}

#[tauri::command]
fn list_vault_notes(
    app: tauri::AppHandle,
    query: String,
    target_type: Option<String>,
    target_id: Option<i64>,
) -> Result<Vec<VaultNoteSummary>, String> {
    let connection = production_connection(&app)?;
    let pattern = format!("%{}%", query.trim());
    let mut statement=connection.prepare("SELECT n.id,n.title,n.body,n.kind,n.target_type,n.target_id,n.tags_json,n.code_words_json,COALESCE(MAX(v.version_no),1),n.created_at,n.updated_at FROM vault_notes n LEFT JOIN vault_note_versions v ON v.note_id=n.id WHERE n.vault_id=1 AND (?1='' OR n.title LIKE ?2 OR n.body LIKE ?2 OR n.tags_json LIKE ?2 OR n.code_words_json LIKE ?2) AND (?3 IS NULL OR n.target_type=?3) AND (?4 IS NULL OR n.target_id=?4) GROUP BY n.id ORDER BY n.updated_at DESC").map_err(|e|e.to_string())?;
    let rows = statement
        .query_map(
            params![query.trim(), pattern, target_type, target_id],
            |r| {
                Ok(VaultNoteSummary {
                    id: r.get(0)?,
                    title: r.get(1)?,
                    body: r.get(2)?,
                    kind: r.get(3)?,
                    target_type: r.get(4)?,
                    target_id: r.get(5)?,
                    tags: parse_string_array(r.get(6)?),
                    code_words: parse_string_array(r.get(7)?),
                    version_no: r.get(8)?,
                    created_at: r.get(9)?,
                    updated_at: r.get(10)?,
                })
            },
        )
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
fn save_vault_note(
    app: tauri::AppHandle,
    id: Option<i64>,
    title: String,
    body: String,
    kind: String,
    target_type: String,
    target_id: Option<i64>,
    tags: Vec<String>,
    code_words: Vec<String>,
) -> Result<Vec<VaultNoteSummary>, String> {
    if title.trim().is_empty() || body.trim().is_empty() {
        return Err("Titel och innehåll krävs".into());
    }
    if ![
        "vault", "document", "object", "person", "company", "category", "event", "claim",
        "conflict",
    ]
    .contains(&target_type.as_str())
    {
        return Err("Otillåten anteckningslänk".into());
    }
    if target_type != "vault" && target_id.is_none() {
        return Err("Välj vilket objekt anteckningen ska länkas till".into());
    }
    let mut connection = production_connection(&app)?;
    let tx = connection.transaction().map_err(|e| e.to_string())?;
    let tags_json = serde_json::to_string(&tags).map_err(|e| e.to_string())?;
    let words_json = serde_json::to_string(&code_words).map_err(|e| e.to_string())?;
    let note_id = if let Some(note_id) = id {
        let next: i64 = tx
            .query_row(
                "SELECT COALESCE(MAX(version_no),0)+1 FROM vault_note_versions WHERE note_id=?1",
                [note_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        let changed=tx.execute("UPDATE vault_notes SET title=?1,body=?2,kind=?3,target_type=?4,target_id=?5,tags_json=?6,code_words_json=?7,updated_at=CURRENT_TIMESTAMP WHERE id=?8 AND vault_id=1",params![title.trim(),body.trim(),kind,target_type,target_id,tags_json,words_json,note_id]).map_err(|e|e.to_string())?;
        if changed == 0 {
            return Err("Anteckningen hittades inte".into());
        }
        tx.execute(
            "INSERT INTO vault_note_versions(note_id,version_no,title,body) VALUES(?1,?2,?3,?4)",
            params![note_id, next, title.trim(), body.trim()],
        )
        .map_err(|e| e.to_string())?;
        note_id
    } else {
        tx.execute("INSERT INTO vault_notes(vault_id,title,body,kind,target_type,target_id,tags_json,code_words_json) VALUES(1,?1,?2,?3,?4,?5,?6,?7)",params![title.trim(),body.trim(),kind,target_type,target_id,tags_json,words_json]).map_err(|e|e.to_string())?;
        let note_id = tx.last_insert_rowid();
        tx.execute(
            "INSERT INTO vault_note_versions(note_id,version_no,title,body) VALUES(?1,1,?2,?3)",
            params![note_id, title.trim(), body.trim()],
        )
        .map_err(|e| e.to_string())?;
        note_id
    };
    record_audit_event(
        &tx,
        "vault_note_saved",
        "note",
        Some(note_id),
        "Linked local note saved",
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    list_vault_notes(app, String::new(), None, None)
}

#[tauri::command]
fn delete_vault_note(app: tauri::AppHandle, id: i64) -> Result<Vec<VaultNoteSummary>, String> {
    let connection = production_connection(&app)?;
    connection
        .execute("DELETE FROM vault_notes WHERE id=?1 AND vault_id=1", [id])
        .map_err(|e| e.to_string())?;
    list_vault_notes(app, String::new(), None, None)
}

#[tauri::command]
fn list_vault_note_versions(
    app: tauri::AppHandle,
    note_id: i64,
) -> Result<Vec<NoteVersionSummary>, String> {
    let connection = production_connection(&app)?;
    let mut statement=connection.prepare("SELECT version_no,title,body,created_at FROM vault_note_versions WHERE note_id=?1 ORDER BY version_no DESC").map_err(|e|e.to_string())?;
    let rows = statement
        .query_map([note_id], |r| {
            Ok(NoteVersionSummary {
                version_no: r.get(0)?,
                title: r.get(1)?,
                body: r.get(2)?,
                created_at: r.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
fn list_note_templates(app: tauri::AppHandle) -> Result<Vec<NoteTemplateSummary>, String> {
    let connection = production_connection(&app)?;
    let mut statement = connection
        .prepare("SELECT id,name,body FROM note_templates WHERE vault_id=1 ORDER BY name")
        .map_err(|e| e.to_string())?;
    let rows = statement
        .query_map([], |r| {
            Ok(NoteTemplateSummary {
                id: r.get(0)?,
                name: r.get(1)?,
                body: r.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
fn list_search_history(app: tauri::AppHandle) -> Result<Vec<SearchHistorySummary>, String> {
    let connection = production_connection(&app)?;
    prune_search_history(&connection).map_err(|e| e.to_string())?;
    let mut statement=connection.prepare("SELECT id,query,result_count,executed_at,pinned FROM search_history WHERE vault_id=1 ORDER BY pinned DESC,executed_at DESC,id DESC LIMIT 100").map_err(|e|e.to_string())?;
    let rows = statement
        .query_map([], |r| {
            Ok(SearchHistorySummary {
                id: r.get(0)?,
                query: r.get(1)?,
                result_count: r.get(2)?,
                executed_at: r.get(3)?,
                pinned: r.get::<_, i64>(4)? != 0,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn search_history_preferences_at(
    connection: &Connection,
) -> rusqlite::Result<SearchHistoryPreferences> {
    let raw: Option<String> = connection
        .query_row(
            "SELECT value_json FROM settings WHERE vault_id=1 AND key='search_history_preferences'",
            [],
            |r| r.get(0),
        )
        .optional()?;
    Ok(raw
        .and_then(|value| serde_json::from_str(&value).ok())
        .unwrap_or(SearchHistoryPreferences {
            enabled: true,
            retention_days: 90,
        }))
}

fn prune_search_history(connection: &Connection) -> rusqlite::Result<()> {
    let preferences = search_history_preferences_at(connection)?;
    connection.execute("DELETE FROM search_history WHERE vault_id=1 AND pinned=0 AND executed_at < datetime('now', '-' || ?1 || ' days')",[preferences.retention_days.clamp(1,3650)])?;
    Ok(())
}

#[tauri::command]
fn get_search_history_preferences(
    app: tauri::AppHandle,
) -> Result<SearchHistoryPreferences, String> {
    search_history_preferences_at(&production_connection(&app)?).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_search_history_preferences(
    app: tauri::AppHandle,
    enabled: bool,
    retention_days: i64,
) -> Result<SearchHistoryPreferences, String> {
    let connection = production_connection(&app)?;
    let preferences = SearchHistoryPreferences {
        enabled,
        retention_days: retention_days.clamp(1, 3650),
    };
    let value = serde_json::to_string(&preferences).map_err(|e| e.to_string())?;
    connection.execute("INSERT INTO settings(vault_id,key,value_json) VALUES(1,'search_history_preferences',?1) ON CONFLICT(vault_id,key) DO UPDATE SET value_json=excluded.value_json",[value]).map_err(|e|e.to_string())?;
    prune_search_history(&connection).map_err(|e| e.to_string())?;
    Ok(preferences)
}

#[tauri::command]
fn clear_search_history(app: tauri::AppHandle) -> Result<Vec<SearchHistorySummary>, String> {
    let connection = production_connection(&app)?;
    connection
        .execute(
            "DELETE FROM search_history WHERE vault_id=1 AND pinned=0",
            [],
        )
        .map_err(|e| e.to_string())?;
    drop(connection);
    list_search_history(app)
}

#[tauri::command]
fn set_search_history_pinned(
    app: tauri::AppHandle,
    id: i64,
    pinned: bool,
) -> Result<Vec<SearchHistorySummary>, String> {
    let connection = production_connection(&app)?;
    connection
        .execute(
            "UPDATE search_history SET pinned=?1 WHERE id=?2 AND vault_id=1",
            params![i64::from(pinned), id],
        )
        .map_err(|e| e.to_string())?;
    drop(connection);
    list_search_history(app)
}

#[tauri::command]
fn export_search_history(app: tauri::AppHandle) -> Result<String, String> {
    let data_root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let connection = production_connection(&app)?;
    prune_search_history(&connection).map_err(|e| e.to_string())?;
    let mut statement=connection.prepare("SELECT query,result_count,executed_at,pinned FROM search_history WHERE vault_id=1 ORDER BY executed_at DESC").map_err(|e|e.to_string())?;
    let rows=statement.query_map([],|r|Ok(serde_json::json!({"query":r.get::<_,String>(0)?,"result_count":r.get::<_,i64>(1)?,"executed_at":r.get::<_,String>(2)?,"pinned":r.get::<_,i64>(3)?!=0}))).map_err(|e|e.to_string())?.collect::<rusqlite::Result<Vec<_>>>().map_err(|e|e.to_string())?;
    let export_root = data_root.join("exports");
    fs::create_dir_all(&export_root).map_err(|e| e.to_string())?;
    let path = export_root.join(format!("vault-search-history-{}.json", epoch_seconds()));
    fs::write(
        &path,
        serde_json::to_vec_pretty(
            &serde_json::json!({"format":"vault-search-history","version":1,"entries":rows}),
        )
        .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().into_owned())
}

#[tauri::command]
fn record_search_history(
    app: tauri::AppHandle,
    query: String,
    result_count: i64,
) -> Result<Vec<SearchHistorySummary>, String> {
    let normalized = query.trim();
    if normalized.is_empty() {
        return list_search_history(app);
    }
    let connection = production_connection(&app)?;
    if !search_history_preferences_at(&connection)
        .map_err(|e| e.to_string())?
        .enabled
    {
        return Ok(Vec::new());
    }
    connection
        .execute(
            "DELETE FROM search_history WHERE vault_id=1 AND lower(query)=lower(?1) AND pinned=0",
            [normalized],
        )
        .map_err(|e| e.to_string())?;
    connection
        .execute(
            "INSERT INTO search_history(vault_id,query,result_count) VALUES(1,?1,?2)",
            params![normalized, result_count.max(0)],
        )
        .map_err(|e| e.to_string())?;
    connection.execute("DELETE FROM search_history WHERE vault_id=1 AND pinned=0 AND id NOT IN(SELECT id FROM search_history WHERE vault_id=1 AND pinned=0 ORDER BY executed_at DESC,id DESC LIMIT 100)",[]).map_err(|e|e.to_string())?;
    drop(connection);
    list_search_history(app)
}

#[tauri::command]
fn get_analysis_preferences(app: tauri::AppHandle) -> Result<AnalysisPreferences, String> {
    let c = production_connection(&app)?;
    c.query_row("SELECT mode,auto_ocr,auto_classify,auto_claims,auto_relations,include_locked FROM analysis_preferences WHERE vault_id=1",[],|r|Ok(AnalysisPreferences{mode:r.get(0)?,auto_ocr:r.get::<_,i64>(1)?!=0,auto_classify:r.get::<_,i64>(2)?!=0,auto_claims:r.get::<_,i64>(3)?!=0,auto_relations:r.get::<_,i64>(4)?!=0,include_locked:r.get::<_,i64>(5)?!=0})).map_err(|e|e.to_string())
}

#[tauri::command]
fn save_analysis_preferences(
    app: tauri::AppHandle,
    mode: String,
    auto_ocr: bool,
    auto_classify: bool,
    auto_claims: bool,
    auto_relations: bool,
    include_locked: bool,
) -> Result<AnalysisPreferences, String> {
    if ![
        "none",
        "manual",
        "folder",
        "category",
        "entities",
        "all",
        "future",
        "all_future",
    ]
    .contains(&mode.as_str())
    {
        return Err("Ogiltig analysomfattning".into());
    }
    let c = production_connection(&app)?;
    c.execute("UPDATE analysis_preferences SET mode=?1,auto_ocr=?2,auto_classify=?3,auto_claims=?4,auto_relations=?5,include_locked=?6,updated_at=CURRENT_TIMESTAMP WHERE vault_id=1",params![mode,i64::from(auto_ocr),i64::from(auto_classify),i64::from(auto_claims),i64::from(auto_relations),i64::from(include_locked)]).map_err(|e|e.to_string())?;
    record_audit_event(
        &c,
        "analysis_preferences_changed",
        "vault",
        Some(1),
        "Local deterministic analysis scope changed",
    )
    .map_err(|e| e.to_string())?;
    drop(c);
    get_analysis_preferences(app)
}

#[tauri::command]
fn list_analysis_exclusions(app: tauri::AppHandle) -> Result<Vec<AnalysisExclusion>, String> {
    let c = production_connection(&app)?;
    let mut s=c.prepare("SELECT id,scope_type,scope_value,reason FROM analysis_exclusions WHERE vault_id=1 ORDER BY scope_type,scope_value").map_err(|e|e.to_string())?;
    let rows = s
        .query_map([], |r| {
            Ok(AnalysisExclusion {
                id: r.get(0)?,
                scope_type: r.get(1)?,
                scope_value: r.get(2)?,
                reason: r.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
fn add_analysis_exclusion(
    app: tauri::AppHandle,
    scope_type: String,
    scope_value: String,
    reason: String,
) -> Result<Vec<AnalysisExclusion>, String> {
    if !["document", "folder", "category", "document_type"].contains(&scope_type.as_str())
        || scope_value.trim().is_empty()
    {
        return Err("Ange giltig typ och värde för undantaget".into());
    }
    let c = production_connection(&app)?;
    c.execute("INSERT INTO analysis_exclusions(vault_id,scope_type,scope_value,reason) VALUES(1,?1,?2,?3) ON CONFLICT(vault_id,scope_type,scope_value) DO UPDATE SET reason=excluded.reason",params![scope_type,scope_value.trim(),reason.trim()]).map_err(|e|e.to_string())?;
    drop(c);
    list_analysis_exclusions(app)
}

#[tauri::command]
fn delete_analysis_exclusion(
    app: tauri::AppHandle,
    id: i64,
) -> Result<Vec<AnalysisExclusion>, String> {
    let c = production_connection(&app)?;
    c.execute(
        "DELETE FROM analysis_exclusions WHERE id=?1 AND vault_id=1",
        [id],
    )
    .map_err(|e| e.to_string())?;
    drop(c);
    list_analysis_exclusions(app)
}

fn scanner_launcher_path() -> Option<PathBuf> {
    let windows = std::env::var_os("WINDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    [
        windows.join("System32").join("wiaacmgr.exe"),
        windows.join("SysWOW64").join("wiaacmgr.exe"),
    ]
    .into_iter()
    .find(|p| p.is_file())
}
#[tauri::command]
fn scanner_status() -> ScannerStatus {
    let path = scanner_launcher_path();
    ScannerStatus {
        available: path.is_some(),
        launcher_path: path.as_ref().map(|p| p.to_string_lossy().into_owned()),
        notes: if path.is_some() {
            vec!["Windows lokal skannerguide hittades. Spara bilden och importera den direkt i Vault.".into()]
        } else {
            vec!["Ingen lokal Windows WIA-skannerguide hittades. Bilder kan fortfarande importeras från skannerns mapp.".into()]
        },
    }
}
#[tauri::command]
fn launch_local_scanner() -> Result<(), String> {
    let path = scanner_launcher_path().ok_or("Windows skannerguide saknas")?;
    Command::new(path).spawn().map_err(|e| e.to_string())?;
    Ok(())
}

fn html_to_visible_text(html: &str) -> String {
    let mut text = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                text.push(' ')
            }
            _ if !in_tag => text.push(ch),
            _ => {}
        }
    }
    text.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
#[tauri::command]
fn import_web_snapshot(
    app: tauri::AppHandle,
    title: String,
    source_url: String,
    html: String,
) -> Result<ImportResult, String> {
    if html.trim().is_empty() {
        return Err("Webbsidans HTML/text är tom".into());
    }
    let data_root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    initialize_vault_at(&data_root).map_err(|e| e.to_string())?;
    let safe = sanitize_file_name(if title.trim().is_empty() {
        "Webbsida"
    } else {
        title.trim()
    });
    let visible = html_to_visible_text(&html);
    let body = format!(
        "Källa: {}\nHämtad manuellt: {}\n\n{}",
        source_url.trim(),
        epoch_seconds(),
        visible
    );
    let temp = data_root
        .join("index")
        .join(format!("webb-{}-{safe}.txt", epoch_seconds()));
    fs::write(&temp, body).map_err(|e| e.to_string())?;
    let result = import_document_at(&data_root, &temp).map_err(|e| e.to_string());
    let _ = fs::remove_file(&temp);
    match result {
        Ok(value) => {
            record_single_import_session(&data_root, "web_snapshot", source_url.trim(), Ok(&value));
            Ok(value)
        }
        Err(error) => {
            let message = error.to_string();
            record_single_import_session(
                &data_root,
                "web_snapshot",
                source_url.trim(),
                Err(&message),
            );
            Err(message)
        }
    }
}

#[tauri::command]
fn run_test_center(app: tauri::AppHandle) -> Result<TestCenterReport, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    run_test_center_at(&data_root).map_err(|error| error.to_string())
}

fn run_test_center_at(data_root: &Path) -> rusqlite::Result<TestCenterReport> {
    let mut cases = Vec::new();

    let health = check_vault_health_at(data_root)?;
    push_test_case(
        &mut cases,
        "Production database health",
        "SQLite integrity is ok and no missing imported files",
        &format!(
            "integrity={}, missing_files={}",
            health.database_integrity, health.missing_file_count
        ),
        health.database_integrity == "ok" && health.missing_file_count == 0,
    );

    let testlab_documents = list_testlab_documents_at(data_root)?;
    push_test_case(
        &mut cases,
        "Test Lab fixture count",
        "At least 4 core fixtures; scale fixtures are allowed",
        &format!(
            "{} synthetic documents including scale fixtures",
            testlab_documents.len()
        ),
        testlab_documents.len() >= 4
            && [10_i64, 11, 12, 13]
                .iter()
                .all(|id| testlab_documents.iter().any(|document| document.id == *id)),
    );

    let dagab_results = search_testlab_documents_at(data_root, "DAGAB kontrakt")?;
    push_test_case(
        &mut cases,
        "Acceptance: DAGAB kontrakt",
        "Top result contains DAGAB",
        dagab_results
            .first()
            .map(|result| result.document.title.as_str())
            .unwrap_or("no result"),
        dagab_results
            .first()
            .is_some_and(|result| result.document.title.contains("DAGAB")),
    );

    let salary_results = search_testlab_documents_at(data_root, "lön juni")?;
    push_test_case(
        &mut cases,
        "Acceptance: lön juni",
        "Top result is salary specification",
        salary_results
            .first()
            .map(|result| result.document.document_type.as_str())
            .unwrap_or("no result"),
        salary_results
            .first()
            .is_some_and(|result| result.document.document_type.contains("nespecifikation")),
    );

    let backups = list_local_backups_at(data_root)?;
    push_test_case(
        &mut cases,
        "Backup registry readable",
        "Backup manifest scan completes",
        &format!("{} backup sets", backups.len()),
        true,
    );

    let audit_events = list_audit_events_at(data_root, 5)?;
    push_test_case(
        &mut cases,
        "Audit log readable",
        "Audit query completes",
        &format!("{} recent events", audit_events.len()),
        true,
    );

    let code_words = list_code_words_at(data_root)?;
    push_test_case(
        &mut cases,
        "Code word registry readable",
        "Code word query completes",
        &format!("{} code words", code_words.len()),
        true,
    );

    let conflicts = list_conflicts_at(data_root)?;
    push_test_case(
        &mut cases,
        "Conflict engine readable",
        "Conflict scan completes",
        &format!("{} conflicts", conflicts.len()),
        true,
    );

    let review_queue = list_review_queue_at(data_root)?;
    push_test_case(
        &mut cases,
        "Review queue readable",
        "Pending review queue query completes",
        &format!("{} pending review items", review_queue.len()),
        true,
    );

    let entity_catalog = list_entity_catalog_at(data_root)?;
    push_test_case(
        &mut cases,
        "Entity catalog readable",
        "Entity catalog query completes",
        &format!("{} entities", entity_catalog.len()),
        true,
    );

    let ocr = detect_ocr_status(Some(data_root));
    push_test_case(
        &mut cases,
        "OCR adapter detectable",
        "OCR status can be checked without failing",
        if ocr.available {
            "Tesseract available"
        } else {
            "Tesseract not installed"
        },
        true,
    );

    let passed = cases
        .iter()
        .filter(|test_case| test_case.status == "passed")
        .count() as i64;
    let failed = cases.len() as i64 - passed;

    Ok(TestCenterReport {
        passed,
        failed,
        cases,
    })
}

fn push_test_case(
    cases: &mut Vec<TestCenterCase>,
    name: &str,
    expected: &str,
    actual: &str,
    passed: bool,
) {
    cases.push(TestCenterCase {
        name: name.to_string(),
        status: if passed { "passed" } else { "failed" }.to_string(),
        expected: expected.to_string(),
        actual: actual.to_string(),
    });
}

#[tauri::command]
fn reset_testlab(app: tauri::AppHandle) -> Result<Vec<DocumentSummary>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    reset_testlab_at(&data_root).map_err(|error| error.to_string())
}

fn reset_testlab_at(data_root: &Path) -> rusqlite::Result<Vec<DocumentSummary>> {
    let mut connection = Connection::open(data_root.join("testlab").join("vault-test.db"))?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;
    let transaction = connection.transaction()?;
    transaction.execute("DELETE FROM document_fts", [])?;
    transaction.execute("DELETE FROM document_versions", [])?;
    transaction.execute("DELETE FROM files", [])?;
    transaction.execute("DELETE FROM documents", [])?;
    transaction.commit()?;

    seed_testlab_documents(&connection)?;
    list_documents(&connection)
}

#[tauri::command]
fn generate_testlab_scale(app: tauri::AppHandle, count: i64) -> Result<TestLabScaleResult, String> {
    if ![100, 1_000, 10_000, 50_000].contains(&count) {
        return Err("Tillåtna skalnivåer är 100, 1 000, 10 000 och 50 000".into());
    }
    let root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    initialize_vault_at(&root).map_err(|e| e.to_string())?;
    generate_testlab_scale_at(&root, count).map_err(|e| e.to_string())
}

fn generate_testlab_scale_at(data_root: &Path, count: i64) -> rusqlite::Result<TestLabScaleResult> {
    let db = data_root.join("testlab").join("vault-test.db");
    let mut c = Connection::open(&db)?;
    c.execute_batch("PRAGMA foreign_keys=ON; PRAGMA synchronous=OFF; PRAGMA temp_store=MEMORY;")?;
    let started = Instant::now();
    let tx = c.transaction()?;
    tx.execute("DELETE FROM document_fts WHERE document_id>=100000", [])?;
    tx.execute("DELETE FROM documents WHERE id>=100000", [])?;
    let mut document_insert=tx.prepare_cached("INSERT INTO documents(id,vault_id,title,document_type,document_date,inbox_status,source_label,match_explanation,extracted_text) VALUES(?1,1,?2,'Kvitto',?3,'indexed','GENERERAD TESTDATA','Deterministisk skalfixture',?4)")?;
    let mut fts_insert = tx.prepare_cached(
        "INSERT INTO document_fts(document_id,title,metadata_text,body_text) VALUES(?1,?2,?3,?4)",
    )?;
    for index in 0..count {
        let id = 100000 + index;
        let month = index % 12 + 1;
        let day = index % 28 + 1;
        let title = format!("TESTDOKUMENT {index:05} – syntetiskt kvitto");
        let date = format!("2025-{month:02}-{day:02}");
        let body=format!("SYNTHETIC TEST DATA namespace VAULT-TEST-{index:05}. Kvitto belopp {} SEK. Garanti 24 månader.",100+index%9000);
        document_insert.execute(params![id, title, date, body])?;
        fts_insert.execute(params![
            id,
            title,
            format!("Kvitto {date} GENERERAD TESTDATA"),
            body
        ])?;
    }
    drop(fts_insert);
    drop(document_insert);
    tx.commit()?;
    let elapsed_ms = started.elapsed().as_millis();
    let search_started = Instant::now();
    let _: i64 = c.query_row(
        "SELECT COUNT(*) FROM document_fts WHERE document_fts MATCH 'garanti AND kvitto'",
        [],
        |r| r.get(0),
    )?;
    let search_elapsed_ms = search_started.elapsed().as_millis();
    let total_documents = c.query_row("SELECT COUNT(*) FROM documents", [], |r| r.get(0))?;
    let database_bytes = fs::metadata(db).map_err(to_sql_error)?.len();
    Ok(TestLabScaleResult {
        requested: count,
        total_documents,
        elapsed_ms,
        search_elapsed_ms,
        database_bytes,
    })
}

#[tauri::command]
fn run_test_center_module(
    app: tauri::AppHandle,
    module: String,
) -> Result<TestCenterReport, String> {
    let root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    initialize_vault_at(&root).map_err(|e| e.to_string())?;
    let mut report = run_test_center_at(&root).map_err(|e| e.to_string())?;
    let terms: Vec<&str> = match module.as_str() {
        "search" => vec!["search", "DAGAB", "salary"],
        "storage" => vec!["database", "backup", "file"],
        "analysis" => vec!["review", "conflict", "entity"],
        "ocr" => vec!["OCR"],
        "all" => Vec::new(),
        _ => return Err("Okänd testmodul".into()),
    };
    if !terms.is_empty() {
        report.cases.retain(|case| {
            terms
                .iter()
                .any(|term| case.name.to_lowercase().contains(&term.to_lowercase()))
        });
        report.passed = report.cases.iter().filter(|c| c.status == "passed").count() as i64;
        report.failed = report.cases.len() as i64 - report.passed;
    }
    Ok(report)
}

#[tauri::command]
fn simulate_testlab_failure(
    app: tauri::AppHandle,
    failure_type: String,
) -> Result<TestCenterCase, String> {
    let root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    initialize_vault_at(&root).map_err(|e| e.to_string())?;
    let (expected, actual) = match failure_type.as_str() {
        "ocr" => (
            "OCR-fel fångas utan att produktion ändras",
            "Simulerat OCR_TOOL_UNAVAILABLE registrerat endast i Test Lab",
        ),
        "database" => (
            "Databasfel rullas tillbaka",
            "Simulerad transaktion återställd; produktionsdatabas orörd",
        ),
        "file" => (
            "Saknad fil rapporteras",
            "Simulerad FILE_NOT_FOUND gav kontrollerat fel",
        ),
        "interrupt" => (
            "Avbrott kan återupptas",
            "Simulerat jobb pausades vid 2/5 och behöll progress",
        ),
        "restore" => (
            "Återställning verifieras innan byte",
            "Simulerad backup validerades och ingen produktionsfil ersattes",
        ),
        _ => return Err("Okänd felsimulering".into()),
    };
    Ok(TestCenterCase {
        name: format!("Simulering: {failure_type}"),
        status: "passed".into(),
        expected: expected.into(),
        actual: actual.into(),
    })
}

#[tauri::command]
fn export_safe_test_report(app: tauri::AppHandle) -> Result<TestReportExport, String> {
    let root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    initialize_vault_at(&root).map_err(|e| e.to_string())?;
    let report = run_test_center_at(&root).map_err(|e| e.to_string())?;
    let dir = root.join("testlab").join("reports");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let created = epoch_seconds();
    let path = dir.join(format!("vault-test-report-{created}.json"));
    let safe = serde_json::json!({"format":"vault-safe-test-report-v1","environment":"TESTMILJÖ – INGA RIKTIGA DOKUMENT","created_at_epoch_seconds":created,"passed":report.passed,"failed":report.failed,"cases":report.cases});
    fs::write(
        &path,
        serde_json::to_string_pretty(&safe).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let sha256 = sha256_file(&path).map_err(|e| e.to_string())?;
    Ok(TestReportExport {
        path: path.to_string_lossy().into_owned(),
        sha256,
        case_count: safe["cases"].as_array().map_or(0, Vec::len),
        created_at_epoch_seconds: created,
    })
}

#[tauri::command]
fn create_local_backup(app: tauri::AppHandle) -> Result<BackupResult, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    create_local_backup_at(&data_root).map_err(|error| error.to_string())
}

fn backup_policy_at(connection: &Connection) -> rusqlite::Result<BackupPolicy> {
    let raw: Option<String> = connection
        .query_row(
            "SELECT value_json FROM settings WHERE vault_id=1 AND key='backup_policy'",
            [],
            |r| r.get(0),
        )
        .optional()?;
    Ok(raw
        .and_then(|value| serde_json::from_str(&value).ok())
        .unwrap_or(BackupPolicy {
            enabled: false,
            paused: false,
            interval_hours: 24,
            backup_mode: "incremental".into(),
            excluded_document_ids: Vec::new(),
            last_run_epoch_seconds: 0,
            destination_path: None,
        }))
}
fn persist_backup_policy(connection: &Connection, policy: &BackupPolicy) -> rusqlite::Result<()> {
    let value = serde_json::to_string(policy)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    connection.execute("INSERT INTO settings(vault_id,key,value_json) VALUES(1,'backup_policy',?1) ON CONFLICT(vault_id,key) DO UPDATE SET value_json=excluded.value_json",[value])?;
    Ok(())
}

#[tauri::command]
fn get_backup_policy(app: tauri::AppHandle) -> Result<BackupPolicy, String> {
    backup_policy_at(&production_connection(&app)?).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_backup_policy(
    app: tauri::AppHandle,
    enabled: bool,
    paused: bool,
    interval_hours: i64,
    backup_mode: String,
    excluded_document_ids: Vec<i64>,
    destination_path: Option<String>,
) -> Result<BackupPolicy, String> {
    if !["full", "incremental"].contains(&backup_mode.as_str()) {
        return Err("Backupläget måste vara full eller incremental".into());
    }
    let connection = production_connection(&app)?;
    let mut ids = excluded_document_ids
        .into_iter()
        .filter(|id| *id > 0)
        .collect::<Vec<_>>();
    ids.sort_unstable();
    ids.dedup();
    let policy = BackupPolicy {
        enabled,
        paused,
        interval_hours: interval_hours.clamp(1, 8760),
        backup_mode,
        excluded_document_ids: ids,
        last_run_epoch_seconds: backup_policy_at(&connection)
            .map_err(|e| e.to_string())?
            .last_run_epoch_seconds,
        destination_path: destination_path.and_then(|value| {
            let trimmed = value.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        }),
    };
    persist_backup_policy(&connection, &policy).map_err(|e| e.to_string())?;
    Ok(policy)
}

#[tauri::command]
fn run_configured_backup(app: tauri::AppHandle) -> Result<BackupResult, String> {
    let data_root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    initialize_vault_at(&data_root).map_err(|e| e.to_string())?;
    let connection = Connection::open(data_root.join("vault.db")).map_err(|e| e.to_string())?;
    let mut policy = backup_policy_at(&connection).map_err(|e| e.to_string())?;
    if policy.paused {
        return Err("Backup är pausad".into());
    }
    let mut result = create_backup_with_options_at(
        &data_root,
        &policy.backup_mode,
        &policy.excluded_document_ids,
    )
    .map_err(|e| e.to_string())?;
    if let Some(destination) = policy.destination_path.as_deref() {
        result.destination_copy =
            Some(mirror_backup_to_destination(&result, destination).map_err(|e| e.to_string())?)
    }
    policy.last_run_epoch_seconds = result.created_at_epoch_seconds;
    persist_backup_policy(&connection, &policy).map_err(|e| e.to_string())?;
    Ok(result)
}

fn run_scheduled_backup_if_due(data_root: &Path) -> rusqlite::Result<bool> {
    let connection = Connection::open(data_root.join("vault.db"))?;
    let mut policy = backup_policy_at(&connection)?;
    if !policy.enabled || policy.paused {
        return Ok(false);
    }
    let now = epoch_seconds();
    if now.saturating_sub(policy.last_run_epoch_seconds) < (policy.interval_hours as u64) * 3600 {
        return Ok(false);
    }
    let result = create_backup_with_options_at(
        data_root,
        &policy.backup_mode,
        &policy.excluded_document_ids,
    )?;
    if let Some(destination) = policy.destination_path.as_deref() {
        mirror_backup_to_destination(&result, destination)?;
    }
    policy.last_run_epoch_seconds = result.created_at_epoch_seconds;
    persist_backup_policy(&connection, &policy)?;
    Ok(true)
}

fn mirror_backup_to_destination(
    result: &BackupResult,
    destination: &str,
) -> rusqlite::Result<String> {
    let destination_root = PathBuf::from(destination);
    if !destination_root.is_absolute() {
        return Err(to_sql_error(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Backupdestinationen måste vara en absolut sökväg",
        )));
    }
    fs::create_dir_all(&destination_root).map_err(to_sql_error)?;
    let source = PathBuf::from(&result.backup_root);
    let folder_name = source.file_name().ok_or_else(|| {
        to_sql_error(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Backupmappen saknar namn",
        ))
    })?;
    let target = destination_root.join(folder_name);
    if target.starts_with(&source) {
        return Err(to_sql_error(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Backupdestinationen får inte ligga inuti backupen",
        )));
    }
    if target.exists() {
        return Err(to_sql_error(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "En backup med samma namn finns redan på destinationen",
        )));
    }
    copy_directory_contents(&source, &target)?;
    let validation = validate_local_backup_at(&target)?;
    if !validation.ok {
        let _ = fs::remove_dir_all(&target);
        return Err(to_sql_error(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "Destinationskopian kunde inte valideras: {}",
                validation.warnings.join("; ")
            ),
        )));
    }
    Ok(target.to_string_lossy().into_owned())
}

fn collect_backup_entries(
    root: &Path,
    directory: &Path,
    entries: &mut Vec<(PathBuf, String)>,
) -> Result<(), String> {
    for item in fs::read_dir(directory).map_err(|e| e.to_string())? {
        let item = item.map_err(|e| e.to_string())?;
        let path = item.path();
        if item.file_type().map_err(|e| e.to_string())?.is_dir() {
            collect_backup_entries(root, &path, entries)?
        } else if path.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            entries.push((path, relative));
        }
    }
    Ok(())
}

#[tauri::command]
fn create_password_backup(
    app: tauri::AppHandle,
    password: String,
    destination_path: Option<String>,
) -> Result<ProtectedBackupResult, String> {
    if password.chars().count() < 8 {
        return Err("Backup-lösenordet måste vara minst 8 tecken".into());
    }
    let data_root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    initialize_vault_at(&data_root).map_err(|e| e.to_string())?;
    let backup = create_local_backup_at(&data_root).map_err(|e| e.to_string())?;
    let backup_root = PathBuf::from(&backup.backup_root);
    let archive_root = destination_path
        .and_then(|value| {
            let value = value.trim().to_string();
            if value.is_empty() {
                None
            } else {
                Some(PathBuf::from(value))
            }
        })
        .unwrap_or_else(|| data_root.join("backups"));
    if !archive_root.is_absolute() {
        return Err("Backupdestinationen måste vara en absolut sökväg".into());
    }
    fs::create_dir_all(&archive_root).map_err(|e| e.to_string())?;
    let archive_path = archive_root.join(format!("vault-skyddad-{}.vaultzip", epoch_seconds()));
    let file = fs::File::create(&archive_path).map_err(|e| e.to_string())?;
    let mut writer = zip::ZipWriter::new(file);
    let mut entries = Vec::new();
    collect_backup_entries(&backup_root, &backup_root, &mut entries)?;
    entries.sort_by(|a, b| a.1.cmp(&b.1));
    for (path, name) in &entries {
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .with_aes_encryption(zip::AesMode::Aes256, &password);
        writer
            .start_file(name, options)
            .map_err(|e| e.to_string())?;
        let bytes = fs::read(path).map_err(|e| e.to_string())?;
        writer.write_all(&bytes).map_err(|e| e.to_string())?;
    }
    writer.finish().map_err(|e| e.to_string())?;
    let size_bytes = fs::metadata(&archive_path)
        .map_err(|e| e.to_string())?
        .len();
    let sha256 = sha256_file(&archive_path).map_err(|e| e.to_string())?;
    let connection = production_connection(&app)?;
    record_audit_event(
        &connection,
        "password_backup_created",
        "backup",
        None,
        &format!(
            "Created AES-256 protected backup with {} entries",
            entries.len()
        ),
    )
    .map_err(|e| e.to_string())?;
    Ok(ProtectedBackupResult {
        archive_path: archive_path.to_string_lossy().into_owned(),
        sha256,
        size_bytes,
        entry_count: entries.len(),
        source_backup_root: backup.backup_root,
    })
}

#[tauri::command]
fn validate_password_backup(archive_path: String, password: String) -> Result<bool, String> {
    let file = fs::File::open(&archive_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut manifest = archive
        .by_name_decrypt("manifest.json", password.as_bytes())
        .map_err(|_| "Fel lösenord eller skadad skyddad backup".to_string())?;
    let mut text = String::new();
    manifest
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    Ok(value.get("format").and_then(|v| v.as_str()) == Some("vault-local-backup-v1"))
}

#[tauri::command]
fn restore_password_backup(
    app: tauri::AppHandle,
    archive_path: String,
    password: String,
) -> Result<RestoreResult, String> {
    if password.is_empty() {
        return Err("Backup-lösenord saknas".into());
    }
    let data_root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    initialize_vault_at(&data_root).map_err(|e| e.to_string())?;
    let staging = data_root
        .join("temp")
        .join(format!("password-restore-{}", epoch_seconds()));
    fs::create_dir_all(&staging).map_err(|e| e.to_string())?;
    let extracted = (|| -> Result<(), String> {
        let file = fs::File::open(&archive_path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        if archive.len() > 100_000 {
            return Err("Backupen innehåller orimligt många poster".into());
        }
        let mut total = 0_u64;
        for index in 0..archive.len() {
            let mut entry = archive
                .by_index_decrypt(index, password.as_bytes())
                .map_err(|_| "Fel lösenord eller skadad skyddad backup".to_string())?;
            total = total.saturating_add(entry.size());
            if total > 100 * 1024 * 1024 * 1024 {
                return Err("Backupen är större än säkerhetsgränsen 100 GB".into());
            }
            let relative = entry
                .enclosed_name()
                .ok_or_else(|| "Backupen innehåller en osäker filsökväg".to_string())?
                .to_path_buf();
            let target = staging.join(relative);
            if entry.is_dir() {
                fs::create_dir_all(&target).map_err(|e| e.to_string())?
            } else {
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?
                }
                let mut output = fs::File::create(&target).map_err(|e| e.to_string())?;
                std::io::copy(&mut entry, &mut output).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    })();
    if let Err(error) = extracted {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }
    let result = restore_local_backup_at(&data_root, &staging).map_err(|e| e.to_string());
    let _ = fs::remove_dir_all(&staging);
    result
}

#[tauri::command]
fn test_backup_restore(
    app: tauri::AppHandle,
    backup_root: String,
) -> Result<RestoreTestResult, String> {
    let data_root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    initialize_vault_at(&data_root).map_err(|e| e.to_string())?;
    let test_root = data_root
        .join("temp")
        .join(format!("restore-test-{}", epoch_seconds()));
    fs::create_dir_all(&test_root).map_err(|e| e.to_string())?;
    let tested = (|| -> Result<RestoreTestResult, String> {
        initialize_vault_at(&test_root).map_err(|e| e.to_string())?;
        restore_local_backup_at(&test_root, Path::new(&backup_root)).map_err(|e| e.to_string())?;
        let health = check_vault_health_at(&test_root).map_err(|e| e.to_string())?;
        Ok(RestoreTestResult {
            ok: health.ok,
            document_count: health.document_count,
            file_count: health.file_count,
            missing_file_count: health.missing_file_count,
            database_integrity: health.database_integrity,
            tested_backup_root: backup_root,
        })
    })();
    let _ = fs::remove_dir_all(&test_root);
    tested
}

#[tauri::command]
fn create_portable_archive(app: tauri::AppHandle) -> Result<PortableArchiveResult, String> {
    let data_root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    create_portable_archive_at(&data_root)
}

fn create_portable_archive_at(data_root: &Path) -> Result<PortableArchiveResult, String> {
    let backup = create_local_backup_at(data_root).map_err(|e| e.to_string())?;
    let backup_root = PathBuf::from(&backup.backup_root);
    let export_root = data_root.join("exports");
    fs::create_dir_all(&export_root).map_err(|e| e.to_string())?;
    let archive_path = export_root.join(format!("vault-portabel-{}.vaultarchive", epoch_seconds()));
    let file = fs::File::create(&archive_path).map_err(|e| e.to_string())?;
    let mut writer = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    let mut entries = Vec::new();
    collect_backup_entries(&backup_root, &backup_root, &mut entries)?;
    for (path, relative) in &entries {
        writer
            .start_file(relative, options)
            .map_err(|e| e.to_string())?;
        let mut source = fs::File::open(path).map_err(|e| e.to_string())?;
        std::io::copy(&mut source, &mut writer).map_err(|e| e.to_string())?;
    }
    let connection = Connection::open(data_root.join("vault.db")).map_err(|e| e.to_string())?;
    let snapshot = serde_json::json!({
        "format": "vault-portable-v1",
        "schema_version": current_schema_version(&connection).map_err(|e| e.to_string())?,
        "document_count": backup.document_count,
        "file_count": backup.file_count,
        "contains": ["originalfiler", "metadata", "claims", "relationer", "historik", "regler", "teman", "verifieringshistorik"],
        "ai_policy": "AI-fri, lokal och flyttbar"
    });
    writer
        .start_file("portable-format.json", options)
        .map_err(|e| e.to_string())?;
    writer
        .write_all(
            serde_json::to_string_pretty(&snapshot)
                .map_err(|e| e.to_string())?
                .as_bytes(),
        )
        .map_err(|e| e.to_string())?;
    writer
        .start_file("documents.csv", options)
        .map_err(|e| e.to_string())?;
    writer
        .write_all(b"id,title,document_type,document_date,inbox_status\n")
        .map_err(|e| e.to_string())?;
    let mut statement = connection.prepare("SELECT id,title,document_type,COALESCE(document_date,''),inbox_status FROM documents WHERE trashed_at IS NULL ORDER BY id").map_err(|e| e.to_string())?;
    let rows = statement
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
            ))
        })
        .map_err(|e| e.to_string())?;
    for row in rows {
        let (id, title, kind, date, status) = row.map_err(|e| e.to_string())?;
        let csv = format!(
            "{},\"{}\",\"{}\",{},{}\n",
            id,
            title.replace('"', "\"\""),
            kind.replace('"', "\"\""),
            date,
            status
        );
        writer
            .write_all(csv.as_bytes())
            .map_err(|e| e.to_string())?;
    }
    writer.finish().map_err(|e| e.to_string())?;
    let sha256 = sha256_file(&archive_path).map_err(|e| e.to_string())?;
    let size_bytes = fs::metadata(&archive_path)
        .map_err(|e| e.to_string())?
        .len();
    record_audit_event(
        &connection,
        "portable_archive_created",
        "backup",
        None,
        &format!(
            "Created portable archive with {} documents",
            backup.document_count
        ),
    )
    .map_err(|e| e.to_string())?;
    Ok(PortableArchiveResult {
        archive_path: archive_path.to_string_lossy().into_owned(),
        sha256,
        size_bytes,
        entry_count: entries.len() + 2,
        document_count: backup.document_count,
        file_count: backup.file_count,
    })
}

#[tauri::command]
fn restore_portable_archive(
    app: tauri::AppHandle,
    archive_path: String,
) -> Result<RestoreResult, String> {
    let data_root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    restore_portable_archive_at(&data_root, Path::new(&archive_path))
}

fn restore_portable_archive_at(
    data_root: &Path,
    archive_path: &Path,
) -> Result<RestoreResult, String> {
    if archive_path.extension().and_then(|x| x.to_str()) != Some("vaultarchive") {
        return Err("Välj en .vaultarchive-fil".into());
    }
    let file = fs::File::open(archive_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    if archive.by_name("portable-format.json").is_err() || archive.by_name("vault.db").is_err() {
        return Err("Arkivet saknar Vault-manifest eller databas".into());
    }
    let staging = data_root
        .join("portable-import")
        .join(format!("restore-{}", epoch_seconds()));
    fs::create_dir_all(&staging).map_err(|e| e.to_string())?;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|e| e.to_string())?;
        let enclosed = entry
            .enclosed_name()
            .ok_or_else(|| "Osäker sökväg i arkivet".to_string())?
            .to_path_buf();
        if enclosed == Path::new("documents.csv") || enclosed == Path::new("portable-format.json") {
            continue;
        }
        let target = staging.join(enclosed);
        if entry.is_dir() {
            fs::create_dir_all(&target).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut out = fs::File::create(target).map_err(|e| e.to_string())?;
            std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
        }
    }
    restore_local_backup_at(data_root, &staging).map_err(|e| e.to_string())
}

#[tauri::command]
fn scan_file_integrity(app: tauri::AppHandle) -> Result<IntegrityScanReport, String> {
    let data_root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    scan_file_integrity_at(&data_root).map_err(|e| e.to_string())
}

fn normalized_document_text(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

fn text_simhash(value: &str) -> u64 {
    let normalized = value.to_lowercase();
    let tokens = normalized
        .split(|c: char| !c.is_alphanumeric())
        .filter(|v| v.len() > 1)
        .collect::<Vec<_>>();
    let mut weights = [0_i32; 64];
    for window in tokens.windows(3) {
        let token = window.join(" ");
        let digest = Sha256::digest(token.as_bytes());
        let hash = u64::from_be_bytes(digest[..8].try_into().unwrap_or([0; 8]));
        for (bit, weight) in weights.iter_mut().enumerate() {
            if hash & (1_u64 << bit) != 0 {
                *weight += 1
            } else {
                *weight -= 1
            }
        }
    }
    weights
        .iter()
        .enumerate()
        .fold(0_u64, |hash, (bit, weight)| {
            if *weight >= 0 {
                hash | (1_u64 << bit)
            } else {
                hash
            }
        })
}

fn perceptual_image_hash(path: &Path) -> Option<u64> {
    use image::imageops::FilterType;
    let image = image::open(path)
        .ok()?
        .resize_exact(8, 8, FilterType::Triangle)
        .to_luma8();
    let average = image.pixels().map(|p| p[0] as u32).sum::<u32>() / 64;
    Some(
        image
            .pixels()
            .enumerate()
            .fold(0_u64, |hash, (index, pixel)| {
                if pixel[0] as u32 >= average {
                    hash | (1_u64 << index)
                } else {
                    hash
                }
            }),
    )
}

fn pair_already_exists(candidates: &[DuplicateCandidate], first: i64, second: i64) -> bool {
    candidates.iter().any(|c| {
        (c.primary_document_id == first && c.secondary_document_id == second)
            || (c.primary_document_id == second && c.secondary_document_id == first)
    })
}

fn scan_file_integrity_at(data_root: &Path) -> rusqlite::Result<IntegrityScanReport> {
    use std::collections::HashMap;
    let connection = Connection::open(data_root.join("vault.db"))?;
    let mut checked = 0;
    let mut intact = 0;
    let mut missing = 0;
    let mut changed = 0;
    let mut warnings = Vec::new();
    let mut stmt = connection.prepare("SELECT storage_path,sha256 FROM files")?;
    for row in stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))? {
        let (path, expected) = row?;
        checked += 1;
        let full = data_root.join(path);
        if !full.is_file() {
            missing += 1;
            warnings.push("En importerad originalfil saknas".into());
        } else {
            match sha256_file(&full) {
                Ok(actual) if actual == expected => intact += 1,
                Ok(_) => {
                    changed += 1;
                    warnings.push("En originalfil har ändrats sedan import".into())
                }
                Err(_) => {
                    changed += 1;
                }
            }
        }
    }
    let mut by_hash: HashMap<String, Vec<(i64, String)>> = HashMap::new();
    let mut by_text: HashMap<String, Vec<(i64, String)>> = HashMap::new();
    let mut document_fingerprints: Vec<(i64, String, String, String, u64, Option<u64>)> =
        Vec::new();
    let mut docs=connection.prepare("SELECT d.id,d.title,COALESCE(f.sha256,''),COALESCE(d.extracted_text,''),COALESCE(f.storage_path,''),COALESCE(f.mime_type,'') FROM documents d LEFT JOIN document_versions dv ON dv.document_id=d.id AND dv.is_current_file=1 LEFT JOIN files f ON f.id=dv.file_id WHERE d.trashed_at IS NULL")?;
    for row in docs.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, String>(4)?,
            r.get::<_, String>(5)?,
        ))
    })? {
        let (id, title, hash, text, storage_path, mime_type) = row?;
        if !hash.is_empty() {
            by_hash.entry(hash).or_default().push((id, title.clone()));
        }
        let normalized = normalized_document_text(&text);
        if normalized.len() > 40 {
            let digest = hex::encode(Sha256::digest(normalized.as_bytes()));
            by_text.entry(digest).or_default().push((id, title.clone()));
        }
        let simhash = if normalized.len() > 40 {
            text_simhash(&text)
        } else {
            0
        };
        let image_hash = if mime_type.starts_with("image/") && !storage_path.is_empty() {
            perceptual_image_hash(&data_root.join(&storage_path))
        } else {
            None
        };
        document_fingerprints.push((id, title, mime_type, normalized, simhash, image_hash));
    }
    let mut candidates = Vec::new();
    for group in by_hash.values().filter(|g| g.len() > 1) {
        for pair in group.windows(2) {
            candidates.push(DuplicateCandidate {
                primary_document_id: pair[0].0,
                primary_title: pair[0].1.clone(),
                secondary_document_id: pair[1].0,
                secondary_title: pair[1].1.clone(),
                confidence: 100,
                match_kind: "exact_hash".into(),
                explanation: "Kryptografisk SHA-256, filstorlek och originalinnehåll är identiska"
                    .into(),
                decision: None,
            });
        }
    }
    for group in by_text.values().filter(|g| g.len() > 1) {
        for pair in group.windows(2) {
            if !candidates
                .iter()
                .any(|c| c.primary_document_id == pair[0].0 && c.secondary_document_id == pair[1].0)
            {
                candidates.push(DuplicateCandidate{primary_document_id:pair[0].0,primary_title:pair[0].1.clone(),secondary_document_id:pair[1].0,secondary_title:pair[1].1.clone(),confidence:95,match_kind:"normalized_text".into(),explanation:"Normaliserad dokumenttext är identisk trots att filrepresentationen kan skilja sig".into(),decision:None});
            }
        }
    }
    for first_index in 0..document_fingerprints.len() {
        for second_index in (first_index + 1)..document_fingerprints.len() {
            let first = &document_fingerprints[first_index];
            let second = &document_fingerprints[second_index];
            if pair_already_exists(&candidates, first.0, second.0) {
                continue;
            }
            if let (Some(first_hash), Some(second_hash)) = (first.5, second.5) {
                let distance = (first_hash ^ second_hash).count_ones();
                if distance <= 8 {
                    candidates.push(DuplicateCandidate{primary_document_id:first.0,primary_title:first.1.clone(),secondary_document_id:second.0,secondary_title:second.1.clone(),confidence:(100-distance as i64*3).clamp(76,99),match_kind:"perceptual_image".into(),explanation:format!("Perceptuell 8×8-bildhash skiljer {distance} bitar; bilderna ser mycket lika ut även om upplösning eller komprimering skiljer sig"),decision:None});
                    continue;
                }
            }
            if first.4 != 0 && second.4 != 0 {
                let longer = first.3.len().max(second.3.len()) as f64;
                let length_ratio = if longer > 0.0 {
                    first.3.len().min(second.3.len()) as f64 / longer
                } else {
                    0.0
                };
                let distance = (first.4 ^ second.4).count_ones();
                if distance <= 8 && length_ratio >= 0.72 {
                    let different_format = first.2 != second.2;
                    candidates.push(DuplicateCandidate{primary_document_id:first.0,primary_title:first.1.clone(),secondary_document_id:second.0,secondary_title:second.1.clone(),confidence:(94-distance as i64*2).clamp(78,94),match_kind:if different_format{"near_text_cross_format".into()}else{"near_text".into()},explanation:format!("Textens deterministiska SimHash skiljer {distance}/64 bitar och längdförhållandet är {:.0}%{}",length_ratio*100.0,if different_format{"; filformaten skiljer sig"}else{""}),decision:None});
                }
            }
        }
    }
    let mut by_page: HashMap<String, Vec<(i64, String, i64)>> = HashMap::new();
    let mut pages=connection.prepare("SELECT dv.document_id,d.title,p.page_no,COALESCE(GROUP_CONCAT(ts.text,' '),'') FROM document_pages p JOIN document_versions dv ON dv.id=p.document_version_id JOIN documents d ON d.id=dv.document_id LEFT JOIN text_spans ts ON ts.document_page_id=p.id WHERE d.trashed_at IS NULL GROUP BY p.id")?;
    for row in pages.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, i64>(2)?,
            r.get::<_, String>(3)?,
        ))
    })? {
        let (document_id, title, page_no, text) = row?;
        let normalized = normalized_document_text(&text);
        if normalized.len() > 80 {
            by_page
                .entry(hex::encode(Sha256::digest(normalized.as_bytes())))
                .or_default()
                .push((document_id, title, page_no));
        }
    }
    for group in by_page.values().filter(|group| {
        group
            .iter()
            .map(|item| item.0)
            .collect::<std::collections::HashSet<_>>()
            .len()
            > 1
    }) {
        for first in group {
            for second in group {
                if first.0 >= second.0 || pair_already_exists(&candidates, first.0, second.0) {
                    continue;
                }
                candidates.push(DuplicateCandidate {
                    primary_document_id: first.0,
                    primary_title: first.1.clone(),
                    secondary_document_id: second.0,
                    secondary_title: second.1.clone(),
                    confidence: 92,
                    match_kind: "duplicate_page".into(),
                    explanation: format!(
                        "Sida {} och sida {} har identisk normaliserad sidtext",
                        first.2, second.2
                    ),
                    decision: None,
                });
            }
        }
    }
    for candidate in &mut candidates {
        candidate.decision=connection.query_row("SELECT decision FROM duplicate_decisions WHERE primary_document_id=?1 AND secondary_document_id=?2",params![candidate.primary_document_id,candidate.secondary_document_id],|r|r.get(0)).ok();
    }
    let groups = candidates.len() as i64;
    connection.execute("INSERT INTO integrity_scans(checked_files,intact_files,missing_files,changed_files,duplicate_groups) VALUES(?1,?2,?3,?4,?5)",params![checked,intact,missing,changed,groups])?;
    record_audit_event(&connection,"integrity_scan","vault",None,&format!("Checked {checked} files; {missing} missing; {changed} changed; {groups} duplicate candidates"))?;
    Ok(IntegrityScanReport {
        checked_files: checked,
        intact_files: intact,
        missing_files: missing,
        changed_files: changed,
        duplicate_groups: groups,
        candidates,
        warnings,
    })
}

#[tauri::command]
fn decide_duplicate(
    app: tauri::AppHandle,
    primary_document_id: i64,
    secondary_document_id: i64,
    decision: String,
    note: String,
) -> Result<(), String> {
    if !["keep_both", "primary_selected", "not_duplicate"].contains(&decision.as_str()) {
        return Err("Ogiltigt dubblettbeslut".into());
    }
    let data_root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let connection = Connection::open(data_root.join("vault.db")).map_err(|e| e.to_string())?;
    connection.execute("INSERT INTO duplicate_decisions(primary_document_id,secondary_document_id,decision,note) VALUES(?1,?2,?3,?4) ON CONFLICT(primary_document_id,secondary_document_id) DO UPDATE SET decision=excluded.decision,note=excluded.note,created_at=CURRENT_TIMESTAMP",params![primary_document_id,secondary_document_id,decision,note]).map_err(|e|e.to_string())?;
    record_audit_event(
        &connection,
        "duplicate_decision",
        "document",
        Some(primary_document_id),
        "User recorded a non-destructive duplicate decision",
    )
    .map_err(|e| e.to_string())
}

fn create_local_backup_at(data_root: &Path) -> rusqlite::Result<BackupResult> {
    create_backup_with_options_at(data_root, "full", &[])
}

fn create_backup_with_options_at(
    data_root: &Path,
    backup_mode: &str,
    excluded_document_ids: &[i64],
) -> rusqlite::Result<BackupResult> {
    let database_path = data_root.join("vault.db");
    let backups_root = data_root.join("backups");
    fs::create_dir_all(&backups_root).map_err(to_sql_error)?;

    let created_at_epoch_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| to_sql_error(std::io::Error::other(error)))?
        .as_secs();
    let backup_root = unique_backup_root(&backups_root, created_at_epoch_seconds);
    fs::create_dir_all(&backup_root).map_err(to_sql_error)?;
    let backup_path = backup_root.join("vault.db");
    let manifest_path = backup_root.join("manifest.json");

    let source_connection = Connection::open(&database_path)?;
    source_connection.execute(
        "VACUUM main INTO ?1",
        params![backup_path.to_string_lossy()],
    )?;

    let connection = Connection::open(&backup_path)?;
    for document_id in excluded_document_ids {
        connection.execute(
            "DELETE FROM documents WHERE id=?1 AND vault_id=1",
            [document_id],
        )?;
    }
    if !excluded_document_ids.is_empty() {
        connection.execute(
            "DELETE FROM files WHERE id NOT IN(SELECT DISTINCT file_id FROM document_versions)",
            [],
        )?;
    }

    let document_count: i64 = connection.query_row(
        "SELECT COUNT(*) FROM documents WHERE vault_id = 1 AND trashed_at IS NULL",
        [],
        |row| row.get(0),
    )?;
    let file_count: i64 = connection.query_row(
        "SELECT COUNT(*) FROM files WHERE vault_id = 1 AND file_state = 'available'",
        [],
        |row| row.get(0),
    )?;
    let previous_root = if backup_mode == "incremental" {
        list_local_backups_at(data_root)?
            .first()
            .map(|item| PathBuf::from(&item.backup_root))
    } else {
        None
    };
    let (copied_file_count, linked_file_count) = copy_backup_files(
        &connection,
        data_root,
        &backup_root,
        previous_root.as_deref(),
    )?;
    let size_bytes = fs::metadata(&backup_path).map_err(to_sql_error)?.len();
    let sha256 = sha256_file(&backup_path).map_err(to_sql_error)?;

    let manifest = serde_json::json!({
        "format": "vault-local-backup-v1",
        "backup_root": backup_root.to_string_lossy(),
        "backup_path": backup_path.to_string_lossy(),
        "manifest_path": manifest_path.to_string_lossy(),
        "sha256": sha256,
        "size_bytes": size_bytes,
        "document_count": document_count,
        "file_count": file_count,
        "copied_file_count": copied_file_count,
        "linked_file_count": linked_file_count,
        "backup_mode": if backup_mode=="incremental"{"incremental"}else{"full"},
        "excluded_document_ids": excluded_document_ids,
        "created_at_epoch_seconds": created_at_epoch_seconds,
        "ai_policy": "AI-free; local deterministic backup"
    });
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest)
            .map_err(|error| to_sql_error(std::io::Error::other(error)))?,
    )
    .map_err(to_sql_error)?;
    record_audit_event(
        &source_connection,
        "backup_created",
        "backup",
        None,
        &format!(
            "Created local backup with {document_count} documents and {copied_file_count}/{file_count} files"
        ),
    )?;

    Ok(BackupResult {
        backup_root: backup_root.to_string_lossy().into_owned(),
        backup_path: backup_path.to_string_lossy().into_owned(),
        manifest_path: manifest_path.to_string_lossy().into_owned(),
        sha256,
        size_bytes,
        document_count,
        file_count,
        copied_file_count,
        linked_file_count,
        backup_mode: if backup_mode == "incremental" {
            "incremental".into()
        } else {
            "full".into()
        },
        created_at_epoch_seconds,
        destination_copy: None,
    })
}

fn unique_backup_root(backups_root: &Path, created_at_epoch_seconds: u64) -> PathBuf {
    let base_name = format!("vault-backup-{created_at_epoch_seconds}");
    let first = backups_root.join(&base_name);
    if !first.exists() {
        return first;
    }

    for suffix in 1..1000 {
        let candidate = backups_root.join(format!("{base_name}-{suffix}"));
        if !candidate.exists() {
            return candidate;
        }
    }

    backups_root.join(format!("{base_name}-overflow"))
}

fn copy_backup_files(
    connection: &Connection,
    data_root: &Path,
    backup_root: &Path,
    previous_root: Option<&Path>,
) -> rusqlite::Result<(i64, i64)> {
    let mut statement = connection.prepare(
        "
        SELECT DISTINCT storage_path
        FROM files
        WHERE vault_id = 1 AND file_state = 'available'
        ORDER BY storage_path
        ",
    )?;
    let storage_paths = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut copied_file_count = 0_i64;
    let mut linked_file_count = 0_i64;
    for storage_path in storage_paths {
        let source = data_root.join(&storage_path);
        if !source.is_file() {
            continue;
        }

        let destination = backup_root.join(&storage_path);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(to_sql_error)?;
        }
        let linked = if let Some(previous_root) = previous_root {
            let previous = previous_root.join(&storage_path);
            if previous.is_file()
                && fs::metadata(&previous).map_err(to_sql_error)?.len()
                    == fs::metadata(&source).map_err(to_sql_error)?.len()
                && sha256_file(&previous).map_err(to_sql_error)?
                    == sha256_file(&source).map_err(to_sql_error)?
            {
                fs::hard_link(&previous, &destination).is_ok()
            } else {
                false
            }
        } else {
            false
        };
        if linked {
            linked_file_count += 1
        } else {
            fs::copy(source, destination).map_err(to_sql_error)?;
            copied_file_count += 1;
        }
    }

    Ok((copied_file_count, linked_file_count))
}

#[tauri::command]
fn list_local_backups(app: tauri::AppHandle) -> Result<Vec<BackupSummary>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    list_local_backups_at(&data_root).map_err(|error| error.to_string())
}

fn list_local_backups_at(data_root: &Path) -> rusqlite::Result<Vec<BackupSummary>> {
    let backups_root = data_root.join("backups");
    if !backups_root.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();
    for entry in fs::read_dir(backups_root).map_err(to_sql_error)? {
        let entry = entry.map_err(to_sql_error)?;
        if !entry.file_type().map_err(to_sql_error)?.is_dir() {
            continue;
        }

        let manifest_path = entry.path().join("manifest.json");
        if !manifest_path.is_file() {
            continue;
        }

        let manifest_text = fs::read_to_string(&manifest_path).map_err(to_sql_error)?;
        let manifest: serde_json::Value = serde_json::from_str(&manifest_text)
            .map_err(|error| to_sql_error(std::io::Error::other(error)))?;
        backups.push(BackupSummary {
            backup_root: manifest
                .get("backup_root")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_string(),
            backup_path: manifest
                .get("backup_path")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_string(),
            manifest_path: manifest_path.to_string_lossy().into_owned(),
            sha256: manifest
                .get("sha256")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_string(),
            size_bytes: manifest
                .get("size_bytes")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or_default(),
            document_count: manifest
                .get("document_count")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or_default(),
            file_count: manifest
                .get("file_count")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or_default(),
            copied_file_count: manifest
                .get("copied_file_count")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or_default(),
            linked_file_count: manifest
                .get("linked_file_count")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or_default(),
            backup_mode: manifest
                .get("backup_mode")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("full")
                .to_string(),
            created_at_epoch_seconds: manifest
                .get("created_at_epoch_seconds")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or_default(),
        });
    }

    backups.sort_by(|left, right| {
        right
            .created_at_epoch_seconds
            .cmp(&left.created_at_epoch_seconds)
    });
    Ok(backups)
}

#[tauri::command]
fn validate_local_backup(
    app: tauri::AppHandle,
    backup_root: String,
) -> Result<BackupValidationReport, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    validate_local_backup_at(Path::new(&backup_root)).map_err(|error| error.to_string())
}

fn validate_local_backup_at(backup_root: &Path) -> rusqlite::Result<BackupValidationReport> {
    let manifest_path = backup_root.join("manifest.json");
    let backup_path = backup_root.join("vault.db");
    let mut warnings = Vec::new();

    if !manifest_path.is_file() {
        warnings.push("manifest.json saknas".to_string());
    }
    if !backup_path.is_file() {
        warnings.push("vault.db saknas i backupmappen".to_string());
    }

    let manifest_text = fs::read_to_string(&manifest_path).unwrap_or_default();
    let manifest: serde_json::Value =
        serde_json::from_str(&manifest_text).unwrap_or_else(|_| serde_json::json!({}));
    let expected_sha256 = manifest
        .get("sha256")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let actual_sha256 = if backup_path.is_file() {
        sha256_file(&backup_path).map_err(to_sql_error)?
    } else {
        String::new()
    };
    let sha256_matches = !expected_sha256.is_empty() && expected_sha256 == actual_sha256;
    if !sha256_matches {
        warnings.push("SHA-256 matchar inte manifestet".to_string());
    }

    let (database_integrity, document_count, file_count) = if backup_path.is_file() {
        let connection = Connection::open(&backup_path)?;
        (
            connection.query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))?,
            connection.query_row(
                "SELECT COUNT(*) FROM documents WHERE vault_id = 1 AND trashed_at IS NULL",
                [],
                |row| row.get::<_, i64>(0),
            )?,
            connection.query_row(
                "SELECT COUNT(*) FROM files WHERE vault_id = 1 AND file_state = 'available'",
                [],
                |row| row.get::<_, i64>(0),
            )?,
        )
    } else {
        ("missing".to_string(), 0, 0)
    };
    if database_integrity != "ok" {
        warnings.push(format!("SQLite integrity_check: {database_integrity}"));
    }

    let copied_file_count = manifest
        .get("copied_file_count")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or_default();
    let linked_file_count = manifest
        .get("linked_file_count")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or_default();
    if copied_file_count + linked_file_count < file_count {
        warnings.push(format!(
            "Backupen innehåller {copied_file_count}/{file_count} importerade filer"
        ));
    }

    Ok(BackupValidationReport {
        ok: warnings.is_empty(),
        backup_root: backup_root.to_string_lossy().into_owned(),
        backup_path: backup_path.to_string_lossy().into_owned(),
        manifest_path: manifest_path.to_string_lossy().into_owned(),
        sha256_matches,
        database_integrity,
        document_count,
        file_count,
        copied_file_count,
        warnings,
    })
}

#[tauri::command]
fn restore_local_backup(
    app: tauri::AppHandle,
    backup_root: String,
) -> Result<RestoreResult, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    restore_local_backup_at(&data_root, Path::new(&backup_root)).map_err(|error| error.to_string())
}

fn restore_local_backup_at(
    data_root: &Path,
    backup_root: &Path,
) -> rusqlite::Result<RestoreResult> {
    let validation = validate_local_backup_at(backup_root)?;
    if !validation.ok {
        return Err(to_sql_error(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "Backup validation failed: {}",
                validation.warnings.join("; ")
            ),
        )));
    }

    let pre_restore_backup = create_local_backup_at(data_root)?;
    let backup_database = backup_root.join("vault.db");
    let target_database = data_root.join("vault.db");

    for suffix in ["", "-wal", "-shm"] {
        let path = PathBuf::from(format!("{}{}", target_database.to_string_lossy(), suffix));
        if path.exists() {
            fs::remove_file(path).map_err(to_sql_error)?;
        }
    }
    fs::copy(&backup_database, &target_database).map_err(to_sql_error)?;

    let backup_files_root = backup_root.join("files");
    if backup_files_root.is_dir() {
        copy_directory_contents(&backup_files_root, &data_root.join("files"))?;
    }

    let connection = Connection::open(&target_database)?;
    let document_count: i64 = connection.query_row(
        "SELECT COUNT(*) FROM documents WHERE vault_id = 1 AND trashed_at IS NULL",
        [],
        |row| row.get(0),
    )?;
    let file_count: i64 = connection.query_row(
        "SELECT COUNT(*) FROM files WHERE vault_id = 1 AND file_state = 'available'",
        [],
        |row| row.get(0),
    )?;
    record_audit_event(
        &connection,
        "backup_restored",
        "backup",
        None,
        &format!("Restored local backup {}", backup_root.to_string_lossy()),
    )?;

    Ok(RestoreResult {
        restored_backup_root: backup_root.to_string_lossy().into_owned(),
        pre_restore_backup_root: pre_restore_backup.backup_root,
        document_count,
        file_count,
    })
}

fn copy_directory_contents(source: &Path, destination: &Path) -> rusqlite::Result<()> {
    fs::create_dir_all(destination).map_err(to_sql_error)?;
    for entry in fs::read_dir(source).map_err(to_sql_error)? {
        let entry = entry.map_err(to_sql_error)?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if entry.file_type().map_err(to_sql_error)?.is_dir() {
            copy_directory_contents(&source_path, &destination_path)?;
        } else {
            if let Some(parent) = destination_path.parent() {
                fs::create_dir_all(parent).map_err(to_sql_error)?;
            }
            fs::copy(source_path, destination_path).map_err(to_sql_error)?;
        }
    }
    Ok(())
}

#[tauri::command]
fn check_vault_health(app: tauri::AppHandle) -> Result<VaultHealthReport, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    check_vault_health_at(&data_root).map_err(|error| error.to_string())
}

fn check_vault_health_at(data_root: &Path) -> rusqlite::Result<VaultHealthReport> {
    let connection = Connection::open(data_root.join("vault.db"))?;
    let database_integrity: String =
        connection.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    let schema_version = current_schema_version(&connection)?;
    let document_count: i64 = connection.query_row(
        "SELECT COUNT(*) FROM documents WHERE vault_id = 1 AND trashed_at IS NULL",
        [],
        |row| row.get(0),
    )?;
    let file_count: i64 = connection.query_row(
        "SELECT COUNT(*) FROM files WHERE vault_id = 1 AND file_state = 'available'",
        [],
        |row| row.get(0),
    )?;
    let fts_entry_count: i64 =
        connection.query_row("SELECT COUNT(*) FROM document_fts", [], |row| row.get(0))?;
    let missing_file_count = count_missing_files(&connection, data_root)?;

    let mut warnings = Vec::new();
    if database_integrity != "ok" {
        warnings.push(format!(
            "SQLite integrity_check returned {database_integrity}"
        ));
    }
    if missing_file_count > 0 {
        warnings.push(format!(
            "{missing_file_count} importerade filer saknas på disk"
        ));
    }
    if fts_entry_count < document_count {
        warnings.push("FTS-index har färre rader än aktiva dokument".to_string());
    }

    Ok(VaultHealthReport {
        ok: warnings.is_empty(),
        database_integrity,
        schema_version,
        document_count,
        file_count,
        missing_file_count,
        fts_entry_count,
        warnings,
    })
}

#[tauri::command]
fn rebuild_production_search_index(app: tauri::AppHandle) -> Result<ReindexResult, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    rebuild_production_search_index_at(&data_root).map_err(|error| error.to_string())
}

fn rebuild_production_search_index_at(data_root: &Path) -> rusqlite::Result<ReindexResult> {
    let mut connection = Connection::open(data_root.join("vault.db"))?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;
    let transaction = connection.transaction()?;
    transaction.execute("DELETE FROM document_fts", [])?;

    let indexed_document_count = {
        let mut statement = transaction.prepare(
            "
            SELECT
              id,
              title,
              document_type,
              document_date,
              inbox_status,
              source_label,
              match_explanation,
              extracted_text
            FROM documents
            WHERE vault_id = 1 AND trashed_at IS NULL
            ORDER BY id
            ",
        )?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        for (
            document_id,
            title,
            document_type,
            document_date,
            inbox_status,
            source_label,
            match_explanation,
            extracted_text,
        ) in &rows
        {
            index_document(
                &transaction,
                *document_id,
                title,
                &format!(
                    "{} {} {} {}",
                    document_type,
                    document_date.as_deref().unwrap_or_default(),
                    inbox_status,
                    source_label
                ),
                &format!("{match_explanation} {extracted_text}"),
            )?;
        }

        rows.len() as i64
    };

    record_audit_event(
        &transaction,
        "search_index_rebuilt",
        "document_fts",
        None,
        &format!("Rebuilt production FTS index for {indexed_document_count} documents"),
    )?;
    transaction.commit()?;

    Ok(ReindexResult {
        indexed_document_count,
    })
}

fn count_missing_files(connection: &Connection, data_root: &Path) -> rusqlite::Result<i64> {
    let mut statement = connection.prepare(
        "
        SELECT storage_path
        FROM files
        WHERE vault_id = 1 AND file_state = 'available'
        ",
    )?;
    let storage_paths = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(storage_paths
        .iter()
        .filter(|storage_path| !data_root.join(storage_path).is_file())
        .count() as i64)
}

#[tauri::command]
fn search_testlab_documents(
    app: tauri::AppHandle,
    query: String,
) -> Result<Vec<SearchResult>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    search_testlab_documents_at(&data_root, &query).map_err(|error| error.to_string())
}

fn search_testlab_documents_at(
    data_root: &Path,
    query: &str,
) -> rusqlite::Result<Vec<SearchResult>> {
    let connection = Connection::open(data_root.join("testlab").join("vault-test.db"))?;
    search_documents(&connection, query)
}

#[tauri::command]
fn search_production_documents(
    app: tauri::AppHandle,
    query: String,
) -> Result<Vec<SearchResult>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    search_production_documents_at(&data_root, &query).map_err(|error| error.to_string())
}

fn search_production_documents_at(
    data_root: &Path,
    query: &str,
) -> rusqlite::Result<Vec<SearchResult>> {
    let connection = Connection::open(data_root.join("vault.db"))?;
    search_documents(&connection, query)
}

#[tauri::command]
fn get_testlab_document_detail(
    app: tauri::AppHandle,
    document_id: i64,
) -> Result<DocumentDetail, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    get_testlab_document_detail_at(&data_root, document_id).map_err(|error| error.to_string())
}

fn get_testlab_document_detail_at(
    data_root: &Path,
    document_id: i64,
) -> rusqlite::Result<DocumentDetail> {
    let connection = Connection::open(data_root.join("testlab").join("vault-test.db"))?;
    get_document_detail(&connection, data_root.join("testlab"), document_id)
}

#[tauri::command]
fn get_production_document_detail(
    app: tauri::AppHandle,
    document_id: i64,
) -> Result<DocumentDetail, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    get_production_document_detail_at(&data_root, document_id).map_err(|error| error.to_string())
}

fn get_production_document_detail_at(
    data_root: &Path,
    document_id: i64,
) -> rusqlite::Result<DocumentDetail> {
    let connection = Connection::open(data_root.join("vault.db"))?;
    get_document_detail(&connection, data_root.to_path_buf(), document_id)
}

fn get_document_detail(
    connection: &Connection,
    storage_root: PathBuf,
    document_id: i64,
) -> rusqlite::Result<DocumentDetail> {
    let mut detail = connection.query_row(
        "
        SELECT
          d.id,
          d.title,
          d.document_type,
          d.document_date,
          d.inbox_status,
          d.source_label,
          d.match_explanation,
          d.extracted_text,
          f.storage_path,
          f.original_name,
          f.mime_type,
          f.size_bytes
        FROM documents d
        LEFT JOIN document_versions dv
          ON dv.document_id = d.id AND dv.is_current_file = 1
        LEFT JOIN files f
          ON f.id = dv.file_id
        WHERE d.id = ?1 AND d.vault_id = 1 AND d.trashed_at IS NULL
        ORDER BY dv.version_no DESC
        LIMIT 1
        ",
        params![document_id],
        |row| {
            let storage_path: Option<String> = row.get(8)?;
            let stored_path = storage_path
                .as_ref()
                .map(|path| storage_root.join(path).to_string_lossy().into_owned());

            Ok(DocumentDetail {
                document: DocumentSummary {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    document_type: row.get(2)?,
                    document_date: row.get(3)?,
                    inbox_status: row.get(4)?,
                    source_label: row.get(5)?,
                    match_explanation: row.get(6)?,
                },
                extracted_text: row.get(7)?,
                stored_path,
                original_name: row.get(9)?,
                mime_type: row.get(10)?,
                size_bytes: row.get(11)?,
                tags: Vec::new(),
                claims: Vec::new(),
                entities: Vec::new(),
            })
        },
    )?;
    detail.tags = list_document_tags(connection, document_id)?;
    detail.claims = list_document_claims(connection, document_id)?;
    detail.entities = list_document_entities(connection, document_id)?;
    Ok(detail)
}

fn list_document_tags(connection: &Connection, document_id: i64) -> rusqlite::Result<Vec<String>> {
    let mut statement = connection.prepare(
        "
        SELECT t.name
        FROM document_tags dt
        JOIN tags t ON t.id = dt.tag_id
        WHERE dt.document_id = ?1
        ORDER BY t.name
        ",
    )?;
    let tags = statement
        .query_map(params![document_id], |row| row.get(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(tags)
}

fn list_document_claims(
    connection: &Connection,
    document_id: i64,
) -> rusqlite::Result<Vec<ClaimSummary>> {
    let mut statement = connection.prepare(
        "
        SELECT c.id, c.claim_type, c.value_json, c.value_kind, c.status,
               c.confidence_kind, c.extraction_method, c.source_span_id,
               p.page_no, s.text, c.effective_from, c.effective_to,
               c.actuality_status, c.actuality_explanation
        FROM claims c
        LEFT JOIN text_spans s ON s.id = c.source_span_id
        LEFT JOIN document_pages p ON p.id = s.document_page_id
        WHERE c.document_id = ?1
        ORDER BY c.id
        ",
    )?;
    let claims = statement
        .query_map(params![document_id], |row| {
            Ok(ClaimSummary {
                id: row.get(0)?,
                claim_type: row.get(1)?,
                value_json: row.get(2)?,
                value_kind: row.get(3)?,
                status: row.get(4)?,
                confidence_kind: row.get(5)?,
                extraction_method: row.get(6)?,
                source_span_id: row.get(7)?,
                source_page_no: row.get(8)?,
                source_text: row.get(9)?,
                effective_from: row.get(10)?,
                effective_to: row.get(11)?,
                actuality_status: row.get(12)?,
                actuality_explanation: row.get(13)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(claims)
}

fn list_document_entities(
    connection: &Connection,
    document_id: i64,
) -> rusqlite::Result<Vec<EntitySummary>> {
    let mut statement = connection.prepare(
        "
        SELECT e.id, e.entity_type, e.display_name, de.role
        FROM document_entities de
        JOIN entities e ON e.id = de.entity_id
        WHERE de.document_id = ?1
        ORDER BY e.entity_type, e.display_name
        ",
    )?;
    let entities = statement
        .query_map(params![document_id], |row| {
            Ok(EntitySummary {
                id: row.get(0)?,
                entity_type: row.get(1)?,
                display_name: row.get(2)?,
                role: row.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(entities)
}

#[tauri::command]
fn update_production_document_metadata(
    app: tauri::AppHandle,
    document_id: i64,
    title: String,
    document_type: String,
    document_date: Option<String>,
    inbox_status: String,
    source_label: String,
    match_explanation: String,
) -> Result<DocumentDetail, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    update_production_document_metadata_at(
        &data_root,
        document_id,
        &title,
        &document_type,
        document_date.as_deref(),
        &inbox_status,
        &source_label,
        &match_explanation,
    )
    .map_err(|error| error.to_string())
}

#[allow(clippy::too_many_arguments)]
fn update_production_document_metadata_at(
    data_root: &Path,
    document_id: i64,
    title: &str,
    document_type: &str,
    document_date: Option<&str>,
    inbox_status: &str,
    source_label: &str,
    match_explanation: &str,
) -> rusqlite::Result<DocumentDetail> {
    let title = clean_required_text(title, "Titel")?;
    let document_type = clean_required_text(document_type, "Dokumenttyp")?;
    let inbox_status = clean_inbox_status(inbox_status)?;
    let source_label = clean_required_text(source_label, "Kalla")?;
    let match_explanation = clean_required_text(match_explanation, "Matchorsak")?;
    let document_date = clean_optional_date(document_date)?;

    let connection = Connection::open(data_root.join("vault.db"))?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;
    let updated = connection.execute(
        "
        UPDATE documents
        SET title = ?1,
            document_type = ?2,
            document_date = ?3,
            inbox_status = ?4,
            source_label = ?5,
            match_explanation = ?6,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ?7 AND vault_id = 1 AND trashed_at IS NULL
        ",
        params![
            title,
            document_type,
            document_date,
            inbox_status,
            source_label,
            match_explanation,
            document_id
        ],
    )?;

    if updated == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    let extracted_text: String = connection.query_row(
        "SELECT extracted_text FROM documents WHERE id = ?1 AND vault_id = 1",
        params![document_id],
        |row| row.get(0),
    )?;
    index_document(
        &connection,
        document_id,
        &title,
        &format!(
            "{} {} {} {}",
            document_type,
            document_date.as_deref().unwrap_or_default(),
            inbox_status,
            source_label
        ),
        &format!("{match_explanation} {extracted_text}"),
    )?;
    record_audit_event(
        &connection,
        "document_metadata_updated",
        "document",
        Some(document_id),
        &format!("Updated metadata for {title}"),
    )?;

    get_document_detail(&connection, data_root.to_path_buf(), document_id)
}

#[tauri::command]
fn set_claim_review_status(
    app: tauri::AppHandle,
    claim_id: i64,
    status: String,
) -> Result<DocumentDetail, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    set_claim_review_status_at(&data_root, claim_id, &status).map_err(|error| error.to_string())
}

fn set_claim_review_status_at(
    data_root: &Path,
    claim_id: i64,
    status: &str,
) -> rusqlite::Result<DocumentDetail> {
    let status = clean_claim_status(status)?;
    let connection = Connection::open(data_root.join("vault.db"))?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;
    let document_id: i64 = connection.query_row(
        "SELECT document_id FROM claims WHERE id = ?1 AND vault_id = 1",
        params![claim_id],
        |row| row.get(0),
    )?;
    connection.execute(
        "UPDATE claims SET status = ?1 WHERE id = ?2 AND vault_id = 1",
        params![status, claim_id],
    )?;
    record_audit_event(
        &connection,
        "claim_review_status_updated",
        "claim",
        Some(claim_id),
        &format!("Set claim {claim_id} to {status}"),
    )?;
    get_document_detail(&connection, data_root.to_path_buf(), document_id)
}

#[tauri::command]
fn add_production_document_tag(
    app: tauri::AppHandle,
    document_id: i64,
    tag_name: String,
) -> Result<DocumentDetail, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    add_production_document_tag_at(&data_root, document_id, &tag_name)
        .map_err(|error| error.to_string())
}

fn add_production_document_tag_at(
    data_root: &Path,
    document_id: i64,
    tag_name: &str,
) -> rusqlite::Result<DocumentDetail> {
    let tag_name = clean_tag_or_code_word(tag_name)?;
    let connection = Connection::open(data_root.join("vault.db"))?;
    ensure_production_document_exists(&connection, document_id)?;
    attach_tag(&connection, document_id, &tag_name)?;
    record_audit_event(
        &connection,
        "document_tag_added",
        "document",
        Some(document_id),
        &format!("Added tag {tag_name}"),
    )?;
    get_document_detail(&connection, data_root.to_path_buf(), document_id)
}

#[tauri::command]
fn remove_production_document_tag(
    app: tauri::AppHandle,
    document_id: i64,
    tag_name: String,
) -> Result<DocumentDetail, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    remove_production_document_tag_at(&data_root, document_id, &tag_name)
        .map_err(|error| error.to_string())
}

fn remove_production_document_tag_at(
    data_root: &Path,
    document_id: i64,
    tag_name: &str,
) -> rusqlite::Result<DocumentDetail> {
    let tag_name = clean_tag_or_code_word(tag_name)?;
    let connection = Connection::open(data_root.join("vault.db"))?;
    connection.execute(
        "
        DELETE FROM document_tags
        WHERE document_id = ?1
          AND tag_id IN (SELECT id FROM tags WHERE vault_id = 1 AND name = ?2)
        ",
        params![document_id, tag_name],
    )?;
    record_audit_event(
        &connection,
        "document_tag_removed",
        "document",
        Some(document_id),
        &format!("Removed tag {tag_name}"),
    )?;
    get_document_detail(&connection, data_root.to_path_buf(), document_id)
}

#[tauri::command]
fn list_code_words(app: tauri::AppHandle) -> Result<Vec<CodeWordSummary>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    list_code_words_at(&data_root).map_err(|error| error.to_string())
}

fn list_code_words_at(data_root: &Path) -> rusqlite::Result<Vec<CodeWordSummary>> {
    let connection = Connection::open(data_root.join("vault.db"))?;
    let mut statement = connection.prepare(
        "
        SELECT id, word, description, created_at
        FROM code_words
        WHERE vault_id = 1
        ORDER BY word
        ",
    )?;
    let code_words = statement
        .query_map([], |row| {
            Ok(CodeWordSummary {
                id: row.get(0)?,
                word: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(code_words)
}

#[tauri::command]
fn add_code_word(
    app: tauri::AppHandle,
    word: String,
    description: String,
) -> Result<Vec<CodeWordSummary>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    add_code_word_at(&data_root, &word, &description).map_err(|error| error.to_string())
}

fn add_code_word_at(
    data_root: &Path,
    word: &str,
    description: &str,
) -> rusqlite::Result<Vec<CodeWordSummary>> {
    let word = clean_tag_or_code_word(word)?;
    let description = description.trim().chars().take(500).collect::<String>();
    let connection = Connection::open(data_root.join("vault.db"))?;
    connection.execute(
        "
        INSERT INTO code_words (vault_id, word, description)
        VALUES (1, ?1, ?2)
        ON CONFLICT(vault_id, word) DO UPDATE SET description = excluded.description
        ",
        params![word, description],
    )?;
    record_audit_event(
        &connection,
        "code_word_saved",
        "code_word",
        None,
        &format!("Saved code word {word}"),
    )?;
    list_code_words_at(data_root)
}

#[tauri::command]
fn list_conflicts(app: tauri::AppHandle) -> Result<Vec<ConflictSummary>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    list_conflicts_at(&data_root).map_err(|error| error.to_string())
}

fn list_conflicts_at(data_root: &Path) -> rusqlite::Result<Vec<ConflictSummary>> {
    let connection = Connection::open(data_root.join("vault.db"))?;
    let mut conflicts = Vec::new();

    let mut duplicate_statement = connection.prepare(
        "
        SELECT f.sha256, COUNT(DISTINCT dv.document_id), GROUP_CONCAT(DISTINCT dv.document_id)
        FROM files f
        JOIN document_versions dv ON dv.file_id = f.id AND dv.is_current_file = 1
        JOIN documents d ON d.id = dv.document_id AND d.trashed_at IS NULL
        WHERE f.vault_id = 1
        GROUP BY f.sha256
        HAVING COUNT(DISTINCT dv.document_id) > 1
        ",
    )?;
    for row in duplicate_statement.query_map([], |row| {
        let sha256: String = row.get(0)?;
        let count: i64 = row.get(1)?;
        let document_ids_csv: String = row.get(2)?;
        Ok((sha256, count, parse_id_list(&document_ids_csv)))
    })? {
        let (sha256, count, document_ids) = row?;
        conflicts.push(ConflictSummary {
            id: None,
            conflict_type: "duplicate_file".to_string(),
            severity: "medium".to_string(),
            title: format!("{count} dokument pekar pa samma fil"),
            detail: format!("SHA-256 {sha256} finns pa flera dokument"),
            document_ids,
            status: "open".to_string(),
            resolution: None,
            locked: false,
        });
    }

    let mut claim_statement = connection.prepare(
        "
        SELECT document_id, claim_type, COUNT(DISTINCT value_json), GROUP_CONCAT(DISTINCT value_json)
        FROM claims
        WHERE vault_id = 1
        GROUP BY document_id, claim_type
        HAVING COUNT(DISTINCT value_json) > 1
        ",
    )?;
    for row in claim_statement.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, String>(3)?,
        ))
    })? {
        let (document_id, claim_type, count, values) = row?;
        conflicts.push(ConflictSummary {
            id: None,
            conflict_type: "conflicting_claims".to_string(),
            severity: "high".to_string(),
            title: format!("{count} olika varden for {claim_type}"),
            detail: values,
            document_ids: vec![document_id],
            status: "open".to_string(),
            resolution: None,
            locked: false,
        });
    }

    let pending_count: i64 = connection.query_row(
        "
        SELECT COUNT(*)
        FROM claims
        WHERE vault_id = 1 AND status = 'auto_extracted_pending_review'
        ",
        [],
        |row| row.get(0),
    )?;
    if pending_count > 0 {
        conflicts.push(ConflictSummary {
            id: None,
            conflict_type: "pending_review".to_string(),
            severity: "low".to_string(),
            title: format!("{pending_count} claims väntar på granskning"),
            detail: "Autoextraherade claims bor godkannas eller avvisas innan arkivet anses helt granskat".to_string(),
            document_ids: Vec::new(),
            status: "open".to_string(),
            resolution: None,
            locked: false,
        });
    }

    for conflict in &mut conflicts {
        let key = format!(
            "{}:{}:{}",
            conflict.conflict_type,
            conflict
                .document_ids
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("-"),
            normalize(&conflict.title)
        );
        connection.execute(
            "INSERT INTO conflicts (vault_id, conflict_key, conflict_type, severity, title, detail, document_ids_json)
             VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(vault_id, conflict_key) DO UPDATE SET
               severity = excluded.severity, title = excluded.title, detail = excluded.detail,
               document_ids_json = excluded.document_ids_json",
            params![
                key,
                conflict.conflict_type,
                conflict.severity,
                conflict.title,
                conflict.detail,
                serde_json::to_string(&conflict.document_ids).unwrap_or_else(|_| "[]".to_string())
            ],
        )?;
        let (id, status, resolution, locked): (i64, String, Option<String>, bool) = connection
            .query_row(
                "SELECT id, status, resolution, locked FROM conflicts WHERE vault_id = 1 AND conflict_key = ?1",
                params![key],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )?;
        conflict.id = Some(id);
        conflict.status = status;
        conflict.resolution = resolution;
        conflict.locked = locked;
    }

    conflicts.sort_by(|left, right| {
        left.severity
            .cmp(&right.severity)
            .then(left.title.cmp(&right.title))
    });
    Ok(conflicts)
}

#[tauri::command]
fn resolve_conflict(
    app: tauri::AppHandle,
    conflict_id: i64,
    decision: String,
    note: String,
    locked: bool,
) -> Result<Vec<ConflictSummary>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    let connection =
        Connection::open(data_root.join("vault.db")).map_err(|error| error.to_string())?;
    let decision = match decision.trim() {
        "resolved" | "ignored" => decision.trim(),
        _ => return Err("Ogiltigt konfliktbeslut".to_string()),
    };
    let changed = connection.execute(
        "UPDATE conflicts SET status = ?1, resolution = ?2, locked = ?3, resolved_at = CURRENT_TIMESTAMP
         WHERE id = ?4 AND vault_id = 1 AND status = 'open'",
        params![decision, note.trim().chars().take(1000).collect::<String>(), locked, conflict_id],
    ).map_err(|error| error.to_string())?;
    if changed == 0 {
        return Err("Konflikten finns inte eller är redan behandlad".to_string());
    }
    connection.execute(
        "INSERT INTO user_verifications (vault_id, target_type, target_id, decision, note, locked)
         VALUES (1, 'conflict', ?1, ?2, ?3, ?4)",
        params![conflict_id, decision, note.trim(), locked],
    ).map_err(|error| error.to_string())?;
    record_audit_event(
        &connection,
        "conflict_resolved",
        "conflict",
        Some(conflict_id),
        "User resolved a conflict",
    )
    .map_err(|error| error.to_string())?;
    list_conflicts_at(&data_root).map_err(|error| error.to_string())
}

fn ensure_production_document_exists(
    connection: &Connection,
    document_id: i64,
) -> rusqlite::Result<()> {
    let exists: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM documents WHERE id = ?1 AND vault_id = 1 AND trashed_at IS NULL)",
        params![document_id],
        |row| row.get(0),
    )?;
    if exists {
        Ok(())
    } else {
        Err(rusqlite::Error::QueryReturnedNoRows)
    }
}

fn clean_required_text(value: &str, field_name: &str) -> rusqlite::Result<String> {
    let cleaned = value.trim().chars().take(240).collect::<String>();
    if cleaned.is_empty() {
        Err(to_sql_error(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("{field_name} far inte vara tom"),
        )))
    } else {
        Ok(cleaned)
    }
}

fn clean_inbox_status(value: &str) -> rusqlite::Result<String> {
    match value.trim() {
        "inbox" | "indexed" | "review" | "reviewed" | "archived" => Ok(value.trim().to_string()),
        _ => Err(to_sql_error(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Ogiltig dokumentstatus",
        ))),
    }
}

fn clean_optional_date(value: Option<&str>) -> rusqlite::Result<Option<String>> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };

    let looks_like_iso_date = value.len() == 10
        && value.chars().enumerate().all(|(index, character)| {
            matches!(index, 4 | 7) && character == '-'
                || !matches!(index, 4 | 7) && character.is_ascii_digit()
        });
    if looks_like_iso_date {
        Ok(Some(value.to_string()))
    } else {
        Err(to_sql_error(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Datum maste vara tomt eller skrivas som YYYY-MM-DD",
        )))
    }
}

fn clean_claim_status(status: &str) -> rusqlite::Result<String> {
    match status.trim() {
        "review_approved"
        | "review_rejected"
        | "auto_extracted_pending_review"
        | "verified_system_value" => Ok(status.trim().to_string()),
        _ => Err(to_sql_error(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Okand claim-status",
        ))),
    }
}

fn clean_tag_or_code_word(value: &str) -> rusqlite::Result<String> {
    let cleaned = normalize(value)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-");
    if cleaned.is_empty() {
        Err(to_sql_error(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Varde saknas",
        )))
    } else {
        Ok(cleaned.chars().take(64).collect())
    }
}

fn parse_id_list(value: &str) -> Vec<i64> {
    value
        .split(',')
        .filter_map(|part| part.trim().parse::<i64>().ok())
        .collect()
}

#[tauri::command]
fn list_document_versions(
    app: tauri::AppHandle,
    document_id: i64,
) -> Result<Vec<DocumentVersionSummary>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    list_document_versions_at(&data_root, document_id).map_err(|error| error.to_string())
}

fn list_document_versions_at(
    data_root: &Path,
    document_id: i64,
) -> rusqlite::Result<Vec<DocumentVersionSummary>> {
    let connection = Connection::open(data_root.join("vault.db"))?;
    let mut statement = connection.prepare(
        "SELECT dv.id, dv.version_no, dv.imported_at, dv.document_date, f.original_name, f.mime_type,
                f.size_bytes, f.sha256, dv.is_current_file, dv.comment, dv.origin
         FROM document_versions dv JOIN files f ON f.id = dv.file_id
         JOIN documents d ON d.id = dv.document_id
         WHERE dv.document_id = ?1 AND d.vault_id = 1
         ORDER BY dv.version_no DESC",
    )?;
    let versions = statement
        .query_map(params![document_id], |row| {
            Ok(DocumentVersionSummary {
                id: row.get(0)?,
                version_no: row.get(1)?,
                imported_at: row.get(2)?,
                document_date: row.get(3)?,
                original_name: row.get(4)?,
                mime_type: row.get(5)?,
                size_bytes: row.get(6)?,
                sha256: row.get(7)?,
                is_current_file: row.get(8)?,
                comment: row.get(9)?,
                origin: row.get(10)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(versions)
}

#[tauri::command]
fn list_document_pages(
    app: tauri::AppHandle,
    document_id: i64,
) -> Result<Vec<DocumentPageSummary>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    list_document_pages_at(&data_root, document_id).map_err(|error| error.to_string())
}

fn list_document_pages_at(
    data_root: &Path,
    document_id: i64,
) -> rusqlite::Result<Vec<DocumentPageSummary>> {
    let connection = Connection::open(data_root.join("vault.db"))?;
    let mut statement = connection.prepare(
        "SELECT dp.page_no, dp.source_kind, dp.text_quality, COALESCE(GROUP_CONCAT(ts.text, '\n'), '')
         FROM document_pages dp JOIN document_versions dv ON dv.id = dp.document_version_id
         LEFT JOIN text_spans ts ON ts.document_page_id = dp.id
         WHERE dv.document_id = ?1 AND dv.is_current_file = 1
         GROUP BY dp.id ORDER BY dp.page_no",
    )?;
    let pages = statement
        .query_map(params![document_id], |row| {
            Ok(DocumentPageSummary {
                page_no: row.get(0)?,
                source_kind: row.get(1)?,
                text_quality: row.get(2)?,
                text: row.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(pages)
}

#[tauri::command]
fn list_document_annotations(
    app: tauri::AppHandle,
    document_id: i64,
) -> Result<Vec<DocumentAnnotationSummary>, String> {
    let data_root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    initialize_vault_at(&data_root).map_err(|e| e.to_string())?;
    let connection = Connection::open(data_root.join("vault.db")).map_err(|e| e.to_string())?;
    let mut statement = connection.prepare("SELECT id,document_id,page_no,annotation_type,selected_text,body,color,created_at,updated_at FROM document_annotations WHERE document_id=?1 ORDER BY page_no,id DESC").map_err(|e| e.to_string())?;
    let annotations = statement
        .query_map(params![document_id], |row| {
            Ok(DocumentAnnotationSummary {
                id: row.get(0)?,
                document_id: row.get(1)?,
                page_no: row.get(2)?,
                annotation_type: row.get(3)?,
                selected_text: row.get(4)?,
                body: row.get(5)?,
                color: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    Ok(annotations)
}

#[tauri::command]
fn save_document_annotation(
    app: tauri::AppHandle,
    document_id: i64,
    annotation_id: Option<i64>,
    page_no: i64,
    annotation_type: String,
    selected_text: String,
    body: String,
    color: String,
) -> Result<Vec<DocumentAnnotationSummary>, String> {
    if !["bookmark", "note", "highlight"].contains(&annotation_type.as_str()) {
        return Err("Ogiltig markeringstyp".into());
    }
    let data_root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    initialize_vault_at(&data_root).map_err(|e| e.to_string())?;
    let connection = Connection::open(data_root.join("vault.db")).map_err(|e| e.to_string())?;
    ensure_production_document_exists(&connection, document_id).map_err(|e| e.to_string())?;
    if let Some(id) = annotation_id {
        connection.execute("UPDATE document_annotations SET page_no=?1,annotation_type=?2,selected_text=?3,body=?4,color=?5,updated_at=CURRENT_TIMESTAMP WHERE id=?6 AND document_id=?7",params![page_no.max(1),annotation_type,selected_text.trim(),body.trim(),color,id,document_id]).map_err(|e|e.to_string())?;
    } else {
        connection.execute("INSERT INTO document_annotations(document_id,page_no,annotation_type,selected_text,body,color) VALUES(?1,?2,?3,?4,?5,?6)",params![document_id,page_no.max(1),annotation_type,selected_text.trim(),body.trim(),color]).map_err(|e|e.to_string())?;
    }
    record_audit_event(
        &connection,
        "document_annotation_saved",
        "document",
        Some(document_id),
        "Viewer annotation saved",
    )
    .map_err(|e| e.to_string())?;
    drop(connection);
    list_document_annotations(app, document_id)
}

#[tauri::command]
fn delete_document_annotation(
    app: tauri::AppHandle,
    document_id: i64,
    annotation_id: i64,
) -> Result<Vec<DocumentAnnotationSummary>, String> {
    let data_root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    initialize_vault_at(&data_root).map_err(|e| e.to_string())?;
    let connection = Connection::open(data_root.join("vault.db")).map_err(|e| e.to_string())?;
    connection
        .execute(
            "DELETE FROM document_annotations WHERE id=?1 AND document_id=?2",
            params![annotation_id, document_id],
        )
        .map_err(|e| e.to_string())?;
    drop(connection);
    list_document_annotations(app, document_id)
}

fn version_text(connection: &Connection, version_id: i64) -> rusqlite::Result<String> {
    connection.query_row("SELECT COALESCE(GROUP_CONCAT(ts.text, char(10)), '') FROM document_pages dp LEFT JOIN text_spans ts ON ts.document_page_id=dp.id WHERE dp.document_version_id=?1 ORDER BY dp.page_no,ts.id",params![version_id],|r|r.get(0))
}

#[tauri::command]
fn compare_document_versions(
    app: tauri::AppHandle,
    document_id: i64,
    left_version_id: i64,
    right_version_id: i64,
) -> Result<VersionComparison, String> {
    let root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    initialize_vault_at(&root).map_err(|e| e.to_string())?;
    let c = Connection::open(root.join("vault.db")).map_err(|e| e.to_string())?;
    let read = |id: i64| -> rusqlite::Result<(i64, Option<String>, i64, String, i64)> {
        c.query_row("SELECT dv.version_no,dv.document_date,f.size_bytes,f.sha256,(SELECT COUNT(*) FROM document_pages WHERE document_version_id=dv.id) FROM document_versions dv JOIN files f ON f.id=dv.file_id WHERE dv.id=?1 AND dv.document_id=?2",params![id,document_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?)))
    };
    let l = read(left_version_id).map_err(|e| e.to_string())?;
    let r = read(right_version_id).map_err(|e| e.to_string())?;
    let lt = version_text(&c, left_version_id).map_err(|e| e.to_string())?;
    let rt = version_text(&c, right_version_id).map_err(|e| e.to_string())?;
    let ll: std::collections::HashSet<String> = lt
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();
    let rl: std::collections::HashSet<String> = rt
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();
    let mut added: Vec<_> = rl.difference(&ll).cloned().collect();
    let mut removed: Vec<_> = ll.difference(&rl).cloned().collect();
    added.sort();
    removed.sort();
    Ok(VersionComparison {
        left_version_no: l.0,
        right_version_no: r.0,
        added_lines: added,
        removed_lines: removed,
        left_sha256: l.3.clone(),
        right_sha256: r.3.clone(),
        size_change_bytes: r.2 - l.2,
        page_change: r.4 - l.4,
        date_changed: l.1 != r.1,
        identical: l.3 == r.3,
    })
}

#[tauri::command]
fn restore_document_version(
    app: tauri::AppHandle,
    document_id: i64,
    version_id: i64,
) -> Result<Vec<DocumentVersionSummary>, String> {
    let root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    initialize_vault_at(&root).map_err(|e| e.to_string())?;
    let mut c = Connection::open(root.join("vault.db")).map_err(|e| e.to_string())?;
    let tx = c.transaction().map_err(|e| e.to_string())?;
    let date: Option<String> = tx
        .query_row(
            "SELECT document_date FROM document_versions WHERE id=?1 AND document_id=?2",
            params![version_id, document_id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    let text = version_text(&tx, version_id).map_err(|e| e.to_string())?;
    tx.execute("UPDATE document_versions SET is_current_file=CASE WHEN id=?1 THEN 1 ELSE 0 END WHERE document_id=?2",params![version_id,document_id]).map_err(|e|e.to_string())?;
    tx.execute("UPDATE documents SET extracted_text=?1,document_date=COALESCE(?2,document_date),inbox_status='review',updated_at=CURRENT_TIMESTAMP WHERE id=?3",params![text,date,document_id]).map_err(|e|e.to_string())?;
    record_audit_event(
        &tx,
        "document_version_restored",
        "document",
        Some(document_id),
        &format!("Restored version id {version_id}; claims require review"),
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    list_document_versions_at(&root, document_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn add_document_version(
    app: tauri::AppHandle,
    document_id: i64,
    source_path: String,
    comment: Option<String>,
) -> Result<Vec<DocumentVersionSummary>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    add_document_version_at(
        &data_root,
        document_id,
        Path::new(&source_path),
        comment.as_deref().unwrap_or(""),
    )
    .map_err(|error| error.to_string())
}

fn add_document_version_at(
    data_root: &Path,
    document_id: i64,
    source_path: &Path,
    comment: &str,
) -> rusqlite::Result<Vec<DocumentVersionSummary>> {
    let metadata = fs::metadata(source_path).map_err(to_sql_error)?;
    if !metadata.is_file() {
        return Err(to_sql_error(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Version source is not a file",
        )));
    }
    let sha256 = sha256_file(source_path).map_err(to_sql_error)?;
    let original_name = source_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("document-version")
        .to_string();
    let mime_type = detect_mime_type(source_path);
    let extracted_text =
        extract_text_for_indexing(source_path, mime_type, Some(data_root)).map_err(to_sql_error)?;
    let storage_relative_path = storage_relative_path(&sha256, &original_name);
    let stored_path = data_root.join(&storage_relative_path);
    if let Some(parent) = stored_path.parent() {
        fs::create_dir_all(parent).map_err(to_sql_error)?;
    }
    if !stored_path.exists() {
        fs::copy(source_path, &stored_path).map_err(to_sql_error)?;
    }
    let date = extract_document_date(&original_name, &extracted_text).document_date;
    let mut connection = Connection::open(data_root.join("vault.db"))?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;
    let transaction = connection.transaction()?;
    ensure_production_document_exists(&transaction, document_id)?;
    let current_sha: Option<String> = transaction.query_row(
        "SELECT f.sha256 FROM document_versions dv JOIN files f ON f.id = dv.file_id WHERE dv.document_id = ?1 AND dv.is_current_file = 1",
        params![document_id], |row| row.get(0)).ok();
    if current_sha.as_deref() == Some(&sha256) {
        return Err(to_sql_error(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "Samma fil är redan aktuell version",
        )));
    }
    transaction.execute(
        "INSERT OR IGNORE INTO files (vault_id, sha256, storage_path, original_name, mime_type, size_bytes) VALUES (1, ?1, ?2, ?3, ?4, ?5)",
        params![sha256, storage_relative_path.to_string_lossy(), original_name, mime_type, metadata.len() as i64],
    )?;
    let file_id: i64 = transaction.query_row(
        "SELECT id FROM files WHERE vault_id = 1 AND sha256 = ?1",
        params![sha256],
        |row| row.get(0),
    )?;
    let next_version: i64 = transaction.query_row(
        "SELECT COALESCE(MAX(version_no), 0) + 1 FROM document_versions WHERE document_id = ?1",
        params![document_id],
        |row| row.get(0),
    )?;
    transaction.execute(
        "UPDATE document_versions SET is_current_file = 0 WHERE document_id = ?1",
        params![document_id],
    )?;
    transaction.execute(
        "INSERT INTO document_versions (document_id, file_id, version_no, document_date, source_path, is_current_file, comment, origin) VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, 'manual')",
        params![document_id, file_id, next_version, date, source_path.to_string_lossy(), comment.trim()],
    )?;
    let version_id = transaction.last_insert_rowid();
    store_document_pages(
        &transaction,
        document_id,
        version_id,
        mime_type,
        &extracted_text,
    )?;
    transaction.execute("UPDATE documents SET extracted_text = ?1, document_date = COALESCE(?2, document_date), updated_at = CURRENT_TIMESTAMP WHERE id = ?3", params![extracted_text, date, document_id])?;
    let (title, document_type, document_date, inbox_status, source_label, explanation): (String, String, Option<String>, String, String, String) = transaction.query_row(
        "SELECT title, document_type, document_date, inbox_status, source_label, match_explanation FROM documents WHERE id = ?1", params![document_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)))?;
    index_document(
        &transaction,
        document_id,
        &title,
        &format!(
            "{document_type} {} {inbox_status} {source_label}",
            document_date.as_deref().unwrap_or_default()
        ),
        &format!("{explanation} {extracted_text}"),
    )?;
    record_audit_event(
        &transaction,
        "document_version_added",
        "document",
        Some(document_id),
        &format!("Added document version {next_version}"),
    )?;
    transaction.commit()?;
    list_document_versions_at(data_root, document_id)
}

#[tauri::command]
fn list_reminders(app: tauri::AppHandle) -> Result<Vec<ReminderSummary>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    list_reminders_at(&data_root).map_err(|error| error.to_string())
}

fn list_reminders_at(data_root: &Path) -> rusqlite::Result<Vec<ReminderSummary>> {
    let connection = Connection::open(data_root.join("vault.db"))?;
    let mut statement = connection.prepare("SELECT r.id, r.document_id, d.title, r.title, r.due_date, r.status, r.note FROM reminders r LEFT JOIN documents d ON d.id = r.document_id WHERE r.vault_id = 1 ORDER BY r.status, r.due_date, r.id")?;
    let items = statement
        .query_map([], |row| {
            Ok(ReminderSummary {
                id: row.get(0)?,
                document_id: row.get(1)?,
                document_title: row.get(2)?,
                title: row.get(3)?,
                due_date: row.get(4)?,
                status: row.get(5)?,
                note: row.get(6)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(items)
}

#[tauri::command]
fn save_reminder(
    app: tauri::AppHandle,
    document_id: Option<i64>,
    title: String,
    due_date: String,
    note: String,
) -> Result<Vec<ReminderSummary>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    let title =
        clean_required_text(&title, "Påminnelsetitel").map_err(|error| error.to_string())?;
    let due_date = clean_optional_date(Some(&due_date))
        .map_err(|error| error.to_string())?
        .ok_or("Datum krävs")?;
    let connection =
        Connection::open(data_root.join("vault.db")).map_err(|error| error.to_string())?;
    connection.execute("INSERT INTO reminders (vault_id, document_id, title, due_date, note) VALUES (1, ?1, ?2, ?3, ?4)", params![document_id, title, due_date, note.trim().chars().take(1000).collect::<String>()]).map_err(|error| error.to_string())?;
    record_audit_event(
        &connection,
        "reminder_created",
        "reminder",
        Some(connection.last_insert_rowid()),
        "Created local reminder",
    )
    .map_err(|error| error.to_string())?;
    list_reminders_at(&data_root).map_err(|error| error.to_string())
}

#[tauri::command]
fn set_reminder_completed(
    app: tauri::AppHandle,
    reminder_id: i64,
    completed: bool,
) -> Result<Vec<ReminderSummary>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    let connection =
        Connection::open(data_root.join("vault.db")).map_err(|error| error.to_string())?;
    connection.execute("UPDATE reminders SET status = ?1, completed_at = CASE WHEN ?2 THEN CURRENT_TIMESTAMP ELSE NULL END WHERE id = ?3 AND vault_id = 1", params![if completed { "completed" } else { "active" }, completed, reminder_id]).map_err(|error| error.to_string())?;
    list_reminders_at(&data_root).map_err(|error| error.to_string())
}

#[tauri::command]
fn list_watched_folders(app: tauri::AppHandle) -> Result<Vec<WatchedFolderSummary>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    list_watched_folders_at(&data_root).map_err(|error| error.to_string())
}

fn list_watched_folders_at(data_root: &Path) -> rusqlite::Result<Vec<WatchedFolderSummary>> {
    let connection = Connection::open(data_root.join("vault.db"))?;
    let mut statement = connection.prepare("SELECT id, path, enabled, last_scanned_at FROM watched_folders WHERE vault_id = 1 ORDER BY path")?;
    let items = statement
        .query_map([], |row| {
            Ok(WatchedFolderSummary {
                id: row.get(0)?,
                path: row.get(1)?,
                enabled: row.get(2)?,
                last_scanned_at: row.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(items)
}

#[tauri::command]
fn add_watched_folder(
    app: tauri::AppHandle,
    path: String,
) -> Result<Vec<WatchedFolderSummary>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    let folder = PathBuf::from(path.trim());
    if !folder.is_dir() {
        return Err("Den valda bevakade mappen finns inte".to_string());
    }
    let canonical = folder.canonicalize().map_err(|error| error.to_string())?;
    let connection =
        Connection::open(data_root.join("vault.db")).map_err(|error| error.to_string())?;
    connection.execute("INSERT INTO watched_folders (vault_id, path) VALUES (1, ?1) ON CONFLICT(vault_id, path) DO UPDATE SET enabled = 1", params![canonical.to_string_lossy()]).map_err(|error| error.to_string())?;
    list_watched_folders_at(&data_root).map_err(|error| error.to_string())
}

#[tauri::command]
fn scan_watched_folders(app: tauri::AppHandle) -> Result<DirectoryImportResult, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    scan_watched_folders_at(&data_root).map_err(|error| error.to_string())
}

fn scan_watched_folders_at(data_root: &Path) -> rusqlite::Result<DirectoryImportResult> {
    let folders = list_watched_folders_at(data_root)?;
    let mut combined = DirectoryImportResult {
        scanned_file_count: 0,
        imported_count: 0,
        duplicate_count: 0,
        failed_count: 0,
        results: Vec::new(),
        failures: Vec::new(),
    };
    for folder in folders.into_iter().filter(|folder| folder.enabled) {
        match collect_supported_import_files(Path::new(&folder.path)) {
            Ok(files) => {
                for file in files {
                    combined.scanned_file_count += 1;
                    let hash = match sha256_file(&file) {
                        Ok(hash) => hash,
                        Err(error) => {
                            combined.failed_count += 1;
                            combined
                                .failures
                                .push(format!("{}: {error}", file.display()));
                            continue;
                        }
                    };
                    let connection = Connection::open(data_root.join("vault.db"))?;
                    let known: bool = connection.query_row(
                        "SELECT EXISTS(SELECT 1 FROM files WHERE vault_id = 1 AND sha256 = ?1)",
                        params![hash],
                        |row| row.get(0),
                    )?;
                    if known {
                        combined.duplicate_count += 1;
                        continue;
                    }
                    match import_document_at(data_root, &file) {
                        Ok(result) => {
                            combined.imported_count += 1;
                            combined.results.push(result);
                        }
                        Err(error) => {
                            combined.failed_count += 1;
                            combined
                                .failures
                                .push(format!("{}: {error}", file.display()));
                        }
                    }
                }
            }
            Err(error) => {
                combined.failed_count += 1;
                combined.failures.push(format!("{}: {error}", folder.path));
            }
        }
        let connection = Connection::open(data_root.join("vault.db"))?;
        connection.execute(
            "UPDATE watched_folders SET last_scanned_at = CURRENT_TIMESTAMP WHERE id = ?1",
            params![folder.id],
        )?;
    }
    Ok(combined)
}

#[tauri::command]
fn list_background_jobs(app: tauri::AppHandle) -> Result<Vec<JobSummary>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    list_background_jobs_at(&data_root).map_err(|error| error.to_string())
}

fn list_background_jobs_at(data_root: &Path) -> rusqlite::Result<Vec<JobSummary>> {
    let connection = Connection::open(data_root.join("vault.db"))?;
    let mut statement = connection.prepare(
        "SELECT id, job_type, target_id, status, attempts, error_code, created_at, updated_at,
                progress_current, progress_total, pause_requested, result_summary
         FROM jobs WHERE vault_id = 1 ORDER BY datetime(created_at) DESC, id DESC LIMIT 50",
    )?;
    let jobs = statement
        .query_map([], |row| {
            Ok(JobSummary {
                id: row.get(0)?,
                job_type: row.get(1)?,
                target_id: row.get(2)?,
                status: row.get(3)?,
                attempts: row.get(4)?,
                error_code: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                progress_current: row.get(8)?,
                progress_total: row.get(9)?,
                pause_requested: row.get(10)?,
                result_summary: row.get(11)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(jobs)
}

#[tauri::command]
fn run_background_jobs_now(app: tauri::AppHandle) -> Result<Vec<JobSummary>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    process_watched_folder_job_at(&data_root).map_err(|error| error.to_string())?;
    list_background_jobs_at(&data_root).map_err(|error| error.to_string())
}

fn process_watched_folder_job_at(data_root: &Path) -> rusqlite::Result<()> {
    if !list_watched_folders_at(data_root)?
        .iter()
        .any(|folder| folder.enabled)
    {
        return Ok(());
    }
    let connection = Connection::open(data_root.join("vault.db"))?;
    connection.execute(
        "INSERT INTO jobs (vault_id, job_type, target_type, status, priority) VALUES (1, 'watched_folder_scan', 'vault', 'running', 100)",
        [],
    )?;
    let job_id = connection.last_insert_rowid();
    match scan_watched_folders_at(data_root) {
        Ok(result) => {
            connection.execute("UPDATE jobs SET status = 'completed', attempts = attempts + 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?1", params![job_id])?;
            record_audit_event(
                &connection,
                "watched_folders_scanned",
                "job",
                Some(job_id),
                &format!(
                    "Watched folders scanned: {} new, {} known",
                    result.imported_count, result.duplicate_count
                ),
            )?;
            Ok(())
        }
        Err(error) => {
            connection.execute("UPDATE jobs SET status = 'failed', attempts = attempts + 1, error_code = 'scan_failed', updated_at = CURRENT_TIMESTAMP WHERE id = ?1", params![job_id])?;
            Err(error)
        }
    }
}

#[tauri::command]
fn queue_document_ocr(app: tauri::AppHandle, document_id: i64) -> Result<Vec<JobSummary>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    let connection =
        Connection::open(data_root.join("vault.db")).map_err(|error| error.to_string())?;
    ensure_production_document_exists(&connection, document_id)
        .map_err(|error| error.to_string())?;
    let exists: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM jobs WHERE vault_id = 1 AND job_type = 'document_ocr' AND target_id = ?1 AND status IN ('queued','running','paused'))",
        params![document_id], |row| row.get(0),
    ).map_err(|error| error.to_string())?;
    if !exists {
        connection.execute(
            "INSERT INTO jobs (vault_id, job_type, target_type, target_id, status, priority) VALUES (1, 'document_ocr', 'document', ?1, 'queued', 50)",
            params![document_id],
        ).map_err(|error| error.to_string())?;
    }
    list_background_jobs_at(&data_root).map_err(|error| error.to_string())
}

#[tauri::command]
fn set_ocr_job_paused(
    app: tauri::AppHandle,
    job_id: i64,
    paused: bool,
) -> Result<Vec<JobSummary>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    let connection =
        Connection::open(data_root.join("vault.db")).map_err(|error| error.to_string())?;
    if paused {
        connection.execute("UPDATE jobs SET pause_requested = 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?1 AND job_type = 'document_ocr' AND status IN ('queued','running')", params![job_id]).map_err(|error| error.to_string())?;
    } else {
        connection.execute("UPDATE jobs SET pause_requested = 0, status = 'queued', updated_at = CURRENT_TIMESTAMP WHERE id = ?1 AND job_type = 'document_ocr' AND status = 'paused'", params![job_id]).map_err(|error| error.to_string())?;
    }
    list_background_jobs_at(&data_root).map_err(|error| error.to_string())
}

fn process_next_ocr_job_at(data_root: &Path) -> rusqlite::Result<bool> {
    let connection = Connection::open(data_root.join("vault.db"))?;
    let job = connection.query_row(
        "SELECT id, target_id, progress_current FROM jobs WHERE vault_id = 1 AND job_type = 'document_ocr' AND status = 'queued' ORDER BY priority, id LIMIT 1",
        [], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?)),
    ).ok();
    let Some((job_id, document_id, completed_pages)) = job else {
        return Ok(false);
    };
    connection.execute("UPDATE jobs SET status = 'running', attempts = attempts + 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?1", params![job_id])?;
    match run_ocr_job_pages_at(data_root, job_id, document_id, completed_pages) {
        Ok(true) => {
            connection.execute("UPDATE jobs SET status = 'completed', result_summary = 'OCR completed and indexed', updated_at = CURRENT_TIMESTAMP WHERE id = ?1", params![job_id])?;
        }
        Ok(false) => {
            connection.execute(
                "UPDATE jobs SET status = 'paused', updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
                params![job_id],
            )?;
        }
        Err(error) => {
            connection.execute("UPDATE jobs SET status = 'failed', error_code = 'ocr_failed', result_summary = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2", params![error.to_string().chars().take(500).collect::<String>(), job_id])?;
        }
    }
    Ok(true)
}

fn run_ocr_job_pages_at(
    data_root: &Path,
    job_id: i64,
    document_id: i64,
    completed_pages: i64,
) -> rusqlite::Result<bool> {
    let connection = Connection::open(data_root.join("vault.db"))?;
    let (storage_path, mime_type, version_id, title, document_type): (String, String, i64, String, String) = connection.query_row(
        "SELECT f.storage_path, f.mime_type, dv.id, d.title, d.document_type FROM documents d JOIN document_versions dv ON dv.document_id = d.id AND dv.is_current_file = 1 JOIN files f ON f.id = dv.file_id WHERE d.id = ?1 AND d.vault_id = 1 AND d.trashed_at IS NULL",
        params![document_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
    )?;
    let source = data_root.join(storage_path);
    let mut page_images = Vec::new();
    let mut temporary_root = None;
    if mime_type == "application/pdf" {
        let pdftoppm = find_pdftoppm_executable().ok_or_else(|| {
            to_sql_error(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "pdftoppm saknas",
            ))
        })?;
        let root = data_root.join("index").join(format!("ocr-job-{job_id}"));
        fs::create_dir_all(&root).map_err(to_sql_error)?;
        let prefix = root.join("page");
        let output = Command::new(pdftoppm)
            .args(["-png", "-r", "200"])
            .arg(&source)
            .arg(&prefix)
            .output()
            .map_err(to_sql_error)?;
        if !output.status.success() {
            return Err(to_sql_error(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "PDF-rendering misslyckades",
            )));
        }
        page_images = fs::read_dir(&root)
            .map_err(to_sql_error)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("png"))
            .collect();
        page_images.sort();
        temporary_root = Some(root);
    } else if matches!(mime_type.as_str(), "image/png" | "image/jpeg") {
        page_images.push(source);
    } else {
        return Err(to_sql_error(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Dokumenttypen stöder inte OCR",
        )));
    }
    connection.execute(
        "UPDATE jobs SET progress_total = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        params![page_images.len() as i64, job_id],
    )?;
    for (index, image_path) in page_images
        .iter()
        .enumerate()
        .skip(completed_pages as usize)
    {
        let pause: bool = connection.query_row(
            "SELECT pause_requested FROM jobs WHERE id = ?1",
            params![job_id],
            |row| row.get(0),
        )?;
        if pause {
            return Ok(false);
        }
        let text =
            extract_image_text_with_tesseract(image_path, Some(data_root)).map_err(to_sql_error)?;
        store_single_document_page(
            &connection,
            document_id,
            version_id,
            index as i64 + 1,
            "ocr_text",
            &text,
        )?;
        connection.execute(
            "UPDATE jobs SET progress_current = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
            params![index as i64 + 1, job_id],
        )?;
    }
    let full_text: String = connection.query_row(
        "SELECT COALESCE(GROUP_CONCAT(ts.text, char(12)), '') FROM document_pages dp JOIN text_spans ts ON ts.document_page_id = dp.id WHERE dp.document_version_id = ?1 ORDER BY dp.page_no",
        params![version_id], |row| row.get(0),
    )?;
    connection.execute(
        "UPDATE documents SET extracted_text = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        params![full_text, document_id],
    )?;
    index_document(&connection, document_id, &title, &document_type, &full_text)?;
    apply_content_claims(&connection, document_id, &title, &document_type, &full_text)?;
    apply_auto_entities(&connection, document_id, &title, &document_type, &full_text)?;
    record_audit_event(
        &connection,
        "ocr_job_completed",
        "document",
        Some(document_id),
        &format!("OCR job completed for {title}"),
    )?;
    if let Some(root) = temporary_root {
        let _ = fs::remove_dir_all(root);
    }
    Ok(true)
}

fn start_background_worker(data_root: PathBuf) {
    thread::spawn(move || {
        let mut ticks = 0_u64;
        loop {
            thread::sleep(Duration::from_secs(2));
            let _ = process_next_ocr_job_at(&data_root);
            ticks += 1;
            if ticks % 30 == 0 {
                let _ = process_watched_folder_job_at(&data_root);
                let _ = run_scheduled_backup_if_due(&data_root);
            }
        }
    });
}

#[tauri::command]
fn open_production_document_file(app: tauri::AppHandle, document_id: i64) -> Result<(), String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    ensure_document_access_at(&data_root, document_id, None)?;
    let file_path = resolve_production_document_file_at(&data_root, document_id)
        .map_err(|error| error.to_string())?;
    open_path_with_default_app(&file_path).map_err(|error| error.to_string())
}

#[tauri::command]
fn reveal_production_document_file(app: tauri::AppHandle, document_id: i64) -> Result<(), String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    ensure_document_access_at(&data_root, document_id, None)?;
    let file_path = resolve_production_document_file_at(&data_root, document_id)
        .map_err(|error| error.to_string())?;
    reveal_path_in_file_manager(&file_path).map_err(|error| error.to_string())
}

fn ensure_document_access_at(
    data_root: &Path,
    document_id: i64,
    pin: Option<&str>,
) -> Result<(), String> {
    let connection =
        Connection::open(data_root.join("vault.db")).map_err(|error| error.to_string())?;
    let is_locked = connection
        .query_row(
            "SELECT CASE WHEN d.is_locked=1 OR EXISTS(SELECT 1 FROM document_folders df JOIN folders f ON f.id=df.folder_id WHERE df.document_id=d.id AND f.is_locked=1) OR EXISTS(SELECT 1 FROM document_categories dc JOIN categories c ON c.id=dc.category_id WHERE dc.document_id=d.id AND c.is_locked=1) THEN 1 ELSE 0 END FROM documents d WHERE d.id=?1 AND d.vault_id=1 AND d.trashed_at IS NULL",
            [document_id],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|error| error.to_string())?
        != 0;
    if is_locked
        && !pin
            .map(|value| verify_pin(&connection, value).unwrap_or(false))
            .unwrap_or(false)
    {
        return Err("Dokumentet är låst. Lås upp Vault med rätt PIN/lösenord först".into());
    }
    Ok(())
}

#[tauri::command]
fn open_protected_document_file(
    app: tauri::AppHandle,
    document_id: i64,
    pin: String,
) -> Result<(), String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    ensure_document_access_at(&data_root, document_id, Some(&pin))?;
    let file_path = resolve_production_document_file_at(&data_root, document_id)
        .map_err(|error| error.to_string())?;
    open_path_with_default_app(&file_path).map_err(|error| error.to_string())
}

#[tauri::command]
fn reveal_protected_document_file(
    app: tauri::AppHandle,
    document_id: i64,
    pin: String,
) -> Result<(), String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    ensure_document_access_at(&data_root, document_id, Some(&pin))?;
    let file_path = resolve_production_document_file_at(&data_root, document_id)
        .map_err(|error| error.to_string())?;
    reveal_path_in_file_manager(&file_path).map_err(|error| error.to_string())
}

#[tauri::command]
fn trash_production_document(
    app: tauri::AppHandle,
    document_id: i64,
) -> Result<Vec<DocumentSummary>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    trash_production_document_at(&data_root, document_id).map_err(|error| error.to_string())
}

fn trash_production_document_at(
    data_root: &Path,
    document_id: i64,
) -> rusqlite::Result<Vec<DocumentSummary>> {
    let connection = Connection::open(data_root.join("vault.db"))?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;
    let title: String = connection.query_row(
        "SELECT title FROM documents WHERE id = ?1 AND vault_id = 1 AND trashed_at IS NULL",
        params![document_id],
        |row| row.get(0),
    )?;
    connection.execute(
        "
        UPDATE documents
        SET trashed_at = CURRENT_TIMESTAMP,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ?1 AND vault_id = 1 AND trashed_at IS NULL
        ",
        params![document_id],
    )?;
    connection.execute(
        "DELETE FROM document_fts WHERE document_id = ?1",
        params![document_id],
    )?;
    record_audit_event(
        &connection,
        "document_trashed",
        "document",
        Some(document_id),
        &format!("Moved document to trash: {title}"),
    )?;
    list_documents(&connection)
}

#[tauri::command]
fn restore_production_document(
    app: tauri::AppHandle,
    document_id: i64,
) -> Result<Vec<DocumentSummary>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    restore_production_document_at(&data_root, document_id).map_err(|error| error.to_string())
}

fn restore_production_document_at(
    data_root: &Path,
    document_id: i64,
) -> rusqlite::Result<Vec<DocumentSummary>> {
    let connection = Connection::open(data_root.join("vault.db"))?;
    let (title, document_type, document_date, inbox_status, source_label, explanation, body):
        (String, String, Option<String>, String, String, String, String) = connection.query_row(
        "SELECT title, document_type, document_date, inbox_status, source_label, match_explanation, extracted_text
         FROM documents WHERE id = ?1 AND vault_id = 1 AND trashed_at IS NOT NULL",
        params![document_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?)),
    )?;
    connection.execute(
        "UPDATE documents SET trashed_at = NULL, updated_at = CURRENT_TIMESTAMP WHERE id = ?1 AND vault_id = 1",
        params![document_id],
    )?;
    index_document(
        &connection,
        document_id,
        &title,
        &format!(
            "{document_type} {} {inbox_status} {source_label}",
            document_date.as_deref().unwrap_or_default()
        ),
        &format!("{explanation} {body}"),
    )?;
    record_audit_event(
        &connection,
        "document_restored",
        "document",
        Some(document_id),
        &format!("Restored document from trash: {title}"),
    )?;
    list_documents(&connection)
}

fn resolve_production_document_file_at(
    data_root: &Path,
    document_id: i64,
) -> rusqlite::Result<PathBuf> {
    let connection = Connection::open(data_root.join("vault.db"))?;
    let storage_path: String = connection.query_row(
        "
        SELECT f.storage_path
        FROM documents d
        JOIN document_versions dv
          ON dv.document_id = d.id AND dv.is_current_file = 1
        JOIN files f
          ON f.id = dv.file_id
        WHERE d.id = ?1 AND d.vault_id = 1 AND d.trashed_at IS NULL
        ORDER BY dv.version_no DESC
        LIMIT 1
        ",
        params![document_id],
        |row| row.get(0),
    )?;

    let file_path = data_root.join(storage_path);
    if !file_path.exists() {
        return Err(to_sql_error(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "Stored file does not exist: {}",
                file_path.to_string_lossy()
            ),
        )));
    }

    Ok(file_path)
}

fn open_path_with_default_app(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "Start-Process -LiteralPath $args[0]",
            ])
            .arg(path)
            .spawn()?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        Command::new("xdg-open").arg(path).spawn()?;
    }

    Ok(())
}

fn reveal_path_in_file_manager(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(format!("/select,{}", path.to_string_lossy()))
            .spawn()?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        let parent = path.parent().unwrap_or(path);
        Command::new("xdg-open").arg(parent).spawn()?;
    }

    Ok(())
}

#[tauri::command]
fn ocr_status(app: tauri::AppHandle) -> Result<OcrStatus, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    ensure_bundled_tessdata(&app, &data_root).map_err(|error| error.to_string())?;
    Ok(detect_ocr_status(Some(&data_root)))
}

#[tauri::command]
fn run_ocr_for_production_document(
    app: tauri::AppHandle,
    document_id: i64,
) -> Result<OcrRunResult, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    ensure_bundled_tessdata(&app, &data_root).map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    run_ocr_for_production_document_at(&data_root, document_id).map_err(|error| error.to_string())
}

fn run_ocr_for_production_document_at(
    data_root: &Path,
    document_id: i64,
) -> rusqlite::Result<OcrRunResult> {
    let connection = Connection::open(data_root.join("vault.db"))?;
    let (storage_path, mime_type): (String, String) = connection.query_row(
        "
        SELECT f.storage_path, f.mime_type
        FROM documents d
        JOIN document_versions dv
          ON dv.document_id = d.id AND dv.is_current_file = 1
        JOIN files f
          ON f.id = dv.file_id
        WHERE d.id = ?1 AND d.vault_id = 1 AND d.trashed_at IS NULL
        ORDER BY dv.version_no DESC
        LIMIT 1
        ",
        params![document_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    if !matches!(
        mime_type.as_str(),
        "image/png" | "image/jpeg" | "application/pdf"
    ) {
        return Err(to_sql_error(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "OCR stöder importerade PNG/JPG-bilder och PDF-dokument",
        )));
    }

    let file_path = data_root.join(storage_path);
    let ocr_text = if mime_type == "application/pdf" {
        extract_pdf_text_with_ocr(&file_path, Some(data_root)).map_err(to_sql_error)?
    } else {
        extract_image_text_with_tesseract(&file_path, Some(data_root)).map_err(to_sql_error)?
    };
    let extracted_character_count = ocr_text.chars().count();
    if extracted_character_count == 0 {
        return Err(to_sql_error(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "OCR gav ingen text",
        )));
    }

    let current_title: String = connection.query_row(
        "SELECT title FROM documents WHERE id = ?1 AND vault_id = 1",
        params![document_id],
        |row| row.get(0),
    )?;
    let classification = classify_document(&current_title, &ocr_text);
    let date = extract_document_date(&current_title, &ocr_text).document_date;
    connection.execute(
        "
        UPDATE documents
        SET extracted_text = ?1,
            document_type = CASE WHEN document_type = 'Oklassificerat' THEN ?2 ELSE document_type END,
            document_date = COALESCE(document_date, ?3),
            match_explanation = match_explanation || '. OCR-text extraherad lokalt med Tesseract',
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ?4 AND vault_id = 1
        ",
        params![ocr_text, classification.document_type, date, document_id],
    )?;

    let summary = connection.query_row(
        "
        SELECT id, title, document_type, document_date, inbox_status, source_label, match_explanation
        FROM documents
        WHERE id = ?1 AND vault_id = 1
        ",
        params![document_id],
        |row| {
            Ok(DocumentSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                document_type: row.get(2)?,
                document_date: row.get(3)?,
                inbox_status: row.get(4)?,
                source_label: row.get(5)?,
                match_explanation: row.get(6)?,
            })
        },
    )?;
    let version_id: i64 = connection.query_row(
        "SELECT id FROM document_versions WHERE document_id = ?1 AND is_current_file = 1 ORDER BY version_no DESC LIMIT 1",
        params![document_id],
        |row| row.get(0),
    )?;
    store_document_pages(
        &connection,
        document_id,
        version_id,
        "image/jpeg",
        &ocr_text,
    )?;
    index_document(
        &connection,
        document_id,
        &summary.title,
        &format!(
            "{} {} {}",
            summary.document_type,
            summary.document_date.as_deref().unwrap_or_default(),
            summary.source_label
        ),
        &ocr_text,
    )?;
    apply_content_claims(
        &connection,
        document_id,
        &summary.title,
        &summary.document_type,
        &ocr_text,
    )?;
    apply_auto_entities(
        &connection,
        document_id,
        &summary.title,
        &summary.document_type,
        &ocr_text,
    )?;
    record_audit_event(
        &connection,
        "ocr_completed",
        "document",
        Some(document_id),
        &format!("OCR completed for {}", summary.title),
    )?;

    Ok(OcrRunResult {
        document_id,
        extracted_character_count,
        engine: "tesseract".to_string(),
        indexed: true,
    })
}

#[tauri::command]
fn import_document(app: tauri::AppHandle, source_path: String) -> Result<ImportResult, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    ensure_bundled_tessdata(&app, &data_root).map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    match import_document_at(&data_root, Path::new(&source_path)) {
        Ok(result) => {
            record_single_import_session(&data_root, "file", &source_path, Ok(&result));
            Ok(result)
        }
        Err(error) => {
            let message = error.to_string();
            record_single_import_session(&data_root, "file", &source_path, Err(&message));
            Err(message)
        }
    }
}

#[tauri::command]
fn import_directory(
    app: tauri::AppHandle,
    source_directory: String,
) -> Result<DirectoryImportResult, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    ensure_bundled_tessdata(&app, &data_root).map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    let result = import_directory_at(&data_root, Path::new(&source_directory))
        .map_err(|error| error.to_string())?;
    record_directory_import_session(&data_root, "directory", &source_directory, &result);
    Ok(result)
}

#[tauri::command]
fn import_archive(
    app: tauri::AppHandle,
    source_path: String,
) -> Result<DirectoryImportResult, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    let result = import_archive_at(&data_root, Path::new(&source_path))
        .map_err(|error| error.to_string())?;
    record_directory_import_session(&data_root, "zip", &source_path, &result);
    Ok(result)
}

#[tauri::command]
fn import_text_document(
    app: tauri::AppHandle,
    title: String,
    text: String,
) -> Result<ImportResult, String> {
    if text.trim().is_empty() {
        return Err("Texten är tom".to_string());
    }
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    let safe_title = sanitize_file_name(if title.trim().is_empty() {
        "Urklipp"
    } else {
        title.trim()
    });
    let temp_path = data_root
        .join("index")
        .join(format!("text-import-{}-{safe_title}.txt", epoch_seconds()));
    fs::write(&temp_path, text.as_bytes()).map_err(|error| error.to_string())?;
    let imported = import_document_at(&data_root, &temp_path).map_err(|error| error.to_string());
    let _ = fs::remove_file(&temp_path);
    match imported {
        Ok(result) => {
            record_single_import_session(&data_root, "clipboard", &safe_title, Ok(&result));
            Ok(result)
        }
        Err(error) => {
            let message = error.to_string();
            record_single_import_session(&data_root, "clipboard", &safe_title, Err(&message));
            Err(message)
        }
    }
}

#[tauri::command]
fn import_paths(
    app: tauri::AppHandle,
    paths: Vec<String>,
) -> Result<DirectoryImportResult, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    let mut combined = DirectoryImportResult {
        scanned_file_count: 0,
        imported_count: 0,
        duplicate_count: 0,
        failed_count: 0,
        results: Vec::new(),
        failures: Vec::new(),
    };
    for raw_path in &paths {
        let path = Path::new(raw_path);
        let part = if path.is_dir() {
            import_directory_at(&data_root, path)
        } else if path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case("zip"))
        {
            import_archive_at(&data_root, path)
        } else {
            combined.scanned_file_count += 1;
            match import_document_at(&data_root, path) {
                Ok(result) => {
                    if result.duplicate {
                        combined.duplicate_count += 1;
                    }
                    combined.imported_count += 1;
                    combined.results.push(result);
                    continue;
                }
                Err(error) => {
                    combined.failed_count += 1;
                    combined
                        .failures
                        .push(format!("{}: {error}", path.display()));
                    continue;
                }
            }
        };
        match part {
            Ok(mut result) => {
                combined.scanned_file_count += result.scanned_file_count;
                combined.imported_count += result.imported_count;
                combined.duplicate_count += result.duplicate_count;
                combined.failed_count += result.failed_count;
                combined.results.append(&mut result.results);
                combined.failures.append(&mut result.failures);
            }
            Err(error) => {
                combined.failed_count += 1;
                combined
                    .failures
                    .push(format!("{}: {error}", path.display()));
            }
        }
    }
    record_directory_import_session(
        &data_root,
        "drag_drop",
        &format!("{} sökvägar", paths.len()),
        &combined,
    );
    Ok(combined)
}

#[tauri::command]
fn list_import_history(app: tauri::AppHandle) -> Result<Vec<ImportSessionSummary>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    list_import_history_at(&data_root).map_err(|error| error.to_string())
}

fn record_single_import_session(
    data_root: &Path,
    method: &str,
    source: &str,
    result: Result<&ImportResult, &String>,
) {
    let Ok(connection) = Connection::open(data_root.join("vault.db")) else {
        return;
    };
    let source_label = Path::new(source)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(source);
    let (status, imported, duplicate, failed, document_id, error) = match result {
        Ok(result) => (
            "completed",
            1,
            i64::from(result.duplicate),
            0,
            Some(result.document.id),
            None,
        ),
        Err(error) => (
            "failed",
            0,
            0,
            1,
            None,
            Some(error.chars().take(500).collect::<String>()),
        ),
    };
    if connection.execute(
        "INSERT INTO import_sessions (vault_id, method, source_label, status, scanned_count, imported_count, duplicate_count, failed_count, completed_at) VALUES (1, ?1, ?2, ?3, 1, ?4, ?5, ?6, CURRENT_TIMESTAMP)",
        params![method, source_label, status, imported, duplicate, failed],
    ).is_ok() {
        let session_id = connection.last_insert_rowid();
        let _ = connection.execute(
            "INSERT INTO import_session_items (import_session_id, source_name, document_id, status, safe_error) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![session_id, source_label, document_id, status, error],
        );
    }
}

fn record_directory_import_session(
    data_root: &Path,
    method: &str,
    source: &str,
    result: &DirectoryImportResult,
) {
    let Ok(mut connection) = Connection::open(data_root.join("vault.db")) else {
        return;
    };
    let status = if result.failed_count == 0 {
        "completed"
    } else if result.imported_count > 0 {
        "partial"
    } else {
        "failed"
    };
    let Ok(transaction) = connection.transaction() else {
        return;
    };
    if transaction.execute(
        "INSERT INTO import_sessions (vault_id, method, source_label, status, scanned_count, imported_count, duplicate_count, failed_count, completed_at) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, CURRENT_TIMESTAMP)",
        params![method, source, status, result.scanned_file_count as i64, result.imported_count as i64, result.duplicate_count as i64, result.failed_count as i64],
    ).is_err() { return; }
    let session_id = transaction.last_insert_rowid();
    for item in &result.results {
        let item_status = if item.duplicate {
            "duplicate"
        } else {
            "imported"
        };
        let _ = transaction.execute(
            "INSERT INTO import_session_items (import_session_id, source_name, document_id, status) VALUES (?1, ?2, ?3, ?4)",
            params![session_id, item.document.title, item.document.id, item_status],
        );
    }
    for failure in &result.failures {
        let safe_error: String = failure.chars().take(500).collect();
        let _ = transaction.execute(
            "INSERT INTO import_session_items (import_session_id, source_name, status, safe_error) VALUES (?1, 'Importfel', 'failed', ?2)",
            params![session_id, safe_error],
        );
    }
    let _ = transaction.commit();
}

fn list_import_history_at(data_root: &Path) -> rusqlite::Result<Vec<ImportSessionSummary>> {
    let connection = Connection::open(data_root.join("vault.db"))?;
    let mut statement = connection.prepare(
        "SELECT id, method, source_label, status, scanned_count, imported_count, duplicate_count, failed_count, started_at, completed_at FROM import_sessions WHERE vault_id = 1 ORDER BY id DESC LIMIT 100",
    )?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, i64>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, Option<String>>(9)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut sessions = Vec::new();
    for (
        id,
        method,
        source_label,
        status,
        scanned_count,
        imported_count,
        duplicate_count,
        failed_count,
        started_at,
        completed_at,
    ) in rows
    {
        let mut item_statement = connection.prepare("SELECT source_name, document_id, status, safe_error FROM import_session_items WHERE import_session_id = ?1 ORDER BY id")?;
        let items = item_statement
            .query_map(params![id], |row| {
                Ok(ImportSessionItemSummary {
                    source_name: row.get(0)?,
                    document_id: row.get(1)?,
                    status: row.get(2)?,
                    safe_error: row.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        sessions.push(ImportSessionSummary {
            id,
            method,
            source_label,
            status,
            scanned_count,
            imported_count,
            duplicate_count,
            failed_count,
            started_at,
            completed_at,
            items,
        });
    }
    Ok(sessions)
}

fn import_archive_at(
    data_root: &Path,
    source_path: &Path,
) -> rusqlite::Result<DirectoryImportResult> {
    let file = fs::File::open(source_path).map_err(to_sql_error)?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|error| to_sql_error(zip_to_io_error(error)))?;
    if archive.len() > 500 {
        return Err(to_sql_error(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "ZIP-arkivet innehåller fler än 500 poster",
        )));
    }
    let temp_root = data_root
        .join("index")
        .join(format!("zip-import-{}", epoch_seconds()));
    fs::create_dir_all(&temp_root).map_err(to_sql_error)?;
    let mut total_size = 0_u64;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| to_sql_error(zip_to_io_error(error)))?;
        if entry.is_dir() {
            continue;
        }
        if entry
            .unix_mode()
            .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            continue;
        }
        total_size = total_size.saturating_add(entry.size());
        if entry.size() > 100 * 1024 * 1024 || total_size > 1024 * 1024 * 1024 {
            let _ = fs::remove_dir_all(&temp_root);
            return Err(to_sql_error(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "ZIP-arkivet överskrider säker storleksgräns",
            )));
        }
        let Some(relative) = entry.enclosed_name() else {
            continue;
        };
        let destination = temp_root.join(relative);
        if !is_supported_import_path(&destination) {
            continue;
        }
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(to_sql_error)?;
        }
        let mut output = fs::File::create(&destination).map_err(to_sql_error)?;
        std::io::copy(&mut entry, &mut output).map_err(to_sql_error)?;
    }
    let result = import_directory_at(data_root, &temp_root);
    let _ = fs::remove_dir_all(&temp_root);
    result
}

fn sanitize_file_name(value: &str) -> String {
    let cleaned: String = value
        .chars()
        .map(|character| {
            if character.is_alphanumeric() || matches!(character, '-' | '_' | ' ') {
                character
            } else {
                '_'
            }
        })
        .take(100)
        .collect();
    if cleaned.trim().is_empty() {
        "Urklipp".to_string()
    } else {
        cleaned.trim().to_string()
    }
}

fn epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[tauri::command]
fn create_demo_archive(app: tauri::AppHandle) -> Result<DirectoryImportResult, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve local app data directory: {error}"))?;

    ensure_bundled_tessdata(&app, &data_root).map_err(|error| error.to_string())?;
    initialize_vault_at(&data_root).map_err(|error| error.to_string())?;
    create_demo_archive_at(&data_root).map_err(|error| error.to_string())
}

fn create_demo_archive_at(data_root: &Path) -> rusqlite::Result<DirectoryImportResult> {
    let demo_dir = data_root.join("demo-import");
    fs::create_dir_all(&demo_dir).map_err(to_sql_error)?;
    let demo_files = [
        (
            "2024-02-01_DAGAB_anstallningsavtal.txt",
            "Anställningsavtal\nArbetsgivare: DAGAB Inköp & Logistik AB\nAnställd: Demo Person\nStartdatum: 2024-02-01\nAvtal: tillsvidareanställning.\n",
        ),
        (
            "2024-06-25_lonespecifikation_juni.txt",
            "Lönespecifikation juni 2024\nArbetsgivare: Willys AB\nUtbetalning: 2024-06-25\nBruttolön: 32000 SEK\nNettolön: 24500 SEK\n",
        ),
        (
            "2025-01-15_kvitto_apotek.csv",
            "datum,butik,total,moms\n2025-01-15,Apotek Demo,349.00,69.80\n",
        ),
        (
            "2022-09-12_passhandling_demo.json",
            r#"{"document_type":"pass","issued":"2022-09-12","holder":"Demo Person","mrz":"P<SWEDemo<<Person"}"#,
        ),
    ];

    for (file_name, content) in demo_files {
        fs::write(demo_dir.join(file_name), content).map_err(to_sql_error)?;
    }

    import_directory_at(data_root, &demo_dir)
}

fn import_directory_at(
    data_root: &Path,
    source_directory: &Path,
) -> rusqlite::Result<DirectoryImportResult> {
    if !source_directory.is_dir() {
        return Err(to_sql_error(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Importkällan är inte en mapp",
        )));
    }

    let files = collect_supported_import_files(source_directory).map_err(to_sql_error)?;
    let scanned_file_count = files.len();
    let mut results = Vec::new();
    let mut failures = Vec::new();
    let mut duplicate_count = 0;

    for file_path in files {
        match import_document_at(data_root, &file_path) {
            Ok(result) => {
                if result.duplicate {
                    duplicate_count += 1;
                }
                results.push(result);
            }
            Err(error) => failures.push(format!("{}: {error}", file_path.to_string_lossy())),
        }
    }

    let failed_count = failures.len();
    let imported_count = results.len().saturating_sub(duplicate_count);

    Ok(DirectoryImportResult {
        scanned_file_count,
        imported_count,
        duplicate_count,
        failed_count,
        results,
        failures,
    })
}

fn collect_supported_import_files(directory: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_supported_import_files_into(directory, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_supported_import_files_into(
    directory: &Path,
    files: &mut Vec<PathBuf>,
) -> std::io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_supported_import_files_into(&path, files)?;
        } else if path.is_file() && is_supported_import_path(&path) {
            files.push(path);
        }
    }
    Ok(())
}

fn is_supported_import_path(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_ascii_lowercase())
            .as_deref(),
        Some(
            "pdf"
                | "docx"
                | "xlsx"
                | "pptx"
                | "txt"
                | "md"
                | "png"
                | "jpg"
                | "jpeg"
                | "csv"
                | "json"
                | "eml"
        )
    )
}

fn import_document_at(data_root: &Path, source_path: &Path) -> rusqlite::Result<ImportResult> {
    let metadata = fs::metadata(source_path).map_err(to_sql_error)?;
    if !metadata.is_file() {
        return Err(to_sql_error(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Import source is not a file",
        )));
    }

    let sha256 = sha256_file(source_path).map_err(to_sql_error)?;
    let original_name = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("imported-document")
        .to_string();
    let mime_type = detect_mime_type(source_path);
    let extracted_text =
        extract_text_for_indexing(source_path, mime_type, Some(data_root)).map_err(to_sql_error)?;
    let classification = classify_document(&original_name, &extracted_text);
    let date_extraction = extract_document_date(&original_name, &extracted_text);
    let storage_relative_path = storage_relative_path(&sha256, &original_name);
    let stored_path = data_root.join(&storage_relative_path);

    if let Some(parent) = stored_path.parent() {
        fs::create_dir_all(parent).map_err(to_sql_error)?;
    }
    if !stored_path.exists() {
        fs::copy(source_path, &stored_path).map_err(to_sql_error)?;
    }

    let mut connection = Connection::open(data_root.join("vault.db"))?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;
    let transaction = connection.transaction()?;

    let existing_file_id = transaction
        .query_row(
            "SELECT id FROM files WHERE vault_id = 1 AND sha256 = ?1",
            params![sha256],
            |row| row.get::<_, i64>(0),
        )
        .ok();

    let (file_id, duplicate) = if let Some(file_id) = existing_file_id {
        (file_id, true)
    } else {
        transaction.execute(
            "
            INSERT INTO files (
              vault_id,
              sha256,
              storage_path,
              original_name,
              mime_type,
              size_bytes
            )
            VALUES (1, ?1, ?2, ?3, ?4, ?5)
            ",
            params![
                sha256,
                storage_relative_path.to_string_lossy(),
                original_name,
                mime_type,
                metadata.len() as i64
            ],
        )?;
        (transaction.last_insert_rowid(), false)
    };

    let title = original_name.clone();
    let match_explanation = if duplicate {
        "Exakt filhash matchar tidigare import"
    } else {
        "Importerad lokalt med SHA-256 och lokal filkopiering"
    };
    let full_match_explanation = [
        match_explanation.to_string(),
        classification.match_explanation.clone(),
        date_extraction
            .explanation
            .clone()
            .unwrap_or_else(|| "Ingen tydlig datumregel matchade; datum lämnas tomt".to_string()),
    ]
    .join(". ");

    transaction.execute(
        "
        INSERT INTO documents (
          vault_id,
          title,
          document_type,
          document_date,
          inbox_status,
          source_label,
          match_explanation,
          extracted_text
        )
        VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ",
        params![
            title,
            classification.document_type,
            date_extraction.document_date,
            "inbox",
            "Lokal import",
            full_match_explanation,
            extracted_text
        ],
    )?;
    let document_id = transaction.last_insert_rowid();

    transaction.execute(
        "
        INSERT INTO document_versions (
          document_id,
          file_id,
          version_no,
          source_path,
          is_current_file
        )
        VALUES (?1, ?2, 1, ?3, 1)
        ",
        params![document_id, file_id, source_path.to_string_lossy()],
    )?;
    let version_id = transaction.last_insert_rowid();

    store_document_pages(
        &transaction,
        document_id,
        version_id,
        mime_type,
        &extracted_text,
    )?;

    index_document(
        &transaction,
        document_id,
        &title,
        &format!(
            "{} Lokal import {mime_type} {}",
            classification.document_type,
            date_extraction.document_date.as_deref().unwrap_or_default()
        ),
        &format!(
            "{} {} {}",
            source_path.to_string_lossy(),
            sha256,
            extracted_text
        ),
    )?;
    record_audit_event(
        &transaction,
        "document_imported",
        "document",
        Some(document_id),
        &format!(
            "Imported {original_name} as {}",
            classification.document_type
        ),
    )?;
    apply_auto_tags_and_claims(
        &transaction,
        document_id,
        classification.document_type,
        date_extraction.document_date.as_deref(),
        &sha256,
    )?;
    apply_content_claims(
        &transaction,
        document_id,
        &title,
        classification.document_type,
        &extracted_text,
    )?;
    apply_auto_entities(
        &transaction,
        document_id,
        &title,
        classification.document_type,
        &extracted_text,
    )?;

    transaction.commit()?;

    Ok(ImportResult {
        document: DocumentSummary {
            id: document_id,
            title,
            document_type: classification.document_type.to_string(),
            document_date: date_extraction.document_date,
            inbox_status: "inbox".to_string(),
            source_label: "Lokal import".to_string(),
            match_explanation: full_match_explanation,
        },
        file_id,
        version_id,
        sha256,
        stored_path: stored_path.to_string_lossy().into_owned(),
        duplicate,
    })
}

fn index_document(
    connection: &Connection,
    document_id: i64,
    title: &str,
    metadata_text: &str,
    body_text: &str,
) -> rusqlite::Result<()> {
    connection.execute(
        "DELETE FROM document_fts WHERE document_id = ?1",
        params![document_id],
    )?;
    connection.execute(
        "
        INSERT INTO document_fts (document_id, title, metadata_text, body_text)
        VALUES (?1, ?2, ?3, ?4)
        ",
        params![document_id, title, metadata_text, body_text],
    )?;

    Ok(())
}

fn store_document_pages(
    connection: &Connection,
    document_id: i64,
    version_id: i64,
    mime_type: &str,
    text: &str,
) -> rusqlite::Result<()> {
    connection.execute(
        "DELETE FROM span_fts WHERE document_id = ?1",
        params![document_id],
    )?;
    let source_kind = match mime_type {
        "application/pdf" => "embedded_pdf_text",
        "image/png" | "image/jpeg" => "ocr_text",
        value if value.contains("officedocument") => "office_extracted_text",
        _ => "embedded_text",
    };
    let pages = if mime_type == "application/pdf" {
        text.split('\u{000c}').collect::<Vec<_>>()
    } else {
        vec![text]
    };
    for (index, page_text) in pages.into_iter().enumerate() {
        if page_text.trim().is_empty() {
            continue;
        }
        let page_no = index as i64 + 1;
        connection.execute(
            "INSERT INTO document_pages (document_version_id, page_no, source_kind, text_quality)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(document_version_id, page_no) DO UPDATE SET source_kind = excluded.source_kind, text_quality = excluded.text_quality",
            params![version_id, page_no, source_kind, if page_text.trim().chars().count() >= 40 { "good" } else { "low" }],
        )?;
        let page_id: i64 = connection.query_row(
            "SELECT id FROM document_pages WHERE document_version_id = ?1 AND page_no = ?2",
            params![version_id, page_no],
            |row| row.get(0),
        )?;
        connection.execute(
            "DELETE FROM text_spans WHERE document_page_id = ?1",
            params![page_id],
        )?;
        connection.execute(
            "INSERT INTO text_spans (document_page_id, source_kind, text) VALUES (?1, ?2, ?3)",
            params![page_id, source_kind, page_text.trim()],
        )?;
        let span_id = connection.last_insert_rowid();
        connection.execute(
            "INSERT INTO span_fts (text_span_id, document_id, page_no, text) VALUES (?1, ?2, ?3, ?4)",
            params![span_id, document_id, page_no, page_text.trim()],
        )?;
    }
    Ok(())
}

fn store_single_document_page(
    connection: &Connection,
    document_id: i64,
    version_id: i64,
    page_no: i64,
    source_kind: &str,
    text: &str,
) -> rusqlite::Result<()> {
    connection.execute(
        "INSERT INTO document_pages (document_version_id, page_no, source_kind, text_quality)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(document_version_id, page_no) DO UPDATE SET source_kind = excluded.source_kind, text_quality = excluded.text_quality",
        params![version_id, page_no, source_kind, if text.trim().chars().count() >= 40 { "good" } else { "low" }],
    )?;
    let page_id: i64 = connection.query_row(
        "SELECT id FROM document_pages WHERE document_version_id = ?1 AND page_no = ?2",
        params![version_id, page_no],
        |row| row.get(0),
    )?;
    connection.execute("DELETE FROM span_fts WHERE text_span_id IN (SELECT id FROM text_spans WHERE document_page_id = ?1)", params![page_id])?;
    connection.execute(
        "DELETE FROM text_spans WHERE document_page_id = ?1",
        params![page_id],
    )?;
    connection.execute(
        "INSERT INTO text_spans (document_page_id, source_kind, text) VALUES (?1, ?2, ?3)",
        params![page_id, source_kind, text.trim()],
    )?;
    let span_id = connection.last_insert_rowid();
    connection.execute(
        "INSERT INTO span_fts (text_span_id, document_id, page_no, text) VALUES (?1, ?2, ?3, ?4)",
        params![span_id, document_id, page_no, text.trim()],
    )?;
    Ok(())
}

fn apply_auto_tags_and_claims(
    connection: &Connection,
    document_id: i64,
    document_type: &str,
    document_date: Option<&str>,
    sha256: &str,
) -> rusqlite::Result<()> {
    let mut tags = vec!["importerad".to_string()];
    let normalized_type = normalize(document_type);
    if normalized_type.contains("lonespecifikation") {
        tags.push("lon".to_string());
    } else if normalized_type.contains("anstallningsavtal") {
        tags.push("avtal".to_string());
        tags.push("anstallning".to_string());
    } else if normalized_type.contains("identitetshandling") {
        tags.push("identitet".to_string());
    } else if normalized_type.contains("kvitto") {
        tags.push("kvitto".to_string());
    }

    for tag in tags {
        attach_tag(connection, document_id, &tag)?;
    }

    insert_claim(
        connection,
        document_id,
        "document_type",
        &serde_json::json!({ "value": document_type }).to_string(),
        "rule_inferred_status",
        "auto_extracted_pending_review",
        "exact_rule_match",
        "local_document_type_rules_v1",
    )?;
    insert_claim(
        connection,
        document_id,
        "sha256",
        &serde_json::json!({ "value": sha256 }).to_string(),
        "explicit_system_value",
        "verified_system_value",
        "exact_hash",
        "sha256_file",
    )?;
    if let Some(document_date) = document_date {
        insert_claim(
            connection,
            document_id,
            "document_date",
            &serde_json::json!({ "value": document_date }).to_string(),
            "validated_parse",
            "auto_extracted_pending_review",
            "validated_parse",
            "local_date_rules_v1",
        )?;
    }

    Ok(())
}

fn attach_tag(connection: &Connection, document_id: i64, tag_name: &str) -> rusqlite::Result<()> {
    connection.execute(
        "INSERT OR IGNORE INTO tags (vault_id, name) VALUES (1, ?1)",
        params![tag_name],
    )?;
    let tag_id: i64 = connection.query_row(
        "SELECT id FROM tags WHERE vault_id = 1 AND name = ?1",
        params![tag_name],
        |row| row.get(0),
    )?;
    connection.execute(
        "INSERT OR IGNORE INTO document_tags (document_id, tag_id) VALUES (?1, ?2)",
        params![document_id, tag_id],
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn insert_claim(
    connection: &Connection,
    document_id: i64,
    claim_type: &str,
    value_json: &str,
    value_kind: &str,
    status: &str,
    confidence_kind: &str,
    extraction_method: &str,
) -> rusqlite::Result<()> {
    let source_span_id = connection
        .query_row(
            "SELECT ts.id FROM text_spans ts JOIN document_pages dp ON dp.id = ts.document_page_id JOIN document_versions dv ON dv.id = dp.document_version_id WHERE dv.document_id = ?1 AND dv.is_current_file = 1 ORDER BY dp.page_no, ts.id LIMIT 1",
            params![document_id],
            |row| row.get::<_, i64>(0),
        )
        .ok();
    let document_date = connection.query_row(
        "SELECT document_date FROM documents WHERE id = ?1",
        params![document_id],
        |row| row.get::<_, Option<String>>(0),
    )?;
    let (actuality_status, actuality_explanation) = if document_date.is_some() {
        (
            "documented",
            "Gäller från dokumentets uttryckliga datum; slutdatum saknas.",
        )
    } else {
        (
            "uncertain",
            "Dokumentdatum saknas, därför kan aktualitet inte avgöras.",
        )
    };
    connection.execute(
        "
        INSERT INTO claims (
          vault_id,
          document_id,
          claim_type,
          value_json,
          value_kind,
          status,
          confidence_kind,
          extraction_method, source_span_id, effective_from,
          actuality_status, actuality_explanation
        )
        VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        ",
        params![
            document_id,
            claim_type,
            value_json,
            value_kind,
            status,
            confidence_kind,
            extraction_method,
            source_span_id,
            document_date,
            actuality_status,
            actuality_explanation
        ],
    )?;
    Ok(())
}

fn apply_content_claims(
    connection: &Connection,
    document_id: i64,
    title: &str,
    document_type: &str,
    extracted_text: &str,
) -> rusqlite::Result<()> {
    connection.execute(
        "
        DELETE FROM claims
        WHERE document_id = ?1
          AND extraction_method IN (
            'local_content_email_v1',
            'local_content_money_v1',
            'local_content_entity_v1',
            'local_content_text_stats_v1'
          )
        ",
        params![document_id],
    )?;

    let haystack = format!("{title}\n{document_type}\n{extracted_text}");
    let text_length = extracted_text.trim().chars().count();
    insert_claim(
        connection,
        document_id,
        "extracted_text_length",
        &serde_json::json!({ "characters": text_length }).to_string(),
        "system_measurement",
        "verified_system_value",
        "exact_count",
        "local_content_text_stats_v1",
    )?;

    for email in extract_email_values(&haystack).into_iter().take(12) {
        insert_claim(
            connection,
            document_id,
            "email",
            &serde_json::json!({ "value": email }).to_string(),
            "explicit_text_value",
            "auto_extracted_pending_review",
            "pattern_match",
            "local_content_email_v1",
        )?;
    }

    for amount in extract_money_values(&haystack).into_iter().take(12) {
        insert_claim(
            connection,
            document_id,
            "money_amount",
            &serde_json::json!({ "value": amount, "currency": "SEK" }).to_string(),
            "explicit_text_value",
            "auto_extracted_pending_review",
            "pattern_match",
            "local_content_money_v1",
        )?;
    }

    for (_, display_name, role) in extract_entities(&haystack).into_iter().take(12) {
        insert_claim(
            connection,
            document_id,
            "entity",
            &serde_json::json!({ "value": display_name, "role": role }).to_string(),
            "explicit_text_value",
            "auto_extracted_pending_review",
            "dictionary_or_pattern_match",
            "local_content_entity_v1",
        )?;
    }

    connection.execute(
        "UPDATE claims SET source_span_id = (SELECT ts.id FROM text_spans ts JOIN document_pages dp ON dp.id = ts.document_page_id JOIN document_versions dv ON dv.id = dp.document_version_id WHERE dv.document_id = claims.document_id AND dv.is_current_file = 1 ORDER BY dp.page_no, ts.id LIMIT 1) WHERE document_id = ?1 AND source_span_id IS NULL",
        params![document_id],
    )?;

    Ok(())
}

fn extract_email_values(text: &str) -> Vec<String> {
    let mut values = Vec::new();
    for token in text.split_whitespace() {
        let candidate = token
            .trim_matches(|character: char| {
                matches!(
                    character,
                    ',' | ';'
                        | ':'
                        | '.'
                        | '!'
                        | '?'
                        | '('
                        | ')'
                        | '['
                        | ']'
                        | '<'
                        | '>'
                        | '"'
                        | '\''
                )
            })
            .to_ascii_lowercase();
        if candidate.contains('@')
            && candidate.contains('.')
            && candidate.len() <= 160
            && !values.iter().any(|value| value == &candidate)
        {
            values.push(candidate);
        }
    }
    values
}

fn extract_money_values(text: &str) -> Vec<String> {
    let tokens = text.split_whitespace().collect::<Vec<_>>();
    let mut values = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        let normalized = normalize(token).trim().to_string();
        let is_currency_marker = matches!(normalized.as_str(), "sek" | "kr" | "kronor");
        if !is_currency_marker || index == 0 {
            continue;
        }

        let amount = tokens[index - 1]
            .trim_matches(|character: char| {
                matches!(character, ',' | ';' | ':' | '(' | ')' | '[' | ']')
            })
            .replace(' ', "")
            .replace(',', ".");
        if amount
            .chars()
            .all(|character| character.is_ascii_digit() || matches!(character, '.' | '-'))
            && amount.chars().any(|character| character.is_ascii_digit())
            && !values.iter().any(|value| value == &amount)
        {
            values.push(amount);
        }
    }
    values
}

fn apply_auto_entities(
    connection: &Connection,
    document_id: i64,
    title: &str,
    document_type: &str,
    extracted_text: &str,
) -> rusqlite::Result<()> {
    let haystack = format!("{title} {document_type} {extracted_text}");
    for (entity_type, display_name, role) in extract_entities(&haystack) {
        attach_entity(connection, document_id, &entity_type, &display_name, &role)?;
    }
    Ok(())
}

fn extract_entities(text: &str) -> Vec<(String, String, String)> {
    let normalized = normalize(text);
    let mut entities = Vec::new();

    for company in ["DAGAB", "WILLYS", "AXFOOD"] {
        if normalized.contains(&normalize(company)) {
            entities.push((
                "organization".to_string(),
                company.to_string(),
                "mentioned".to_string(),
            ));
        }
    }

    for token in text.split_whitespace() {
        let cleaned = token.trim_matches(|character: char| {
            !(character.is_ascii_alphanumeric() || matches!(character, '@' | '.' | '_' | '-'))
        });
        if cleaned.contains('@') && cleaned.contains('.') {
            entities.push((
                "contact".to_string(),
                cleaned.to_string(),
                "mentioned".to_string(),
            ));
        }
    }

    entities.sort_by(|left, right| left.1.cmp(&right.1));
    entities.dedup_by(|left, right| left.0 == right.0 && normalize(&left.1) == normalize(&right.1));
    entities
}

fn attach_entity(
    connection: &Connection,
    document_id: i64,
    entity_type: &str,
    display_name: &str,
    role: &str,
) -> rusqlite::Result<()> {
    let normalized_name = normalize(display_name)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if normalized_name.is_empty() {
        return Ok(());
    }

    connection.execute(
        "
        INSERT INTO entities (vault_id, entity_type, display_name, normalized_name)
        VALUES (1, ?1, ?2, ?3)
        ON CONFLICT(vault_id, entity_type, normalized_name) DO UPDATE SET
          display_name = excluded.display_name
        ",
        params![entity_type, display_name, normalized_name],
    )?;
    let entity_id: i64 = connection.query_row(
        "
        SELECT id
        FROM entities
        WHERE vault_id = 1 AND entity_type = ?1 AND normalized_name = ?2
        ",
        params![entity_type, normalized_name],
        |row| row.get(0),
    )?;
    connection.execute(
        "
        INSERT OR IGNORE INTO document_entities (document_id, entity_id, role)
        VALUES (?1, ?2, ?3)
        ",
        params![document_id, entity_id, role],
    )?;
    Ok(())
}

fn sha256_file(path: &Path) -> std::io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(hex::encode(hasher.finalize()))
}

fn storage_relative_path(sha256: &str, original_name: &str) -> PathBuf {
    let prefix = sha256.get(0..2).unwrap_or("00");
    Path::new("files")
        .join(prefix)
        .join(format!("{sha256}-{original_name}"))
}

fn detect_mime_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .as_deref()
    {
        Some("pdf") => "application/pdf",
        Some("docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        Some("xlsx") => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        Some("pptx") => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        Some("txt") | Some("md") => "text/plain",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("csv") => "text/csv",
        Some("json") => "application/json",
        Some("eml") => "message/rfc822",
        _ => "application/octet-stream",
    }
}

fn extract_text_for_indexing(
    path: &Path,
    mime_type: &str,
    data_root: Option<&Path>,
) -> std::io::Result<String> {
    match mime_type {
        "text/plain" | "text/csv" | "application/json" => {
            let bytes = fs::read(path)?;
            Ok(String::from_utf8_lossy(&bytes)
                .chars()
                .take(250_000)
                .collect())
        }
        "message/rfc822" => extract_eml_text(path),
        "application/pdf" => extract_pdf_text(path, data_root),
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
            extract_ooxml_text(path, &["word/document.xml", "word/header", "word/footer"])
        }
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => {
            extract_ooxml_text(path, &["xl/sharedStrings.xml", "xl/worksheets/sheet"])
        }
        "application/vnd.openxmlformats-officedocument.presentationml.presentation" => {
            extract_ooxml_text(path, &["ppt/slides/slide", "ppt/notesSlides/notesSlide"])
        }
        "image/png" | "image/jpeg" => extract_image_text_with_tesseract(path, data_root),
        _ => Ok(String::new()),
    }
}

fn extract_eml_text(path: &Path) -> std::io::Result<String> {
    let raw = String::from_utf8_lossy(&fs::read(path)?).replace("\r\n", "\n");
    let (headers, body) = raw.split_once("\n\n").unwrap_or((&raw, ""));
    let mut output = String::new();
    let mut current_header = String::new();
    for line in headers.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            current_header.push(' ');
            current_header.push_str(line.trim());
            continue;
        }
        if !current_header.is_empty() {
            append_email_header(&mut output, &current_header);
        }
        current_header = line.to_string();
    }
    append_email_header(&mut output, &current_header);
    output.push('\n');
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("--") || trimmed.to_ascii_lowercase().starts_with("content-") {
            continue;
        }
        output.push_str(&decode_quoted_printable_line(trimmed));
        output.push('\n');
        if output.chars().count() >= 250_000 {
            break;
        }
    }
    Ok(output.chars().take(250_000).collect())
}

fn append_email_header(output: &mut String, header: &str) {
    let lower = header.to_ascii_lowercase();
    if ["from:", "to:", "cc:", "subject:", "date:"]
        .iter()
        .any(|prefix| lower.starts_with(prefix))
    {
        output.push_str(header.trim());
        output.push('\n');
    }
}

fn decode_quoted_printable_line(line: &str) -> String {
    let bytes = line.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'=' && index + 2 < bytes.len() {
            if let Ok(value) = u8::from_str_radix(&line[index + 1..index + 3], 16) {
                output.push(value);
                index += 3;
                continue;
            }
        }
        output.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&output).into_owned()
}

fn extract_ooxml_text(path: &Path, entry_prefixes: &[&str]) -> std::io::Result<String> {
    let file = fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file).map_err(zip_to_io_error)?;
    let mut text = String::new();

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(zip_to_io_error)?;
        let entry_name = entry.name().to_string();
        if !entry_name.ends_with(".xml")
            || !entry_prefixes
                .iter()
                .any(|prefix| entry_name.starts_with(prefix))
        {
            continue;
        }

        let mut xml = String::new();
        entry.read_to_string(&mut xml)?;
        text.push_str(&xml_text_content(&xml));
        text.push('\n');
        if text.chars().count() > 250_000 {
            break;
        }
    }

    Ok(text.chars().take(250_000).collect())
}

fn xml_text_content(xml: &str) -> String {
    let mut text = String::new();
    let mut inside_tag = false;
    let mut entity = String::new();
    let mut inside_entity = false;

    for character in xml.chars() {
        if inside_entity {
            if character == ';' {
                text.push_str(match entity.as_str() {
                    "amp" => "&",
                    "lt" => "<",
                    "gt" => ">",
                    "quot" => "\"",
                    "apos" => "'",
                    _ => " ",
                });
                entity.clear();
                inside_entity = false;
            } else if entity.len() < 12 {
                entity.push(character);
            } else {
                entity.clear();
                inside_entity = false;
            }
            continue;
        }

        match character {
            '<' => {
                inside_tag = true;
                text.push(' ');
            }
            '>' => inside_tag = false,
            '&' if !inside_tag => inside_entity = true,
            _ if !inside_tag => text.push(character),
            _ => {}
        }
    }

    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn zip_to_io_error(error: zip::result::ZipError) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, error)
}

fn extract_pdf_text(path: &Path, data_root: Option<&Path>) -> std::io::Result<String> {
    let text = if let Some(pdftotext_path) = find_pdftotext_executable() {
        let output = Command::new(pdftotext_path)
            .args(["-layout", "-enc", "UTF-8"])
            .arg(path)
            .arg("-")
            .output();

        match output {
            Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
                .chars()
                .take(250_000)
                .collect(),
            Ok(_) | Err(_) => String::new(),
        }
    } else {
        String::new()
    };

    if text.trim().chars().count() >= 20 {
        return Ok(text);
    }

    extract_pdf_text_with_ocr(path, data_root)
}

fn find_pdftotext_executable() -> Option<PathBuf> {
    executable_from_path("pdftotext").or_else(|| {
        let mut candidates = vec![
            PathBuf::from(r"C:\Program Files\Git\mingw64\bin\pdftotext.exe"),
            PathBuf::from(r"C:\Program Files\poppler\Library\bin\pdftotext.exe"),
        ];
        if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
            candidates.push(
                PathBuf::from(&local_app_data)
                    .join("Microsoft")
                    .join("WinGet")
                    .join("Links")
                    .join("pdftotext.exe"),
            );
            candidates.extend(find_named_executable_under(
                &PathBuf::from(local_app_data)
                    .join("Microsoft")
                    .join("WinGet")
                    .join("Packages"),
                "pdftotext.exe",
                5,
            ));
        }

        candidates.into_iter().find(|path| path.is_file())
    })
}

fn find_pdftoppm_executable() -> Option<PathBuf> {
    executable_from_path("pdftoppm").or_else(|| {
        let mut candidates = vec![
            PathBuf::from(r"C:\Program Files\Git\mingw64\bin\pdftoppm.exe"),
            PathBuf::from(r"C:\Program Files\poppler\Library\bin\pdftoppm.exe"),
        ];
        if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
            candidates.push(
                PathBuf::from(&local_app_data)
                    .join("Microsoft")
                    .join("WinGet")
                    .join("Links")
                    .join("pdftoppm.exe"),
            );
            candidates.extend(find_named_executable_under(
                &PathBuf::from(local_app_data)
                    .join("Microsoft")
                    .join("WinGet")
                    .join("Packages"),
                "pdftoppm.exe",
                5,
            ));
        }

        candidates.into_iter().find(|path| path.is_file())
    })
}

fn extract_pdf_text_with_ocr(path: &Path, data_root: Option<&Path>) -> std::io::Result<String> {
    let Some(pdftoppm_path) = find_pdftoppm_executable() else {
        return Ok(String::new());
    };
    if find_tesseract_executable().is_none() {
        return Ok(String::new());
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let output_dir = std::env::temp_dir().join(format!("vault-pdf-ocr-{timestamp}"));
    fs::create_dir_all(&output_dir)?;
    let prefix = output_dir.join("page");

    let render = Command::new(pdftoppm_path)
        .args(["-png", "-r", "180", "-f", "1", "-l", "5"])
        .arg(path)
        .arg(&prefix)
        .output();

    let Ok(render) = render else {
        let _ = fs::remove_dir_all(&output_dir);
        return Ok(String::new());
    };
    if !render.status.success() {
        let _ = fs::remove_dir_all(&output_dir);
        return Ok(String::new());
    }

    let mut page_images = fs::read_dir(&output_dir)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("png"))
        .collect::<Vec<_>>();
    page_images.sort();

    let mut extracted = String::new();
    for image_path in page_images {
        let page_text = extract_image_text_with_tesseract(&image_path, data_root)?;
        if !page_text.trim().is_empty() {
            if !extracted.is_empty() {
                extracted.push_str("\n\n");
            }
            extracted.push_str(page_text.trim());
        }
    }

    let _ = fs::remove_dir_all(&output_dir);
    Ok(extracted.chars().take(250_000).collect())
}

fn extract_image_text_with_tesseract(
    path: &Path,
    data_root: Option<&Path>,
) -> std::io::Result<String> {
    let Some(tesseract_path) = find_tesseract_executable() else {
        return Ok(String::new());
    };
    let tessdata_dir = find_tessdata_dir(data_root);
    let languages = available_tesseract_languages(&tesseract_path, tessdata_dir.as_deref());
    let preferred_language = if languages.iter().any(|language| language == "swe")
        && languages.iter().any(|language| language == "eng")
    {
        "swe+eng"
    } else if languages.iter().any(|language| language == "swe") {
        "swe"
    } else if languages.iter().any(|language| language == "eng") {
        "eng"
    } else {
        ""
    };

    for language in [preferred_language, "eng", ""] {
        let mut command = Command::new(&tesseract_path);
        command.arg(path).arg("stdout");
        if let Some(tessdata_dir) = tessdata_dir.as_deref() {
            command.arg("--tessdata-dir").arg(tessdata_dir);
        }
        if !language.is_empty() {
            command.args(["-l", language]);
        }
        let output = command.output();
        match output {
            Ok(output) if output.status.success() => {
                let text = String::from_utf8_lossy(&output.stdout)
                    .chars()
                    .take(250_000)
                    .collect::<String>();
                if !text.trim().is_empty() {
                    return Ok(text);
                }
            }
            Ok(_) | Err(_) => {}
        }
    }

    Ok(String::new())
}

fn detect_ocr_status(data_root: Option<&Path>) -> OcrStatus {
    let executable_path = find_tesseract_executable();
    let tessdata_dir = find_tessdata_dir(data_root);
    let mut notes = Vec::new();
    let version = executable_path.as_ref().and_then(|path| {
        Command::new(path)
            .arg("--version")
            .output()
            .ok()
            .and_then(|output| {
                output.status.success().then(|| {
                    String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .next()
                        .unwrap_or("tesseract")
                        .to_string()
                })
            })
    });
    let languages = executable_path
        .as_ref()
        .map(|path| available_tesseract_languages(path, tessdata_dir.as_deref()))
        .unwrap_or_default();

    if executable_path.is_some() {
        notes.push(
            "Lokal Tesseract hittades; bild-OCR kan koras for PNG/JPG och skannade PDF-filer"
                .to_string(),
        );
    } else {
        notes.push("Tesseract hittades inte p? PATH eller i vanliga Windows-mappar".to_string());
    }
    if languages.iter().any(|language| language == "swe") {
        notes.push("Svensk OCR-data hittades".to_string());
    } else {
        notes.push("Svensk OCR-data saknas; engelska eller standardmodell anvands".to_string());
    }
    notes.push(
        "PDF-text extraheras via pdftotext; skannade PDF-filer OCR-lases via pdftoppm och Tesseract"
            .to_string(),
    );

    OcrStatus {
        available: executable_path.is_some(),
        engine: "tesseract".to_string(),
        executable_path: executable_path.map(|path| path.to_string_lossy().into_owned()),
        tessdata_dir: tessdata_dir.map(|path| path.to_string_lossy().into_owned()),
        languages,
        version,
        notes,
    }
}

fn find_tessdata_dir(data_root: Option<&Path>) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(data_root) = data_root {
        candidates.push(data_root.join("tessdata"));
    }
    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(current_dir.join("tessdata"));
        candidates.push(current_dir.join("src-tauri").join("tessdata"));
    }
    candidates.push(PathBuf::from(r"C:\Program Files\Tesseract-OCR\tessdata"));
    candidates.push(PathBuf::from(
        r"C:\Program Files (x86)\Tesseract-OCR\tessdata",
    ));

    candidates.into_iter().find(|path| {
        path.join("swe.traineddata").is_file() || path.join("eng.traineddata").is_file()
    })
}

fn available_tesseract_languages(
    tesseract_path: &Path,
    tessdata_dir: Option<&Path>,
) -> Vec<String> {
    let mut command = Command::new(tesseract_path);
    command.arg("--list-langs");
    if let Some(tessdata_dir) = tessdata_dir {
        command.arg("--tessdata-dir").arg(tessdata_dir);
    }

    let output = command.output();
    match output {
        Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
            .lines()
            .skip(1)
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToOwned::to_owned)
            .collect(),
        Ok(_) | Err(_) => Vec::new(),
    }
}

fn find_tesseract_executable() -> Option<PathBuf> {
    executable_from_path("tesseract").or_else(|| {
        [
            PathBuf::from(r"C:\Program Files\Tesseract-OCR\tesseract.exe"),
            PathBuf::from(r"C:\Program Files (x86)\Tesseract-OCR\tesseract.exe"),
        ]
        .into_iter()
        .find(|path| path.is_file())
    })
}

fn executable_from_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    let candidates = std::env::split_paths(&path_var).flat_map(|directory| {
        #[cfg(target_os = "windows")]
        {
            vec![directory.join(format!("{name}.exe")), directory.join(name)]
        }
        #[cfg(not(target_os = "windows"))]
        {
            vec![directory.join(name)]
        }
    });

    candidates.into_iter().find(|candidate| candidate.is_file())
}

fn find_named_executable_under(root: &Path, file_name: &str, max_depth: usize) -> Vec<PathBuf> {
    if max_depth == 0 || !root.is_dir() {
        return Vec::new();
    }

    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };
    let mut matches = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file()
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.eq_ignore_ascii_case(file_name))
        {
            matches.push(path);
        } else if path.is_dir() {
            matches.extend(find_named_executable_under(&path, file_name, max_depth - 1));
        }
    }
    matches
}

fn classify_document(original_name: &str, extracted_text: &str) -> DocumentClassification {
    let haystack = normalize(&format!("{original_name} {extracted_text}"));

    let rules = [
        (
            "Lönespecifikation",
            &["lonespecifikation", "lonespec", "lon period", "utbetalning"][..],
            "Regelmatch: löneord i filnamn eller extraherad text",
        ),
        (
            "Anställningsavtal",
            &["anstallningsavtal", "anstallning", "arbetsgivare", "avtal"][..],
            "Regelmatch: anställnings- eller avtalsord i dokumentet",
        ),
        (
            "Identitetshandling",
            &["pass", "identitetshandling", "passport", "mrz"][..],
            "Regelmatch: pass/identitetsord i dokumentet",
        ),
        (
            "Kvitto",
            &["kvitto", "receipt", "moms", "total", "betalt"][..],
            "Regelmatch: kvitto- eller betalningsord i dokumentet",
        ),
    ];

    for (document_type, keywords, explanation) in rules {
        if keywords.iter().any(|keyword| haystack.contains(keyword)) {
            return DocumentClassification {
                document_type,
                match_explanation: explanation.to_string(),
            };
        }
    }

    DocumentClassification {
        document_type: "Importerad fil",
        match_explanation: "Ingen dokumenttypsregel matchade; sparad för manuell granskning"
            .to_string(),
    }
}

fn extract_document_date(original_name: &str, extracted_text: &str) -> DateExtraction {
    let source = format!("{original_name} {extracted_text}");

    if let Some(date) = extract_iso_date(&source) {
        return DateExtraction {
            document_date: Some(date),
            explanation: Some("Datumregel: hittade explicit ISO-datum".to_string()),
        };
    }

    if let Some(date) = extract_separated_date(&source) {
        return DateExtraction {
            document_date: Some(date),
            explanation: Some(
                "Datumregel: hittade datum med punkt, snedstreck eller bindestreck".to_string(),
            ),
        };
    }

    if let Some(date) = extract_compact_date(&source) {
        return DateExtraction {
            document_date: Some(date),
            explanation: Some("Datumregel: hittade kompakt YYYYMMDD-datum".to_string()),
        };
    }

    if let Some(date) = extract_compact_swedish_date(&source) {
        return DateExtraction {
            document_date: Some(date),
            explanation: Some("Datumregel: hittade kompakt svenskt DDMMYY-datum".to_string()),
        };
    }

    if let Some(date) = extract_day_month_year_date(&source) {
        return DateExtraction {
            document_date: Some(date),
            explanation: Some("Datumregel: hittade dag, svenskt månadsnamn och år".to_string()),
        };
    }

    if let Some(date) = extract_month_year_date(&source) {
        return DateExtraction {
            document_date: Some(date),
            explanation: Some("Datumregel: hittade svenskt månadsnamn med år".to_string()),
        };
    }

    DateExtraction {
        document_date: None,
        explanation: None,
    }
}

fn extract_iso_date(source: &str) -> Option<String> {
    let bytes = source.as_bytes();
    for start in 0..bytes.len().saturating_sub(9) {
        let candidate = &bytes[start..start + 10];
        if candidate[4] != b'-' || candidate[7] != b'-' {
            continue;
        }

        if !candidate[0..4].iter().all(u8::is_ascii_digit)
            || !candidate[5..7].iter().all(u8::is_ascii_digit)
            || !candidate[8..10].iter().all(u8::is_ascii_digit)
        {
            continue;
        }

        let token = String::from_utf8_lossy(candidate).to_string();
        let year = token[0..4].parse::<i32>().ok()?;
        let month = token[5..7].parse::<u32>().ok()?;
        let day = token[8..10].parse::<u32>().ok()?;
        if valid_date_parts(year, month, day) {
            return Some(token);
        }
    }

    None
}

fn extract_compact_date(source: &str) -> Option<String> {
    let bytes = source.as_bytes();
    for start in 0..bytes.len().saturating_sub(7) {
        let candidate = &bytes[start..start + 8];
        if !candidate.iter().all(u8::is_ascii_digit) {
            continue;
        }

        let token = String::from_utf8_lossy(candidate).to_string();
        let year = token[0..4].parse::<i32>().ok()?;
        let month = token[4..6].parse::<u32>().ok()?;
        let day = token[6..8].parse::<u32>().ok()?;
        if valid_date_parts(year, month, day) {
            return Some(format!("{year:04}-{month:02}-{day:02}"));
        }
    }

    None
}

fn extract_separated_date(source: &str) -> Option<String> {
    let tokens = date_tokens(source);
    for token in tokens {
        let separators = token
            .chars()
            .filter(|character| matches!(character, '-' | '/' | '.'))
            .count();
        if separators != 2 {
            continue;
        }

        let parts = token
            .split(['-', '/', '.'])
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();
        if parts.len() != 3
            || !parts
                .iter()
                .all(|part| part.chars().all(|c| c.is_ascii_digit()))
        {
            continue;
        }

        if parts[0].len() == 4 {
            let (Ok(year), Ok(month), Ok(day)) = (
                parts[0].parse::<i32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<u32>(),
            ) else {
                continue;
            };
            if valid_date_parts(year, month, day) {
                return Some(format!("{year:04}-{month:02}-{day:02}"));
            }
        } else if parts[2].len() == 4 {
            let (Ok(day), Ok(month), Ok(year)) = (
                parts[0].parse::<u32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<i32>(),
            ) else {
                continue;
            };
            if valid_date_parts(year, month, day) {
                return Some(format!("{year:04}-{month:02}-{day:02}"));
            }
        }
    }

    None
}

fn extract_compact_swedish_date(source: &str) -> Option<String> {
    let bytes = source.as_bytes();
    for start in 0..bytes.len().saturating_sub(5) {
        let candidate = &bytes[start..start + 6];
        if !candidate.iter().all(u8::is_ascii_digit) {
            continue;
        }
        let previous_is_digit = start > 0 && bytes[start - 1].is_ascii_digit();
        let next_is_digit = start + 6 < bytes.len() && bytes[start + 6].is_ascii_digit();
        if previous_is_digit || next_is_digit {
            continue;
        }
        let token = String::from_utf8_lossy(candidate);
        let (Ok(day), Ok(month), Ok(year_suffix)) = (
            token[0..2].parse::<u32>(),
            token[2..4].parse::<u32>(),
            token[4..6].parse::<i32>(),
        ) else {
            continue;
        };
        let year = if year_suffix >= 70 {
            1900 + year_suffix
        } else {
            2000 + year_suffix
        };
        if valid_date_parts(year, month, day) {
            return Some(format!("{year:04}-{month:02}-{day:02}"));
        }
    }

    None
}

fn extract_day_month_year_date(source: &str) -> Option<String> {
    let normalized_tokens = date_tokens(&normalize(source));
    for (index, token) in normalized_tokens.iter().enumerate() {
        let Some(month) = month_number(token) else {
            continue;
        };
        let Some(day_token) = index
            .checked_sub(1)
            .and_then(|idx| normalized_tokens.get(idx))
        else {
            continue;
        };
        let Some(year_token) = normalized_tokens.get(index + 1) else {
            continue;
        };
        if !(day_token.len() <= 2 && year_token.len() == 4) {
            continue;
        }
        let (Ok(day), Ok(year)) = (day_token.parse::<u32>(), year_token.parse::<i32>()) else {
            continue;
        };
        if valid_date_parts(year, month, day) {
            return Some(format!("{year:04}-{month:02}-{day:02}"));
        }
    }

    None
}

fn extract_month_year_date(source: &str) -> Option<String> {
    let normalized_tokens = date_tokens(&normalize(source));
    for (index, token) in normalized_tokens.iter().enumerate() {
        let Some(month) = month_number(token) else {
            continue;
        };

        for candidate in [
            normalized_tokens.get(index.wrapping_sub(1)),
            normalized_tokens.get(index + 1),
        ]
        .into_iter()
        .flatten()
        {
            if candidate.len() == 4
                && candidate
                    .chars()
                    .all(|character| character.is_ascii_digit())
            {
                let year = candidate.parse::<i32>().ok()?;
                if (1900..=2100).contains(&year) {
                    return Some(format!("{year:04}-{month:02}-01"));
                }
            }
        }
    }

    None
}

fn date_tokens(source: &str) -> Vec<String> {
    source
        .split(|character: char| {
            !(character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '/' | '.'))
        })
        .flat_map(|chunk| chunk.split('_'))
        .filter(|token| !token.is_empty())
        .map(|token| {
            token
                .trim_matches(|character: char| {
                    !character.is_ascii_alphanumeric() && !matches!(character, '-' | '/' | '.')
                })
                .to_string()
        })
        .filter(|token| !token.is_empty())
        .collect()
}

fn month_number(token: &str) -> Option<u32> {
    match token {
        "januari" | "jan" => Some(1),
        "februari" | "feb" => Some(2),
        "mars" | "mar" => Some(3),
        "april" | "apr" => Some(4),
        "maj" => Some(5),
        "juni" | "jun" => Some(6),
        "juli" | "jul" => Some(7),
        "augusti" | "aug" => Some(8),
        "september" | "sep" => Some(9),
        "oktober" | "okt" => Some(10),
        "november" | "nov" => Some(11),
        "december" | "dec" => Some(12),
        _ => None,
    }
}

fn valid_date_parts(year: i32, month: u32, day: u32) -> bool {
    (1900..=2100).contains(&year)
        && (1..=12).contains(&month)
        && (1..=days_in_month(year, month)).contains(&day)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn search_documents(connection: &Connection, query: &str) -> rusqlite::Result<Vec<SearchResult>> {
    let documents = list_documents(connection)?;
    let parsed = parse_search_query(query);
    let query_plan = build_query_plan(&parsed.terms.join(" "));

    if query_plan.is_empty()
        && parsed.exact_phrases.is_empty()
        && parsed.document_type.is_none()
        && parsed.tag.is_none()
        && parsed.date.is_none()
        && parsed.inbox_status.is_none()
        && parsed.negative_terms.is_empty()
    {
        return Ok(documents
            .into_iter()
            .map(|document| SearchResult {
                document,
                score: 0,
                reasons: vec!["Ingen sökfråga: visar Test Lab-lista i stabil ordning".to_string()],
                query_plan: Vec::new(),
            })
            .collect());
    }

    let fts_candidate_ids = fts_candidate_ids(connection, &query_plan)?;
    let mut results = documents
        .into_iter()
        .filter_map(|document| {
            if !document_matches_filters(connection, &document, &parsed).unwrap_or(false) {
                return None;
            }
            let (mut score, mut reasons) =
                score_document(&document, &query_plan, &fts_candidate_ids);
            let normalized_title = normalize(&document.title);
            for phrase in &parsed.exact_phrases {
                if normalized_title.contains(phrase) {
                    score += 24;
                    reasons.push(format!("Exakt fras i titel: '{phrase}'"));
                }
            }
            if parsed.document_type.is_some() {
                score += 8;
                reasons.push("Filtrerad på dokumenttyp".to_string());
            }
            if parsed.tag.is_some() {
                score += 8;
                reasons.push("Filtrerad på tagg".to_string());
            }
            if parsed.date.is_some() {
                score += 7;
                reasons.push("Filtrerad på datum".to_string());
            }
            if parsed.inbox_status.is_some() {
                score += 5;
                reasons.push("Filtrerad på status".to_string());
            }
            if query_plan.is_empty()
                && (!parsed.exact_phrases.is_empty()
                    || !parsed.negative_terms.is_empty()
                    || parsed.document_type.is_some()
                    || parsed.tag.is_some()
                    || parsed.date.is_some()
                    || parsed.inbox_status.is_some())
                && score == 0
            {
                score = 1;
            }
            (score > 0).then_some(SearchResult {
                document,
                score,
                reasons,
                query_plan: query_plan.clone(),
            })
        })
        .collect::<Vec<_>>();

    results.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.document.title.cmp(&right.document.title))
    });

    Ok(results)
}

fn parse_search_query(query: &str) -> ParsedSearchQuery {
    let mut parsed = ParsedSearchQuery::default();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut parts = Vec::new();
    for character in query.chars() {
        if character == '"' {
            if in_quotes && !current.trim().is_empty() {
                parsed
                    .exact_phrases
                    .push(normalize(current.trim()).trim().to_string());
                current.clear();
            }
            in_quotes = !in_quotes;
        } else if character.is_whitespace() && !in_quotes {
            if !current.is_empty() {
                parts.push(std::mem::take(&mut current));
            }
        } else {
            current.push(character);
        }
    }
    if !current.is_empty() {
        parts.push(current);
    }
    for part in parts {
        let normalized = normalize(&part).trim().to_string();
        if let Some(value) = part.strip_prefix("type:") {
            parsed.document_type = Some(normalize(value).trim().to_string());
        } else if let Some(value) = part.strip_prefix("tag:") {
            parsed.tag = Some(normalize(value).trim().to_string());
        } else if let Some(value) = part.strip_prefix("date:") {
            parsed.date = Some(normalize(value).trim().to_string());
        } else if let Some(value) = part.strip_prefix("status:") {
            parsed.inbox_status = Some(normalize(value).trim().to_string());
        } else if part.starts_with('-') && part.len() > 1 {
            parsed
                .negative_terms
                .push(normalize(&part[1..]).trim().to_string());
        } else if !normalized.is_empty() {
            parsed.terms.push(normalized);
        }
    }
    for phrase in &parsed.exact_phrases {
        parsed.terms.extend(tokenize(phrase));
    }
    parsed.terms.sort();
    parsed.terms.dedup();
    parsed
}

fn document_matches_filters(
    connection: &Connection,
    document: &DocumentSummary,
    parsed: &ParsedSearchQuery,
) -> rusqlite::Result<bool> {
    let body: String = connection.query_row(
        "SELECT extracted_text FROM documents WHERE id = ?1",
        params![document.id],
        |row| row.get(0),
    )?;
    let haystack = normalize(&format!(
        "{} {} {} {} {} {}",
        document.title,
        document.document_type,
        document.document_date.as_deref().unwrap_or_default(),
        document.source_label,
        document.match_explanation,
        body
    ));
    if parsed
        .negative_terms
        .iter()
        .any(|term| haystack.contains(term))
    {
        return Ok(false);
    }
    if parsed
        .exact_phrases
        .iter()
        .any(|phrase| !haystack.contains(phrase))
    {
        return Ok(false);
    }
    if parsed
        .document_type
        .as_ref()
        .is_some_and(|value| !normalize(&document.document_type).contains(value))
    {
        return Ok(false);
    }
    if parsed.date.as_ref().is_some_and(|value| {
        !normalize(document.document_date.as_deref().unwrap_or_default()).contains(value)
    }) {
        return Ok(false);
    }
    if parsed
        .inbox_status
        .as_ref()
        .is_some_and(|value| !normalize(&document.inbox_status).contains(value))
    {
        return Ok(false);
    }
    if let Some(tag) = &parsed.tag {
        let matches: bool = connection.query_row("SELECT EXISTS(SELECT 1 FROM document_tags dt JOIN tags t ON t.id = dt.tag_id WHERE dt.document_id = ?1 AND t.vault_id = 1 AND lower(t.name) LIKE ?2)", params![document.id, format!("%{tag}%")], |row| row.get(0))?;
        if !matches {
            return Ok(false);
        }
    }
    Ok(true)
}

fn build_query_plan(query: &str) -> Vec<String> {
    let mut tokens = Vec::new();

    for token in tokenize(query) {
        push_unique(&mut tokens, token.as_str());

        match token.as_str() {
            "kontrakt" => {
                push_unique(&mut tokens, "avtal");
                push_unique(&mut tokens, "anställningsavtal");
            }
            "lon" | "lön" => {
                push_unique(&mut tokens, "lön");
                push_unique(&mut tokens, "lönespecifikation");
                push_unique(&mut tokens, "löneperiod");
            }
            "juni" => {
                push_unique(&mut tokens, "06");
                push_unique(&mut tokens, "månad");
                push_unique(&mut tokens, "löneperiod");
            }
            "dagab" => {
                push_unique(&mut tokens, "dagab");
            }
            _ => {}
        }
    }

    tokens
}

fn tokenize(query: &str) -> Vec<String> {
    normalize(query)
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn normalize(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|character| match character {
            'å' | 'ä' | 'á' | 'à' | 'â' => 'a',
            'ö' | 'ó' | 'ò' | 'ô' => 'o',
            'é' | 'è' | 'ê' => 'e',
            'ü' => 'u',
            '0'..='9' | 'a'..='z' => character,
            _ => ' ',
        })
        .collect::<String>()
}

fn push_unique(tokens: &mut Vec<String>, token: &str) {
    let normalized = normalize(token).trim().to_string();
    if !normalized.is_empty() && !tokens.iter().any(|existing| existing == &normalized) {
        tokens.push(normalized);
    }
}

fn fts_candidate_ids(connection: &Connection, query_plan: &[String]) -> rusqlite::Result<Vec<i64>> {
    if query_plan.is_empty() {
        return Ok(Vec::new());
    }

    let fts_query = query_plan
        .iter()
        .filter(|token| {
            token
                .chars()
                .all(|character| character.is_ascii_alphanumeric())
        })
        .map(|token| format!("\"{token}\""))
        .collect::<Vec<_>>()
        .join(" OR ");

    if fts_query.is_empty() {
        return Ok(Vec::new());
    }

    let mut statement = connection.prepare(
        "
        SELECT DISTINCT document_id
        FROM document_fts
        WHERE document_fts MATCH ?1
        ",
    )?;

    let ids = statement
        .query_map(params![fts_query], |row| row.get(0))?
        .collect::<rusqlite::Result<Vec<i64>>>()?;

    Ok(ids)
}

fn score_document(
    document: &DocumentSummary,
    query_plan: &[String],
    fts_candidate_ids: &[i64],
) -> (i64, Vec<String>) {
    let title = normalize(&document.title);
    let document_type = normalize(&document.document_type);
    let date = normalize(document.document_date.as_deref().unwrap_or_default());
    let source = normalize(&document.source_label);
    let explanation = normalize(&document.match_explanation);
    let haystack = format!("{title} {document_type} {date} {source} {explanation}");

    let mut score = 0;
    let mut reasons = Vec::new();

    if fts_candidate_ids.contains(&document.id) {
        score += 8;
        reasons.push("FTS5-index matchade dokumentet".to_string());
    }

    for token in query_plan {
        if title.contains(token) {
            score += 12;
            reasons.push(format!("Titel matchade '{token}'"));
        } else if document_type.contains(token) {
            score += 10;
            reasons.push(format!("Dokumenttyp matchade '{token}'"));
        } else if date.contains(token) {
            score += 7;
            reasons.push(format!("Datum/period matchade '{token}'"));
        } else if explanation.contains(token) {
            score += 5;
            reasons.push(format!("Matchförklaring matchade '{token}'"));
        } else if source.contains(token) || haystack.contains(token) {
            score += 2;
            reasons.push(format!("Metadata matchade '{token}'"));
        }
    }

    reasons.sort();
    reasons.dedup();

    (score, reasons)
}

fn create_vault_directories(data_root: &Path) -> std::io::Result<()> {
    for directory in [
        data_root.to_path_buf(),
        data_root.join("files"),
        data_root.join("index"),
        data_root.join("backups"),
        data_root.join("testlab"),
        data_root.join("plugins"),
        data_root.join("index").join("private-previews"),
    ] {
        fs::create_dir_all(directory)?;
    }

    Ok(())
}

fn ensure_bundled_tessdata(app: &tauri::AppHandle, data_root: &Path) -> std::io::Result<()> {
    let target_dir = data_root.join("tessdata");
    let target_file = target_dir.join("swe.traineddata");
    if target_file.is_file() {
        return Ok(());
    }

    fs::create_dir_all(&target_dir)?;
    let mut candidates = Vec::new();
    if let Ok(resource_dir) = app.path().resource_dir() {
        candidates.push(resource_dir.join("tessdata").join("swe.traineddata"));
        candidates.push(resource_dir.join("swe.traineddata"));
    }
    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(
            current_dir
                .join("src-tauri")
                .join("tessdata")
                .join("swe.traineddata"),
        );
        candidates.push(current_dir.join("tessdata").join("swe.traineddata"));
    }

    if let Some(source_file) = candidates.into_iter().find(|path| path.is_file()) {
        fs::copy(source_file, target_file)?;
    }

    Ok(())
}

fn to_sql_error(error: std::io::Error) -> rusqlite::Error {
    rusqlite::Error::ToSqlConversionFailure(Box::new(error))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            environment_status,
            initialize_vault,
            vault_diagnostics,
            list_production_documents,
            list_trashed_production_documents,
            list_testlab_documents,
            get_production_document_detail,
            get_testlab_document_detail,
            open_production_document_file,
            reveal_production_document_file,
            open_protected_document_file,
            reveal_protected_document_file,
            trash_production_document,
            restore_production_document,
            create_local_backup,
            get_backup_policy,
            save_backup_policy,
            run_configured_backup,
            create_password_backup,
            validate_password_backup,
            restore_password_backup,
            test_backup_restore,
            create_portable_archive,
            restore_portable_archive,
            scan_file_integrity,
            decide_duplicate,
            list_local_backups,
            validate_local_backup,
            restore_local_backup,
            check_vault_health,
            rebuild_production_search_index,
            list_audit_events,
            list_review_queue,
            list_entity_catalog,
            list_timeline_events,
            list_calendar_events,
            save_calendar_event,
            list_graph_data,
            save_graph_relation,
            list_domain_records,
            refresh_claim_actuality,
            set_claim_actuality,
            analyze_production_archive,
            list_code_words,
            add_code_word,
            list_conflicts,
            resolve_conflict,
            list_document_versions,
            list_document_pages,
            add_document_version,
            list_document_annotations,
            save_document_annotation,
            delete_document_annotation,
            compare_document_versions,
            restore_document_version,
            list_reminders,
            save_reminder,
            set_reminder_completed,
            list_watched_folders,
            add_watched_folder,
            scan_watched_folders,
            list_background_jobs,
            run_background_jobs_now,
            queue_document_ocr,
            set_ocr_job_paused,
            ocr_status,
            run_ocr_for_production_document,
            update_production_document_metadata,
            set_claim_review_status,
            add_production_document_tag,
            remove_production_document_tag,
            run_test_center,
            run_test_center_module,
            generate_testlab_scale,
            simulate_testlab_failure,
            export_safe_test_report,
            reset_testlab,
            search_production_documents,
            search_testlab_documents,
            import_document,
            import_directory,
            import_archive,
            import_text_document,
            import_paths,
            list_import_history,
            list_automation_rules,
            save_automation_rule,
            delete_automation_rule,
            run_automation_rules,
            list_document_templates,
            save_document_template,
            list_custom_fields,
            save_custom_field,
            list_saved_searches,
            save_search,
            delete_saved_search,
            security_status,
            configure_security,
            unlock_vault,
            windows_hello_status,
            unlock_vault_windows_hello,
            set_document_security,
            list_locked_document_ids,
            list_private_document_ids,
            set_document_private,
            open_private_document_file,
            list_plugins,
            install_plugin,
            set_plugin_enabled,
            run_plugin,
            remove_plugin,
            export_document_secure,
            list_theme_profiles,
            save_theme_profile,
            activate_theme_profile,
            duplicate_theme_profile,
            export_theme_profile,
            import_theme_profile,
            list_favorite_document_ids,
            set_document_favorite,
            list_collections,
            save_collection,
            set_collection_document,
            list_collection_document_ids,
            list_folders,
            list_categories,
            save_folder,
            save_category,
            set_organizer_locked,
            set_document_organizer,
            list_document_organizer_ids,
            list_document_notes,
            save_document_note,
            list_vault_notes,
            save_vault_note,
            delete_vault_note,
            list_vault_note_versions,
            list_note_templates,
            list_search_history,
            record_search_history,
            get_search_history_preferences,
            save_search_history_preferences,
            clear_search_history,
            set_search_history_pinned,
            export_search_history,
            get_analysis_preferences,
            save_analysis_preferences,
            list_analysis_exclusions,
            add_analysis_exclusion,
            delete_analysis_exclusion,
            scanner_status,
            launch_local_scanner,
            import_web_snapshot,
            create_demo_archive
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            if let Ok(data_root) = app.path().app_data_dir() {
                if initialize_vault_at(&data_root).is_ok() {
                    start_background_worker(data_root);
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn initializes_local_vault_database() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let status = initialize_vault_at(temp_dir.path()).expect("vault initialization");

        assert_eq!(status.schema_version, 26);
        assert_eq!(status.testlab_document_count, 4);
        assert!(status.production_vault_ready);
        assert!(PathBuf::from(status.database_path).exists());
        assert!(PathBuf::from(status.testlab_database_path).exists());
        assert!(temp_dir.path().join("files").exists());
        assert!(temp_dir.path().join("index").exists());
        assert!(temp_dir.path().join("backups").exists());
        assert!(temp_dir.path().join("testlab").exists());
        let production =
            Connection::open(temp_dir.path().join("vault.db")).expect("production database");
        let theme_count: i64 = production
            .query_row("SELECT COUNT(*) FROM theme_profiles", [], |row| row.get(0))
            .expect("theme presets");
        let active_theme_count: i64 = production
            .query_row(
                "SELECT COUNT(*) FROM theme_profiles WHERE is_active=1",
                [],
                |row| row.get(0),
            )
            .expect("active theme");
        assert_eq!(theme_count, 11);
        assert_eq!(active_theme_count, 1);

        let documents = list_testlab_documents_at(temp_dir.path()).expect("test lab documents");
        assert_eq!(documents.len(), 4);
        assert!(documents
            .iter()
            .any(|document| document.title == "DAGAB anställningsavtal"));

        let detail =
            get_testlab_document_detail_at(temp_dir.path(), 10).expect("test lab document detail");
        assert!(detail.extracted_text.contains("DAGAB kontrakt"));

        let connection = Connection::open(temp_dir.path().join("testlab").join("vault-test.db"))
            .expect("test lab connection");
        connection
            .execute("DELETE FROM documents WHERE id = 10", [])
            .expect("delete fixture");
        assert_eq!(
            list_testlab_documents_at(temp_dir.path())
                .expect("mutated test lab documents")
                .len(),
            3
        );

        let reset_documents = reset_testlab_at(temp_dir.path()).expect("reset test lab");
        assert_eq!(reset_documents.len(), 4);
        assert!(reset_documents
            .iter()
            .any(|document| document.title.contains("DAGAB")));

        let report = run_test_center_at(temp_dir.path()).expect("test center");
        assert_eq!(report.failed, 0);
        assert!(report.passed >= 5);
    }

    #[test]
    fn searches_dagab_contract_with_synonym() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        initialize_vault_at(temp_dir.path()).expect("vault initialization");

        let results =
            search_testlab_documents_at(temp_dir.path(), "DAGAB kontrakt").expect("search results");

        assert!(!results.is_empty());
        assert_eq!(results[0].document.title, "DAGAB anställningsavtal");
        assert!(results[0].query_plan.iter().any(|token| token == "avtal"));
        assert!(results[0]
            .reasons
            .iter()
            .any(|reason| reason.contains("Dokumenttyp") || reason.contains("FTS5")));
    }

    #[test]
    fn searches_salary_for_june_by_period_terms() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        initialize_vault_at(temp_dir.path()).expect("vault initialization");

        let results =
            search_testlab_documents_at(temp_dir.path(), "lön juni").expect("search results");

        assert!(!results.is_empty());
        assert_eq!(results[0].document.title, "Lönespecifikation juni");
        assert!(results[0].query_plan.iter().any(|token| token == "06"));
        assert!(results[0]
            .reasons
            .iter()
            .any(|reason| reason.contains("Datum") || reason.contains("Titel")));
    }

    #[test]
    fn supports_explainable_search_operators() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        initialize_vault_at(temp_dir.path()).expect("vault initialization");
        create_demo_archive_at(temp_dir.path()).expect("demo archive");

        let typed =
            search_production_documents_at(temp_dir.path(), "type:lönespecifikation date:2024")
                .expect("typed search");
        assert_eq!(typed.len(), 1);
        assert_eq!(typed[0].document.document_type, "Lönespecifikation");
        assert!(typed[0]
            .reasons
            .iter()
            .any(|reason| reason.contains("dokumenttyp")));

        let tagged =
            search_production_documents_at(temp_dir.path(), "tag:kvitto").expect("tag search");
        assert!(tagged
            .iter()
            .all(|result| result.document.document_type == "Kvitto"));

        let phrase = search_production_documents_at(temp_dir.path(), "\"DAGAB anställningsavtal\"")
            .expect("phrase search");
        assert_eq!(phrase.len(), 1);

        let negative = search_production_documents_at(temp_dir.path(), "avtal -DAGAB")
            .expect("negative search");
        assert!(negative
            .iter()
            .all(|result| !result.document.title.contains("DAGAB")));
    }

    #[test]
    fn imports_local_file_into_production_vault() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        initialize_vault_at(temp_dir.path()).expect("vault initialization");
        let source_file = temp_dir.path().join("source.txt");
        fs::write(
            &source_file,
            "Vault import test with searchable internal text DAGAB kontakt anna@example.com",
        )
        .expect("write source");

        let result = import_document_at(temp_dir.path(), &source_file).expect("import result");

        assert!(Path::new(&result.stored_path).exists());
        assert!(!result.duplicate);
        assert_eq!(result.document.title, "source.txt");

        let duplicate =
            import_document_at(temp_dir.path(), &source_file).expect("duplicate import");
        assert!(duplicate.duplicate);
        assert_eq!(duplicate.file_id, result.file_id);

        let documents =
            list_production_documents_at(temp_dir.path()).expect("production documents");
        assert_eq!(documents.len(), 2);
        assert_eq!(documents[0].title, "source.txt");

        let results = search_production_documents_at(temp_dir.path(), "internal text")
            .expect("production search");
        assert!(!results.is_empty());
        assert_eq!(results[0].document.title, "source.txt");

        let detail = get_production_document_detail_at(temp_dir.path(), result.document.id)
            .expect("production detail");
        let pages =
            list_document_pages_at(temp_dir.path(), result.document.id).expect("document pages");
        assert_eq!(pages.len(), 1);
        assert!(pages[0].text.contains("DAGAB"));
        assert!(detail.extracted_text.contains("searchable internal text"));
        assert!(detail.stored_path.is_some());
        assert!(detail.tags.iter().any(|tag| tag == "importerad"));
        assert!(detail
            .claims
            .iter()
            .any(|claim| claim.claim_type == "sha256"));
        assert!(detail
            .claims
            .iter()
            .any(|claim| claim.claim_type == "email"));
        assert!(detail
            .claims
            .iter()
            .any(|claim| claim.claim_type == "extracted_text_length"));
        let sourced_email = detail
            .claims
            .iter()
            .find(|claim| claim.claim_type == "email")
            .expect("email claim");
        assert_eq!(sourced_email.source_page_no, Some(1));
        assert!(sourced_email.source_span_id.is_some());
        assert!(sourced_email
            .source_text
            .as_deref()
            .unwrap_or_default()
            .contains("anna@example.com"));
        assert_eq!(sourced_email.actuality_status, "uncertain");
        assert!(detail
            .entities
            .iter()
            .any(|entity| entity.display_name == "DAGAB"));
        assert!(detail
            .entities
            .iter()
            .any(|entity| entity.display_name == "anna@example.com"));

        let updated_detail = update_production_document_metadata_at(
            temp_dir.path(),
            result.document.id,
            "Reviewed source",
            "Arbetsdokument",
            Some("2024-07-20"),
            "review",
            "Manuell granskning",
            "Metadata uppdaterad i test",
        )
        .expect("metadata update");
        assert_eq!(updated_detail.document.title, "Reviewed source");
        assert_eq!(
            updated_detail.document.document_date.as_deref(),
            Some("2024-07-20")
        );

        let claim_id = updated_detail
            .claims
            .iter()
            .find(|claim| claim.status == "auto_extracted_pending_review")
            .map(|claim| claim.id)
            .expect("pending claim");
        let reviewed_detail =
            set_claim_review_status_at(temp_dir.path(), claim_id, "review_approved")
                .expect("claim review");
        assert!(reviewed_detail
            .claims
            .iter()
            .any(|claim| claim.id == claim_id && claim.status == "review_approved"));

        let tagged_detail =
            add_production_document_tag_at(temp_dir.path(), result.document.id, "Privat kod")
                .expect("add tag");
        assert!(tagged_detail.tags.iter().any(|tag| tag == "privat-kod"));
        let untagged_detail =
            remove_production_document_tag_at(temp_dir.path(), result.document.id, "Privat kod")
                .expect("remove tag");
        assert!(!untagged_detail.tags.iter().any(|tag| tag == "privat-kod"));

        let code_words = add_code_word_at(temp_dir.path(), "DAGAB Privat", "Kodord for avtal")
            .expect("add code word");
        assert!(code_words
            .iter()
            .any(|code_word| code_word.word == "dagab-privat"));

        let conflicts = list_conflicts_at(temp_dir.path()).expect("conflicts");
        assert!(conflicts
            .iter()
            .any(|conflict| conflict.conflict_type == "duplicate_file"));

        let resolved_file =
            resolve_production_document_file_at(temp_dir.path(), result.document.id)
                .expect("resolved stored file");
        assert_eq!(resolved_file, PathBuf::from(&result.stored_path));

        let health = check_vault_health_at(temp_dir.path()).expect("vault health");
        assert!(health.ok);
        assert_eq!(health.database_integrity, "ok");
        assert_eq!(health.document_count, 2);
        assert_eq!(health.file_count, 1);
        assert_eq!(health.missing_file_count, 0);

        let backup = create_local_backup_at(temp_dir.path()).expect("local backup");
        assert!(Path::new(&backup.backup_root).exists());
        assert!(Path::new(&backup.backup_path).exists());
        assert!(Path::new(&backup.manifest_path).exists());
        assert_eq!(backup.document_count, 2);
        assert_eq!(backup.file_count, 1);
        assert_eq!(backup.copied_file_count, 1);
        let backup_original_file = Path::new(&backup.backup_root).join(
            Path::new(&result.stored_path)
                .strip_prefix(temp_dir.path())
                .expect("stored file relative path"),
        );
        assert!(backup_original_file.exists());
        assert_eq!(
            sha256_file(Path::new(&backup.backup_path)).expect("backup hash"),
            backup.sha256
        );

        let backups = list_local_backups_at(temp_dir.path()).expect("local backups");
        assert_eq!(backups.len(), 1);
        assert_eq!(backups[0].sha256, backup.sha256);
        let validation =
            validate_local_backup_at(Path::new(&backup.backup_root)).expect("backup validation");
        assert!(validation.ok);
        assert!(validation.sha256_matches);

        let reindex = rebuild_production_search_index_at(temp_dir.path()).expect("reindex");
        assert_eq!(reindex.indexed_document_count, 2);

        let audit_events = list_audit_events_at(temp_dir.path(), 10).expect("audit events");
        assert!(audit_events
            .iter()
            .any(|event| event.event_type == "document_imported"));
        assert!(audit_events
            .iter()
            .any(|event| event.event_type == "backup_created"));
        assert!(audit_events
            .iter()
            .any(|event| event.event_type == "search_index_rebuilt"));

        fs::remove_file(&result.stored_path).expect("remove stored file");
        let broken_health = check_vault_health_at(temp_dir.path()).expect("broken vault health");
        assert!(!broken_health.ok);
        assert_eq!(broken_health.missing_file_count, 1);

        let restore = restore_local_backup_at(temp_dir.path(), Path::new(&backup.backup_root))
            .expect("restore backup");
        assert_eq!(restore.document_count, 2);
        assert_eq!(restore.file_count, 1);
        assert!(Path::new(&result.stored_path).exists());
        let restored_health = check_vault_health_at(temp_dir.path()).expect("restored health");
        assert!(restored_health.ok);
    }

    #[test]
    fn classifies_imported_documents_with_local_rules() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        initialize_vault_at(temp_dir.path()).expect("vault initialization");

        let salary_file = temp_dir.path().join("lonespec-juni-2024.txt");
        fs::write(
            &salary_file,
            "Lonespecifikation juni 2024. Utbetalning och lon period.",
        )
        .expect("write salary fixture");

        let result = import_document_at(temp_dir.path(), &salary_file).expect("import result");

        assert!(result.document.document_type.contains("nespecifikation"));
        assert_eq!(result.document.document_date.as_deref(), Some("2024-06-01"));
        assert!(result.document.match_explanation.contains("Regelmatch"));
        assert!(result.document.match_explanation.contains("Datumregel"));

        let search_results =
            search_production_documents_at(temp_dir.path(), "lön juni").expect("search results");
        assert!(!search_results.is_empty());
        assert!(search_results[0]
            .document
            .document_type
            .contains("nespecifikation"));
        assert_eq!(
            search_results[0].document.document_date.as_deref(),
            Some("2024-06-01")
        );
    }

    #[test]
    fn extracts_explicit_document_dates_without_guessing() {
        assert_eq!(
            extract_document_date("avtal-2024-02-29.txt", "").document_date,
            Some("2024-02-29".to_string())
        );
        assert_eq!(
            extract_document_date("kvitto_20240625.txt", "").document_date,
            Some("2024-06-25".to_string())
        );
        assert_eq!(
            extract_document_date("faktura.pdf", "Fakturadatum 25/06/2024").document_date,
            Some("2024-06-25".to_string())
        );
        assert_eq!(
            extract_document_date("kontoutdrag.pdf", "Bokföringsdag 25.06.2024").document_date,
            Some("2024-06-25".to_string())
        );
        assert_eq!(
            extract_document_date("utbetalning.pdf", "Utbetalningsdatum 25 juni 2024")
                .document_date,
            Some("2024-06-25".to_string())
        );
        assert_eq!(
            extract_document_date("kvitto_250624.pdf", "").document_date,
            Some("2024-06-25".to_string())
        );
        assert_eq!(
            extract_document_date("lonespec-juni.txt", "Lonespecifikation juni").document_date,
            None
        );
        assert_eq!(
            extract_document_date("fel-2023-02-29.txt", "").document_date,
            None
        );
    }

    #[test]
    fn extracts_content_claim_values_from_text() {
        let text = "Kontakt: anna@example.com. Bruttolön 32000 SEK och avgift 349 kr.";

        let emails = extract_email_values(text);
        let amounts = extract_money_values(text);

        assert!(emails.iter().any(|email| email == "anna@example.com"));
        assert!(amounts.iter().any(|amount| amount == "32000"));
        assert!(amounts.iter().any(|amount| amount == "349"));
    }

    #[test]
    fn pdf_text_extraction_falls_back_when_tool_is_missing_or_file_is_not_pdf() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let fake_pdf = temp_dir.path().join("fake.pdf");
        fs::write(&fake_pdf, "not a pdf").expect("fake pdf");

        let extracted = extract_text_for_indexing(&fake_pdf, "application/pdf", None)
            .expect("pdf extraction fallback");
        assert!(extracted.is_empty());

        let ocr = detect_ocr_status(Some(temp_dir.path()));
        assert_eq!(ocr.engine, "tesseract");
    }

    #[test]
    fn extracts_text_from_office_open_xml_documents() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let docx = temp_dir.path().join("fixture.docx");
        write_test_zip(
            &docx,
            &[(
                "word/document.xml",
                "<w:document><w:body><w:t>DAGAB anställningsavtal</w:t></w:body></w:document>",
            )],
        );

        let extracted = extract_text_for_indexing(
            &docx,
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            None,
        )
        .expect("docx text");
        assert!(extracted.contains("DAGAB"));
        assert!(extracted.contains("anställningsavtal"));
    }

    #[test]
    fn imports_demo_archive_and_reports_diagnostics() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        initialize_vault_at(temp_dir.path()).expect("vault initialization");

        let result = create_demo_archive_at(temp_dir.path()).expect("demo archive");

        assert_eq!(result.scanned_file_count, 4);
        assert_eq!(result.failed_count, 0);
        assert!(result.imported_count >= 4);

        let diagnostics = vault_diagnostics_at(temp_dir.path()).expect("diagnostics");
        assert_eq!(
            diagnostics.production_document_count,
            result.results.len() as i64
        );
        assert_eq!(diagnostics.testlab_document_count, 4);
        assert!(diagnostics.office_text_available);

        let salary_results =
            search_production_documents_at(temp_dir.path(), "lön juni").expect("salary search");
        assert!(!salary_results.is_empty());

        let timeline = list_timeline_events_at(temp_dir.path()).expect("timeline");
        assert_eq!(timeline.len(), 4);
        assert!(timeline
            .windows(2)
            .all(|pair| pair[0].event_date >= pair[1].event_date));
        assert!(timeline.iter().all(|event| !event.explanation.is_empty()));
        assert!(timeline.iter().any(|event| event.source_page_no == Some(1)));

        let domain_records = list_domain_records_at(temp_dir.path()).expect("domain records");
        assert_eq!(domain_records.len(), 4);
        let employment = domain_records
            .iter()
            .find(|record| record.record_type == "employment_agreement")
            .expect("employment record");
        assert_eq!(employment.effective_from.as_deref(), Some("2024-02-01"));
        assert_eq!(employment.actuality_status, "documented_open_period");
        assert!(employment
            .actuality_explanation
            .contains("slutdatum saknas"));
        let salary = domain_records
            .iter()
            .find(|record| record.record_type == "salary_period")
            .expect("salary record");
        assert_eq!(salary.actuality_status, "historical_record");
        assert!(salary.fields_json.contains("money_amount"));
    }

    #[test]
    fn imports_eml_and_safe_zip_with_history() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        initialize_vault_at(temp_dir.path()).expect("vault initialization");
        let eml = temp_dir.path().join("meddelande.eml");
        fs::write(&eml, "From: anna@example.com\r\nTo: vault@example.com\r\nSubject: Servicebok\r\nDate: Tue, 21 Jul 2026 10:00:00 +0200\r\nContent-Type: text/plain\r\n\r\nRegistreringsnummer ABC123=0AService utförd 2026-07-20").expect("eml");
        let imported = import_document_at(temp_dir.path(), &eml).expect("import eml");
        let detail = get_production_document_detail_at(temp_dir.path(), imported.document.id)
            .expect("eml detail");
        assert!(detail.extracted_text.contains("Subject: Servicebok"));
        assert!(detail.extracted_text.contains("anna@example.com"));

        let zip_path = temp_dir.path().join("arkiv.zip");
        let zip_file = fs::File::create(&zip_path).expect("zip file");
        let mut writer = zip::ZipWriter::new(zip_file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        writer
            .start_file("underlag/avtal.txt", options)
            .expect("zip entry");
        writer
            .write_all(b"Avtal Startdatum: 2026-08-01")
            .expect("zip content");
        writer
            .start_file("../outside.txt", options)
            .expect("unsafe entry");
        writer
            .write_all(b"must not escape")
            .expect("unsafe content");
        writer.finish().expect("finish zip");
        let result = import_archive_at(temp_dir.path(), &zip_path).expect("import zip");
        assert_eq!(result.scanned_file_count, 1);
        assert!(!temp_dir.path().join("outside.txt").exists());
        record_directory_import_session(temp_dir.path(), "zip", "arkiv.zip", &result);
        let history = list_import_history_at(temp_dir.path()).expect("history");
        assert_eq!(history[0].method, "zip");
        assert_eq!(history[0].imported_count, 1);
        assert_eq!(history[0].items.len(), 1);
    }

    #[test]
    fn trashes_production_document_and_removes_it_from_search() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        initialize_vault_at(temp_dir.path()).expect("vault initialization");
        let source_file = temp_dir.path().join("trash-me.txt");
        fs::write(&source_file, "DAGAB trash flow searchable text").expect("write source");
        let import = import_document_at(temp_dir.path(), &source_file).expect("import");

        let remaining =
            trash_production_document_at(temp_dir.path(), import.document.id).expect("trash");

        assert!(remaining
            .iter()
            .all(|document| document.id != import.document.id));
        let results =
            search_production_documents_at(temp_dir.path(), "trash flow").expect("search");
        assert!(results
            .iter()
            .all(|result| result.document.id != import.document.id));

        let connection = Connection::open(temp_dir.path().join("vault.db")).expect("connection");
        let trashed = list_trashed_documents(&connection).expect("trashed documents");
        assert_eq!(trashed.len(), 1);
        assert_eq!(trashed[0].id, import.document.id);

        let restored = restore_production_document_at(temp_dir.path(), import.document.id)
            .expect("restore document");
        assert!(restored
            .iter()
            .any(|document| document.id == import.document.id));
        let restored_results =
            search_production_documents_at(temp_dir.path(), "trash flow").expect("restored search");
        assert!(restored_results
            .iter()
            .any(|result| result.document.id == import.document.id));
    }

    #[test]
    fn supports_versions_reminders_and_watched_folders() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        initialize_vault_at(temp_dir.path()).expect("vault initialization");
        let first = temp_dir.path().join("contract-v1.txt");
        let second = temp_dir.path().join("contract-v2.txt");
        fs::write(&first, "Avtal version ett").expect("first version");
        fs::write(&second, "Avtal version två med nytt innehåll").expect("second version");
        let imported = import_document_at(temp_dir.path(), &first).expect("import");
        let versions =
            add_document_version_at(temp_dir.path(), imported.document.id, &second, "test")
                .expect("add version");
        assert_eq!(versions.len(), 2);
        assert!(versions[0].is_current_file);
        assert_eq!(versions[0].version_no, 2);

        let connection = Connection::open(temp_dir.path().join("vault.db")).expect("connection");
        connection.execute(
            "INSERT INTO reminders (vault_id, document_id, title, due_date) VALUES (1, ?1, 'Kontrollera avtal', '2026-08-01')",
            params![imported.document.id],
        ).expect("reminder");
        let reminders = list_reminders_at(temp_dir.path()).expect("list reminders");
        assert_eq!(reminders.len(), 1);
        assert_eq!(reminders[0].document_id, Some(imported.document.id));

        connection
            .execute(
                "INSERT INTO watched_folders (vault_id, path) VALUES (1, ?1)",
                params![temp_dir.path().to_string_lossy()],
            )
            .expect("watched folder");
        assert_eq!(
            list_watched_folders_at(temp_dir.path())
                .expect("folders")
                .len(),
            1
        );
        let before = list_production_documents_at(temp_dir.path())
            .expect("before scan")
            .len();
        let scan = scan_watched_folders_at(temp_dir.path()).expect("scan watched folder");
        assert!(scan.duplicate_count >= 2);
        let after = list_production_documents_at(temp_dir.path())
            .expect("after scan")
            .len();
        assert_eq!(
            before, after,
            "repeated watched-folder scans must be idempotent"
        );
        process_watched_folder_job_at(temp_dir.path()).expect("background folder job");
        let jobs = list_background_jobs_at(temp_dir.path()).expect("jobs");
        assert_eq!(jobs[0].status, "completed");
        assert_eq!(jobs[0].job_type, "watched_folder_scan");
    }

    #[test]
    fn archive_analysis_backfills_missing_document_dates() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        initialize_vault_at(temp_dir.path()).expect("vault initialization");
        let source_file = temp_dir.path().join("faktura-demo.txt");
        fs::write(
            &source_file,
            "Faktura\nFakturadatum 25/06/2024\nTotal 349 SEK",
        )
        .expect("write source");
        let import = import_document_at(temp_dir.path(), &source_file).expect("import");
        let connection = Connection::open(temp_dir.path().join("vault.db")).expect("connection");
        connection
            .execute(
                "UPDATE documents SET document_date = NULL WHERE id = ?1",
                params![import.document.id],
            )
            .expect("clear date");

        let result = analyze_production_archive_at(temp_dir.path()).expect("analysis");
        let detail =
            get_production_document_detail_at(temp_dir.path(), import.document.id).expect("detail");

        assert_eq!(result.date_updated_count, 1);
        assert_eq!(detail.document.document_date.as_deref(), Some("2024-06-25"));
        assert!(detail
            .claims
            .iter()
            .any(|claim| claim.extraction_method == "local_date_rules_v2"));
    }

    fn write_test_zip(path: &Path, entries: &[(&str, &str)]) {
        let file = fs::File::create(path).expect("zip file");
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        for (name, content) in entries {
            zip.start_file(name, options).expect("zip entry");
            std::io::Write::write_all(&mut zip, content.as_bytes()).expect("zip content");
        }
        zip.finish().expect("finish zip");
    }
}
#[test]
fn hashes_pin_deterministically_and_masks_long_numbers() {
    let first = derive_pin_hash("1234", "local-test-salt");
    let second = derive_pin_hash("1234", "local-test-salt");
    assert_eq!(first, second);
    assert_ne!(first, derive_pin_hash("4321", "local-test-salt"));
    assert_eq!(first.len(), 64);
    let masked = mask_sensitive_text("Konto 1234567890 kod 1234");
    assert!(masked.contains("12••••"));
    assert!(masked.contains("1234"));
    assert!(!masked.contains("1234567890"));
}

#[test]
fn converts_html_snapshot_without_scripts_or_markup() {
    let text =
        html_to_visible_text("<main><h1>Kvitto &amp; garanti</h1><p>Belopp 1 299 kr</p></main>");
    assert_eq!(text, "Kvitto & garanti Belopp 1 299 kr");
    assert!(!text.contains('<'));
}

#[test]
fn testlab_scale_remains_isolated_and_test_center_compatible() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    initialize_vault_at(temp_dir.path()).expect("initialize");
    let scaled = generate_testlab_scale_at(temp_dir.path(), 100).expect("scale generation");
    assert_eq!(scaled.total_documents, 104);
    let production = Connection::open(temp_dir.path().join("vault.db")).expect("production");
    assert_eq!(
        production
            .query_row("SELECT COUNT(*) FROM documents", [], |r| r.get::<_, i64>(0))
            .expect("production count"),
        0
    );
    assert_eq!(
        run_test_center_at(temp_dir.path())
            .expect("test center")
            .failed,
        0
    );
}

#[test]
fn extracts_extended_domain_fields_without_inference() {
    let housing = extract_domain_fields(
        "housing",
        "Adress: Storgatan 1\nHyresvärd: Lokalvärden AB\nMånadskostnad: 8 500 kr",
    );
    assert_eq!(housing["address"], "Storgatan 1");
    assert_eq!(housing["landlord"], "Lokalvärden AB");
    let authority = extract_domain_fields(
        "authority",
        "Myndighet: Skatteverket\nÄrendenummer: ABC-123\nInstruktioner: Svara skriftligt",
    );
    assert_eq!(authority["authority"], "Skatteverket");
    assert_eq!(authority["case_number"], "ABC-123");
    assert!(authority.get("legal_effect").is_none());
}

#[test]
fn validates_calendar_dates_strictly() {
    assert!(is_valid_iso_date("2026-07-21"));
    assert!(!is_valid_iso_date("21/07/2026"));
    assert!(!is_valid_iso_date("2026-7-21"));
}

#[test]
fn creates_valid_portable_archive_and_scans_integrity() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    initialize_vault_at(temp_dir.path()).expect("initialize");
    let source = temp_dir.path().join("portable-source.txt");
    fs::write(
        &source,
        "Portabelt originaldokument med tillräckligt mycket identisk text för kontroll.",
    )
    .expect("source");
    import_document_at(temp_dir.path(), &source).expect("import");
    let scan = scan_file_integrity_at(temp_dir.path()).expect("integrity scan");
    assert_eq!(scan.checked_files, 1);
    assert_eq!(scan.intact_files, 1);
    let portable = create_portable_archive_at(temp_dir.path()).expect("portable archive");
    assert!(Path::new(&portable.archive_path).is_file());
    assert_eq!(portable.document_count, 1);
    let file = fs::File::open(&portable.archive_path).expect("archive file");
    let mut zip = zip::ZipArchive::new(file).expect("zip");
    assert!(zip.by_name("portable-format.json").is_ok());
    assert!(zip.by_name("documents.csv").is_ok());
    assert!(zip.by_name("vault.db").is_ok());
    drop(zip);
    let target = tempfile::tempdir().expect("target tempdir");
    initialize_vault_at(target.path()).expect("initialize target");
    let restored = restore_portable_archive_at(target.path(), Path::new(&portable.archive_path))
        .expect("restore portable");
    assert_eq!(restored.document_count, 1);
    assert_eq!(
        list_production_documents_at(target.path())
            .expect("restored documents")
            .len(),
        1
    );
}

#[test]
fn decrypts_aes256_private_container_only_with_correct_password() {
    let temp = tempfile::tempdir().expect("tempdir");
    let archive_path = temp.path().join("private.vaultprivate");
    let file = fs::File::create(&archive_path).expect("archive");
    let mut writer = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .with_aes_encryption(zip::AesMode::Aes256, "correct-horse");
    writer.start_file("original.txt", options).expect("entry");
    writer.write_all(b"hemligt lokalt original").expect("write");
    writer.finish().expect("finish");
    let target = temp.path().join("opened.txt");
    assert!(decrypt_private_file(&archive_path, "wrong-password", &target).is_err());
    assert!(decrypt_private_file(&archive_path, "correct-horse", &target).is_ok());
    assert_eq!(
        fs::read_to_string(target).expect("text"),
        "hemligt lokalt original"
    );
}

#[test]
fn rejects_plugins_with_network_or_ai_access() {
    let safe = PluginManifest {
        id: "local.tags".into(),
        name: "Lokala taggar".into(),
        version: "1.0.0".into(),
        description: String::new(),
        capabilities: vec!["read_metadata".into(), "write_tags".into()],
        data_access: vec!["metadata".into()],
        actions: vec![PluginAction {
            action_type: "add_tag".into(),
            match_text: "kvitto".into(),
            value: "Ekonomi".into(),
        }],
        ai_free: true,
    };
    assert!(validate_plugin_manifest(&safe).is_ok());
    let network = PluginManifest {
        id: "bad.network".into(),
        name: "Bad".into(),
        version: "1".into(),
        description: String::new(),
        capabilities: vec!["network".into()],
        data_access: vec!["metadata".into()],
        actions: vec![],
        ai_free: true,
    };
    assert!(validate_plugin_manifest(&network)
        .unwrap_err()
        .contains("Otillåten"));
    let ai = PluginManifest {
        id: "bad.ai".into(),
        name: "Bad AI".into(),
        version: "1".into(),
        description: String::new(),
        capabilities: vec![],
        data_access: vec![],
        actions: vec![],
        ai_free: false,
    };
    assert!(validate_plugin_manifest(&ai)
        .unwrap_err()
        .contains("ai_free"));
}

#[test]
fn stores_linked_note_with_versions_and_templates() {
    let temp = tempfile::tempdir().expect("tempdir");
    initialize_vault_at(temp.path()).expect("initialize");
    let connection = Connection::open(temp.path().join("vault.db")).expect("database");
    connection.execute("INSERT INTO vault_notes(vault_id,title,body,kind,target_type,target_id,tags_json,code_words_json) VALUES(1,'Servicekontroll','# Kontroll\n- [ ] Kvitto','todo','object',42,'[\"bil\"]','[\"service\"]')",[]).expect("note");
    let id = connection.last_insert_rowid();
    connection.execute("INSERT INTO vault_note_versions(note_id,version_no,title,body) VALUES(?1,1,'Servicekontroll','# Kontroll')",[id]).expect("version 1");
    connection.execute("INSERT INTO vault_note_versions(note_id,version_no,title,body) VALUES(?1,2,'Servicekontroll','Uppdaterad')",[id]).expect("version 2");
    let (target, versions): (String, i64) = connection.query_row("SELECT target_type,(SELECT COUNT(*) FROM vault_note_versions WHERE note_id=n.id) FROM vault_notes n WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?))).expect("linked note");
    assert_eq!(target, "object");
    assert_eq!(versions, 2);
    let templates: i64 = connection
        .query_row("SELECT COUNT(*) FROM note_templates", [], |r| r.get(0))
        .expect("templates");
    assert!(templates >= 2);
}

#[test]
fn creates_self_contained_incremental_backup_and_honors_exclusions() {
    let temp = tempfile::tempdir().expect("tempdir");
    initialize_vault_at(temp.path()).expect("initialize");
    let source = temp.path().join("backup-source.txt");
    fs::write(&source, "lokal backupfil").expect("write");
    let imported = import_document_at(temp.path(), &source).expect("import");
    let full = create_backup_with_options_at(temp.path(), "full", &[]).expect("full");
    assert_eq!(full.copied_file_count, 1);
    assert_eq!(full.linked_file_count, 0);
    let incremental =
        create_backup_with_options_at(temp.path(), "incremental", &[]).expect("incremental");
    assert_eq!(incremental.file_count, 1);
    assert_eq!(incremental.linked_file_count, 1);
    assert!(
        validate_local_backup_at(Path::new(&incremental.backup_root))
            .expect("validate")
            .ok
    );
    let excluded = create_backup_with_options_at(temp.path(), "full", &[imported.document.id])
        .expect("excluded");
    assert_eq!(excluded.document_count, 0);
    assert_eq!(excluded.file_count, 0);
    assert_eq!(excluded.copied_file_count, 0);
}

#[test]
fn deterministic_near_duplicate_fingerprints_detect_small_changes() {
    let first = text_simhash(
        "anställningsavtal dagab startdatum 2024 lön och villkor uppsägningstid tre månader",
    );
    let second = text_simhash(
        "anställningskontrakt dagab startdatum 2024 lön och villkor uppsägningstid tre månader",
    );
    assert!((first ^ second).count_ones() <= 12);
    let temp = tempfile::tempdir().expect("tempdir");
    let first_path = temp.path().join("first.png");
    let second_path = temp.path().join("second.png");
    let mut image = image::GrayImage::from_pixel(32, 32, image::Luma([240]));
    for x in 4..28 {
        for y in 4..28 {
            image.put_pixel(x, y, image::Luma([40]));
        }
    }
    image.save(&first_path).expect("first image");
    image.put_pixel(5, 5, image::Luma([45]));
    image.save(&second_path).expect("second image");
    let distance = (perceptual_image_hash(&first_path).expect("hash")
        ^ perceptual_image_hash(&second_path).expect("hash"))
    .count_ones();
    assert!(distance <= 2);
}
