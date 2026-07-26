import { useEffect, useMemo, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { lazy, Suspense } from 'react'
import type { CSSProperties } from 'react'
import { open } from '@tauri-apps/plugin-dialog'
import { getCurrentWindow } from '@tauri-apps/api/window'
import './App.css'
const DocumentViewer = lazy(() => import('./DocumentViewer').then((module) => ({ default: module.DocumentViewer })))

type EnvironmentStatus = {
  app_mode: string
  storage: string
  database: string
  search_index: string
  network: string
  ai_policy: string
}

type VaultInitStatus = {
  data_root: string
  database_path: string
  testlab_database_path: string
  schema_version: number
  production_vault_ready: boolean
  testlab_isolated: boolean
  testlab_document_count: number
}

type DocumentSummary = {
  id: number
  title: string
  document_type: string
  document_date: string | null
  inbox_status: string
  source_label: string
  match_explanation: string
}

type SearchResult = {
  document: DocumentSummary
  score: number
  reasons: string[]
  query_plan: string[]
}

type ImportResult = {
  document: DocumentSummary
  file_id: number
  version_id: number
  sha256: string
  stored_path: string
  duplicate: boolean
}

type DirectoryImportResult = {
  scanned_file_count: number
  imported_count: number
  duplicate_count: number
  failed_count: number
  results: ImportResult[]
  failures: string[]
}

type ImportSessionSummary = {
  id: number
  method: string
  source_label: string
  status: string
  scanned_count: number
  imported_count: number
  duplicate_count: number
  failed_count: number
  started_at: string
  completed_at: string | null
  items: { source_name: string; document_id: number | null; status: string; safe_error: string | null }[]
}

type AutomationRuleSummary = { id:number; name:string; enabled:boolean; priority:number; match_field:string; match_operator:string; match_value:string; action_type:string; action_value:string; approval_policy:string; version:number }
type DocumentTemplateSummary = { id:number; name:string; document_type:string; category:string; default_tags:string; required_fields:string }
type CustomFieldSummary = { id:number; name:string; field_type:string; applies_to:string; required:boolean }
type SavedSearchSummary = { id:number; name:string; query:string; pinned:boolean }
type RuleRunResult = { scanned_document_count:number; matched_document_count:number; applied_action_count:number; explanations:string[] }
type SecurityStatus = { mode:string; pin_configured:boolean; auto_lock_minutes:number; mask_sensitive:boolean; locked_document_count:number;private_document_count:number }
type WindowsHelloStatus = {available:boolean;explanation:string}
type ProtectedExportResult = { export_path:string; sha256:string; size_bytes:number; masked:boolean }
type DataExportResult = { export_path:string; format:string; table_count:number; row_count:number; file_count:number; sha256:string|null }
type ProfileImportResult = { saved_searches:number; themes:number; code_words:number; rules:number; dashboard_layout_json:string }

type DocumentDetail = {
  document: DocumentSummary
  extracted_text: string
  stored_path: string | null
  original_name: string | null
  mime_type: string | null
  size_bytes: number | null
  tags: string[]
  claims: ClaimSummary[]
  entities: EntitySummary[]
}

type EntitySummary = {
  id: number
  entity_type: string
  display_name: string
  role: string
}

type EntityCatalogItem = {
  id: number
  entity_type: string
  display_name: string
  document_count: number
}

type TimelineEventSummary = {
  event_date: string
  event_type: string
  title: string
  document_id: number | null
  entity_names: string[]
  source_page_no: number | null
  status: string
  explanation: string
}

type DomainRecordSummary = {
  id: number
  document_id: number
  document_title: string
  domain_type: string
  record_type: string
  subject: string
  effective_from: string | null
  effective_to: string | null
  actuality_status: string
  actuality_explanation: string
  fields_json: string
  source_page_no: number | null
  extraction_method: string
  manually_locked: boolean
}

type ReviewQueueItem = {
  claim_id: number
  document_id: number
  document_title: string
  claim_type: string
  value_json: string
  status: string
  extraction_method: string
}

type ArchiveAnalysisResult = {
  document_count: number
  reindexed_document_count: number
  ocr_attempt_count: number
  ocr_updated_count: number
  reclassified_document_count: number
  date_updated_count: number
  entity_link_count: number
  pending_review_count: number
}

type ClaimSummary = {
  id: number
  claim_type: string
  value_json: string
  value_kind: string
  status: string
  confidence_kind: string
  extraction_method: string
  source_span_id: number | null
  source_page_no: number | null
  source_text: string | null
  effective_from: string | null
  effective_to: string | null
  actuality_status: string
  actuality_explanation: string
}

type CodeWordSummary = {
  id: number
  word: string
  description: string
  created_at: string
}

type ConflictSummary = {
  id: number | null
  conflict_type: string
  severity: string
  title: string
  detail: string
  document_ids: number[]
  status: string
  resolution: string | null
  locked: boolean
}

type DocumentVersionSummary = {
  id: number
  version_no: number
  imported_at: string
  document_date: string | null
  original_name: string
  mime_type: string
  size_bytes: number
  sha256: string
  is_current_file: boolean
  comment: string
  origin: string
}

type DocumentAnnotationSummary={id:number;document_id:number;page_no:number;annotation_type:'bookmark'|'note'|'highlight';selected_text:string;body:string;color:string;created_at:string;updated_at:string}
type VersionComparison={left_version_no:number;right_version_no:number;added_lines:string[];removed_lines:string[];left_sha256:string;right_sha256:string;size_change_bytes:number;page_change:number;date_changed:boolean;identical:boolean;left_lines:string[];right_lines:string[]}
type DashboardSize='small'|'medium'|'large'
type DashboardLayout={name:string;widgets:string[];sizes:Record<string,DashboardSize>}

type DocumentPageSummary = {
  page_no: number
  source_kind: string
  text_quality: string
  text: string
}

type ReminderSummary = {
  id: number
  document_id: number | null
  document_title: string | null
  title: string
  due_date: string
  status: string
  note: string
}

type WatchedFolderSummary = {
  id: number
  path: string
  enabled: boolean
  last_scanned_at: string | null
}

type JobSummary = {
  id: number
  job_type: string
  target_id: number | null
  status: string
  attempts: number
  error_code: string | null
  created_at: string
  updated_at: string
  progress_current: number
  progress_total: number
  pause_requested: boolean
  result_summary: string | null
}

type OcrStatus = {
  available: boolean
  engine: string
  executable_path: string | null
  tessdata_dir: string | null
  languages: string[]
  version: string | null
  notes: string[]
}

type BackupResult = {
  backup_root: string
  backup_path: string
  manifest_path: string
  sha256: string
  size_bytes: number
  document_count: number
  file_count: number
  copied_file_count: number
  linked_file_count: number
  backup_mode: string
  created_at_epoch_seconds: number
  destination_copy: string | null
}

type BackupSummary = BackupResult
type BackupPolicy = {enabled:boolean;paused:boolean;interval_hours:number;backup_mode:string;excluded_document_ids:number[];last_run_epoch_seconds:number;destination_path:string|null}
type RestoreTestResult={ok:boolean;document_count:number;file_count:number;missing_file_count:number;database_integrity:string;tested_backup_root:string}

type ProtectedBackupResult = { archive_path:string; sha256:string; size_bytes:number; entry_count:number; source_backup_root:string }
type PortableArchiveResult = {archive_path:string;sha256:string;size_bytes:number;entry_count:number;document_count:number;file_count:number}
type DuplicateCandidate = {primary_document_id:number;primary_title:string;secondary_document_id:number;secondary_title:string;confidence:number;match_kind:string;explanation:string;decision:string|null}
type IntegrityScanReport = {checked_files:number;intact_files:number;missing_files:number;changed_files:number;duplicate_groups:number;candidates:DuplicateCandidate[];warnings:string[]}
type PluginSummary={id:string;name:string;version:string;description:string;capabilities:string[];data_access:string[];enabled:boolean;approved:boolean;last_run_at:string|null;last_result:string|null;api_version:string;adapter_kind:string;manifest_sha256:string;signature_status:string;resource_limit:number}
type PluginRunResult={plugin_id:string;scanned_documents:number;changed_documents:number;explanation:string}
type ExternalImportSource={provider:string;display_name:string;enabled:boolean;approved:boolean;local_staging_path:string|null;last_preview_at:string|null;last_import_at:string|null;last_result:string|null}
type ExternalImportFile={path:string;relative_path:string;size_bytes:number}
type ExternalImportPreview={provider:string;files:ExternalImportFile[];skipped_count:number;total_size_bytes:number}

type VaultHealthReport = {
  ok: boolean
  database_integrity: string
  schema_version: number
  document_count: number
  file_count: number
  missing_file_count: number
  fts_entry_count: number
  warnings: string[]
}

type ReindexResult = {
  indexed_document_count: number
}

type BackupValidationReport = {
  ok: boolean
  backup_root: string
  backup_path: string
  manifest_path: string
  sha256_matches: boolean
  database_integrity: string
  document_count: number
  file_count: number
  copied_file_count: number
  warnings: string[]
}

type RestoreResult = {
  restored_backup_root: string
  pre_restore_backup_root: string
  document_count: number
  file_count: number
}

type AuditEventSummary = {
  id: number
  event_type: string
  target_type: string
  target_id: number | null
  safe_summary: string
  created_at: string
}

type TestCenterCase = {
  name: string
  status: string
  expected: string
  actual: string
}

type TestCenterReport = {
  passed: number
  failed: number
  cases: TestCenterCase[]
}
type TestLabScaleResult = {requested:number;total_documents:number;elapsed_ms:number;search_elapsed_ms:number;database_bytes:number;import_budget_ms:number;search_budget_ms:number;budgets_met:boolean}
type TestReportExport = {path:string;sha256:string;case_count:number;created_at_epoch_seconds:number}

type VaultDiagnostics = {
  data_root: string
  database_path: string
  testlab_database_path: string
  schema_version: number
  production_document_count: number
  testlab_document_count: number
  pending_review_count: number
  entity_count: number
  conflict_count: number
  backup_count: number
  audit_event_count: number
  ocr_available: boolean
  tesseract_path: string | null
  tessdata_dir: string | null
  ocr_languages: string[]
  pdf_text_available: boolean
  pdftotext_path: string | null
  office_text_available: boolean
  issues: string[]
}

type ThemeMode = 'system' | 'light' | 'dark'
type ThemeProfile = { id:number;name:string;base_mode:'light'|'dark';accent:string;surface_main:string;surface_sidebar:string;surface_raised:string;text_primary:string;text_secondary:string;border_color:string;radius_px:number;font_scale:number;density:'compact'|'comfortable'|'spacious';motion:'off'|'reduced'|'normal';is_builtin:boolean;is_active:boolean;version:number;shadow_strength:number;transparency:number;blur_px:number;font_family:'system'|'serif'|'mono';line_height:number;animation_speed:number;preview_ratio:'document'|'square'|'wide';background_image:string;schedule_mode:'manual'|'day_night'|'windows';follow_windows:boolean }
type CollectionSummary = {id:number;name:string;description:string;color:string;is_pinned:boolean;document_count:number}
type DocumentViewMode = 'list'|'grid'|'table'|'gallery'|'kanban'|'split'
type AnalysisPreferences = {mode:string;auto_ocr:boolean;auto_classify:boolean;auto_claims:boolean;auto_relations:boolean;include_locked:boolean}
type AnalysisExclusion = {id:number;scope_type:string;scope_value:string;reason:string}
type ScannerStatus = {available:boolean;launcher_path:string|null;notes:string[]}
type OrganizerSummary = {id:number;name:string;color:string;description:string;is_pinned:boolean;is_locked:boolean;document_count:number}
type DocumentNoteSummary = {id:number;document_id:number;body:string;kind:string;version_no:number;created_at:string;updated_at:string}
type SearchHistorySummary = {id:number;query:string;result_count:number;executed_at:string;pinned:boolean}
type SearchHistoryPreferences = {enabled:boolean;retention_days:number}
type VaultNoteSummary = {id:number;title:string;body:string;kind:string;target_type:string;target_id:number|null;tags:string[];code_words:string[];version_no:number;created_at:string;updated_at:string}
type NoteVersionSummary = {version_no:number;title:string;body:string;created_at:string}
type NoteTemplateSummary = {id:number;name:string;body:string}
type CalendarEventSummary = {id:string;event_date:string;title:string;event_type:string;source_kind:string;status:string;document_id:number|null;explanation:string}
type GraphNodeSummary = {key:string;node_type:string;label:string;document_id:number|null}
type GraphEdgeSummary = {id:number;source_key:string;target_key:string;relation_type:string;status:string;valid_from:string|null;valid_to:string|null;explanation:string}
type GraphDataSummary = {nodes:GraphNodeSummary[];edges:GraphEdgeSummary[]}
type ActiveView =
  | 'start'
  | 'documents'
  | 'inbox'
  | 'archive'
  | 'favorites'
  | 'recent'
  | 'collections'
  | 'folders'
  | 'categories'
  | 'searchhistory'
  | 'notes'
  | 'capture'
  | 'analysissettings'
  | 'trash'
  | 'review'
  | 'relations'
  | 'people'
  | 'companies'
  | 'objects'
  | 'employment'
  | 'education'
  | 'contracts'
  | 'timeline'
  | 'calendar'
  | 'graph'
  | 'housing'
  | 'travel'
  | 'authorities'
  | 'actuality'
  | 'conflicts'
  | 'reminders'
  | 'automation'
  | 'importhistory'
  | 'rules'
  | 'security'
  | 'themes'
  | 'backup'
  | 'integrity'
  | 'plugins'
  | 'testcenter'

const fallbackDocuments: DocumentSummary[] = [
  {
    id: 10,
    title: 'DAGAB anställningsavtal',
    document_type: 'Anställningsavtal',
    document_date: '2024-02-01',
    inbox_status: 'inbox',
    source_label: 'Syntetiskt Test Lab-underlag',
    match_explanation: 'Matchar företag, avtal och anställning',
  },
  {
    id: 11,
    title: 'Lönespecifikation juni',
    document_type: 'Lönespecifikation',
    document_date: '2024-06-25',
    inbox_status: 'indexed',
    source_label: 'Syntetiskt Test Lab-underlag',
    match_explanation: 'Juni tolkas som månad 6 och löneperiod prioriteras',
  },
  {
    id: 12,
    title: 'Passhandling syntetisk person',
    document_type: 'Identitetshandling',
    document_date: '2022-09-12',
    inbox_status: 'review',
    source_label: 'Syntetiskt Test Lab-underlag',
    match_explanation: 'MRZ-parser planerad, OCR saknas lokalt',
  },
]

const emptyDocument: DocumentSummary = {
  id: 0,
  title: 'Inget dokument valt',
  document_type: '—',
  document_date: null,
  inbox_status: 'inbox',
  source_label: '—',
  match_explanation: 'Välj ett dokument i listan för att visa detaljer.',
}

const defaultStatus: EnvironmentStatus = {
  app_mode: 'web preview',
  storage: 'local storage planned',
  database: 'SQLite planned',
  search_index: 'SQLite FTS5 planned',
  network: 'no required network services',
  ai_policy: 'AI-free by product rule',
}

const defaultVaultInit: VaultInitStatus = {
  data_root: 'Endast tillgänglig i EXE-läge',
  database_path: 'Endast tillgänglig i EXE-läge',
  testlab_database_path: 'Endast tillgänglig i EXE-läge',
  schema_version: 0,
  production_vault_ready: false,
  testlab_isolated: false,
  testlab_document_count: fallbackDocuments.length,
}

const defaultOcrStatus: OcrStatus = {
  available: false,
  engine: 'tesseract',
  executable_path: null,
  tessdata_dir: null,
  languages: [],
  version: null,
  notes: ['OCR-status inte kontrollerad'],
}

function formatInboxStatus(status: string) {
  const labels: Record<string, string> = {
    inbox: 'Inkorg',
    indexed: 'Indexerat',
    review: 'Granskning',
    reviewed: 'Granskat',
    archived: 'Arkiverat',
  }

  return labels[status] ?? status
}

function formatDomainStatus(status: string) {
  const labels: Record<string, string> = {
    current: 'Aktuell',
    historical: 'Historisk',
    future: 'Framtida',
    uncertain: 'Osäker',
    historical_record: 'Historikpost',
    documented_open_period: 'Dokumenterad öppen period',
  }
  return labels[status] ?? status
}

function formatRecordType(type: string) {
  const labels: Record<string, string> = {
    employment_agreement: 'Anställningsavtal',
    salary_period: 'Löneperiod',
    education_document: 'Utbildningsdokument',
    vehicle_document: 'Fordonsdokument',
    identity_document: 'Identitetshandling',
    agreement: 'Avtal',
    purchase_record: 'Inköpspost',
    housing_record: 'Boendepost',
    travel_record: 'Resepost',
    authority_case: 'Myndighetsärende',
  }
  return labels[type] ?? type
}

function localFallbackSearch(documents: DocumentSummary[], query: string): SearchResult[] {
  const normalizedQuery = query.trim().toLowerCase()

  if (!normalizedQuery) {
    return documents.map((document) => ({
      document,
      score: 0,
      reasons: ['Ingen sökfråga: visar Test Lab-lista i stabil ordning'],
      query_plan: [],
    }))
  }

  return documents
    .filter((document) =>
      [
        document.title,
        document.document_type,
        document.document_date ?? '',
        document.source_label,
        document.match_explanation,
      ]
        .join(' ')
        .toLowerCase()
        .includes(normalizedQuery),
    )
    .map((document) => ({
      document,
      score: 1,
      reasons: ['Web-preview fallback matchade text lokalt'],
      query_plan: [normalizedQuery],
    }))
}

function App() {
  const [status, setStatus] = useState<EnvironmentStatus>(defaultStatus)
  const [vaultInit, setVaultInit] = useState<VaultInitStatus>(defaultVaultInit)
  const [testlabDocuments, setTestlabDocuments] = useState<DocumentSummary[]>(fallbackDocuments)
  const [productionDocuments, setProductionDocuments] = useState<DocumentSummary[]>([])
  const [trashedDocuments, setTrashedDocuments] = useState<DocumentSummary[]>([])
  const [searchResults, setSearchResults] = useState<SearchResult[]>([])
  const [query, setQuery] = useState('')
  const [showSearchFilters,setShowSearchFilters]=useState(false)
  const [searchFilters,setSearchFilters]=useState({type:'',category:'',employer:'',status:'',mime:'',from:'',to:''})
  const [selectedIndex, setSelectedIndex] = useState(0)
  const [resultWindowStart,setResultWindowStart]=useState(0)
  const [batchMode,setBatchMode]=useState(false)
  const [selectedDocumentIds,setSelectedDocumentIds]=useState<number[]>([])
  const [batchDraft,setBatchDraft]=useState({action:'add_tag',value:''})
  const [lastImport, setLastImport] = useState<ImportResult | null>(null)
  const [documentDetail, setDocumentDetail] = useState<DocumentDetail | null>(null)
  const [importError, setImportError] = useState<string | null>(null)
  const [lastDirectoryImport, setLastDirectoryImport] = useState<DirectoryImportResult | null>(null)
  const [importHistory, setImportHistory] = useState<ImportSessionSummary[]>([])
  const [automationRules, setAutomationRules] = useState<AutomationRuleSummary[]>([])
  const [documentTemplates, setDocumentTemplates] = useState<DocumentTemplateSummary[]>([])
  const [customFields, setCustomFields] = useState<CustomFieldSummary[]>([])
  const [savedSearches, setSavedSearches] = useState<SavedSearchSummary[]>([])
  const [ruleRun, setRuleRun] = useState<RuleRunResult | null>(null)
  const [securityStatus, setSecurityStatus] = useState<SecurityStatus>({mode:'comfortable',pin_configured:false,auto_lock_minutes:15,mask_sensitive:true,locked_document_count:0,private_document_count:0})
  const [lockedDocumentIds, setLockedDocumentIds] = useState<number[]>([])
  const [privateDocumentIds,setPrivateDocumentIds]=useState<number[]>([])
  const [guestMode,setGuestMode]=useState(false)
  const [securityDraft, setSecurityDraft] = useState({mode:'comfortable',pin:'',autoLockMinutes:15,maskSensitive:true})
  const [unlockPin, setUnlockPin] = useState('')
  const [securityCredential, setSecurityCredential] = useState('')
  const [vaultUnlocked, setVaultUnlocked] = useState(true)
  const [windowsHello,setWindowsHello]=useState<WindowsHelloStatus>({available:false,explanation:'Kontrollerar Windows Hello…'})
  const [quickUnlockEnabled,setQuickUnlockEnabled]=useState(false)
  const [lastProtectedExport, setLastProtectedExport] = useState<ProtectedExportResult | null>(null)
  const [ruleDraft, setRuleDraft] = useState({name:'Avtal får etikett',match_field:'text',match_operator:'contains',match_value:'avtal',logic_operator:'AND',second_field:'title',second_operator:'contains',second_value:'',action_type:'add_tag',action_value:'Avtal',approval_policy:'suggest'})
  const [templateDraft, setTemplateDraft] = useState({name:'',document_type:'',category:'',default_tags:'',required_fields:''})
  const [fieldDraft, setFieldDraft] = useState({name:'',field_type:'text',applies_to:'all',required:false})
  const [searchDraft, setSearchDraft] = useState({name:'',query:'',pinned:true})
  const [clipboardDraft, setClipboardDraft] = useState({ title: 'Urklipp', text: '' })
  const [showClipboardImport, setShowClipboardImport] = useState(false)
  const [isDraggingFiles, setIsDraggingFiles] = useState(false)
  const [fileActionError, setFileActionError] = useState<string | null>(null)
  const [backupError, setBackupError] = useState<string | null>(null)
  const [lastBackup, setLastBackup] = useState<BackupResult | null>(null)
  const [backupPassword, setBackupPassword] = useState('')
  const [protectedBackup, setProtectedBackup] = useState<ProtectedBackupResult | null>(null)
  const [portableArchive,setPortableArchive]=useState<PortableArchiveResult|null>(null)
  const [dataExport,setDataExport]=useState<DataExportResult|null>(null)
  const [dataExportDraft,setDataExportDraft]=useState({format:'package',includeOriginals:true,folderLayout:'vault'})
  const [integrityReport,setIntegrityReport]=useState<IntegrityScanReport|null>(null)
  const [plugins,setPlugins]=useState<PluginSummary[]>([])
  const [pluginRun,setPluginRun]=useState<PluginRunResult|null>(null)
  const [pluginError,setPluginError]=useState<string|null>(null)
  const [externalImportSources,setExternalImportSources]=useState<ExternalImportSource[]>([])
  const [externalImportPreview,setExternalImportPreview]=useState<ExternalImportPreview|null>(null)
  const [externalImportSelection,setExternalImportSelection]=useState<string[]>([])
  const [backups, setBackups] = useState<BackupSummary[]>([])
  const [backupPolicy,setBackupPolicy]=useState<BackupPolicy>({enabled:false,paused:false,interval_hours:24,backup_mode:'incremental',excluded_document_ids:[],last_run_epoch_seconds:0,destination_path:null})
  const [backupExclusions,setBackupExclusions]=useState('')
  const [restoreTest,setRestoreTest]=useState<RestoreTestResult|null>(null)
  const [passwordRestorePath,setPasswordRestorePath]=useState('')
  const [healthError, setHealthError] = useState<string | null>(null)
  const [healthReport, setHealthReport] = useState<VaultHealthReport | null>(null)
  const [reindexError, setReindexError] = useState<string | null>(null)
  const [lastReindex, setLastReindex] = useState<ReindexResult | null>(null)
  const [backupValidation, setBackupValidation] = useState<BackupValidationReport | null>(null)
  const [restoreError, setRestoreError] = useState<string | null>(null)
  const [lastRestore, setLastRestore] = useState<RestoreResult | null>(null)
  const [auditEvents, setAuditEvents] = useState<AuditEventSummary[]>([])
  const [testCenterError, setTestCenterError] = useState<string | null>(null)
  const [testCenterReport, setTestCenterReport] = useState<TestCenterReport | null>(null)
  const [testLabScale,setTestLabScale]=useState<TestLabScaleResult|null>(null)
  const [testReportExport,setTestReportExport]=useState<TestReportExport|null>(null)
  const [simulatedCases,setSimulatedCases]=useState<TestCenterCase[]>([])
  const [testModule,setTestModule]=useState('all')
  const [diagnostics, setDiagnostics] = useState<VaultDiagnostics | null>(null)
  const [diagnosticsError, setDiagnosticsError] = useState<string | null>(null)
  const [codeWords, setCodeWords] = useState<CodeWordSummary[]>([])
  const [conflicts, setConflicts] = useState<ConflictSummary[]>([])
  const [versions, setVersions] = useState<DocumentVersionSummary[]>([])
  const [documentPages, setDocumentPages] = useState<DocumentPageSummary[]>([])
  const [selectedPageNo, setSelectedPageNo] = useState(1)
  const [viewerZoom,setViewerZoom]=useState(100)
  const [viewerRotation,setViewerRotation]=useState(0)
  const [viewerSearch,setViewerSearch]=useState('')
  const [viewerFullscreen,setViewerFullscreen]=useState(false)
  const [internalViewerOpen,setInternalViewerOpen]=useState(false)
  const [viewerSourceClaim,setViewerSourceClaim]=useState<ClaimSummary|null>(null)
  const [annotations,setAnnotations]=useState<DocumentAnnotationSummary[]>([])
  const [annotationDraft,setAnnotationDraft]=useState({type:'note' as 'bookmark'|'note'|'highlight',selectedText:'',body:'',color:'#ffe08a'})
  const [versionSelection,setVersionSelection]=useState<number[]>([])
  const [versionComparison,setVersionComparison]=useState<VersionComparison|null>(null)
  const [versionComment,setVersionComment]=useState('')
  const [reminders, setReminders] = useState<ReminderSummary[]>([])
  const [watchedFolders, setWatchedFolders] = useState<WatchedFolderSummary[]>([])
  const [backgroundJobs, setBackgroundJobs] = useState<JobSummary[]>([])
  const [systemJobType,setSystemJobType]=useState('archive_analysis')
  const [reminderDraft, setReminderDraft] = useState({ title: '', due_date: '', note: '' })
  const [workflowError, setWorkflowError] = useState<string | null>(null)
  const [isAddingVersion, setIsAddingVersion] = useState(false)
  const [isScanningFolders, setIsScanningFolders] = useState(false)
  const [reviewQueue, setReviewQueue] = useState<ReviewQueueItem[]>([])
  const [entityCatalog, setEntityCatalog] = useState<EntityCatalogItem[]>([])
  const [timelineEvents, setTimelineEvents] = useState<TimelineEventSummary[]>([])
  const [domainRecords, setDomainRecords] = useState<DomainRecordSummary[]>([])
  const [lastAnalysis, setLastAnalysis] = useState<ArchiveAnalysisResult | null>(null)
  const [analysisError, setAnalysisError] = useState<string | null>(null)
  const [ocrStatus, setOcrStatus] = useState<OcrStatus>(defaultOcrStatus)
  const [ocrError, setOcrError] = useState<string | null>(null)
  const [metadataError, setMetadataError] = useState<string | null>(null)
  const [tagInput, setTagInput] = useState('')
  const [codeWordInput, setCodeWordInput] = useState('')
  const [codeWordDescription, setCodeWordDescription] = useState('')
  const [metadataDraft, setMetadataDraft] = useState({
    title: '',
    document_type: '',
    document_date: '',
    inbox_status: 'inbox',
    source_label: '',
    match_explanation: '',
  })
  const [isImporting, setIsImporting] = useState(false)
  const [isImportingDirectory, setIsImportingDirectory] = useState(false)
  const [isCreatingDemoArchive, setIsCreatingDemoArchive] = useState(false)
  const [isResettingTestlab, setIsResettingTestlab] = useState(false)
  const [isCreatingBackup, setIsCreatingBackup] = useState(false)
  const [isCheckingHealth, setIsCheckingHealth] = useState(false)
  const [isReindexing, setIsReindexing] = useState(false)
  const [isValidatingBackup, setIsValidatingBackup] = useState(false)
  const [isRestoringBackup, setIsRestoringBackup] = useState(false)
  const [isRunningTestCenter, setIsRunningTestCenter] = useState(false)
  const [isSavingMetadata, setIsSavingMetadata] = useState(false)
  const [isSavingTag, setIsSavingTag] = useState(false)
  const [isSavingCodeWord, setIsSavingCodeWord] = useState(false)
  const [isCheckingOcr, setIsCheckingOcr] = useState(false)
  const [isAnalyzingArchive, setIsAnalyzingArchive] = useState(false)
  const [isTrashingDocument, setIsTrashingDocument] = useState(false)
  const [activeView, setActiveView] = useState<ActiveView>('start')
  const [themeMode, setThemeMode] = useState<ThemeMode>('system')
  const [themeProfiles,setThemeProfiles]=useState<ThemeProfile[]>([])
  const [themeDraft,setThemeDraft]=useState<Omit<ThemeProfile,'id'|'is_builtin'|'is_active'|'version'>>({name:'Mitt tema',base_mode:'dark',accent:'#8bd8a5',surface_main:'#11130f',surface_sidebar:'#0c0e0b',surface_raised:'#20241d',text_primary:'#f4f1e8',text_secondary:'#d3cfc3',border_color:'#31372c',radius_px:12,font_scale:1,density:'comfortable',motion:'normal',shadow_strength:.25,transparency:1,blur_px:0,font_family:'system',line_height:1.45,animation_speed:1,preview_ratio:'document',background_image:'',schedule_mode:'manual',follow_windows:false})
  const [themeMessage,setThemeMessage]=useState<string|null>(null)
  const [favoriteDocumentIds,setFavoriteDocumentIds]=useState<number[]>([])
  const [collections,setCollections]=useState<CollectionSummary[]>([])
  const [selectedCollectionId,setSelectedCollectionId]=useState<number|null>(null)
  const [collectionDocumentIds,setCollectionDocumentIds]=useState<number[]>([])
  const [collectionDraft,setCollectionDraft]=useState({name:'',description:'',color:'#7fd3a6',isPinned:true})
  const [documentViewMode,setDocumentViewMode]=useState<DocumentViewMode>('list')
  const [sidebarVisible,setSidebarVisible]=useState(()=>localStorage.getItem('vault.layout.sidebarVisible')!=='false')
  const [sidebarCollapsed,setSidebarCollapsed]=useState(()=>localStorage.getItem('vault.layout.sidebarCollapsed')==='true')
  const [sidebarWidth,setSidebarWidth]=useState(()=>Number(localStorage.getItem('vault.layout.sidebarWidth')||260))
  const [previewVisible,setPreviewVisible]=useState(()=>localStorage.getItem('vault.layout.previewVisible')!=='false')
  const [detailsVisible,setDetailsVisible]=useState(()=>localStorage.getItem('vault.layout.detailsVisible')!=='false')
  const [detailsWidth,setDetailsWidth]=useState(()=>Number(localStorage.getItem('vault.layout.detailsWidth')||310))
  const [commandPaletteOpen,setCommandPaletteOpen]=useState(false)
  const [commandQuery,setCommandQuery]=useState('')
  const [commandShortcutEditing,setCommandShortcutEditing]=useState(false)
  const [commandShortcuts,setCommandShortcuts]=useState<Record<string,string>>(()=>{try{return JSON.parse(localStorage.getItem('vault.command.shortcuts')||'{}')}catch{return {}}})
  const [analysisPreferences,setAnalysisPreferences]=useState<AnalysisPreferences>({mode:'manual',auto_ocr:true,auto_classify:true,auto_claims:true,auto_relations:true,include_locked:false})
  const [analysisExclusions,setAnalysisExclusions]=useState<AnalysisExclusion[]>([])
  const [exclusionDraft,setExclusionDraft]=useState({scopeType:'document_type',scopeValue:'',reason:''})
  const [scannerStatus,setScannerStatus]=useState<ScannerStatus>({available:false,launcher_path:null,notes:[]})
  const [webDraft,setWebDraft]=useState({title:'Sparad webbsida',sourceUrl:'',html:''})
  const [folders,setFolders]=useState<OrganizerSummary[]>([])
  const [categories,setCategories]=useState<OrganizerSummary[]>([])
  const [folderDraft,setFolderDraft]=useState({name:'',color:'#7fd3a6',isPinned:true})
  const [categoryDraft,setCategoryDraft]=useState({name:'',description:'',color:'#84aef5',isPinned:true})
  const [documentFolderIds,setDocumentFolderIds]=useState<number[]>([])
  const [documentCategoryIds,setDocumentCategoryIds]=useState<number[]>([])
  const [documentNotes,setDocumentNotes]=useState<DocumentNoteSummary[]>([])
  const [noteDraft,setNoteDraft]=useState({id:null as number|null,body:'',kind:'note'})
  const [searchHistory,setSearchHistory]=useState<SearchHistorySummary[]>([])
  const [searchHistoryPreferences,setSearchHistoryPreferences]=useState<SearchHistoryPreferences>({enabled:true,retention_days:90})
  const [searchHistoryMessage,setSearchHistoryMessage]=useState<string|null>(null)
  const [vaultNotes,setVaultNotes]=useState<VaultNoteSummary[]>([])
  const [noteVersions,setNoteVersions]=useState<NoteVersionSummary[]>([])
  const [noteTemplates,setNoteTemplates]=useState<NoteTemplateSummary[]>([])
  const [noteSearch,setNoteSearch]=useState('')
  const [vaultNoteDraft,setVaultNoteDraft]=useState({id:null as number|null,title:'',body:'',kind:'note',targetType:'vault',targetId:'',tags:'',codeWords:''})
  const [calendarEvents,setCalendarEvents]=useState<CalendarEventSummary[]>([])
  const [calendarDraft,setCalendarDraft]=useState({title:'',eventDate:'',eventType:'reminder',note:''})
  const [calendarMonth,setCalendarMonth]=useState(new Date().toISOString().slice(0,7))
  const [graphData,setGraphData]=useState<GraphDataSummary>({nodes:[],edges:[]})
  const [graphQuery,setGraphQuery]=useState('')
  const [graphTypes,setGraphTypes]=useState<string[]>(['document','person','organization','category','object'])
  const [relationDraft,setRelationDraft]=useState({sourceKey:'',targetKey:'',relationType:'relaterad till',validFrom:'',validTo:''})
  const defaultDashboardWidgets=['review','recent','uncategorized','undated','expiry','favorites','storage','conflicts','backup','index']
  const [dashboardWidgets,setDashboardWidgets]=useState<string[]>(()=>{try{return JSON.parse(localStorage.getItem('vault.dashboard.widgets')||'null')||defaultDashboardWidgets}catch{return defaultDashboardWidgets}})
  const [dashboardEditing,setDashboardEditing]=useState(false)
  const [dashboardSizes,setDashboardSizes]=useState<Record<string,DashboardSize>>(()=>{try{return JSON.parse(localStorage.getItem('vault.dashboard.sizes')||'{}')}catch{return {}}})
  const [dashboardLayouts,setDashboardLayouts]=useState<DashboardLayout[]>(()=>{try{return JSON.parse(localStorage.getItem('vault.dashboard.layouts')||'[]')}catch{return []}})
  const [activeDashboardLayout,setActiveDashboardLayout]=useState('Standard')
  const [dashboardLayoutName,setDashboardLayoutName]=useState('Min layout')

  useEffect(() => {
    document.documentElement.dataset.theme = themeMode
  }, [themeMode])

  useEffect(()=>localStorage.setItem('vault.dashboard.widgets',JSON.stringify(dashboardWidgets)),[dashboardWidgets])
  useEffect(()=>localStorage.setItem('vault.dashboard.sizes',JSON.stringify(dashboardSizes)),[dashboardSizes])
  useEffect(()=>localStorage.setItem('vault.dashboard.layouts',JSON.stringify(dashboardLayouts)),[dashboardLayouts])
  useEffect(()=>localStorage.setItem('vault.command.shortcuts',JSON.stringify(commandShortcuts)),[commandShortcuts])

  useEffect(()=>{const theme=themeProfiles.find(item=>item.is_active);if(!theme)return;const root=document.documentElement;const scheduledMode=theme.schedule_mode==='day_night'?(new Date().getHours()>=7&&new Date().getHours()<19?'light':'dark'):theme.base_mode;root.dataset.theme=theme.follow_windows||theme.schedule_mode==='windows'?(window.matchMedia('(prefers-color-scheme: dark)').matches?'dark':'light'):scheduledMode;root.dataset.density=theme.density;root.dataset.motion=theme.motion;root.dataset.previewRatio=theme.preview_ratio;root.style.setProperty('--accent',theme.accent);root.style.setProperty('--accent-strong',theme.accent);root.style.setProperty('--focus',theme.accent);root.style.setProperty('--surface-main',theme.surface_main);root.style.setProperty('--surface-sidebar',theme.surface_sidebar);root.style.setProperty('--surface-raised',theme.surface_raised);root.style.setProperty('--surface-row',theme.surface_raised);root.style.setProperty('--surface-input',theme.surface_raised);root.style.setProperty('--text-primary',theme.text_primary);root.style.setProperty('--text-secondary',theme.text_secondary);root.style.setProperty('--border',theme.border_color);root.style.setProperty('--radius',`${theme.radius_px}px`);root.style.setProperty('--shadow',`0 10px 35px rgb(0 0 0 / ${theme.shadow_strength})`);root.style.setProperty('--panel-opacity',String(theme.transparency));root.style.setProperty('--panel-blur',`${theme.blur_px}px`);root.style.setProperty('--line-height',String(theme.line_height));root.style.setProperty('--animation-speed',String(theme.animation_speed));root.style.setProperty('--font-family',theme.font_family==='serif'?'Georgia,serif':theme.font_family==='mono'?'Consolas,monospace':'Inter,Segoe UI,sans-serif');root.style.setProperty('--background-image',theme.background_image?`url("${theme.background_image.replaceAll('"','')}")`:'none');root.style.fontSize=`${theme.font_scale*100}%`},[themeProfiles])
  useEffect(()=>{localStorage.setItem('vault.layout.sidebarVisible',String(sidebarVisible));localStorage.setItem('vault.layout.sidebarCollapsed',String(sidebarCollapsed));localStorage.setItem('vault.layout.sidebarWidth',String(sidebarWidth));localStorage.setItem('vault.layout.previewVisible',String(previewVisible));localStorage.setItem('vault.layout.detailsVisible',String(detailsVisible));localStorage.setItem('vault.layout.detailsWidth',String(detailsWidth))},[sidebarVisible,sidebarCollapsed,sidebarWidth,previewVisible,detailsVisible,detailsWidth])

  useEffect(() => {
    if (securityStatus.mode==='comfortable' || !vaultUnlocked) return
    const timeout=window.setTimeout(()=>{setSecurityCredential('');setVaultUnlocked(false)},securityStatus.auto_lock_minutes*60_000)
    return ()=>window.clearTimeout(timeout)
  },[securityStatus.mode,securityStatus.auto_lock_minutes,vaultUnlocked])

  useEffect(() => {
    if (securityStatus.mode==='comfortable' || !vaultUnlocked) return
    let hiddenAt:number|null=null
    const lock=()=>{setSecurityCredential('');setVaultUnlocked(false)}
    const onVisibility=()=>{
      if(document.hidden){hiddenAt=Date.now();return}
      // A Windows lock/sleep cycle suspends the webview. Lock immediately when
      // it returns; short task switches remain governed by the inactivity timer.
      if(hiddenAt!==null && Date.now()-hiddenAt>=15_000)lock()
      hiddenAt=null
    }
    const onPageHide=()=>lock()
    document.addEventListener('visibilitychange',onVisibility)
    window.addEventListener('pagehide',onPageHide)
    return()=>{document.removeEventListener('visibilitychange',onVisibility);window.removeEventListener('pagehide',onPageHide)}
  },[securityStatus.mode,vaultUnlocked])

  useEffect(() => {
    invoke<EnvironmentStatus>('environment_status')
      .then(setStatus)
      .catch(() => setStatus(defaultStatus))

    invoke<VaultInitStatus>('initialize_vault')
      .then(setVaultInit)
      .catch(() => setVaultInit(defaultVaultInit))

    invoke<DocumentSummary[]>('list_testlab_documents')
      .then((loadedDocuments) => {
        if (loadedDocuments.length > 0) {
          setTestlabDocuments(loadedDocuments)
          setSelectedIndex(0)
        }
      })
      .catch(() => setTestlabDocuments(fallbackDocuments))

    invoke<DocumentSummary[]>('list_production_documents')
      .then(setProductionDocuments)
      .catch(() => setProductionDocuments([]))

    invoke<DocumentSummary[]>('list_trashed_production_documents')
      .then(setTrashedDocuments)
      .catch(() => setTrashedDocuments([]))

    invoke<BackupSummary[]>('list_local_backups')
      .then(setBackups)
      .catch(() => setBackups([]))
    invoke<BackupPolicy>('get_backup_policy').then(policy=>{setBackupPolicy(policy);setBackupExclusions(policy.excluded_document_ids.join(', '))}).catch(()=>{})

    invoke<AuditEventSummary[]>('list_audit_events')
      .then(setAuditEvents)
      .catch(() => setAuditEvents([]))

    invoke<CodeWordSummary[]>('list_code_words')
      .then(setCodeWords)
      .catch(() => setCodeWords([]))

    invoke<ConflictSummary[]>('list_conflicts')
      .then(setConflicts)
      .catch(() => setConflicts([]))

    invoke<ReminderSummary[]>('list_reminders').then(setReminders).catch(() => setReminders([]))
    invoke<WatchedFolderSummary[]>('list_watched_folders').then(setWatchedFolders).catch(() => setWatchedFolders([]))
    invoke<JobSummary[]>('list_background_jobs').then(setBackgroundJobs).catch(() => setBackgroundJobs([]))

    invoke<ReviewQueueItem[]>('list_review_queue')
      .then(setReviewQueue)
      .catch(() => setReviewQueue([]))

    invoke<EntityCatalogItem[]>('list_entity_catalog')
      .then(setEntityCatalog)
      .catch(() => setEntityCatalog([]))
    invoke<TimelineEventSummary[]>('list_timeline_events').then(setTimelineEvents).catch(() => setTimelineEvents([]))
    invoke<DomainRecordSummary[]>('list_domain_records').then(setDomainRecords).catch(() => setDomainRecords([]))
    invoke<ImportSessionSummary[]>('list_import_history').then(setImportHistory).catch(() => setImportHistory([]))
    invoke<AutomationRuleSummary[]>('list_automation_rules').then(setAutomationRules).catch(() => setAutomationRules([]))
    invoke<DocumentTemplateSummary[]>('list_document_templates').then(setDocumentTemplates).catch(() => setDocumentTemplates([]))
    invoke<CustomFieldSummary[]>('list_custom_fields').then(setCustomFields).catch(() => setCustomFields([]))
    invoke<SavedSearchSummary[]>('list_saved_searches').then(setSavedSearches).catch(() => setSavedSearches([]))
    invoke<SecurityStatus>('security_status').then(result=>{setSecurityStatus(result);setSecurityDraft({mode:result.mode,pin:'',autoLockMinutes:result.auto_lock_minutes,maskSensitive:result.mask_sensitive});setVaultUnlocked(result.mode==='comfortable')}).catch(()=>undefined)
    invoke<WindowsHelloStatus>('windows_hello_status').then(setWindowsHello).catch(error=>setWindowsHello({available:false,explanation:String(error)}))
    invoke<boolean>('quick_unlock_status').then(setQuickUnlockEnabled).catch(()=>setQuickUnlockEnabled(false))
    invoke<number[]>('list_locked_document_ids').then(setLockedDocumentIds).catch(()=>setLockedDocumentIds([]))
    invoke<number[]>('list_private_document_ids').then(setPrivateDocumentIds).catch(()=>setPrivateDocumentIds([]))
    invoke<PluginSummary[]>('list_plugins').then(setPlugins).catch(()=>setPlugins([]))
    invoke<ExternalImportSource[]>('list_external_import_sources').then(setExternalImportSources).catch(()=>setExternalImportSources([]))
    invoke<ThemeProfile[]>('list_theme_profiles').then(setThemeProfiles).catch(()=>setThemeProfiles([]))
    invoke<number[]>('list_favorite_document_ids').then(setFavoriteDocumentIds).catch(()=>setFavoriteDocumentIds([]))
    invoke<CollectionSummary[]>('list_collections').then(setCollections).catch(()=>setCollections([]))
    invoke<AnalysisPreferences>('get_analysis_preferences').then(setAnalysisPreferences).catch(()=>undefined)
    invoke<AnalysisExclusion[]>('list_analysis_exclusions').then(setAnalysisExclusions).catch(()=>setAnalysisExclusions([]))
    invoke<ScannerStatus>('scanner_status').then(setScannerStatus).catch(()=>undefined)
    invoke<OrganizerSummary[]>('list_folders').then(setFolders).catch(()=>setFolders([]))
    invoke<OrganizerSummary[]>('list_categories').then(setCategories).catch(()=>setCategories([]))
    invoke<SearchHistorySummary[]>('list_search_history').then(setSearchHistory).catch(()=>setSearchHistory([]))
    invoke<SearchHistoryPreferences>('get_search_history_preferences').then(setSearchHistoryPreferences).catch(()=>{})
    invoke<VaultNoteSummary[]>('list_vault_notes',{query:'',targetType:null,targetId:null}).then(setVaultNotes).catch(()=>setVaultNotes([]))
    invoke<NoteTemplateSummary[]>('list_note_templates').then(setNoteTemplates).catch(()=>setNoteTemplates([]))
    invoke<CalendarEventSummary[]>('list_calendar_events').then(setCalendarEvents).catch(()=>setCalendarEvents([]))
    invoke<GraphDataSummary>('list_graph_data').then(setGraphData).catch(()=>setGraphData({nodes:[],edges:[]}))
    invoke<number>('refresh_claim_actuality').catch(() => undefined)

    invoke<OcrStatus>('ocr_status')
      .then(setOcrStatus)
      .catch(() => setOcrStatus(defaultOcrStatus))

    invoke<VaultDiagnostics>('vault_diagnostics')
      .then((result) => {
        setDiagnostics(result)
        setDiagnosticsError(null)
      })
      .catch((error) => {
        setDiagnostics(null)
        setDiagnosticsError(error instanceof Error ? error.message : String(error))
      })
  }, [])


  useEffect(() => {
    let unlisten: (() => void) | undefined
    getCurrentWindow().onDragDropEvent((event) => {
      if (event.payload.type === 'enter' || event.payload.type === 'over') setIsDraggingFiles(true)
      if (event.payload.type === 'leave') setIsDraggingFiles(false)
      if (event.payload.type === 'drop') {
        setIsDraggingFiles(false)
        void handleDroppedPaths(event.payload.paths)
      }
    }).then((dispose) => { unlisten = dispose }).catch(() => undefined)
    return () => unlisten?.()
  }, [])

  const hasProductionDocuments = productionDocuments.length > 0
  const activeDocuments = hasProductionDocuments ? productionDocuments : testlabDocuments
  const activeArchiveLabel = hasProductionDocuments ? 'Production' : 'Developer Test Lab'
  const appReadyScore = [
    vaultInit.production_vault_ready,
    vaultInit.testlab_isolated,
    ocrStatus.available,
    diagnostics?.pdf_text_available ?? false,
    diagnostics?.office_text_available ?? false,
    (diagnostics?.issues.length ?? 1) === 0,
  ].filter(Boolean).length

  useEffect(() => {
    const trimmedQuery = query.trim()
    if (!trimmedQuery) {
      setSearchResults([])
      return
    }

    const timeoutId = window.setTimeout(() => {
      const searchCommand = hasProductionDocuments
        ? 'search_production_documents'
        : 'search_testlab_documents'

      invoke<SearchResult[]>(searchCommand, { query: trimmedQuery })
        .then((results) => {
          setSearchResults(results)
          setSelectedIndex(0)
        })
        .catch(() => {
          setSearchResults(localFallbackSearch(activeDocuments, trimmedQuery))
          setSelectedIndex(0)
        })
    }, 320)

    return () => window.clearTimeout(timeoutId)
  }, [activeDocuments, hasProductionDocuments, query])

  useEffect(() => {
    if (query.trim() && activeView === 'start') {
      setActiveView('documents')
    }
  }, [activeView, query])

  const accessibleProductionDocuments=useMemo(()=>guestMode?productionDocuments.filter(document=>!privateDocumentIds.includes(document.id)):productionDocuments,[guestMode,privateDocumentIds,productionDocuments])

  const documentViewDocuments = useMemo(() => {
    if (activeView === 'trash') return trashedDocuments
    if (!hasProductionDocuments) return testlabDocuments
    if (activeView === 'inbox') return accessibleProductionDocuments.filter((document) => document.inbox_status === 'inbox')
    if (activeView === 'archive') return accessibleProductionDocuments.filter((document) => document.inbox_status === 'archived')
    if (activeView === 'favorites') return accessibleProductionDocuments.filter(document=>favoriteDocumentIds.includes(document.id))
    if (activeView === 'recent') return accessibleProductionDocuments.slice(0,20)
    return accessibleProductionDocuments
  }, [accessibleProductionDocuments, activeView, favoriteDocumentIds, hasProductionDocuments, testlabDocuments, trashedDocuments])

  const visibleResults = useMemo(() => {
    if (activeView === 'trash') {
      return trashedDocuments.map((document) => ({ document, score: 0, reasons: ['Dokumentet ligger i papperskorgen'], query_plan: [] }))
    }
    if (searchResults.length > 0 || query.trim()) {
      const allowedIds = new Set(documentViewDocuments.map((document) => document.id))
      return searchResults.filter((result) => allowedIds.has(result.document.id))
    }

    return documentViewDocuments.map((document) => ({
      document,
      score: 0,
      reasons: ['Ingen sökfråga: visar Test Lab-lista i stabil ordning'],
      query_plan: [],
    }))
  }, [activeView, documentViewDocuments, query, searchResults, trashedDocuments])

  const selectedResult = visibleResults[selectedIndex] ?? visibleResults[0]
  const resultWindowSize=100
  const renderedResults=visibleResults.slice(resultWindowStart,resultWindowStart+resultWindowSize)
  useEffect(()=>{setResultWindowStart(0)},[activeView,query])
  const selectedDocument = selectedResult?.document ?? emptyDocument
  const selectedDocumentId = selectedResult?.document.id ?? null
  const previewText = documentDetail?.extracted_text.trim()
  const selectedPage = documentPages.find((page) => page.page_no === selectedPageNo)
  const displayedPreviewText = selectedPage?.text.trim() || previewText
  const canUseStoredFile = hasProductionDocuments && Boolean(documentDetail?.stored_path)
  const canEditDocument = hasProductionDocuments && activeView !== 'trash' && Boolean(selectedResult)
  const canRunOcr =
    hasProductionDocuments &&
    ocrStatus.available &&
    ['image/png', 'image/jpeg', 'application/pdf'].includes(documentDetail?.mime_type ?? '')
  const selectedOcrJob = backgroundJobs.find(
    (job) => job.job_type === 'document_ocr' && job.target_id === selectedDocumentId && job.status !== 'completed' && job.status !== 'failed',
  )
  const latestCompletedOcrJob = backgroundJobs.find((job) => job.job_type === 'document_ocr' && job.status === 'completed')

  useEffect(() => {
    if (!selectedDocumentId || !hasProductionDocuments || latestCompletedOcrJob?.target_id !== selectedDocumentId) return
    invoke<DocumentDetail>('get_production_document_detail', { documentId: selectedDocumentId }).then(setDocumentDetail)
    invoke<DocumentPageSummary[]>('list_document_pages', { documentId: selectedDocumentId }).then(setDocumentPages)
  }, [hasProductionDocuments, latestCompletedOcrJob?.id, latestCompletedOcrJob?.target_id, selectedDocumentId])

  useEffect(() => {
    const hasActiveJobs = backgroundJobs.some((job) => ['queued', 'running'].includes(job.status))
    if (!hasActiveJobs) return
    const interval = window.setInterval(() => {
      invoke<JobSummary[]>('list_background_jobs')
        .then((jobs) => {
          setBackgroundJobs(jobs)
          if (!jobs.some((job) => ['queued', 'running'].includes(job.status))) {
            refreshOperationalData()
            if (selectedDocumentId && hasProductionDocuments) {
              invoke<DocumentDetail>('get_production_document_detail', { documentId: selectedDocumentId }).then(setDocumentDetail)
              invoke<DocumentPageSummary[]>('list_document_pages', { documentId: selectedDocumentId }).then(setDocumentPages)
            }
          }
        })
        .catch(() => undefined)
    }, 2000)
    return () => window.clearInterval(interval)
  }, [backgroundJobs, hasProductionDocuments, selectedDocumentId])

  useEffect(() => {
    if (!selectedDocumentId || activeView === 'trash') {
      setDocumentDetail(null)
      setVersions([])
      setDocumentPages([])
      setAnnotations([])
      setVersionComparison(null)
      setVersionSelection([])
      return
    }

    const detailCommand = hasProductionDocuments
      ? 'get_production_document_detail'
      : 'get_testlab_document_detail'

    invoke<DocumentDetail>(detailCommand, { documentId: selectedDocumentId })
      .then(setDocumentDetail)
      .catch(() => setDocumentDetail(null))
    if (hasProductionDocuments) {
      invoke<DocumentVersionSummary[]>('list_document_versions', { documentId: selectedDocumentId })
        .then(setVersions)
        .catch(() => setVersions([]))
      invoke<DocumentPageSummary[]>('list_document_pages', { documentId: selectedDocumentId })
        .then((pages) => { setDocumentPages(pages); setSelectedPageNo(pages[0]?.page_no ?? 1) })
        .catch(() => setDocumentPages([]))
      invoke<DocumentAnnotationSummary[]>('list_document_annotations',{documentId:selectedDocumentId}).then(setAnnotations).catch(()=>setAnnotations([]))
      invoke<number[]>('list_document_organizer_ids',{kind:'folder',documentId:selectedDocumentId}).then(setDocumentFolderIds).catch(()=>setDocumentFolderIds([]))
      invoke<number[]>('list_document_organizer_ids',{kind:'category',documentId:selectedDocumentId}).then(setDocumentCategoryIds).catch(()=>setDocumentCategoryIds([]))
      invoke<DocumentNoteSummary[]>('list_document_notes',{documentId:selectedDocumentId}).then(setDocumentNotes).catch(()=>setDocumentNotes([]))
    } else {
      setVersions([])
      setDocumentPages([])
    }
  }, [activeView, hasProductionDocuments, selectedDocumentId])

  useEffect(() => {
    const document = documentDetail?.document ?? selectedDocument
    setMetadataDraft({
      title: document.title,
      document_type: document.document_type,
      document_date: document.document_date ?? '',
      inbox_status: document.inbox_status,
      source_label: document.source_label,
      match_explanation: document.match_explanation,
    })
    setMetadataError(null)
    setTagInput('')
  }, [documentDetail, selectedDocument])

  async function handleImport() {
    setImportError(null)
    setFileActionError(null)
    setIsImporting(true)

    try {
      const selectedPath = await open({
        multiple: false,
        title: 'Importera dokument till Vault',
        filters: [
          {
            name: 'Dokument',
            extensions: [
              'pdf',
              'docx',
              'xlsx',
              'pptx',
              'txt',
              'md',
              'png',
              'jpg',
              'jpeg',
              'csv',
              'json',
              'eml',
            ],
          },
        ],
      })

      if (typeof selectedPath !== 'string') {
        return
      }

      const result = await invoke<ImportResult>('import_document', { sourcePath: selectedPath })
      setLastImport(result)
      setProductionDocuments((currentDocuments) => [result.document, ...currentDocuments])
      setSelectedIndex(0)
      setActiveView('documents')
      refreshOperationalData()
    } catch (error) {
      setImportError(error instanceof Error ? error.message : String(error))
    } finally {
      setIsImporting(false)
    }
  }

  async function handleImportDirectory() {
    setImportError(null)
    setFileActionError(null)
    setIsImportingDirectory(true)

    try {
      const selectedPath = await open({
        directory: true,
        multiple: false,
        title: 'Importera mapp till Vault',
      })

      if (typeof selectedPath !== 'string') {
        return
      }

      const result = await invoke<DirectoryImportResult>('import_directory', {
        sourceDirectory: selectedPath,
      })
      setLastDirectoryImport(result)
      setLastImport(result.results[0] ?? null)
      const documents = await invoke<DocumentSummary[]>('list_production_documents')
      setProductionDocuments(documents)
      setSelectedIndex(0)
      setActiveView('start')
      refreshOperationalData()
    } catch (error) {
      setImportError(error instanceof Error ? error.message : String(error))
    } finally {
      setIsImportingDirectory(false)
    }
  }

  async function applyBatchImportResult(result: DirectoryImportResult, nextView: ActiveView = 'importhistory') {
    setLastDirectoryImport(result)
    setLastImport(result.results[0] ?? null)
    setProductionDocuments(await invoke<DocumentSummary[]>('list_production_documents'))
    setSelectedIndex(0)
    setActiveView(nextView)
    await refreshOperationalData()
  }

  async function handleImportArchive() {
    setImportError(null)
    setIsImportingDirectory(true)
    try {
      const selectedPath = await open({ multiple: false, title: 'Importera ZIP-arkiv', filters: [{ name: 'ZIP-arkiv', extensions: ['zip'] }] })
      if (typeof selectedPath !== 'string') return
      await applyBatchImportResult(await invoke<DirectoryImportResult>('import_archive', { sourcePath: selectedPath }))
    } catch (error) {
      setImportError(error instanceof Error ? error.message : String(error))
    } finally {
      setIsImportingDirectory(false)
    }
  }

  async function handleClipboardRead() {
    setImportError(null)
    try {
      const text = await navigator.clipboard.readText()
      setClipboardDraft((draft) => ({ ...draft, text }))
    } catch {
      setClipboardDraft((draft) => ({ ...draft, text: '' }))
    }
    setShowClipboardImport(true)
    setActiveView('importhistory')
  }

  async function handleClipboardImport() {
    if (!clipboardDraft.text.trim()) return
    setIsImporting(true)
    setImportError(null)
    try {
      const result = await invoke<ImportResult>('import_text_document', clipboardDraft)
      setLastImport(result)
      setClipboardDraft({ title: 'Urklipp', text: '' })
      setShowClipboardImport(false)
      setProductionDocuments(await invoke<DocumentSummary[]>('list_production_documents'))
      await refreshOperationalData()
    } catch (error) {
      setImportError(error instanceof Error ? error.message : String(error))
    } finally {
      setIsImporting(false)
    }
  }

  async function handleDroppedPaths(paths: string[]) {
    if (!paths.length) return
    setIsImportingDirectory(true)
    setImportError(null)
    try {
      await applyBatchImportResult(await invoke<DirectoryImportResult>('import_paths', { paths }))
    } catch (error) {
      setImportError(error instanceof Error ? error.message : String(error))
    } finally {
      setIsImportingDirectory(false)
    }
  }

  async function handleCreateDemoArchive() {
    setImportError(null)
    setIsCreatingDemoArchive(true)

    try {
      const result = await invoke<DirectoryImportResult>('create_demo_archive')
      setLastDirectoryImport(result)
      setLastImport(result.results[0] ?? null)
      const documents = await invoke<DocumentSummary[]>('list_production_documents')
      setProductionDocuments(documents)
      setSelectedIndex(0)
      setActiveView('start')
      refreshOperationalData()
    } catch (error) {
      setImportError(error instanceof Error ? error.message : String(error))
    } finally {
      setIsCreatingDemoArchive(false)
    }
  }

  async function handleOpenStoredFile() {
    if (!selectedResult || !canUseStoredFile) {
      return
    }

    setFileActionError(null)

    try {
      if(privateDocumentIds.includes(selectedResult.document.id)){await invoke('open_private_document_file',{documentId:selectedResult.document.id,pin:securityCredential});return}
      const locked=lockedDocumentIds.includes(selectedResult.document.id)||Boolean(securityCredential)
      await invoke(locked?'open_protected_document_file':'open_production_document_file', { documentId: selectedResult.document.id, ...(locked?{pin:securityCredential}:{}) })
    } catch (error) {
      setFileActionError(error instanceof Error ? error.message : String(error))
    }
  }

  async function handleRevealStoredFile() {
    if (!selectedResult || !canUseStoredFile) {
      return
    }

    setFileActionError(null)

    try {
      if(privateDocumentIds.includes(selectedResult.document.id)){throw new Error('Privata original visas inte i Utforskaren. Använd Öppna privat original för en tillfällig dekrypterad förhandsvisning.')}
      const locked=lockedDocumentIds.includes(selectedResult.document.id)||Boolean(securityCredential)
      await invoke(locked?'reveal_protected_document_file':'reveal_production_document_file', { documentId: selectedResult.document.id, ...(locked?{pin:securityCredential}:{}) })
    } catch (error) {
      setFileActionError(error instanceof Error ? error.message : String(error))
    }
  }

  async function handleTrashSelectedDocument() {
    if (!selectedResult || !hasProductionDocuments) {
      return
    }

    setFileActionError(null)
    setIsTrashingDocument(true)

    try {
      const documents = await invoke<DocumentSummary[]>('trash_production_document', {
        documentId: selectedResult.document.id,
      })
      setProductionDocuments(documents)
      setDocumentDetail(null)
      setSelectedIndex(0)
      refreshOperationalData()
    } catch (error) {
      setFileActionError(error instanceof Error ? error.message : String(error))
    } finally {
      setIsTrashingDocument(false)
    }
  }

  async function handleResetTestlab() {
    setFileActionError(null)
    setIsResettingTestlab(true)

    try {
      const resetDocuments = await invoke<DocumentSummary[]>('reset_testlab')
      setTestlabDocuments(resetDocuments)
      if (!hasProductionDocuments) {
        setSearchResults([])
        setSelectedIndex(0)
      }
    } catch (error) {
      setFileActionError(error instanceof Error ? error.message : String(error))
    } finally {
      setIsResettingTestlab(false)
    }
  }

  async function handleCreateBackup() {
    setBackupError(null)
    setIsCreatingBackup(true)

    try {
      const backup = await invoke<BackupResult>('create_local_backup')
      setLastBackup(backup)
      setBackups((currentBackups) => [backup, ...currentBackups])
      refreshOperationalData()
      invoke<AuditEventSummary[]>('list_audit_events')
        .then(setAuditEvents)
        .catch(() => undefined)
    } catch (error) {
      setBackupError(error instanceof Error ? error.message : String(error))
    } finally {
      setIsCreatingBackup(false)
    }
  }

  async function handleSaveBackupPolicy(){try{const ids=backupExclusions.split(',').map(value=>Number(value.trim())).filter(value=>Number.isInteger(value)&&value>0);setBackupPolicy(await invoke('save_backup_policy',{enabled:backupPolicy.enabled,paused:backupPolicy.paused,intervalHours:backupPolicy.interval_hours,backupMode:backupPolicy.backup_mode,excludedDocumentIds:ids,destinationPath:backupPolicy.destination_path}));setBackupError(null)}catch(error){setBackupError(String(error))}}
  async function handleRunConfiguredBackup(){setIsCreatingBackup(true);try{const backup=await invoke<BackupResult>('run_configured_backup');setLastBackup(backup);setBackups(await invoke('list_local_backups'));setBackupPolicy(await invoke('get_backup_policy'));setBackupError(null)}catch(error){setBackupError(String(error))}finally{setIsCreatingBackup(false)}}

  async function handleCreatePasswordBackup() {
    setBackupError(null)
    try { const result=await invoke<ProtectedBackupResult>('create_password_backup',{password:backupPassword,destinationPath:backupPolicy.destination_path});setProtectedBackup(result);setBackupPassword('') } catch(error){setBackupError(error instanceof Error?error.message:String(error))}
  }

  async function handleChooseBackupDestination(){const selected=await open({directory:true,multiple:false,title:'Välj lokal disk, extern disk eller nätverksmapp'});if(typeof selected==='string')setBackupPolicy({...backupPolicy,destination_path:selected})}
  async function handleTestLatestRestore(){if(!backups[0])return;setRestoreError(null);try{setRestoreTest(await invoke('test_backup_restore',{backupRoot:backups[0].backup_root}))}catch(error){setRestoreError(String(error))}}
  async function handleChoosePasswordBackup(){const selected=await open({multiple:false,filters:[{name:'Krypterad Vault-backup',extensions:['vaultzip']}],title:'Välj lösenordsskyddad backup'});if(typeof selected==='string')setPasswordRestorePath(selected)}
  async function handleRestorePasswordBackup(){if(!passwordRestorePath||backupPassword.length<8)return;if(!window.confirm('Vault skapar först en säkerhetskopia av nuvarande arkiv och återställer sedan den krypterade backupen. Fortsätta?'))return;setRestoreError(null);try{setLastRestore(await invoke('restore_password_backup',{archivePath:passwordRestorePath,password:backupPassword}));setBackupPassword('');setProductionDocuments(await invoke('list_production_documents'));setBackups(await invoke('list_local_backups'));setHealthReport(await invoke('check_vault_health'))}catch(error){setRestoreError(String(error))}}

  async function handleCreatePortableArchive(){setBackupError(null);try{setPortableArchive(await invoke('create_portable_archive'))}catch(error){setBackupError(String(error))}}
  async function handleCreateDataExport(){setBackupError(null);try{setDataExport(await invoke('create_data_export',{format:dataExportDraft.format,includeOriginals:dataExportDraft.includeOriginals,folderLayout:dataExportDraft.folderLayout}))}catch(error){setBackupError(String(error))}}
  async function handleExportUserProfile(){try{const path=await invoke<string>('export_user_profile',{dashboardLayoutJson:JSON.stringify({name:activeDashboardLayout,widgets:dashboardWidgets,sizes:dashboardSizes,layouts:dashboardLayouts})});setBackupError(`Profil exporterad: ${path}`)}catch(error){setBackupError(String(error))}}
  async function handleImportUserProfile(){const selected=await open({multiple:false,filters:[{name:'Vault-profil',extensions:['json']}],title:'Importera Vault-profil'});if(typeof selected!=='string')return;try{const result=await invoke<ProfileImportResult>('import_user_profile',{path:selected});const layout=JSON.parse(result.dashboard_layout_json);if(Array.isArray(layout.widgets))setDashboardWidgets(layout.widgets);if(layout.sizes&&typeof layout.sizes==='object')setDashboardSizes(layout.sizes);if(Array.isArray(layout.layouts))setDashboardLayouts(layout.layouts);if(typeof layout.name==='string')setActiveDashboardLayout(layout.name);setSavedSearches(await invoke('list_saved_searches'));setThemeProfiles(await invoke('list_theme_profiles'));setCodeWords(await invoke('list_code_words'));setAutomationRules(await invoke('list_automation_rules'));setBackupError(`Profil importerad: ${result.saved_searches} sökningar, ${result.themes} teman, ${result.code_words} kodord och ${result.rules} regler.`)}catch(error){setBackupError(String(error))}}

  async function handleRestorePortableArchive(){
    const selected=await open({multiple:false,filters:[{name:'Vault portabelt arkiv',extensions:['vaultarchive']}]})
    if(typeof selected!=='string')return
    if(!window.confirm('Det aktuella arkivet säkerhetskopieras först. Vill du sedan ersätta det med det valda portabla arkivet?'))return
    setRestoreError(null)
    try{const result=await invoke<RestoreResult>('restore_portable_archive',{archivePath:selected});setLastRestore(result);setProductionDocuments(await invoke('list_production_documents'));setBackups(await invoke('list_local_backups'));setHealthReport(await invoke('check_vault_health'))}catch(error){setRestoreError(String(error))}
  }

  async function handleIntegrityScan(){setHealthError(null);try{setIntegrityReport(await invoke('scan_file_integrity'))}catch(error){setHealthError(String(error))}}

  async function handleDuplicateDecision(candidate:DuplicateCandidate,decision:string){try{await invoke('decide_duplicate',{primaryDocumentId:candidate.primary_document_id,secondaryDocumentId:candidate.secondary_document_id,decision,note:''});setIntegrityReport(await invoke('scan_file_integrity'))}catch(error){setHealthError(String(error))}}

  async function handleInstallPlugin(){const path=await open({multiple:false,title:'Välj lokalt Vault-pluginmanifest',filters:[{name:'Vault plugin JSON',extensions:['json']}]});if(typeof path!=='string')return;setPluginError(null);try{setPlugins(await invoke('install_plugin',{manifestPath:path}))}catch(error){setPluginError(String(error))}}
  async function handleSetPluginEnabled(plugin:PluginSummary){setPluginError(null);try{setPlugins(await invoke('set_plugin_enabled',{id:plugin.id,enabled:!plugin.enabled}))}catch(error){setPluginError(String(error))}}
  async function handleRunPlugin(plugin:PluginSummary){setPluginError(null);try{setPluginRun(await invoke('run_plugin',{id:plugin.id}));setPlugins(await invoke('list_plugins'));setProductionDocuments(await invoke('list_production_documents'))}catch(error){setPluginError(String(error))}}
  async function handleRemovePlugin(plugin:PluginSummary){if(!window.confirm(`Ta bort plugin ${plugin.name}? Dokumentändringar som redan godkänts behålls.`))return;setPluginError(null);try{setPlugins(await invoke('remove_plugin',{id:plugin.id}))}catch(error){setPluginError(String(error))}}
  async function handleConfigureExternalImport(source:ExternalImportSource){const selected=await open({directory:true,multiple:false,title:`Välj lokal export-/synkmapp för ${source.display_name}`});if(typeof selected!=='string')return;setPluginError(null);try{setExternalImportSources(await invoke('configure_external_import_source',{provider:source.provider,localStagingPath:selected,enabled:true}));setExternalImportPreview(null);setExternalImportSelection([])}catch(error){setPluginError(String(error))}}
  async function handlePreviewExternalImport(source:ExternalImportSource){setPluginError(null);try{const preview=await invoke<ExternalImportPreview>('preview_external_import',{provider:source.provider});setExternalImportPreview(preview);setExternalImportSelection(preview.files.map(file=>file.path))}catch(error){setPluginError(String(error))}}
  async function handleImportExternalSelection(){if(!externalImportPreview||!externalImportSelection.length)return;if(!window.confirm(`Importera ${externalImportSelection.length} exakt valda filer från ${externalImportPreview.provider}?`))return;setPluginError(null);try{const result=await invoke<DirectoryImportResult>('import_external_selection',{provider:externalImportPreview.provider,paths:externalImportSelection});setPluginError(`Import klar: ${result.imported_count} importerade, ${result.duplicate_count} dubletter, ${result.failed_count} fel.`);setProductionDocuments(await invoke('list_production_documents'));setExternalImportSources(await invoke('list_external_import_sources'))}catch(error){setPluginError(String(error))}}

  async function handleCheckHealth() {
    setHealthError(null)
    setIsCheckingHealth(true)

    try {
      const report = await invoke<VaultHealthReport>('check_vault_health')
      setHealthReport(report)
      refreshOperationalData()
    } catch (error) {
      setHealthError(error instanceof Error ? error.message : String(error))
    } finally {
      setIsCheckingHealth(false)
    }
  }

  async function handleReindexSearch() {
    setReindexError(null)
    setIsReindexing(true)

    try {
      const result = await invoke<ReindexResult>('rebuild_production_search_index')
      setLastReindex(result)
      const report = await invoke<VaultHealthReport>('check_vault_health')
      setHealthReport(report)
      refreshOperationalData()
      invoke<AuditEventSummary[]>('list_audit_events')
        .then(setAuditEvents)
        .catch(() => undefined)
    } catch (error) {
      setReindexError(error instanceof Error ? error.message : String(error))
    } finally {
      setIsReindexing(false)
    }
  }

  async function handleValidateLatestBackup() {
    if (!backups.length) {
      setRestoreError('Ingen backup finns att validera')
      return
    }

    setRestoreError(null)
    setIsValidatingBackup(true)

    try {
      const report = await invoke<BackupValidationReport>('validate_local_backup', {
        backupRoot: backups[0].backup_root,
      })
      setBackupValidation(report)
    } catch (error) {
      setRestoreError(error instanceof Error ? error.message : String(error))
    } finally {
      setIsValidatingBackup(false)
    }
  }

  async function handleRestoreLatestBackup() {
    if (!backups.length) {
      setRestoreError('Ingen backup finns att återställa')
      return
    }
    if (!backupValidation?.ok || !restoreTest?.ok) {
      setRestoreError('Validera backupen och kör isolerat återställningsprov först')
      return
    }
    if (!window.confirm(`Full återställning ersätter aktivt arkiv med ${backups[0].document_count} dokument. Vault skapar först en separat återställningspunkt. Fortsätta?`)) return

    setRestoreError(null)
    setIsRestoringBackup(true)

    try {
      const result = await invoke<RestoreResult>('restore_local_backup', {
        backupRoot: backups[0].backup_root,
      })
      setLastRestore(result)
      const documents = await invoke<DocumentSummary[]>('list_production_documents')
      setProductionDocuments(documents)
      const report = await invoke<VaultHealthReport>('check_vault_health')
      setHealthReport(report)
      const refreshedBackups = await invoke<BackupSummary[]>('list_local_backups')
      setBackups(refreshedBackups)
      invoke<AuditEventSummary[]>('list_audit_events')
        .then(setAuditEvents)
        .catch(() => undefined)
    } catch (error) {
      setRestoreError(error instanceof Error ? error.message : String(error))
    } finally {
      setIsRestoringBackup(false)
    }
  }

  async function handleRunTestCenter() {
    setTestCenterError(null)
    setIsRunningTestCenter(true)

    try {
      const report = await invoke<TestCenterReport>('run_test_center')
      setTestCenterReport(report)
      refreshOperationalData()
    } catch (error) {
      setTestCenterError(error instanceof Error ? error.message : String(error))
    } finally {
      setIsRunningTestCenter(false)
    }
  }
  async function handleRunTestModule(){setIsRunningTestCenter(true);setTestCenterError(null);try{setTestCenterReport(await invoke('run_test_center_module',{module:testModule}))}catch(e){setTestCenterError(String(e))}finally{setIsRunningTestCenter(false)}}
  async function handleGenerateTestScale(count:number){setIsRunningTestCenter(true);setTestCenterError(null);try{setTestLabScale(await invoke('generate_testlab_scale',{count}));setTestlabDocuments(await invoke('list_testlab_documents'))}catch(e){setTestCenterError(String(e))}finally{setIsRunningTestCenter(false)}}
  async function handleSimulateFailure(failureType:string){try{const result=await invoke<TestCenterCase>('simulate_testlab_failure',{failureType});setSimulatedCases(items=>[result,...items.filter(item=>item.name!==result.name)])}catch(e){setTestCenterError(String(e))}}
  async function handleExportTestReport(){try{setTestReportExport(await invoke('export_safe_test_report'))}catch(e){setTestCenterError(String(e))}}

  async function refreshOperationalData() {
    const [documents, trashed, conflictsResult, reviewItems, entities, events, diagnosticsResult, reminderResult, folderResult, jobsResult, timelineResult, domainsResult, importsResult] =
      await Promise.allSettled([
        invoke<DocumentSummary[]>('list_production_documents'),
        invoke<DocumentSummary[]>('list_trashed_production_documents'),
        invoke<ConflictSummary[]>('list_conflicts'),
        invoke<ReviewQueueItem[]>('list_review_queue'),
        invoke<EntityCatalogItem[]>('list_entity_catalog'),
        invoke<AuditEventSummary[]>('list_audit_events'),
        invoke<VaultDiagnostics>('vault_diagnostics'),
        invoke<ReminderSummary[]>('list_reminders'),
        invoke<WatchedFolderSummary[]>('list_watched_folders'),
        invoke<JobSummary[]>('list_background_jobs'),
        invoke<TimelineEventSummary[]>('list_timeline_events'),
        invoke<DomainRecordSummary[]>('list_domain_records'),
        invoke<ImportSessionSummary[]>('list_import_history'),
      ])

    if (documents.status === 'fulfilled') setProductionDocuments(documents.value)
    if (trashed.status === 'fulfilled') setTrashedDocuments(trashed.value)
    if (conflictsResult.status === 'fulfilled') setConflicts(conflictsResult.value)
    if (reviewItems.status === 'fulfilled') setReviewQueue(reviewItems.value)
    if (entities.status === 'fulfilled') setEntityCatalog(entities.value)
    if (timelineResult.status === 'fulfilled') setTimelineEvents(timelineResult.value)
    if (domainsResult.status === 'fulfilled') setDomainRecords(domainsResult.value)
    if (importsResult.status === 'fulfilled') setImportHistory(importsResult.value)
    if (events.status === 'fulfilled') setAuditEvents(events.value)
    if (diagnosticsResult.status === 'fulfilled') {
      setDiagnostics(diagnosticsResult.value)
      setDiagnosticsError(null)
    } else {
      setDiagnosticsError(String(diagnosticsResult.reason))
    }
    if (reminderResult.status === 'fulfilled') setReminders(reminderResult.value)
    if (folderResult.status === 'fulfilled') setWatchedFolders(folderResult.value)
    if (jobsResult.status === 'fulfilled') setBackgroundJobs(jobsResult.value)
  }

  async function handleAnalyzeArchive() {
    setAnalysisError(null)
    setIsAnalyzingArchive(true)

    try {
      const result = await invoke<ArchiveAnalysisResult>('analyze_production_archive')
      setLastAnalysis(result)
      await refreshOperationalData()
    } catch (error) {
      setAnalysisError(error instanceof Error ? error.message : String(error))
    } finally {
      setIsAnalyzingArchive(false)
    }
  }

  async function handleCheckOcrStatus() {
    setOcrError(null)
    setIsCheckingOcr(true)

    try {
      const status = await invoke<OcrStatus>('ocr_status')
      setOcrStatus(status)
      refreshOperationalData()
    } catch (error) {
      setOcrError(error instanceof Error ? error.message : String(error))
    } finally {
      setIsCheckingOcr(false)
    }
  }

  async function handleRunOcrForSelectedDocument() {
    if (!selectedResult || !canRunOcr) {
      return
    }

    setOcrError(null)
    try {
      const jobs = await invoke<JobSummary[]>('queue_document_ocr', {
        documentId: selectedResult.document.id,
      })
      setBackgroundJobs(jobs)
    } catch (error) {
      setOcrError(error instanceof Error ? error.message : String(error))
    }
  }

  async function handleToggleOcrPause() {
    if (!selectedOcrJob) return
    try {
      setBackgroundJobs(await invoke<JobSummary[]>('set_ocr_job_paused', {
        jobId: selectedOcrJob.id,
        paused: selectedOcrJob.status !== 'paused',
      }))
    } catch (error) {
      setOcrError(error instanceof Error ? error.message : String(error))
    }
  }

  async function refreshReviewData() {
    refreshOperationalData()
  }

  async function handleSaveMetadata() {
    if (!hasProductionDocuments || !selectedResult) {
      setMetadataError('Metadata kan bara sparas i produktionsarkivet')
      return
    }

    setMetadataError(null)
    setIsSavingMetadata(true)

    try {
      const detail = await invoke<DocumentDetail>('update_production_document_metadata', {
        documentId: selectedResult.document.id,
        title: metadataDraft.title,
        documentType: metadataDraft.document_type,
        documentDate: metadataDraft.document_date.trim() || null,
        inboxStatus: metadataDraft.inbox_status,
        sourceLabel: metadataDraft.source_label,
        matchExplanation: metadataDraft.match_explanation,
      })
      setDocumentDetail(detail)
      setProductionDocuments((currentDocuments) =>
        currentDocuments.map((document) =>
          document.id === detail.document.id ? detail.document : document,
        ),
      )
      refreshReviewData()
    } catch (error) {
      setMetadataError(error instanceof Error ? error.message : String(error))
    } finally {
      setIsSavingMetadata(false)
    }
  }

  async function handleSetClaimStatus(claimId: number, status: string) {
    if (!hasProductionDocuments) {
      return
    }

    setMetadataError(null)

    try {
      const detail = await invoke<DocumentDetail>('set_claim_review_status', {
        claimId,
        status,
      })
      setDocumentDetail(detail)
      refreshReviewData()
    } catch (error) {
      setMetadataError(error instanceof Error ? error.message : String(error))
    }
  }

  async function handleSetClaimActuality(claimId: number, actualityStatus: string) {
    try {
      const detail = await invoke<DocumentDetail>('set_claim_actuality', { claimId, actualityStatus })
      setDocumentDetail(detail)
    } catch (error) {
      setMetadataError(error instanceof Error ? error.message : String(error))
    }
  }

  async function handleAddTag() {
    if (!hasProductionDocuments || !selectedResult || !tagInput.trim()) {
      return
    }

    setMetadataError(null)
    setIsSavingTag(true)

    try {
      const detail = await invoke<DocumentDetail>('add_production_document_tag', {
        documentId: selectedResult.document.id,
        tagName: tagInput,
      })
      setDocumentDetail(detail)
      setTagInput('')
      refreshReviewData()
    } catch (error) {
      setMetadataError(error instanceof Error ? error.message : String(error))
    } finally {
      setIsSavingTag(false)
    }
  }

  async function handleRemoveTag(tagName: string) {
    if (!hasProductionDocuments || !selectedResult) {
      return
    }

    setMetadataError(null)

    try {
      const detail = await invoke<DocumentDetail>('remove_production_document_tag', {
        documentId: selectedResult.document.id,
        tagName,
      })
      setDocumentDetail(detail)
      refreshReviewData()
    } catch (error) {
      setMetadataError(error instanceof Error ? error.message : String(error))
    }
  }

  async function handleAddCodeWord() {
    if (!codeWordInput.trim()) {
      return
    }

    setMetadataError(null)
    setIsSavingCodeWord(true)

    try {
      const savedCodeWords = await invoke<CodeWordSummary[]>('add_code_word', {
        word: codeWordInput,
        description: codeWordDescription,
      })
      setCodeWords(savedCodeWords)
      setCodeWordInput('')
      setCodeWordDescription('')
      refreshReviewData()
    } catch (error) {
      setMetadataError(error instanceof Error ? error.message : String(error))
    } finally {
      setIsSavingCodeWord(false)
    }
  }

  async function handleRestoreSelectedDocument() {
    if (!selectedResult || activeView !== 'trash') return
    setFileActionError(null)
    try {
      const documents = await invoke<DocumentSummary[]>('restore_production_document', {
        documentId: selectedResult.document.id,
      })
      setProductionDocuments(documents)
      setTrashedDocuments((current) => current.filter((document) => document.id !== selectedResult.document.id))
      setSelectedIndex(0)
      await refreshOperationalData()
    } catch (error) {
      setFileActionError(error instanceof Error ? error.message : String(error))
    }
  }

  async function handleAddVersion() {
    if (!canEditDocument || !selectedResult) return
    setWorkflowError(null)
    setIsAddingVersion(true)
    try {
      const selectedPath = await open({ multiple: false, title: 'Välj fil för ny dokumentversion' })
      if (typeof selectedPath !== 'string') return
      const items = await invoke<DocumentVersionSummary[]>('add_document_version', {
        documentId: selectedResult.document.id,
        sourcePath: selectedPath,
        comment: versionComment,
      })
      setVersions(items)
      setVersionComment('')
      const detail = await invoke<DocumentDetail>('get_production_document_detail', { documentId: selectedResult.document.id })
      setDocumentDetail(detail)
      await refreshOperationalData()
    } catch (error) {
      setWorkflowError(error instanceof Error ? error.message : String(error))
    } finally {
      setIsAddingVersion(false)
    }
  }

  async function handleSaveAnnotation(){
    if(!selectedDocumentId)return
    try{setAnnotations(await invoke('save_document_annotation',{documentId:selectedDocumentId,annotationId:null,pageNo:selectedPageNo,annotationType:annotationDraft.type,selectedText:annotationDraft.selectedText,body:annotationDraft.body,color:annotationDraft.color}));setAnnotationDraft({...annotationDraft,selectedText:'',body:''});setWorkflowError(null)}catch(error){setWorkflowError(String(error))}
  }
  async function handleDeleteAnnotation(id:number){if(!selectedDocumentId)return;try{setAnnotations(await invoke('delete_document_annotation',{documentId:selectedDocumentId,annotationId:id}))}catch(error){setWorkflowError(String(error))}}
  function captureViewerSelection(){const text=window.getSelection()?.toString().trim()||'';setAnnotationDraft(current=>({...current,selectedText:text}));if(text)setWorkflowError(null)}
  function jumpToViewerMatch(direction:1|-1){if(!viewerSearch.trim()||!documentPages.length)return;const start=Math.max(0,documentPages.findIndex(p=>p.page_no===selectedPageNo));for(let offset=1;offset<=documentPages.length;offset++){const index=(start+direction*offset+documentPages.length)%documentPages.length;if(documentPages[index].text.toLocaleLowerCase('sv-SE').includes(viewerSearch.toLocaleLowerCase('sv-SE'))){setSelectedPageNo(documentPages[index].page_no);return}}setWorkflowError(`Ingen träff för “${viewerSearch}” i dokumentet`)}
  async function handleCompareVersions(){if(versionSelection.length!==2||!selectedDocumentId)return;try{setVersionComparison(await invoke('compare_document_versions',{documentId:selectedDocumentId,leftVersionId:versionSelection[0],rightVersionId:versionSelection[1]}));setWorkflowError(null)}catch(error){setWorkflowError(String(error))}}
  async function handleRestoreVersion(versionId:number){if(!selectedDocumentId||!window.confirm('Gör denna version aktuell? Nuvarande version sparas i historiken.'))return;try{setVersions(await invoke('restore_document_version',{documentId:selectedDocumentId,versionId}));setDocumentDetail(await invoke('get_production_document_detail',{documentId:selectedDocumentId}));setDocumentPages(await invoke('list_document_pages',{documentId:selectedDocumentId}));await refreshOperationalData()}catch(error){setWorkflowError(String(error))}}

  async function handleResolveConflict(conflictId: number, decision: 'source_a' | 'source_b' | 'both' | 'neither' | 'unresolved') {
    setWorkflowError(null)
    try {
      const items = await invoke<ConflictSummary[]>('resolve_conflict', {
        conflictId,
        decision,
        note: `Användaren valde ${decision} efter manuell källgranskning`,
        locked: decision !== 'unresolved',
      })
      setConflicts(items)
    } catch (error) {
      setWorkflowError(error instanceof Error ? error.message : String(error))
    }
  }

  async function handleSaveReminder() {
    if (!reminderDraft.title.trim() || !reminderDraft.due_date) return
    setWorkflowError(null)
    try {
      const items = await invoke<ReminderSummary[]>('save_reminder', {
        documentId: selectedResult?.document.id ?? null,
        title: reminderDraft.title,
        dueDate: reminderDraft.due_date,
        note: reminderDraft.note,
      })
      setReminders(items)
      setReminderDraft({ title: '', due_date: '', note: '' })
    } catch (error) {
      setWorkflowError(error instanceof Error ? error.message : String(error))
    }
  }

  async function handleToggleReminder(reminder: ReminderSummary) {
    const items = await invoke<ReminderSummary[]>('set_reminder_completed', {
      reminderId: reminder.id,
      completed: reminder.status !== 'completed',
    })
    setReminders(items)
  }

  async function handleAddWatchedFolder() {
    const selectedPath = await open({ directory: true, multiple: false, title: 'Välj bevakad mapp' })
    if (typeof selectedPath !== 'string') return
    try {
      setWatchedFolders(await invoke<WatchedFolderSummary[]>('add_watched_folder', { path: selectedPath }))
    } catch (error) {
      setWorkflowError(error instanceof Error ? error.message : String(error))
    }
  }

  async function handleScanWatchedFolders() {
    setIsScanningFolders(true)
    setWorkflowError(null)
    try {
      const result = await invoke<DirectoryImportResult>('scan_watched_folders')
      setLastDirectoryImport(result)
      await refreshOperationalData()
    } catch (error) {
      setWorkflowError(error instanceof Error ? error.message : String(error))
    } finally {
      setIsScanningFolders(false)
    }
  }

  async function handleRunBackgroundJobs() {
    setIsScanningFolders(true)
    try {
      setBackgroundJobs(await invoke<JobSummary[]>('run_background_jobs_now'))
      await refreshOperationalData()
    } catch (error) {
      setWorkflowError(error instanceof Error ? error.message : String(error))
    } finally {
      setIsScanningFolders(false)
    }
  }
  async function handleQueueSystemJob(){try{setBackgroundJobs(await invoke('queue_system_job',{jobType:systemJobType}));setWorkflowError(null)}catch(error){setWorkflowError(String(error))}}
  async function handleControlJob(job:JobSummary,action:string){try{setBackgroundJobs(await invoke('control_background_job',{jobId:job.id,action}));setWorkflowError(null)}catch(error){setWorkflowError(String(error))}}

  async function handleSaveRule() {
    try {const conditions=[{field:ruleDraft.match_field,operator:ruleDraft.match_operator,value:ruleDraft.match_value},...(ruleDraft.second_value.trim()?[{field:ruleDraft.second_field,operator:ruleDraft.second_operator,value:ruleDraft.second_value}]:[])];setAutomationRules(await invoke('save_automation_rule',{id:null,name:ruleDraft.name,enabled:true,priority:100,matchField:ruleDraft.match_field,matchOperator:ruleDraft.match_operator,matchValue:ruleDraft.match_value,actionType:ruleDraft.action_type,actionValue:ruleDraft.action_value,approvalPolicy:ruleDraft.approval_policy,logicOperator:ruleDraft.logic_operator,conditionsJson:JSON.stringify(conditions),actionsJson:JSON.stringify([{type:ruleDraft.action_type,value:ruleDraft.action_value}])})); setWorkflowError(null) } catch(e){setWorkflowError(String(e))}
  }
  async function handleRunRules() {
    try { setRuleRun(await invoke('run_automation_rules')); setProductionDocuments(await invoke('list_production_documents')); setWorkflowError(null) } catch(e){setWorkflowError(String(e))}
  }
  async function handleApplyBatch(){if(!selectedDocumentIds.length)return;try{setProductionDocuments(await invoke('apply_batch_action',{documentIds:selectedDocumentIds,actionType:batchDraft.action,actionValue:batchDraft.value}));setSelectedDocumentIds([]);setBatchMode(false);await refreshOperationalData();setWorkflowError(null)}catch(error){setWorkflowError(String(error))}}
  async function handleSaveTemplate() {
    try { setDocumentTemplates(await invoke('save_document_template',{name:templateDraft.name,documentType:templateDraft.document_type,category:templateDraft.category,defaultTags:templateDraft.default_tags,requiredFields:templateDraft.required_fields})); setTemplateDraft({name:'',document_type:'',category:'',default_tags:'',required_fields:''}); setWorkflowError(null) } catch(e){setWorkflowError(String(e))}
  }
  async function handleSaveCustomField() {
    try { setCustomFields(await invoke('save_custom_field',{name:fieldDraft.name,fieldType:fieldDraft.field_type,appliesTo:fieldDraft.applies_to,required:fieldDraft.required})); setFieldDraft({name:'',field_type:'text',applies_to:'all',required:false}); setWorkflowError(null) } catch(e){setWorkflowError(String(e))}
  }
  async function handleSaveSearch() {
    try { setSavedSearches(await invoke('save_search',searchDraft)); setSearchDraft({name:'',query:'',pinned:true}); setWorkflowError(null) } catch(e){setWorkflowError(String(e))}
  }
  async function handleConfigureSecurity() {
    try { const result=await invoke<SecurityStatus>('configure_security',{mode:securityDraft.mode,pin:securityDraft.pin,autoLockMinutes:securityDraft.autoLockMinutes,maskSensitive:securityDraft.maskSensitive});setSecurityStatus(result);setSecurityCredential(result.mode==='comfortable'?'':securityDraft.pin);setVaultUnlocked(result.mode==='comfortable');setSecurityDraft(d=>({...d,pin:''}));setWorkflowError(null) } catch(e){setWorkflowError(String(e))}
  }
  async function handleUnlockVault() {
    try { const ok=await invoke<boolean>('unlock_vault',{pin:unlockPin});setVaultUnlocked(ok);setWorkflowError(ok?null:'Fel PIN/lösenord');if(ok){setSecurityCredential(unlockPin);setUnlockPin('')}} catch(e){setWorkflowError(String(e))}
  }
  async function handleWindowsHelloUnlock(){try{const ok=await invoke<boolean>('unlock_vault_windows_hello');setVaultUnlocked(ok);setWorkflowError(ok?null:'Windows Hello kunde inte verifiera dig')}catch(e){setWorkflowError(String(e))}}
  async function handleQuickUnlock(){try{const ok=await invoke<boolean>('unlock_vault_quick');setVaultUnlocked(ok);setWorkflowError(ok?null:'Windows-skyddad upplåsning misslyckades')}catch(e){setWorkflowError(String(e))}}
  async function handleToggleQuickUnlock(){try{const enabled=await invoke<boolean>('set_quick_unlock',{pin:securityCredential,enabled:!quickUnlockEnabled});setQuickUnlockEnabled(enabled);setWorkflowError(null)}catch(e){setWorkflowError(String(e))}}
  async function handleToggleDocumentLock() {
    if(!selectedDocumentId)return;try{const locked=lockedDocumentIds.includes(selectedDocumentId);setSecurityStatus(await invoke('set_document_security',{documentId:selectedDocumentId,isLocked:!locked,sensitivity:!locked?'confidential':'normal'}));setLockedDocumentIds(await invoke('list_locked_document_ids'));setWorkflowError(null)}catch(e){setWorkflowError(String(e))}
  }
  async function handleProtectedExport(masked:boolean) {
    if(!selectedDocumentId)return;try{setLastProtectedExport(await invoke('export_document_secure',{documentId:selectedDocumentId,pin:securityCredential,masked}));setWorkflowError(null)}catch(e){setWorkflowError(String(e))}
  }
  async function handleTogglePrivateDocument(){if(!selectedDocumentId)return;if(!securityCredential){setWorkflowError('Lås upp Vault och ange PIN/lösenord innan privat kryptering');return}try{const makePrivate=!privateDocumentIds.includes(selectedDocumentId);setSecurityStatus(await invoke('set_document_private',{documentId:selectedDocumentId,isPrivate:makePrivate,pin:securityCredential}));setPrivateDocumentIds(await invoke('list_private_document_ids'));setLockedDocumentIds(await invoke('list_locked_document_ids'));setWorkflowError(null)}catch(e){setWorkflowError(String(e))}}
  async function handleOpenPrivateDocument(){if(!selectedDocumentId)return;try{await invoke('open_private_document_file',{documentId:selectedDocumentId,pin:securityCredential});setWorkflowError(null)}catch(e){setWorkflowError(String(e))}}
  async function handleActivateTheme(id:number){try{setThemeProfiles(await invoke('activate_theme_profile',{id}));setThemeMessage('Temat är aktivt och sparat lokalt')}catch(e){setThemeMessage(String(e))}}
  async function handleSaveTheme(){try{setThemeProfiles(await invoke('save_theme_profile',{name:themeDraft.name,baseMode:themeDraft.base_mode,accent:themeDraft.accent,surfaceMain:themeDraft.surface_main,surfaceSidebar:themeDraft.surface_sidebar,surfaceRaised:themeDraft.surface_raised,textPrimary:themeDraft.text_primary,textSecondary:themeDraft.text_secondary,borderColor:themeDraft.border_color,radiusPx:themeDraft.radius_px,fontScale:themeDraft.font_scale,density:themeDraft.density,motion:themeDraft.motion,shadowStrength:themeDraft.shadow_strength,transparency:themeDraft.transparency,blurPx:themeDraft.blur_px,fontFamily:themeDraft.font_family,lineHeight:themeDraft.line_height,animationSpeed:themeDraft.animation_speed,previewRatio:themeDraft.preview_ratio,backgroundImage:themeDraft.background_image,scheduleMode:themeDraft.schedule_mode,followWindows:themeDraft.follow_windows}));setThemeMessage('Det egna temat sparades')}catch(e){setThemeMessage(String(e))}}
  async function handleDuplicateTheme(theme:ThemeProfile){try{setThemeProfiles(await invoke('duplicate_theme_profile',{id:theme.id,name:`${theme.name} kopia`}));setThemeMessage('Temat duplicerades')}catch(e){setThemeMessage(String(e))}}
  async function handleExportTheme(theme:ThemeProfile){try{const path=await invoke<string>('export_theme_profile',{id:theme.id});setThemeMessage(`Temat exporterades: ${path}`)}catch(e){setThemeMessage(String(e))}}
  async function handleImportTheme(){const path=await open({multiple:false,title:'Importera Vault-tema',filters:[{name:'Vault theme',extensions:['json']} ]});if(typeof path!=='string')return;try{setThemeProfiles(await invoke('import_theme_profile',{path}));setThemeMessage('Temat importerades lokalt')}catch(e){setThemeMessage(String(e))}}
  async function handleToggleFavorite(){if(!selectedDocumentId)return;const favorite=favoriteDocumentIds.includes(selectedDocumentId);try{setFavoriteDocumentIds(await invoke('set_document_favorite',{documentId:selectedDocumentId,favorite:!favorite}));setWorkflowError(null)}catch(e){setWorkflowError(String(e))}}
  async function handleSaveCollection(){try{setCollections(await invoke('save_collection',{name:collectionDraft.name,description:collectionDraft.description,color:collectionDraft.color,isPinned:collectionDraft.isPinned}));setCollectionDraft({name:'',description:'',color:'#7fd3a6',isPinned:true});setWorkflowError(null)}catch(e){setWorkflowError(String(e))}}
  async function handleOpenCollection(id:number){setSelectedCollectionId(id);setCollectionDocumentIds(await invoke('list_collection_document_ids',{collectionId:id}))}
  async function handleAddToCollection(collectionId:number){if(!selectedDocumentId)return;try{setCollections(await invoke('set_collection_document',{collectionId,documentId:selectedDocumentId,included:true}));if(selectedCollectionId===collectionId)setCollectionDocumentIds(await invoke('list_collection_document_ids',{collectionId}));setWorkflowError(null)}catch(e){setWorkflowError(String(e))}}
  async function handleSaveAnalysisPreferences(){try{setAnalysisPreferences(await invoke('save_analysis_preferences',{mode:analysisPreferences.mode,autoOcr:analysisPreferences.auto_ocr,autoClassify:analysisPreferences.auto_classify,autoClaims:analysisPreferences.auto_claims,autoRelations:analysisPreferences.auto_relations,includeLocked:analysisPreferences.include_locked}));setWorkflowError(null)}catch(e){setWorkflowError(String(e))}}
  async function handleAddAnalysisExclusion(){try{setAnalysisExclusions(await invoke('add_analysis_exclusion',{scopeType:exclusionDraft.scopeType,scopeValue:exclusionDraft.scopeValue,reason:exclusionDraft.reason}));setExclusionDraft({...exclusionDraft,scopeValue:'',reason:''});setWorkflowError(null)}catch(e){setWorkflowError(String(e))}}
  async function handleImportWebSnapshot(){try{const result=await invoke<ImportResult>('import_web_snapshot',{title:webDraft.title,sourceUrl:webDraft.sourceUrl,html:webDraft.html});setLastImport(result);setWebDraft({...webDraft,html:''});await refreshOperationalData();setWorkflowError(null)}catch(e){setWorkflowError(String(e))}}
  async function handleLaunchScanner(){try{await invoke('launch_local_scanner');setWorkflowError(null)}catch(e){setWorkflowError(String(e))}}
  async function handleSaveFolder(){try{setFolders(await invoke('save_folder',{name:folderDraft.name,color:folderDraft.color,isPinned:folderDraft.isPinned}));setFolderDraft({...folderDraft,name:''});setWorkflowError(null)}catch(e){setWorkflowError(String(e))}}
  async function handleSaveCategory(){try{setCategories(await invoke('save_category',{name:categoryDraft.name,description:categoryDraft.description,color:categoryDraft.color,isPinned:categoryDraft.isPinned}));setCategoryDraft({...categoryDraft,name:'',description:''});setWorkflowError(null)}catch(e){setWorkflowError(String(e))}}
  async function handleOrganizerLock(kind:'folder'|'category',item:OrganizerSummary){try{const updated=await invoke<OrganizerSummary[]>('set_organizer_locked',{kind,organizerId:item.id,locked:!item.is_locked,pin:securityCredential});if(kind==='folder')setFolders(updated);else setCategories(updated);setWorkflowError(null)}catch(e){setWorkflowError(String(e));setActiveView('security')}}
  async function handleAssignOrganizer(kind:'folder'|'category',organizerId:number){if(!selectedDocumentId)return;try{const current=kind==='folder'?documentFolderIds:documentCategoryIds;const ids=await invoke<number[]>('set_document_organizer',{kind,organizerId,documentId:selectedDocumentId,included:!current.includes(organizerId)});if(kind==='folder'){setDocumentFolderIds(ids);setFolders(await invoke('list_folders'))}else{setDocumentCategoryIds(ids);setCategories(await invoke('list_categories'))}setWorkflowError(null)}catch(e){setWorkflowError(String(e))}}
  async function handleSaveNote(){if(!selectedDocumentId)return;try{setDocumentNotes(await invoke('save_document_note',{documentId:selectedDocumentId,noteId:noteDraft.id,body:noteDraft.body,kind:noteDraft.kind}));setNoteDraft({id:null,body:'',kind:'note'});setWorkflowError(null)}catch(e){setWorkflowError(String(e))}}
  async function handleRecordSearch(){if(!query.trim())return;try{setSearchHistory(await invoke('record_search_history',{query,resultCount:visibleResults.length}));setActiveView('documents')}catch(e){setWorkflowError(String(e))}}
  async function handleSaveSearchHistoryPreferences(){try{setSearchHistoryPreferences(await invoke('save_search_history_preferences',{enabled:searchHistoryPreferences.enabled,retentionDays:searchHistoryPreferences.retention_days}));setSearchHistory(await invoke('list_search_history'));setSearchHistoryMessage('Inställningarna sparades lokalt')}catch(e){setSearchHistoryMessage(String(e))}}
  async function handleClearSearchHistory(){try{setSearchHistory(await invoke('clear_search_history'));setSearchHistoryMessage('All ej fäst sökhistorik rensades')}catch(e){setSearchHistoryMessage(String(e))}}
  async function handleExportSearchHistory(){try{const path=await invoke<string>('export_search_history');setSearchHistoryMessage(`Exporterad lokalt: ${path}`)}catch(e){setSearchHistoryMessage(String(e))}}
  async function handleSearchNotes(value=noteSearch){try{setVaultNotes(await invoke('list_vault_notes',{query:value,targetType:null,targetId:null}));setWorkflowError(null)}catch(e){setWorkflowError(String(e))}}
  async function handleSaveVaultNote(){try{setVaultNotes(await invoke('save_vault_note',{id:vaultNoteDraft.id,title:vaultNoteDraft.title,body:vaultNoteDraft.body,kind:vaultNoteDraft.kind,targetType:vaultNoteDraft.targetType,targetId:vaultNoteDraft.targetType==='vault'?null:Number(vaultNoteDraft.targetId),tags:vaultNoteDraft.tags.split(',').map(v=>v.trim()).filter(Boolean),codeWords:vaultNoteDraft.codeWords.split(',').map(v=>v.trim()).filter(Boolean)}));setVaultNoteDraft({id:null,title:'',body:'',kind:'note',targetType:'vault',targetId:'',tags:'',codeWords:''});setNoteVersions([]);setWorkflowError(null)}catch(e){setWorkflowError(String(e))}}
  async function handleEditVaultNote(note:VaultNoteSummary){setVaultNoteDraft({id:note.id,title:note.title,body:note.body,kind:note.kind,targetType:note.target_type,targetId:note.target_id?.toString()??'',tags:note.tags.join(', '),codeWords:note.code_words.join(', ')});setNoteVersions(await invoke('list_vault_note_versions',{noteId:note.id}))}
  async function handleSaveCalendarEvent(){try{setCalendarEvents(await invoke('save_calendar_event',{documentId:selectedDocumentId,title:calendarDraft.title,eventDate:calendarDraft.eventDate,eventType:calendarDraft.eventType,note:calendarDraft.note}));setCalendarMonth(calendarDraft.eventDate.slice(0,7));setCalendarDraft({title:'',eventDate:'',eventType:'reminder',note:''});setWorkflowError(null)}catch(e){setWorkflowError(String(e))}}
  async function handleSaveGraphRelation(){const [sourceType,sourceId]=relationDraft.sourceKey.split(':');const [targetType,targetId]=relationDraft.targetKey.split(':');if(!sourceType||!targetType)return;try{setGraphData(await invoke('save_graph_relation',{sourceType,sourceId:Number(sourceId),targetType,targetId:Number(targetId),relationType:relationDraft.relationType,validFrom:relationDraft.validFrom||null,validTo:relationDraft.validTo||null}));setWorkflowError(null)}catch(e){setWorkflowError(String(e))}}
  const graphVisibleNodes=graphData.nodes.filter(node=>graphTypes.includes(node.node_type)||graphTypes.includes(node.node_type==='entity'?'object':node.node_type)).filter(node=>node.label.toLowerCase().includes(graphQuery.toLowerCase())).slice(0,60)
  const graphNodeKeys=new Set(graphVisibleNodes.map(node=>node.key));const graphVisibleEdges=graphData.edges.filter(edge=>graphNodeKeys.has(edge.source_key)&&graphNodeKeys.has(edge.target_key))
  const dashboardCatalog:Record<string,{title:string;value:string|number;detail:string;view?:ActiveView;query?:string}>={
    review:{title:'Behöver granskas',value:reviewQueue.length,detail:'Claims som väntar på ett beslut',view:'review'},
    recent:{title:'Senaste dokument',value:productionDocuments.length?productionDocuments[0].title:'Inga dokument',detail:`${productionDocuments.length} dokument i arkivet`,view:'recent'},
    uncategorized:{title:'Utan kategori',value:productionDocuments.filter(d=>!d.document_type.trim()).length,detail:'Dokument som behöver organiseras',view:'documents'},
    undated:{title:'Utan dokumentdatum',value:productionDocuments.filter(d=>!d.document_date).length,detail:'Datum saknas eller har inte verifierats',view:'documents'},
    expiry:{title:'Kommande datum',value:calendarEvents.filter(e=>e.event_date>=new Date().toISOString().slice(0,10)).length,detail:'Utgångar, starter och påminnelser',view:'calendar'},
    favorites:{title:'Favoriter',value:favoriteDocumentIds.length,detail:'Snabbåtkomst till markerade dokument',view:'favorites'},
    storage:{title:'Arkivets storlek',value:productionDocuments.length,detail:'Lokalt lagrade dokument',view:'archive'},
    conflicts:{title:'Motstridiga uppgifter',value:conflicts.length,detail:'Vault gissar aldrig mellan källor',view:'conflicts'},
    backup:{title:'Backupstatus',value:backups.length?`${backups.length} st`:'Saknas',detail:backups[0]?.backup_root??'Skapa din första lokala backup',view:'backup'},
    index:{title:'Sökindex',value:healthReport?.fts_entry_count??0,detail:'Lokala FTS-poster',view:'documents'},
    inbox:{title:'Inkorg',value:productionDocuments.filter(d=>d.inbox_status==='inbox').length,detail:'Dokument som ännu inte arkiverats',view:'inbox'},
    today:{title:'Importerade idag',value:importHistory.filter(item=>item.started_at?.slice(0,10)===new Date().toISOString().slice(0,10)).length,detail:'Lokala importkörningar idag',view:'importhistory'},
    ocr:{title:'OCR-kö',value:backgroundJobs.filter(job=>job.job_type==='document_ocr'&&job.status!=='completed').length,detail:'Väntande eller pausade OCR-jobb',view:'documents'},
    reminders:{title:'Aktiva påminnelser',value:reminders.filter(item=>item.status==='active').length,detail:'Datum som kräver uppföljning',view:'reminders'},
    relations:{title:'Relationer',value:entityCatalog.length,detail:'Personer, företag och objekt',view:'relations'},
    trash:{title:'Papperskorg',value:trashedDocuments.length,detail:'Dokument som kan återställas',view:'trash'},
    automation:{title:'Bevakade mappar',value:watchedFolders.filter(item=>item.enabled).length,detail:'Aktiva lokala importkällor',view:'automation'},
    ...Object.fromEntries(savedSearches.map(search=>[`search:${search.id}`,{title:search.name,value:'Sparad sökning',detail:search.query,view:'documents' as ActiveView,query:search.query}])),
  }
  function moveDashboardWidget(id:string,direction:-1|1){setDashboardWidgets(items=>{const index=items.indexOf(id),next=index+direction;if(index<0||next<0||next>=items.length)return items;const copy=[...items];[copy[index],copy[next]]=[copy[next],copy[index]];return copy})}
  function exportDashboardLayout(){const blob=new Blob([JSON.stringify({format:'vault-dashboard-layout',version:2,name:activeDashboardLayout,widgets:dashboardWidgets,sizes:dashboardSizes},null,2)],{type:'application/json'});const url=URL.createObjectURL(blob);const anchor=document.createElement('a');anchor.href=url;anchor.download='vault-dashboard-layout.json';anchor.click();URL.revokeObjectURL(url)}
  function setDashboardWidgetSize(id:string,size:DashboardSize){setDashboardSizes(current=>({...current,[id]:size}))}
  function saveDashboardLayout(){const name=dashboardLayoutName.trim();if(!name)return;setDashboardLayouts(items=>[...items.filter(item=>item.name!==name),{name,widgets:[...dashboardWidgets],sizes:{...dashboardSizes}}]);setActiveDashboardLayout(name)}
  function loadDashboardLayout(name:string){if(name==='Standard'){setDashboardWidgets(defaultDashboardWidgets);setDashboardSizes({});setActiveDashboardLayout(name);return}const layout=dashboardLayouts.find(item=>item.name===name);if(layout){setDashboardWidgets(layout.widgets);setDashboardSizes(layout.sizes);setActiveDashboardLayout(name)}}
  function deleteDashboardLayout(){if(activeDashboardLayout==='Standard')return;setDashboardLayouts(items=>items.filter(item=>item.name!==activeDashboardLayout));loadDashboardLayout('Standard')}
  function openSavedSearch(search:SavedSearchSummary){setQuery(search.query);setActiveView('documents')}
  function applySearchFilters(){const freeText=query.split(/\s+/).filter(part=>!part.includes(':')).join(' ');const operators=[searchFilters.type&&`type:${searchFilters.type}`,searchFilters.category&&`kategori:${searchFilters.category}`,searchFilters.employer&&`arbetsgivare:${searchFilters.employer}`,searchFilters.status&&`status:${searchFilters.status}`,searchFilters.mime&&`mime:${searchFilters.mime}`,searchFilters.from&&`fran:${searchFilters.from}`,searchFilters.to&&`till:${searchFilters.to}`].filter(Boolean);setQuery([freeText,...operators].filter(Boolean).join(' '));setActiveView('documents')}
  const allCommandItems=[
    {id:'documents',label:'Öppna alla dokument',keywords:'dokument arkiv',run:()=>setActiveView('documents')},{id:'favorites',label:'Visa favoriter',keywords:'stjärna favorit',run:()=>setActiveView('favorites')},{id:'recent',label:'Visa senaste',keywords:'nyligen recent',run:()=>setActiveView('recent')},{id:'notes',label:'Öppna anteckningar',keywords:'markdown todo beslut',run:()=>setActiveView('notes')},{id:'collections',label:'Öppna samlingar',keywords:'collection',run:()=>setActiveView('collections')},{id:'folders',label:'Öppna mappar',keywords:'folder mapp',run:()=>setActiveView('folders')},{id:'categories',label:'Öppna kategorier',keywords:'kategori klassificera',run:()=>setActiveView('categories')},{id:'calendar',label:'Öppna kalender',keywords:'datum påminnelse giltighet',run:()=>setActiveView('calendar')},{id:'graph',label:'Öppna graf',keywords:'relation nod kant',run:()=>setActiveView('graph')},{id:'searchhistory',label:'Sökhistorik',keywords:'tidigare sökningar',run:()=>setActiveView('searchhistory')},{id:'backup',label:'Öppna backup',keywords:'säkerhetskopia inkrementell återställ',run:()=>setActiveView('backup')},{id:'integrity',label:'Dubbletter och integritet',keywords:'hash dubblett filkontroll',run:()=>setActiveView('integrity')},{id:'review',label:'Öppna granskningskön',keywords:'claims granska godkänn',run:()=>setActiveView('review')},{id:'import',label:'Importera dokument',keywords:'fil lägg till',run:()=>void handleImport()},{id:'capture',label:'Skanna eller spara webbsida',keywords:'scanner html webb',run:()=>setActiveView('capture')},{id:'analysissettings',label:'Analysinställningar',keywords:'omfattning undantag ocr',run:()=>setActiveView('analysissettings')},{id:'analyze',label:'Analysera arkiv',keywords:'ocr index regler',run:()=>void handleAnalyzeArchive()},{id:'themes',label:'Öppna teman',keywords:'utseende färg',run:()=>setActiveView('themes')},{id:'security',label:'Öppna säkerhet',keywords:'pin lås',run:()=>setActiveView('security')},{id:'testcenter',label:'Öppna Test Center',keywords:'test diagnos',run:()=>setActiveView('testcenter')},{id:'health',label:'Kontrollera arkivets hälsa',keywords:'sqlite filer status',run:()=>void handleCheckHealth()},{id:'reindex',label:'Reparera sökindex',keywords:'fts index sök',run:()=>void handleReindexSearch()}
    ,...savedSearches.map(search=>({id:`saved-search-${search.id}`,label:`Sök: ${search.name}`,keywords:`sparad sökning ${search.query}`,run:()=>openSavedSearch(search)}))
  ]
  const commandItems=allCommandItems.filter(item=>(item.label+' '+item.keywords).toLowerCase().includes(commandQuery.toLowerCase()))
  function runCommand(item:(typeof commandItems)[number]){item.run();setCommandPaletteOpen(false);setCommandQuery('')}

  // The command table deliberately captures the current local app actions.
  // eslint-disable-next-line react-hooks/exhaustive-deps
  useEffect(()=>{const normalize=(event:KeyboardEvent)=>[event.ctrlKey?'Ctrl':'',event.altKey?'Alt':'',event.shiftKey?'Shift':'',event.key.length===1?event.key.toUpperCase():event.key].filter(Boolean).join('+');const onKey=(event:KeyboardEvent)=>{if((event.ctrlKey||event.metaKey)&&event.key.toLowerCase()==='k'){event.preventDefault();setCommandPaletteOpen(open=>!open);return}if(event.key==='Escape'){setCommandPaletteOpen(false);return}const pressed=normalize(event);const item=allCommandItems.find(command=>commandShortcuts[command.id]===pressed);if(item&&!['INPUT','TEXTAREA','SELECT'].includes((event.target as HTMLElement)?.tagName)){event.preventDefault();item.run()}};window.addEventListener('keydown',onKey);return()=>window.removeEventListener('keydown',onKey)},[commandShortcuts])

  return (
    <main className={`vault-shell${sidebarCollapsed?' sidebar-collapsed':''}${!sidebarVisible?' sidebar-hidden':''}`} style={{'--sidebar-width':`${sidebarCollapsed?76:sidebarWidth}px`,'--details-width':`${detailsWidth}px`} as CSSProperties}>
      <a className="skip-link" href="#vault-workspace">Hoppa till arbetsytan</a>
      {internalViewerOpen&&selectedDocumentId?<Suspense fallback={<div className="viewer-module-loading">Startar den lokala dokumentvisaren…</div>}><DocumentViewer documentId={selectedDocumentId} initialPage={selectedPageNo} initialSearch={viewerSourceClaim?.source_text??''} onClose={()=>{setInternalViewerOpen(false);setViewerSourceClaim(null)}} onPageChange={setSelectedPageNo} pin={securityCredential} sourceClaimId={viewerSourceClaim?.id??null} sourceRule={viewerSourceClaim?.extraction_method??''} title={selectedDocument.title}/></Suspense>:null}
      {commandPaletteOpen?<div className="command-palette-backdrop" onMouseDown={()=>setCommandPaletteOpen(false)}><section className="command-palette" onMouseDown={event=>event.stopPropagation()}><header><input aria-label="Kommandopalett" autoFocus placeholder="Sök kommando…" value={commandQuery} onChange={event=>setCommandQuery(event.target.value)} onKeyDown={event=>{if(event.key==='Enter'&&commandItems[0]&&!commandShortcutEditing)runCommand(commandItems[0])}}/><button className="mini-action" onClick={()=>setCommandShortcutEditing(value=>!value)}>{commandShortcutEditing?'Klar':'Kortkommandon'}</button></header><div>{commandItems.map(item=><div className="command-row" key={item.id}><button onClick={()=>runCommand(item)} type="button"><strong>{item.label}</strong><small>{item.keywords}</small></button>{commandShortcutEditing?<input aria-label={`Kortkommando för ${item.label}`} placeholder="Ctrl+Alt+D" value={commandShortcuts[item.id]??''} onChange={event=>setCommandShortcuts(current=>({...current,[item.id]:event.target.value}))} onKeyDown={event=>{event.stopPropagation();if(event.key==='Backspace'||event.key==='Delete'){event.preventDefault();setCommandShortcuts(current=>({...current,[item.id]:''}));return}if(['Control','Alt','Shift','Meta','Tab'].includes(event.key))return;event.preventDefault();const value=[event.ctrlKey?'Ctrl':'',event.altKey?'Alt':'',event.shiftKey?'Shift':'',event.key.length===1?event.key.toUpperCase():event.key].filter(Boolean).join('+');if(value&&!Object.entries(commandShortcuts).some(([id,shortcut])=>id!==item.id&&shortcut===value))setCommandShortcuts(current=>({...current,[item.id]:value}))}}/>:<kbd>{commandShortcuts[item.id]??''}</kbd>}</div>)}</div><footer>Ctrl+K öppnar · Enter kör första · Esc stänger{commandShortcutEditing?' · Tryck önskad tangentkombination i ett fält':''}</footer></section></div>:null}
      {!vaultUnlocked && securityStatus.mode!=='comfortable' ? <div className="vault-lock-screen"><div className="lock-card"><div className="brand-mark">V</div><h1>Vault är låst</h1><p>Arkivet och förhandsvisningar är dolda tills du verifierats lokalt.</p><input aria-label="PIN eller lösenord" autoFocus type="password" value={unlockPin} onChange={e=>setUnlockPin(e.target.value)} onKeyDown={e=>{if(e.key==='Enter')void handleUnlockVault()}}/><button className="primary-action" onClick={handleUnlockVault} type="button">Lås upp med PIN/lösenord</button>{quickUnlockEnabled?<button className="secondary-action" onClick={handleQuickUnlock} type="button">Lås upp med Windows-skydd</button>:null}{windowsHello.available?<button className="secondary-action" onClick={handleWindowsHelloUnlock} type="button">Lås upp med Windows Hello</button>:null}<small>Snabb upplåsning skyddas med DPAPI för den aktuella Windows-användaren.</small>{workflowError?<p className="inline-error">{workflowError}</p>:null}</div></div>:null}
      {!sidebarVisible?<button aria-label="Visa sidopanel" className="sidebar-reopen" onClick={()=>setSidebarVisible(true)} type="button">☰</button>:null}
      <aside className="sidebar" aria-label="Huvudnavigering" hidden={!sidebarVisible}>
        <div className="brand-block">
          <div className="brand-mark" aria-hidden="true">V</div>
          <div>
            <div className="brand-title">Vault</div>
            <div className="brand-subtitle">Lokalt dokumentarkiv</div>
          </div>
          <div className="sidebar-layout-actions"><button aria-label={sidebarCollapsed?'Expandera sidopanel':'Minimera sidopanel'} onClick={()=>setSidebarCollapsed(value=>!value)} type="button">{sidebarCollapsed?'»':'«'}</button><button aria-label="Dölj sidopanel" onClick={()=>setSidebarVisible(false)} type="button">×</button></div>
        </div>
        {!sidebarCollapsed?<label className="sidebar-width-control">Bredd<input aria-label="Sidopanelens bredd" max="380" min="210" onChange={event=>setSidebarWidth(Number(event.target.value))} type="range" value={sidebarWidth}/></label>:null}

        <nav className="nav-section">
          {([
            ['start', 'Start / Testa appen'],
            ['documents', 'Dokument'],
            ['inbox', 'Inkorg (' + productionDocuments.filter((document) => document.inbox_status === 'inbox').length + ')'],
            ['archive', 'Arkiv (' + productionDocuments.filter((document) => document.inbox_status === 'archived').length + ')'],
            ['favorites', 'Favoriter (' + favoriteDocumentIds.length + ')'],
            ['recent', 'Senaste'],
            ['collections', 'Samlingar (' + collections.length + ')'],
            ['folders', 'Mappar (' + folders.length + ')'],
            ['categories', 'Kategorier (' + categories.length + ')'],
            ['notes', 'Anteckningar (' + vaultNotes.length + ')'],
            ['searchhistory', 'Sökhistorik (' + searchHistory.length + ')'],
            ['capture', 'Skanna och webb'],
            ['trash', 'Papperskorg (' + trashedDocuments.length + ')'],
            ['review', 'Granskningskö (' + reviewQueue.length + ')'],
            ['relations', 'Relationer (' + entityCatalog.length + ')'],
            ['graph', 'Grafvy (' + graphData.nodes.length + ')'],
            ['people', 'Personer'],
            ['companies', 'Företag'],
            ['objects', 'Objekt och fordon'],
            ['employment', 'Arbete och lön'],
            ['education', 'Utbildningar'],
            ['contracts', 'Avtal'],
            ['housing', 'Boende'],
            ['travel', 'Resor'],
            ['authorities', 'Myndigheter'],
            ['timeline', 'Tidslinje (' + timelineEvents.length + ')'],
            ['calendar', 'Kalender (' + calendarEvents.length + ')'],
            ['actuality', 'Aktualitet'],
            ['conflicts', 'Konflikter (' + conflicts.filter((conflict) => conflict.status === 'open').length + ')'],
            ['reminders', 'Påminnelser (' + reminders.filter((reminder) => reminder.status === 'active').length + ')'],
            ['automation', 'Bevakade mappar (' + watchedFolders.length + ')'],
            ['importhistory', 'Importhistorik (' + importHistory.length + ')'],
            ['rules', 'Regler och mallar (' + automationRules.length + ')'],
            ['analysissettings', 'Analysomfattning'],
            ['security', 'Säkerhet (' + securityStatus.locked_document_count + ' låsta)'],
            ['themes', 'Teman (' + themeProfiles.length + ')'],
            ['backup', 'Backup (' + backups.length + ')'],
            ['integrity', 'Dubbletter och integritet'],
            ['plugins', 'Plugins (' + plugins.length + ')'],
            ['testcenter', 'Test Center'],
          ] as [ActiveView, string][]).map(([view, label]) => (
            <button
              className={activeView === view ? 'nav-button active' : 'nav-button'}
              key={view}
              onClick={() => setActiveView(view)}
              type="button"
            >
              {label}
            </button>
          ))}
          {savedSearches.filter(search=>search.pinned).length?<div className="nav-subsection"><strong>Sparade sökningar</strong>{savedSearches.filter(search=>search.pinned).map(search=><button className="nav-button saved-search-nav" key={search.id} onClick={()=>openSavedSearch(search)} type="button">⌕ {search.name}</button>)}</div>:null}
          <button
            className="nav-button"
            disabled={isResettingTestlab}
            onClick={handleResetTestlab}
            type="button"
          >
            {isResettingTestlab ? 'Återställer...' : 'Återställ Test Lab'}
          </button>
        </nav>

        <div className="sidebar-note">
          <span className="status-dot"></span>
          AI-fritt läge aktivt
        </div>

        <div className="theme-switcher" aria-label="Tema">
          {(['system', 'light', 'dark'] as ThemeMode[]).map((mode) => (
            <button
              aria-pressed={themeMode === mode}
              className={themeMode === mode ? 'theme-option active' : 'theme-option'}
              key={mode}
              onClick={() => setThemeMode(mode)}
              type="button"
            >
              {mode === 'system' ? 'Auto' : mode === 'light' ? 'Ljus' : 'Mörk'}
            </button>
          ))}
        </div>
      </aside>

      <section className={`workspace${isDraggingFiles ? ' drag-active' : ''}`} aria-label="Vault arbetsyta" id="vault-workspace" tabIndex={-1}>
        {isDraggingFiles ? <div className="drop-overlay">Släpp filer, mappar eller ZIP här för lokal import</div> : null}
        <header className="topbar">
          <label className="search-field" htmlFor="global-search">
            <span>Sök</span>
            <input
              id="global-search"
              onChange={(event) => setQuery(event.target.value)}
              onKeyDown={(event)=>{if(event.key==='Enter')void handleRecordSearch()}}
              placeholder='Sök text eller type:avtal tag:kvitto date:2024 -utkast'
              type="search"
              value={query}
            />
          </label>
          <button className={showSearchFilters?'command-trigger active':'command-trigger'} onClick={()=>setShowSearchFilters(value=>!value)} type="button">Filter</button>
          <button className="command-trigger" onClick={()=>setCommandPaletteOpen(true)} type="button">Kommandon <kbd>Ctrl K</kbd></button>
          <div className="topbar-actions">
            <button
              className="primary-action"
              disabled={isImporting || isImportingDirectory}
              onClick={handleImport}
              type="button"
            >
              {isImporting ? 'Importerar...' : 'Importera dokument'}
            </button>
            <button
              className="ghost-action"
              disabled={isImporting || isImportingDirectory || isCreatingDemoArchive}
              onClick={handleImportDirectory}
              type="button"
            >
              {isImportingDirectory ? 'Importerar mapp...' : 'Importera mapp'}
            </button>
            <button
              className="ghost-action"
              disabled={isCreatingDemoArchive}
              onClick={handleCreateDemoArchive}
              type="button"
            >
              {isCreatingDemoArchive ? 'Skapar demo...' : 'Skapa demoarkiv'}
            </button>
            <button className="ghost-action" disabled={isImportingDirectory} onClick={handleImportArchive} type="button">
              Importera ZIP
            </button>
            <button className="ghost-action" disabled={isImporting} onClick={handleClipboardRead} type="button">
              Klistra in text
            </button>
            <button
              className="ghost-action"
              disabled={isCheckingOcr}
              onClick={handleCheckOcrStatus}
              type="button"
            >
              {isCheckingOcr
                ? 'Kontrollerar OCR...'
                : ocrStatus.available
                  ? 'OCR redo'
                  : 'OCR saknas'}
            </button>
            <button
              className="ghost-action"
              disabled={isAnalyzingArchive}
              onClick={handleAnalyzeArchive}
              type="button"
            >
              {isAnalyzingArchive ? 'Analyserar...' : 'Analysera arkiv'}
            </button>
            <button
              className="ghost-action"
              disabled={isCreatingBackup}
              onClick={handleCreateBackup}
              type="button"
            >
              {isCreatingBackup ? 'Skapar backup...' : 'Skapa backup'}
            </button>
            <button
              className="ghost-action"
              disabled={isCheckingHealth}
              onClick={handleCheckHealth}
              type="button"
            >
              {isCheckingHealth ? 'Kontrollerar...' : 'Kontrollera arkiv'}
            </button>
            <button
              className="ghost-action"
              disabled={isReindexing}
              onClick={handleReindexSearch}
              type="button"
            >
              {isReindexing ? 'Reindexerar...' : 'Reparera sök'}
            </button>
            <button
              className="ghost-action"
              disabled={isRunningTestCenter}
              onClick={handleRunTestCenter}
              type="button"
            >
              {isRunningTestCenter ? 'Testar...' : 'Test Center'}
            </button>
          </div>
        </header>
        {showSearchFilters?<section className="advanced-search-panel" aria-label="Avancerade sökfilter"><label>Dokumenttyp<input value={searchFilters.type} onChange={e=>setSearchFilters({...searchFilters,type:e.target.value})}/></label><label>Kategori<input value={searchFilters.category} onChange={e=>setSearchFilters({...searchFilters,category:e.target.value})}/></label><label>Arbetsgivare eller entitet<input value={searchFilters.employer} onChange={e=>setSearchFilters({...searchFilters,employer:e.target.value})}/></label><label>Status<select value={searchFilters.status} onChange={e=>setSearchFilters({...searchFilters,status:e.target.value})}><option value="">Alla</option><option value="inbox">Inkorg</option><option value="review">Granskning</option><option value="archived">Arkiv</option></select></label><label>Filformat<input placeholder="pdf, image eller text" value={searchFilters.mime} onChange={e=>setSearchFilters({...searchFilters,mime:e.target.value})}/></label><label>Från datum<input type="date" value={searchFilters.from} onChange={e=>setSearchFilters({...searchFilters,from:e.target.value})}/></label><label>Till datum<input type="date" value={searchFilters.to} onChange={e=>setSearchFilters({...searchFilters,to:e.target.value})}/></label><div className="claim-actions"><button className="primary-action" onClick={applySearchFilters} type="button">Använd filter</button><button className="mini-action" onClick={()=>{setSearchFilters({type:'',category:'',employer:'',status:'',mime:'',from:'',to:''});setQuery('')}} type="button">Rensa</button></div></section>:null}

        <div className="overview-strip">
          <div>
            <span className="metric">{testlabDocuments.length}</span>
            <span className="metric-label">syntetiska dokument</span>
          </div>
          <div>
            <span className="metric">{productionDocuments.length}</span>
            <span className="metric-label">importerade dokument</span>
          </div>
          <div>
            <span className="metric">{query.trim() ? visibleResults.length : 0}</span>
            <span className="metric-label">sökträffar</span>
          </div>
          <div>
            <span className="metric">0</span>
            <span className="metric-label">molntjänster</span>
          </div>
        </div>

        {activeView === 'start' ? (
          <section className="start-view" aria-label="Kom igång med Vault">
            <div className="start-header">
              <div>
                <h1>Vault testklarhet</h1>
                <p>
                  {diagnosticsError
                    ? diagnosticsError
                    : diagnostics
                      ? `${appReadyScore}/6 grundkontroller är redo. Importera en fil och kör analys för att testa hela kedjan.`
                      : 'Läser lokal diagnostik...'}
                </p>
              </div>
              <div className="readiness-meter" aria-label="Redo-kontroller">
                <strong>{appReadyScore}/6</strong>
                <span>redo</span>
              </div>
            </div>

            <div className="quick-actions">
              <button className="primary-action" disabled={isImporting} onClick={handleImport} type="button">
                {isImporting ? 'Importerar...' : 'Importera första filen'}
              </button>
              <button
                className="secondary-action"
                disabled={isImportingDirectory}
                onClick={handleImportDirectory}
                type="button"
              >
                {isImportingDirectory ? 'Importerar mapp...' : 'Importera hel mapp'}
              </button>
              <button
                className="secondary-action"
                disabled={isCreatingDemoArchive}
                onClick={handleCreateDemoArchive}
                type="button"
              >
                {isCreatingDemoArchive ? 'Skapar demo...' : 'Skapa demoarkiv'}
              </button>
              <button
                className="secondary-action"
                disabled={isAnalyzingArchive}
                onClick={handleAnalyzeArchive}
                type="button"
              >
                {isAnalyzingArchive ? 'Analyserar...' : 'Analysera arkiv'}
              </button>
              <button
                className="secondary-action"
                disabled={isRunningTestCenter}
                onClick={handleRunTestCenter}
                type="button"
              >
                {isRunningTestCenter ? 'Testar...' : 'Kör Test Center'}
              </button>
              <button
                className="secondary-action"
                disabled={isCheckingHealth}
                onClick={handleCheckHealth}
                type="button"
              >
                {isCheckingHealth ? 'Kontrollerar...' : 'Kontrollera hälsa'}
              </button>
              <button className="secondary-action" onClick={() => setActiveView('documents')} type="button">
                Öppna dokument
              </button>
            </div>

            <div className="dashboard-toolbar">
              <div><strong>Min arkivöversikt</strong><small>Layouten sparas endast lokalt på den här datorn.</small></div>
              <div>
                <select aria-label="Dashboardlayout" value={activeDashboardLayout} onChange={event=>loadDashboardLayout(event.target.value)}><option>Standard</option>{dashboardLayouts.map(layout=><option key={layout.name}>{layout.name}</option>)}</select>
                <button className="secondary-action" onClick={()=>setDashboardEditing(value=>!value)} type="button">{dashboardEditing?'Klar':'Anpassa'}</button>
                <button className="secondary-action" onClick={exportDashboardLayout} type="button">Exportera layout</button>
                {dashboardEditing?<><input aria-label="Namn på layout" value={dashboardLayoutName} onChange={event=>setDashboardLayoutName(event.target.value)}/><button className="secondary-action" onClick={saveDashboardLayout} type="button">Spara layout</button><select aria-label="Lägg till widget" defaultValue="" onChange={event=>{if(event.target.value)setDashboardWidgets(items=>[...items,event.target.value]);event.target.value='' }}><option value="">Lägg till widget…</option>{Object.entries(dashboardCatalog).map(([id,widget])=><option key={id} value={id}>{widget.title}</option>)}</select><button className="secondary-action" onClick={()=>{setDashboardWidgets(defaultDashboardWidgets);setDashboardSizes({});setActiveDashboardLayout('Standard')}} type="button">Återställ</button>{activeDashboardLayout!=='Standard'?<button className="secondary-action danger-action" onClick={deleteDashboardLayout} type="button">Ta bort layout</button>:null}</>:null}
              </div>
            </div>
            <div className="dashboard-widget-grid">
              {dashboardWidgets.map((id,index)=>{const widget=dashboardCatalog[id];if(!widget)return null;const size=dashboardSizes[id]??'small';return <article className={`dashboard-widget size-${size}`} key={`${id}-${index}`}>
                <button className="dashboard-widget-main" onClick={()=>{if(widget.query)setQuery(widget.query);if(widget.view)setActiveView(widget.view)}} type="button"><span>{widget.title}</span><strong>{widget.value}</strong><small>{widget.detail}</small></button>
                {dashboardEditing?<div className="dashboard-widget-controls"><button aria-label={`Flytta ${widget.title} åt vänster`} disabled={index===0} onClick={()=>moveDashboardWidget(id,-1)} type="button">←</button><button aria-label={`Flytta ${widget.title} åt höger`} disabled={index===dashboardWidgets.length-1} onClick={()=>moveDashboardWidget(id,1)} type="button">→</button><select aria-label={`Storlek för ${widget.title}`} value={size} onChange={event=>setDashboardWidgetSize(id,event.target.value as DashboardSize)}><option value="small">Liten</option><option value="medium">Mellan</option><option value="large">Stor</option></select><button aria-label={`Dölj ${widget.title}`} onClick={()=>setDashboardWidgets(items=>items.filter((_,itemIndex)=>itemIndex!==index))} type="button">Dölj</button><button aria-label={`Duplicera ${widget.title}`} onClick={()=>setDashboardWidgets(items=>[...items.slice(0,index+1),id,...items.slice(index+1)])} type="button">Duplicera</button></div>:null}
              </article>})}
            </div>

            <div className="status-grid">
              <div className="status-card">
                <span className="status-label">Databas</span>
                <strong>{vaultInit.production_vault_ready ? 'Redo' : 'Inte redo'}</strong>
                <small>Schema v{diagnostics?.schema_version ?? vaultInit.schema_version}</small>
              </div>
              <div className="status-card">
                <span className="status-label">Dokument</span>
                <strong>{diagnostics?.production_document_count ?? productionDocuments.length}</strong>
                <small>{diagnostics?.testlab_document_count ?? testlabDocuments.length} i Test Lab</small>
              </div>
              <div className="status-card">
                <span className="status-label">OCR</span>
                <strong>{ocrStatus.available ? 'Redo' : 'Saknas'}</strong>
                <small>{ocrStatus.languages.join(', ') || 'inga språk hittade'}</small>
              </div>
              <div className="status-card">
                <span className="status-label">PDF</span>
                <strong>{diagnostics?.pdf_text_available ? 'Redo' : 'Begränsad'}</strong>
                <small>{diagnostics?.pdftotext_path ?? 'pdftotext hittades inte'}</small>
              </div>
              <div className="status-card">
                <span className="status-label">Office</span>
                <strong>{diagnostics?.office_text_available ? 'Redo' : 'Saknas'}</strong>
                <small>DOCX, XLSX och PPTX extraheras lokalt</small>
              </div>
              <div className="status-card">
                <span className="status-label">Granskning</span>
                <strong>{diagnostics?.pending_review_count ?? reviewQueue.length}</strong>
                <small>{diagnostics?.entity_count ?? entityCatalog.length} relationer</small>
              </div>
            </div>

            <div className="start-columns">
              <section className="work-item">
                <strong>Att testa nu</strong>
                <ol className="test-flow">
                  <li>Importera en PDF, bild, DOCX eller TXT.</li>
                  <li>Kör Analysera arkiv.</li>
                  <li>Sök på ord från dokumentet.</li>
                  <li>Öppna Granskningskö och godkänn eller avvisa claims.</li>
                  <li>Skapa backup och kör hälsokontroll.</li>
                </ol>
              </section>
              <section className="work-item">
                <strong>Problem som appen ser</strong>
                {diagnostics?.issues.length ? (
                  <ul className="issue-list">
                    {diagnostics.issues.map((issue) => (
                      <li key={issue}>{issue}</li>
                    ))}
                  </ul>
                ) : (
                  <p>Inga blockerande problem hittade i lokal diagnostik.</p>
                )}
              </section>
              <section className="work-item">
                <strong>Senaste mappimport</strong>
                {lastDirectoryImport ? (
                  <>
                    <span>
                      {lastDirectoryImport.imported_count} nya, {lastDirectoryImport.duplicate_count} dubbletter,
                      {lastDirectoryImport.failed_count} fel av {lastDirectoryImport.scanned_file_count} filer.
                    </span>
                    {lastDirectoryImport.failures.slice(0, 3).map((failure) => (
                      <small className="path-value" key={failure}>{failure}</small>
                    ))}
                  </>
                ) : (
                  <p>Ingen mappimport körd ännu.</p>
                )}
              </section>
              <section className="work-item">
                <strong>Lokala sökvägar</strong>
                <small className="path-value">Data: {diagnostics?.data_root ?? vaultInit.data_root}</small>
                <small className="path-value">Tesseract: {diagnostics?.tesseract_path ?? 'saknas'}</small>
                <small className="path-value">Tessdata: {diagnostics?.tessdata_dir ?? 'saknas'}</small>
              </section>
            </div>
          </section>
        ) : null}

        {!['documents', 'inbox', 'archive', 'favorites', 'recent', 'trash', 'start'].includes(activeView) ? (
          <section className="operational-view" aria-label="Vault kontrollvy">
            <div className="section-heading">
              <div>
                <h1>
                  {activeView === 'review'
                    ? 'Granskningskö'
                    : activeView === 'relations'
                      ? 'Relationer'
                      : activeView === 'graph'
                        ? 'Grafvy'
                      : activeView === 'people'
                        ? 'Personer'
                        : activeView === 'companies'
                          ? 'Företag'
                          : activeView === 'objects'
                            ? 'Objekt och fordon'
                            : activeView === 'employment'
                              ? 'Arbete och lön'
                              : activeView === 'education'
                                ? 'Utbildningar'
                                : activeView === 'contracts'
                                  ? 'Avtal och försäkringar'
                                  : activeView === 'housing'
                                    ? 'Boende'
                                    : activeView === 'travel'
                                      ? 'Resor'
                                      : activeView === 'authorities'
                                        ? 'Myndigheter och ärenden'
                            : activeView === 'timeline'
                              ? 'Tidslinje'
                              : activeView === 'calendar'
                                ? 'Kalender'
                              : activeView === 'actuality'
                                ? 'Aktualitet'
                      : activeView === 'conflicts'
                        ? 'Konflikter'
                        : activeView === 'reminders'
                          ? 'Påminnelser'
                          : activeView === 'automation'
                            ? 'Bevakade mappar'
                            : activeView === 'importhistory'
                              ? 'Importhistorik'
                              : activeView === 'rules'
                                ? 'Regler, mallar och egna fält'
                                : activeView === 'security'
                                  ? 'Säkerhet och skyddad export'
                                  : activeView === 'themes'
                                    ? 'Teman och visuell anpassning'
                                    : activeView === 'collections'
                                      ? 'Samlingar'
                                      : activeView === 'folders'
                                        ? 'Mappar'
                                        : activeView === 'categories'
                                          ? 'Kategorier'
                                          : activeView === 'notes'
                                            ? 'Anteckningar och egna arbetsdata'
                                          : activeView === 'searchhistory'
                                            ? 'Sökhistorik och integritet'
                                      : activeView === 'capture'
                                        ? 'Skanna och spara webbsida'
                                        : activeView === 'analysissettings'
                                          ? 'Analysomfattning och undantag'
                        : activeView === 'backup'
                          ? 'Backup och hälsa'
                          : activeView === 'integrity'
                            ? 'Dubbletter och filintegritet'
                            : activeView === 'plugins'
                              ? 'Lokala plugins och behörigheter'
                          : 'Test Center'}
                </h1>
                <p>
                  {analysisError
                    ? analysisError
                    : lastAnalysis
                      ? `Senaste analys: ${lastAnalysis.document_count} dokument, ${lastAnalysis.reindexed_document_count} omindexerade, ${lastAnalysis.ocr_updated_count} OCR-uppdaterade, ${lastAnalysis.date_updated_count} datum ifyllda, ${lastAnalysis.entity_link_count} nya relationer.`
                      : 'Kör Analysera arkiv för att uppdatera OCR, index, regler och relationer.'}
                </p>
              </div>
              <button
                className="secondary-action"
                disabled={isAnalyzingArchive}
                onClick={handleAnalyzeArchive}
                type="button"
              >
                {isAnalyzingArchive ? 'Analyserar...' : 'Analysera arkiv'}
              </button>
            </div>

            {activeView === 'review' ? (
              <div className="work-panel-grid">
                {reviewQueue.length ? (
                  reviewQueue.map((item) => (
                    <div className="work-item" key={item.claim_id}>
                      <strong>{item.document_title}</strong>
                      <span>{item.claim_type}</span>
                      <code>{item.value_json}</code>
                      <small>{item.extraction_method}</small>
                      <div className="claim-actions">
                        <button
                          className="mini-action"
                          onClick={() => handleSetClaimStatus(item.claim_id, 'review_approved')}
                          type="button"
                        >
                          Godkänn
                        </button>
                        <button
                          className="mini-action"
                          onClick={() => handleSetClaimStatus(item.claim_id, 'review_rejected')}
                          type="button"
                        >
                          Avvisa
                        </button>
                      </div>
                    </div>
                  ))
                ) : (
                  <div className="empty-state">Inga claims väntar på granskning.</div>
                )}
              </div>
            ) : null}

            {activeView === 'importhistory' ? (
              <>
                {showClipboardImport ? (
                  <div className="work-item clipboard-import">
                    <strong>Skapa dokument från urklipp</strong>
                    <input value={clipboardDraft.title} onChange={(event) => setClipboardDraft((draft) => ({ ...draft, title: event.target.value }))} placeholder="Dokumenttitel" />
                    <textarea value={clipboardDraft.text} onChange={(event) => setClipboardDraft((draft) => ({ ...draft, text: event.target.value }))} placeholder="Klistra in text här om automatisk läsning inte tilläts" rows={8} />
                    <div className="claim-actions">
                      <button className="mini-action" disabled={isImporting || !clipboardDraft.text.trim()} onClick={handleClipboardImport} type="button">Importera text</button>
                      <button className="mini-action" onClick={() => setShowClipboardImport(false)} type="button">Avbryt</button>
                    </div>
                  </div>
                ) : null}
                <div className="work-panel-grid">
                  {importHistory.length ? importHistory.map((session) => (
                    <div className="work-item" key={session.id}>
                      <strong>{session.method} · {session.status}</strong>
                      <span>{session.source_label}</span>
                      <small>{session.started_at} · {session.imported_count} importerade · {session.duplicate_count} dubbletter · {session.failed_count} fel</small>
                      {session.items.slice(0, 8).map((item, index) => (
                        <small key={`${item.source_name}-${index}`}>{item.status}: {item.source_name}{item.safe_error ? ` · ${item.safe_error}` : ''}</small>
                      ))}
                    </div>
                  )) : <div className="empty-state">Ingen importhistorik ännu.</div>}
                </div>
              </>
            ) : null}

            {activeView === 'relations' ? (
              <div className="work-panel-grid">
                {entityCatalog.length ? (
                  entityCatalog.map((entity) => (
                    <div className="work-item" key={entity.id}>
                      <strong>{entity.display_name}</strong>
                      <span>{entity.entity_type}</span>
                      <small>{entity.document_count} dokument</small>
                    </div>
                  ))
                ) : (
                  <div className="empty-state">Inga relationer hittade ännu.</div>
                )}
              </div>
            ) : null}

            {activeView==='graph'?<div className="graph-layout"><section className="work-item form-stack graph-controls"><input placeholder="Sök nod…" value={graphQuery} onChange={e=>setGraphQuery(e.target.value)}/><div className="tag-list">{['document','person','organization','category','object'].map(type=><button className={graphTypes.includes(type)?'tag-chip active':'tag-chip'} key={type} onClick={()=>setGraphTypes(types=>types.includes(type)?types.filter(x=>x!==type):[...types,type])}>{type}</button>)}</div><strong>Ny relation</strong><select value={relationDraft.sourceKey} onChange={e=>setRelationDraft({...relationDraft,sourceKey:e.target.value})}><option value="">Från nod…</option>{graphData.nodes.map(n=><option key={n.key} value={n.key}>{n.label}</option>)}</select><select value={relationDraft.relationType} onChange={e=>setRelationDraft({...relationDraft,relationType:e.target.value})}>{['ersätter','äldre version av','bilaga till','kvitto för','avtal med','tillhör','relaterad till','förnyelse av','svar på','skickad tillsammans med','bevis för','gäller objekt','utfärdad av','ändrar','förlänger','avslutar','parallell med','gäller under period'].map(x=><option key={x}>{x}</option>)}</select><select value={relationDraft.targetKey} onChange={e=>setRelationDraft({...relationDraft,targetKey:e.target.value})}><option value="">Till nod…</option>{graphData.nodes.map(n=><option key={n.key} value={n.key}>{n.label}</option>)}</select><div className="inline-control"><input type="date" value={relationDraft.validFrom} onChange={e=>setRelationDraft({...relationDraft,validFrom:e.target.value})}/><input type="date" value={relationDraft.validTo} onChange={e=>setRelationDraft({...relationDraft,validTo:e.target.value})}/></div><button className="primary-action" disabled={!relationDraft.sourceKey||!relationDraft.targetKey} onClick={handleSaveGraphRelation}>Skapa relation</button></section><section className="graph-canvas work-item"><svg viewBox="0 0 900 600" role="img" aria-label="Relationsgraf">{graphVisibleEdges.map(edge=>{const a=graphVisibleNodes.findIndex(n=>n.key===edge.source_key),b=graphVisibleNodes.findIndex(n=>n.key===edge.target_key);const ax=90+(a%7)*125,ay=70+Math.floor(a/7)*95,bx=90+(b%7)*125,by=70+Math.floor(b/7)*95;return <g key={edge.id}><line x1={ax} y1={ay} x2={bx} y2={by}/><title>{edge.relation_type} · {edge.status} · {edge.explanation}</title></g>})}{graphVisibleNodes.map((node,index)=>{const x=90+(index%7)*125,y=70+Math.floor(index/7)*95;return <g className={`graph-node type-${node.node_type}`} key={node.key} onClick={()=>{if(node.document_id){setActiveView('documents');setSelectedIndex(productionDocuments.findIndex(d=>d.id===node.document_id))}}}><circle cx={x} cy={y} r="30"/><text x={x} y={y+45} textAnchor="middle">{node.label.slice(0,18)}</text><title>{node.node_type}: {node.label}</title></g>})}</svg><small>{graphVisibleNodes.length} noder · {graphVisibleEdges.length} synliga relationer</small></section></div>:null}

            {(['people', 'companies', 'objects'] as ActiveView[]).includes(activeView) ? (
              <div className="work-panel-grid">
                {entityCatalog
                  .filter((entity) => activeView === 'people'
                    ? entity.entity_type === 'person'
                    : activeView === 'companies'
                      ? entity.entity_type === 'organization'
                      : !['person', 'organization'].includes(entity.entity_type))
                  .map((entity) => (
                    <div className="work-item" key={entity.id}>
                      <strong>{entity.display_name}</strong>
                      <span>{entity.entity_type}</span>
                      <small>{entity.document_count} källdokument</small>
                    </div>
                  ))}
                {activeView === 'objects' ? domainRecords
                  .filter((record) => ['vehicle', 'product'].includes(record.domain_type))
                  .map((record) => (
                    <div className="work-item" key={`domain-${record.id}`}>
                      <strong>{record.subject}</strong>
                      <span>{formatRecordType(record.record_type)} · {formatDomainStatus(record.actuality_status)}</span>
                      <small>{record.document_title}{record.source_page_no ? ` · källa sida ${record.source_page_no}` : ''}</small>
                      <p>{record.actuality_explanation}</p>
                    </div>
                  )) : null}
              </div>
            ) : null}

            {(['employment', 'education', 'contracts'] as ActiveView[]).includes(activeView) ? (
              <div className="work-panel-grid">
                {domainRecords
                  .filter((record) => record.domain_type === (activeView === 'contracts' ? 'contract' : activeView))
                  .map((record) => (
                    <div className="work-item" key={record.id}>
                      <strong>{record.subject}</strong>
                      <span>{formatRecordType(record.record_type)} · {formatDomainStatus(record.actuality_status)}</span>
                      <small>
                        {record.effective_from ? `Från ${record.effective_from}` : 'Startdatum saknas'}
                        {record.effective_to ? ` · till ${record.effective_to}` : ' · inget uttryckligt slutdatum'}
                      </small>
                      <small>{record.document_title}{record.source_page_no ? ` · källa sida ${record.source_page_no}` : ''}</small>
                      <p>{record.actuality_explanation}</p>
                    </div>
                  ))}
                {!domainRecords.some((record) => record.domain_type === (activeView === 'contracts' ? 'contract' : activeView))
                  ? <div className="empty-state">Inga källbundna poster i detta område ännu.</div>
                  : null}
              </div>
            ) : null}

            {(['housing','travel','authorities'] as ActiveView[]).includes(activeView)?<div className="work-panel-grid">{domainRecords.filter(record=>record.domain_type===(activeView==='authorities'?'authority':activeView)).map(record=>{let fields:{fields?:Record<string,string>}={};try{fields=JSON.parse(record.fields_json)}catch{fields={}}return <div className="work-item" key={record.id}><strong>{record.subject}</strong><span>{formatRecordType(record.record_type)} · {formatDomainStatus(record.actuality_status)}</span><small>{record.effective_from?`Från ${record.effective_from}`:'Startdatum saknas'}{record.effective_to?` · till ${record.effective_to}`:' · inget uttryckligt slutdatum'}</small>{Object.entries(fields.fields??{}).map(([key,value])=><div className="domain-field" key={key}><b>{key.replaceAll('_',' ')}</b><span>{value}</span></div>)}<small>{record.document_title}{record.source_page_no?` · källa sida ${record.source_page_no}`:''}</small><p>{record.actuality_explanation}</p>{record.domain_type==='authority'?<em>Vault återger endast uttryckliga fält och avgör inte juridisk innebörd.</em>:null}</div>})}{!domainRecords.some(record=>record.domain_type===(activeView==='authorities'?'authority':activeView))?<div className="empty-state">Inga källbundna poster i detta område ännu. Importera dokument och kör analys.</div>:null}</div>:null}

            {activeView === 'timeline' ? (
              <div className="work-panel-grid">
                {timelineEvents.length ? timelineEvents.map((event) => (
                  <div className="work-item" key={`${event.document_id}-${event.event_date}-${event.title}`}>
                    <strong>{event.event_date} · {event.title}</strong>
                    <span>{event.event_type} · {event.status}</span>
                    <small>{event.entity_names.join(', ') || 'Ingen kopplad entitet'}{event.source_page_no ? ` · källa sida ${event.source_page_no}` : ''}</small>
                    <p>{event.explanation}</p>
                  </div>
                )) : <div className="empty-state">Dokument med uttryckliga datum visas här efter import eller analys.</div>}
              </div>
            ) : null}

            {activeView==='calendar'?<div className="calendar-layout"><section className="work-item form-stack"><strong>Kalendermånad</strong><input type="month" value={calendarMonth} onChange={e=>setCalendarMonth(e.target.value)}/><div className="calendar-event-list">{calendarEvents.filter(event=>event.event_date.startsWith(calendarMonth)).map(event=><button className="calendar-event" key={event.id} onClick={()=>{if(event.document_id){setActiveView('documents');setSelectedIndex(productionDocuments.findIndex(d=>d.id===event.document_id))}}}><time>{event.event_date}</time><strong>{event.title}</strong><span>{event.event_type} · {event.status}</span><small>{event.explanation}</small></button>)}{!calendarEvents.some(event=>event.event_date.startsWith(calendarMonth))?<p>Inga händelser denna månad.</p>:null}</div></section><section className="work-item form-stack"><strong>Ny lokal kalenderpost</strong><input placeholder="Titel" value={calendarDraft.title} onChange={e=>setCalendarDraft({...calendarDraft,title:e.target.value})}/><input type="date" value={calendarDraft.eventDate} onChange={e=>setCalendarDraft({...calendarDraft,eventDate:e.target.value})}/><select value={calendarDraft.eventType} onChange={e=>setCalendarDraft({...calendarDraft,eventType:e.target.value})}><option value="reminder">Påminnelse</option><option value="renewal">Förnyelse</option><option value="service">Service</option><option value="response_deadline">Sista svarsdatum</option><option value="expiry">Utgångsdatum</option><option value="start">Startdatum</option></select><textarea placeholder="Anteckning" value={calendarDraft.note} onChange={e=>setCalendarDraft({...calendarDraft,note:e.target.value})}/><button className="primary-action" disabled={!calendarDraft.title||!calendarDraft.eventDate} onClick={handleSaveCalendarEvent}>Spara lokal händelse</button><small>Påminnelser är lokala och frivilliga. Domändatum och giltighetsperioder visas automatiskt.</small></section></div>:null}

            {activeView === 'actuality' ? (
              <div className="work-panel-grid">
                {productionDocuments.map((document) => (
                  <div className="work-item" key={document.id}>
                    <strong>{document.title}</strong>
                    <span>{document.document_date ? (document.document_date > new Date().toISOString().slice(0, 10) ? 'framtida' : 'historisk/dokumenterad') : 'osäker'}</span>
                    <small>{document.document_date ?? 'Uttryckligt dokumentdatum saknas'} · systemet använder aldrig importdatum som bevis</small>
                  </div>
                ))}
              </div>
            ) : null}

            {activeView === 'conflicts' ? (
              <div className="work-panel-grid">
                {conflicts.length ? (
                  conflicts.map((conflict) => (
                    <div className="work-item" key={`${conflict.conflict_type}-${conflict.title}`}>
                      <strong>{conflict.title}</strong>
                      <span>{conflict.severity} / {conflict.conflict_type}</span>
                      <span>Status: {conflict.status}{conflict.locked ? ' · låst beslut' : ''}</span>
                      <p>{conflict.detail}</p>
                      <small>
                        {conflict.document_ids.length
                          ? `Dokument: ${conflict.document_ids.join(', ')}`
                          : 'Gäller arkivet'}
                      </small>
                      {conflict.status === 'open' && conflict.id ? (
                        <div className="claim-actions">
                          <button className="mini-action" onClick={() => handleResolveConflict(conflict.id!, 'source_a')} type="button">Välj källa A</button>
                          <button className="mini-action" onClick={() => handleResolveConflict(conflict.id!, 'source_b')} type="button">Välj källa B</button>
                          <button className="mini-action" onClick={() => handleResolveConflict(conflict.id!, 'both')} type="button">Behåll båda</button>
                          <button className="mini-action" onClick={() => handleResolveConflict(conflict.id!, 'neither')} type="button">Avvisa båda</button>
                          <button className="mini-action" onClick={() => handleResolveConflict(conflict.id!, 'unresolved')} type="button">Lämna olöst</button>
                        </div>
                      ) : <small>{conflict.resolution ?? 'Behandlad'}</small>}
                    </div>
                  ))
                ) : (
                  <div className="empty-state">Inga konflikter hittade.</div>
                )}
              </div>
            ) : null}

            {activeView === 'reminders' ? (
              <div className="workflow-stack">
                <div className="work-item workflow-form">
                  <strong>Ny lokal påminnelse</strong>
                  <input placeholder="Titel" value={reminderDraft.title} onChange={(event) => setReminderDraft((draft) => ({ ...draft, title: event.target.value }))} />
                  <input type="date" value={reminderDraft.due_date} onChange={(event) => setReminderDraft((draft) => ({ ...draft, due_date: event.target.value }))} />
                  <textarea placeholder="Anteckning (valfritt)" value={reminderDraft.note} onChange={(event) => setReminderDraft((draft) => ({ ...draft, note: event.target.value }))} />
                  <button className="primary-action" disabled={!reminderDraft.title.trim() || !reminderDraft.due_date} onClick={handleSaveReminder} type="button">Spara påminnelse</button>
                  <small>{selectedResult ? `Kopplas till valt dokument: ${selectedResult.document.title}` : 'Skapas utan dokumentkoppling'}</small>
                </div>
                <div className="work-panel-grid">
                  {reminders.map((reminder) => (
                    <div className="work-item" key={reminder.id}>
                      <strong>{reminder.title}</strong><span>{reminder.due_date} · {reminder.status === 'active' ? 'Aktiv' : 'Klar'}</span>
                      <small>{reminder.document_title ?? 'Ingen dokumentkoppling'}</small><p>{reminder.note}</p>
                      <button className="mini-action" onClick={() => handleToggleReminder(reminder)} type="button">{reminder.status === 'active' ? 'Markera klar' : 'Återaktivera'}</button>
                    </div>
                  ))}
                </div>
              </div>
            ) : null}

            {activeView === 'automation' ? (
              <div className="workflow-stack">
                <div className="quick-actions">
                  <button className="primary-action" onClick={handleAddWatchedFolder} type="button">Lägg till mapp</button>
                  <button className="secondary-action" disabled={!watchedFolders.length || isScanningFolders} onClick={handleScanWatchedFolders} type="button">{isScanningFolders ? 'Skannar...' : 'Skanna nu'}</button>
                  <button className="secondary-action" disabled={!watchedFolders.length || isScanningFolders} onClick={handleRunBackgroundJobs} type="button">Kör bakgrundsjobb nu</button>
                  <select aria-label="Typ av systemjobb" value={systemJobType} onChange={event=>setSystemJobType(event.target.value)}><option value="archive_analysis">Full arkivanalys</option><option value="reindex">Reparera sökindex</option><option value="backup">Lokal backup</option><option value="integrity_scan">Integritetskontroll</option><option value="portable_export">Portabel export</option><option value="dirty_recompute">Uppdatera ändrade dokument</option></select>
                  <button className="primary-action" onClick={handleQueueSystemJob} type="button">Lägg i beständig kö</button>
                </div>
                {workflowError ? <p className="inline-error">{workflowError}</p> : null}
                <div className="work-panel-grid">
                  {watchedFolders.length ? watchedFolders.map((folder) => (
                    <div className="work-item" key={folder.id}><strong>{folder.enabled ? 'Aktiv mapp' : 'Pausad mapp'}</strong><span className="path-value">{folder.path}</span><small>Senast skannad: {folder.last_scanned_at ?? 'aldrig'}</small></div>
                  )) : <div className="empty-state">Ingen bevakad mapp har lagts till.</div>}
                </div>
                <h2>Senaste bakgrundsjobb</h2>
                <div className="work-panel-grid">
                  {backgroundJobs.length ? backgroundJobs.map((job) => (
                    <div className="work-item" key={job.id}>
                      <strong>{job.job_type}</strong><span>{job.status} · {job.progress_current}/{job.progress_total || '?'} · försök {job.attempts}</span>
                      <small>{job.updated_at}{job.error_code ? ` · ${job.error_code}` : ''}</small>
                      {job.result_summary?<small>{job.result_summary}</small>:null}
                      <div className="claim-actions">{['queued','running'].includes(job.status)?<button className="mini-action" onClick={()=>handleControlJob(job,'pause')} type="button">Pausa</button>:null}{job.status==='paused'?<button className="mini-action" onClick={()=>handleControlJob(job,'resume')} type="button">Återuppta</button>:null}{!['completed','cancelled'].includes(job.status)?<button className="mini-action danger-action" onClick={()=>handleControlJob(job,'cancel')} type="button">Avbryt</button>:null}{['failed','cancelled','requires_action'].includes(job.status)?<button className="mini-action" onClick={()=>handleControlJob(job,'retry')} type="button">Försök igen</button>:null}</div>
                    </div>
                  )) : <div className="empty-state">Inga bakgrundsjobb har körts ännu. Automatisk kontroll sker varje minut medan appen är öppen.</div>}
                </div>
              </div>
            ) : null}

            {activeView === 'rules' ? (
              <div className="workflow-stack">
                {workflowError ? <p className="inline-error">{workflowError}</p> : null}
                <div className="work-panel-grid">
                  <div className="work-item form-stack">
                    <strong>Ny deterministisk regel</strong>
                    <input aria-label="Regelnamn" value={ruleDraft.name} onChange={e=>setRuleDraft({...ruleDraft,name:e.target.value})}/>
                    <div className="claim-actions"><select value={ruleDraft.match_field} onChange={e=>setRuleDraft({...ruleDraft,match_field:e.target.value})}><option value="text">Dokumenttext</option><option value="title">Titel</option><option value="document_type">Dokumenttyp</option></select><select value={ruleDraft.match_operator} onChange={e=>setRuleDraft({...ruleDraft,match_operator:e.target.value})}><option value="contains">innehåller</option><option value="equals">är exakt</option><option value="starts_with">börjar med</option></select></div>
                    <input aria-label="Matchningsvärde" value={ruleDraft.match_value} onChange={e=>setRuleDraft({...ruleDraft,match_value:e.target.value})}/>
                    <div className="compound-rule-row"><select aria-label="Villkorslogik" value={ruleDraft.logic_operator} onChange={e=>setRuleDraft({...ruleDraft,logic_operator:e.target.value})}><option value="AND">AND – alla villkor</option><option value="OR">OR – något villkor</option><option value="NOT">NOT – inget villkor</option></select><select value={ruleDraft.second_field} onChange={e=>setRuleDraft({...ruleDraft,second_field:e.target.value})}><option value="text">Dokumenttext</option><option value="title">Titel</option><option value="document_type">Dokumenttyp</option></select><select value={ruleDraft.second_operator} onChange={e=>setRuleDraft({...ruleDraft,second_operator:e.target.value})}><option value="contains">innehåller</option><option value="equals">är exakt</option><option value="starts_with">börjar med</option></select><input aria-label="Andra villkorets värde" placeholder="Valfritt andra villkor" value={ruleDraft.second_value} onChange={e=>setRuleDraft({...ruleDraft,second_value:e.target.value})}/></div>
                    <div className="claim-actions"><select value={ruleDraft.action_type} onChange={e=>setRuleDraft({...ruleDraft,action_type:e.target.value})}><option value="add_tag">Lägg till tagg</option><option value="set_document_type">Sätt dokumenttyp</option><option value="archive">Flytta till arkiv</option></select><input aria-label="Åtgärdsvärde" value={ruleDraft.action_value} onChange={e=>setRuleDraft({...ruleDraft,action_value:e.target.value})}/></div>
                    <select value={ruleDraft.approval_policy} onChange={e=>setRuleDraft({...ruleDraft,approval_policy:e.target.value})}><option value="suggest">Endast föreslå</option><option value="auto">Spara automatiskt</option></select>
                    <div className="claim-actions"><button className="primary-action" onClick={handleSaveRule}>Spara regel</button><button className="secondary-action" onClick={handleRunRules}>Kör aktiva regler</button></div>
                    {ruleRun ? <small>{ruleRun.scanned_document_count} granskade · {ruleRun.matched_document_count} matchningar · {ruleRun.applied_action_count} åtgärder</small>:null}
                  </div>
                  <div className="work-item form-stack"><strong>Dokumentmall</strong><input placeholder="Mallnamn" value={templateDraft.name} onChange={e=>setTemplateDraft({...templateDraft,name:e.target.value})}/><input placeholder="Dokumenttyp" value={templateDraft.document_type} onChange={e=>setTemplateDraft({...templateDraft,document_type:e.target.value})}/><input placeholder="Kategori" value={templateDraft.category} onChange={e=>setTemplateDraft({...templateDraft,category:e.target.value})}/><input placeholder="Standardtaggar, kommaseparerade" value={templateDraft.default_tags} onChange={e=>setTemplateDraft({...templateDraft,default_tags:e.target.value})}/><input placeholder="Obligatoriska fält" value={templateDraft.required_fields} onChange={e=>setTemplateDraft({...templateDraft,required_fields:e.target.value})}/><button className="primary-action" disabled={!templateDraft.name||!templateDraft.document_type} onClick={handleSaveTemplate}>Spara mall</button></div>
                  <div className="work-item form-stack"><strong>Eget metadatafält</strong><input placeholder="Fältnamn" value={fieldDraft.name} onChange={e=>setFieldDraft({...fieldDraft,name:e.target.value})}/><select value={fieldDraft.field_type} onChange={e=>setFieldDraft({...fieldDraft,field_type:e.target.value})}><option value="text">Text</option><option value="number">Tal</option><option value="date">Datum</option><option value="boolean">Ja/nej</option></select><input placeholder="Gäller dokumenttyp eller all" value={fieldDraft.applies_to} onChange={e=>setFieldDraft({...fieldDraft,applies_to:e.target.value})}/><label><input type="checkbox" checked={fieldDraft.required} onChange={e=>setFieldDraft({...fieldDraft,required:e.target.checked})}/> Obligatoriskt</label><button className="primary-action" disabled={!fieldDraft.name} onClick={handleSaveCustomField}>Spara fält</button></div>
                  <div className="work-item form-stack"><strong>Sparad sökning</strong><input placeholder="Namn" value={searchDraft.name} onChange={e=>setSearchDraft({...searchDraft,name:e.target.value})}/><input placeholder="Sökfråga" value={searchDraft.query} onChange={e=>setSearchDraft({...searchDraft,query:e.target.value})}/><button className="primary-action" disabled={!searchDraft.name||!searchDraft.query} onClick={handleSaveSearch}>Spara sökning</button></div>
                </div>
                <h2>Aktiva regler</h2><div className="work-panel-grid">{automationRules.map(rule=><div className="work-item" key={rule.id}><strong>{rule.name} · v{rule.version}</strong><span>{rule.match_field} {rule.match_operator} “{rule.match_value}”</span><small>{rule.approval_policy} → {rule.action_type}: {rule.action_value}</small><button className="mini-action danger-action" onClick={async()=>setAutomationRules(await invoke('delete_automation_rule',{id:rule.id}))}>Ta bort</button></div>)}</div>
                <h2>Mallar, egna fält och sparade sökningar</h2><div className="work-panel-grid">{documentTemplates.map(x=><div className="work-item" key={'t'+x.id}><strong>Mall: {x.name}</strong><span>{x.document_type} · {x.category||'ingen kategori'}</span><small>Taggar: {x.default_tags||'—'} · Krav: {x.required_fields||'—'}</small></div>)}{customFields.map(x=><div className="work-item" key={'f'+x.id}><strong>Fält: {x.name}</strong><span>{x.field_type} · {x.applies_to}</span><small>{x.required?'Obligatoriskt':'Valfritt'}</small></div>)}{savedSearches.map(x=><button className="work-item clickable-card" key={'s'+x.id} onClick={()=>{setQuery(x.query);setActiveView('documents')}}><strong>Sökning: {x.name}</strong><span>{x.query}</span></button>)}</div>
                {ruleRun?.explanations.length ? <><h2>Senaste matchförklaringar</h2><div className="work-panel-grid">{ruleRun.explanations.map((x,i)=><div className="work-item" key={i}><small>{x}</small></div>)}</div></>:null}
              </div>
            ) : null}

            {activeView === 'security' ? <div className="workflow-stack">
              <div className="work-panel-grid">
                <div className="work-item form-stack"><strong>Säkerhetsläge</strong><select value={securityDraft.mode} onChange={e=>setSecurityDraft({...securityDraft,mode:e.target.value})}><option value="comfortable">Bekvämt lokalt läge</option><option value="pin">PIN-läge</option><option value="locked">Fullt låst läge</option><option value="private">Krypterad privat sektion</option></select>{securityDraft.mode!=='comfortable'?<input type="password" placeholder={securityStatus.pin_configured?'Ange nytt PIN/lösenord':'Minst 4 tecken'} value={securityDraft.pin} onChange={e=>setSecurityDraft({...securityDraft,pin:e.target.value})}/>:null}<label>Automatisk låsning (minuter)<input type="number" min="1" max="1440" value={securityDraft.autoLockMinutes} onChange={e=>setSecurityDraft({...securityDraft,autoLockMinutes:Number(e.target.value)})}/></label><label><input type="checkbox" checked={securityDraft.maskSensitive} onChange={e=>setSecurityDraft({...securityDraft,maskSensitive:e.target.checked})}/> Maskera känsliga nummer i säker export</label><button className="primary-action" onClick={handleConfigureSecurity} type="button">Spara säkerhetsläge</button>{securityStatus.mode!=='comfortable'&&vaultUnlocked?<button className="secondary-action" onClick={()=>{setSecurityCredential('');setVaultUnlocked(false)}} type="button">Lås appen nu</button>:null}</div>
                <div className="work-item"><strong>Låsta dokument</strong><span>{securityStatus.locked_document_count} dokument är låsta</span><p>Låsta original kräver korrekt PIN vid export. Maskerad export skapar en separat lokal textkopia och ändrar aldrig originalet.</p></div>
                <div className="work-item form-stack"><strong>Krypterad privat sektion</strong><span>{securityStatus.private_document_count} dokument är AES-256-krypterade på disken</span><p>Välj ett dokument i dokumentvyn och använd knappen Privat sektion. Originalet ersätts först efter verifierad kryptering och kan återställas med rätt PIN/lösenord.</p></div>
                <div className="work-item form-stack"><strong>Gästläge</strong><span>{guestMode?'Aktivt — privata och dolda dokument visas inte':'Avstängt'}</span><p>Gästläget döljer privata dokument från listor, sökresultat och förhandsvisning utan att ändra original eller metadata.</p><button className="secondary-action" onClick={()=>{setGuestMode(value=>!value);setSelectedIndex(0);setDocumentDetail(null)}} type="button">{guestMode?'Avsluta gästläge':'Starta gästläge'}</button></div>
                <div className="work-item form-stack"><strong>Windows-skyddad snabb upplåsning</strong><span>{quickUnlockEnabled?'Aktiverad för denna Windows-användare':'Avstängd'}</span><p>PIN/lösenordet lagras aldrig i klartext. Windows DPAPI krypterar det användarbundet och Vault verifierar fortfarande den vanliga hashningen vid upplåsning.</p><button className="secondary-action" disabled={!quickUnlockEnabled&&!securityCredential} onClick={handleToggleQuickUnlock} type="button">{quickUnlockEnabled?'Ta bort snabb upplåsning':'Aktivera efter upplåsning'}</button></div>
                <div className="work-item"><strong>Senaste skyddade export</strong>{lastProtectedExport?<><span>{lastProtectedExport.masked?'Maskerad text':'Originalkopia'} · {(lastProtectedExport.size_bytes/1024).toFixed(1)} KB</span><small className="path-value">{lastProtectedExport.export_path}</small><code>{lastProtectedExport.sha256.slice(0,24)}…</code></>:<p>Ingen skyddad export skapad i denna session.</p>}</div>
              </div>{workflowError?<p className="inline-error">{workflowError}</p>:null}
            </div>:null}

            {activeView === 'capture'?<div className="capture-layout"><section className="work-item form-stack"><strong>Fysisk skanner</strong><span>{scannerStatus.available?'Windows WIA är redo':'Ingen Windows WIA-enhet hittades'}</span>{scannerStatus.notes.map(note=><p key={note}>{note}</p>)}<small className="path-value">{scannerStatus.launcher_path??'Skannerguide saknas'}</small><button className="primary-action" disabled={!scannerStatus.available} onClick={handleLaunchScanner} type="button">Öppna lokal skannerguide</button><button className="secondary-action" onClick={handleImport} type="button">Importera sparad skanningsfil</button></section><section className="work-item form-stack"><strong>Spara webbsida lokalt</strong><p>Klistra in sidans HTML eller markerade text. Vault tar bort markup deterministiskt och sparar källa och hämtningstid utan extern tjänst.</p><input placeholder="Titel" value={webDraft.title} onChange={e=>setWebDraft({...webDraft,title:e.target.value})}/><input placeholder="https://källa.example/sida" value={webDraft.sourceUrl} onChange={e=>setWebDraft({...webDraft,sourceUrl:e.target.value})}/><textarea rows={12} placeholder="Klistra in HTML eller sidtext" value={webDraft.html} onChange={e=>setWebDraft({...webDraft,html:e.target.value})}/><button className="primary-action" disabled={!webDraft.html.trim()} onClick={handleImportWebSnapshot} type="button">Spara lokal webbkopia</button></section></div>:null}

            {activeView === 'analysissettings'?<div className="analysis-settings-layout"><section className="work-item form-stack"><strong>Analysomfattning</strong><select value={analysisPreferences.mode} onChange={e=>setAnalysisPreferences({...analysisPreferences,mode:e.target.value})}><option value="none">Ingen automatisk analys</option><option value="manual">Endast manuellt valda dokument</option><option value="folder">Valda mappar</option><option value="category">Valda kategorier</option><option value="entities">Valda personer/företag/objekt</option><option value="all">Hela befintliga arkivet</option><option value="future">Alla framtida importer</option><option value="all_future">Hela arkivet och framtida importer</option></select>{([['auto_ocr','Lokal OCR vid behov'],['auto_classify','Regelbaserad klassificering'],['auto_claims','Extrahera definierade claims'],['auto_relations','Föreslå relationer'],['include_locked','Inkludera låsta dokument']] as [keyof AnalysisPreferences,string][]).map(([key,label])=><label key={key}><input type="checkbox" checked={Boolean(analysisPreferences[key])} onChange={e=>setAnalysisPreferences({...analysisPreferences,[key]:e.target.checked})}/>{label}</label>)}<button className="primary-action" onClick={handleSaveAnalysisPreferences} type="button">Spara analysomfattning</button></section><section className="work-item form-stack"><strong>Undantag</strong><select value={exclusionDraft.scopeType} onChange={e=>setExclusionDraft({...exclusionDraft,scopeType:e.target.value})}><option value="document">Dokument-ID</option><option value="folder">Mapp</option><option value="category">Kategori</option><option value="document_type">Dokumenttyp</option></select><input placeholder="Värde" value={exclusionDraft.scopeValue} onChange={e=>setExclusionDraft({...exclusionDraft,scopeValue:e.target.value})}/><input placeholder="Orsak" value={exclusionDraft.reason} onChange={e=>setExclusionDraft({...exclusionDraft,reason:e.target.value})}/><button className="secondary-action" onClick={handleAddAnalysisExclusion} type="button">Lägg till undantag</button>{analysisExclusions.map(item=><div className="exclusion-row" key={item.id}><span><strong>{item.scope_type}</strong> · {item.scope_value}<small>{item.reason}</small></span><button onClick={async()=>setAnalysisExclusions(await invoke('delete_analysis_exclusion',{id:item.id}))} type="button">Ta bort</button></div>)}</section></div>:null}

            {activeView === 'collections' ? <div className="collections-layout"><section><div className="collection-grid">{collections.map(collection=><button className={`collection-card${selectedCollectionId===collection.id?' active':''}`} key={collection.id} onClick={()=>void handleOpenCollection(collection.id)} style={{borderColor:collection.color}} type="button"><span className="collection-color" style={{background:collection.color}}/><strong>{collection.is_pinned?'◆ ':''}{collection.name}</strong><small>{collection.document_count} dokument</small><p>{collection.description||'Ingen beskrivning'}</p></button>)}</div>{selectedCollectionId?<div className="work-item"><strong>Dokument i {collections.find(c=>c.id===selectedCollectionId)?.name}</strong><div className="collection-document-list">{productionDocuments.filter(document=>collectionDocumentIds.includes(document.id)).map(document=><button key={document.id} onClick={()=>{setActiveView('documents');setSelectedIndex(productionDocuments.findIndex(item=>item.id===document.id))}} type="button"><strong>{document.title}</strong><span>{document.document_type}</span></button>)}{!collectionDocumentIds.length?<p>Samlingen är tom. Lägg till ett valt dokument från dokumentpanelen.</p>:null}</div></div>:null}</section><section className="work-item form-stack"><strong>Ny eller uppdaterad samling</strong><input placeholder="Namn" value={collectionDraft.name} onChange={e=>setCollectionDraft({...collectionDraft,name:e.target.value})}/><textarea placeholder="Beskrivning" value={collectionDraft.description} onChange={e=>setCollectionDraft({...collectionDraft,description:e.target.value})}/><label>Färg <input type="color" value={collectionDraft.color} onChange={e=>setCollectionDraft({...collectionDraft,color:e.target.value})}/></label><label><input type="checkbox" checked={collectionDraft.isPinned} onChange={e=>setCollectionDraft({...collectionDraft,isPinned:e.target.checked})}/> Fäst samlingen</label><button className="primary-action" onClick={handleSaveCollection} type="button">Spara samling</button></section></div>:null}
            {activeView === 'folders'?<div className="collections-layout"><section className="collection-grid">{folders.map(item=><div className="collection-card" key={item.id} style={{borderColor:item.color}}><span className="collection-color" style={{background:item.color}}/><strong>{item.is_pinned?'◆ ':''}{item.is_locked?'🔒 ':''}{item.name}</strong><small>{item.document_count} dokument</small><button className="mini-action" disabled={!securityStatus.pin_configured||!vaultUnlocked} onClick={()=>void handleOrganizerLock('folder',item)} type="button">{item.is_locked?'Lås upp mapp':'Lås mapp'}</button></div>)}</section><section className="work-item form-stack"><strong>Ny eller uppdaterad mapp</strong><input placeholder="Mappnamn" value={folderDraft.name} onChange={e=>setFolderDraft({...folderDraft,name:e.target.value})}/><label>Färg <input type="color" value={folderDraft.color} onChange={e=>setFolderDraft({...folderDraft,color:e.target.value})}/></label><label><input type="checkbox" checked={folderDraft.isPinned} onChange={e=>setFolderDraft({...folderDraft,isPinned:e.target.checked})}/> Fäst mappen</label><button className="primary-action" disabled={!folderDraft.name.trim()} onClick={handleSaveFolder}>Spara mapp</button><small>Låsta mappar kräver aktivt PIN-/lösenordsläge och skyddar alla anslutna original vid öppning och export.</small></section></div>:null}
            {activeView === 'categories'?<div className="collections-layout"><section className="collection-grid">{categories.map(item=><div className="collection-card" key={item.id} style={{borderColor:item.color}}><span className="collection-color" style={{background:item.color}}/><strong>{item.is_pinned?'◆ ':''}{item.is_locked?'🔒 ':''}{item.name}</strong><small>{item.document_count} dokument</small><p>{item.description||'Ingen beskrivning'}</p><button className="mini-action" disabled={!securityStatus.pin_configured||!vaultUnlocked} onClick={()=>void handleOrganizerLock('category',item)} type="button">{item.is_locked?'Lås upp kategori':'Lås kategori'}</button></div>)}</section><section className="work-item form-stack"><strong>Ny eller uppdaterad kategori</strong><input placeholder="Kategorinamn" value={categoryDraft.name} onChange={e=>setCategoryDraft({...categoryDraft,name:e.target.value})}/><textarea placeholder="Beskrivning" value={categoryDraft.description} onChange={e=>setCategoryDraft({...categoryDraft,description:e.target.value})}/><label>Färg <input type="color" value={categoryDraft.color} onChange={e=>setCategoryDraft({...categoryDraft,color:e.target.value})}/></label><label><input type="checkbox" checked={categoryDraft.isPinned} onChange={e=>setCategoryDraft({...categoryDraft,isPinned:e.target.checked})}/> Fäst kategorin</label><button className="primary-action" disabled={!categoryDraft.name.trim()} onClick={handleSaveCategory}>Spara kategori</button><small>Låsta kategorier kräver aktivt PIN-/lösenordsläge och skyddar alla anslutna original vid öppning och export.</small></section></div>:null}
            {activeView === 'searchhistory'?<div className="search-history-layout"><section className="work-item form-stack"><strong>Privat sökhistorik</strong><label><input type="checkbox" checked={searchHistoryPreferences.enabled} onChange={e=>setSearchHistoryPreferences({...searchHistoryPreferences,enabled:e.target.checked})}/> Spara sökningar lokalt</label><label>Behåll ej fästa sökningar i dagar<input type="number" min="1" max="3650" value={searchHistoryPreferences.retention_days} onChange={e=>setSearchHistoryPreferences({...searchHistoryPreferences,retention_days:Number(e.target.value)})}/></label><div className="claim-actions"><button className="primary-action" onClick={handleSaveSearchHistoryPreferences} type="button">Spara policy</button><button className="secondary-action" onClick={handleExportSearchHistory} type="button">Exportera JSON</button><button className="mini-action danger" onClick={handleClearSearchHistory} type="button">Rensa ej fästa</button></div>{searchHistoryMessage?<small className="path-value">{searchHistoryMessage}</small>:null}<small>Fästa sökningar undantas från automatisk rensning.</small></section><section className="work-panel-grid">{searchHistory.map(item=><article className="work-item" key={item.id}><button className="clickable-card search-history-open" onClick={()=>{setQuery(item.query);setActiveView('documents')}} type="button"><strong>{item.pinned?'◆ ':''}{item.query}</strong><span>{item.result_count} träffar</span><small>{item.executed_at}</small></button><button className="mini-action" onClick={async()=>setSearchHistory(await invoke('set_search_history_pinned',{id:item.id,pinned:!item.pinned}))} type="button">{item.pinned?'Lossa':'Fäst'}</button></article>)}{!searchHistory.length?<div className="work-item"><strong>Ingen sökhistorik ännu</strong><p>{searchHistoryPreferences.enabled?'Skriv en sökning och tryck Enter för att spara den lokalt.':'Historiken är avstängd.'}</p></div>:null}</section></div>:null}

            {activeView === 'notes'?<div className="notes-workspace"><section className="notes-list"><div className="work-item form-stack"><strong>Sök anteckningar</strong><input placeholder="Titel, text, tagg eller kodord" value={noteSearch} onChange={e=>{setNoteSearch(e.target.value);void handleSearchNotes(e.target.value)}}/></div>{vaultNotes.map(note=><article className="work-item note-card" key={note.id}><button className="clickable-card search-history-open" onClick={()=>void handleEditVaultNote(note)} type="button"><strong>{note.title}</strong><span>{note.kind} · {note.target_type}{note.target_id?` #${note.target_id}`:''}</span><p>{note.body.slice(0,180)}</p><small>Version {note.version_no} · {note.updated_at}</small></button><div className="tag-list">{note.tags.map(tag=><span className="tag-chip" key={tag}>{tag}</span>)}</div><button className="mini-action danger" onClick={async()=>setVaultNotes(await invoke('delete_vault_note',{id:note.id}))} type="button">Ta bort</button></article>)}{!vaultNotes.length?<div className="empty-state">Inga egna anteckningar matchar. Anteckningar visas aldrig som dokumenterade fakta.</div>:null}</section><section className="work-item form-stack note-editor"><strong>{vaultNoteDraft.id?'Redigera anteckning':'Ny anteckning'}</strong><input placeholder="Titel" value={vaultNoteDraft.title} onChange={e=>setVaultNoteDraft({...vaultNoteDraft,title:e.target.value})}/><div className="note-editor-row"><select value={vaultNoteDraft.kind} onChange={e=>setVaultNoteDraft({...vaultNoteDraft,kind:e.target.value})}><option value="note">Anteckning</option><option value="todo">Checklista</option><option value="decision">Beslut</option></select><select value={vaultNoteDraft.targetType} onChange={e=>setVaultNoteDraft({...vaultNoteDraft,targetType:e.target.value,targetId:''})}><option value="vault">Hela Vault</option><option value="document">Dokument</option><option value="person">Person</option><option value="company">Företag</option><option value="object">Objekt</option><option value="category">Kategori</option><option value="event">Händelse</option><option value="claim">Claim</option><option value="conflict">Konflikt</option></select>{vaultNoteDraft.targetType!=='vault'?<input type="number" min="1" placeholder="Länkat ID" value={vaultNoteDraft.targetId} onChange={e=>setVaultNoteDraft({...vaultNoteDraft,targetId:e.target.value})}/>:null}</div><select value="" onChange={e=>{const template=noteTemplates.find(t=>t.id===Number(e.target.value));if(template)setVaultNoteDraft({...vaultNoteDraft,body:template.body})}}><option value="">Infoga mall…</option>{noteTemplates.map(template=><option key={template.id} value={template.id}>{template.name}</option>)}</select><textarea className="markdown-editor" rows={16} placeholder={'Markdown: # rubrik, - [ ] checklista, [länk](vault://document/1), `kod` och tabeller'} value={vaultNoteDraft.body} onChange={e=>setVaultNoteDraft({...vaultNoteDraft,body:e.target.value})}/><div className="note-editor-row"><input placeholder="Taggar, kommaseparerade" value={vaultNoteDraft.tags} onChange={e=>setVaultNoteDraft({...vaultNoteDraft,tags:e.target.value})}/><input placeholder="Kodord, kommaseparerade" value={vaultNoteDraft.codeWords} onChange={e=>setVaultNoteDraft({...vaultNoteDraft,codeWords:e.target.value})}/></div><button className="primary-action" disabled={!vaultNoteDraft.title.trim()||!vaultNoteDraft.body.trim()||(vaultNoteDraft.targetType!=='vault'&&!vaultNoteDraft.targetId)} onClick={handleSaveVaultNote} type="button">{vaultNoteDraft.id?'Spara ny version':'Skapa anteckning'}</button>{vaultNoteDraft.id?<button className="secondary-action" onClick={()=>{setVaultNoteDraft({id:null,title:'',body:'',kind:'note',targetType:'vault',targetId:'',tags:'',codeWords:''});setNoteVersions([])}} type="button">Ny anteckning</button>:null}<div className="markdown-preview"><strong>Markdown-källa</strong><pre>{vaultNoteDraft.body||'Skriv Markdown för att förhandsgranska.'}</pre></div>{noteVersions.length?<details><summary>Versionshistorik ({noteVersions.length})</summary>{noteVersions.map(version=><button className="note-history-row" key={version.version_no} onClick={()=>setVaultNoteDraft({...vaultNoteDraft,title:version.title,body:version.body})} type="button"><strong>Version {version.version_no}</strong><small>{version.created_at}</small></button>)}</details>:null}<small>Egna anteckningar är separat arbetsdata och påverkar aldrig claims eller aktualitetsbedömning.</small></section></div>:null}

            {activeView === 'themes' ? <div className="theme-editor-layout">
              <section className="theme-gallery"><div className="section-heading"><div><h2>Färdiga teman</h2><p>Alla teman lagras lokalt och kan aktiveras, dupliceras eller exporteras.</p></div><button className="mini-action" onClick={handleImportTheme} type="button">Importera theme-fil</button></div><div className="theme-card-grid">{themeProfiles.map(theme=><article className={`theme-card${theme.is_active?' active':''}`} key={theme.id} style={{background:theme.surface_main,color:theme.text_primary,borderColor:theme.accent}}><div className="theme-swatch" style={{background:`linear-gradient(135deg,${theme.surface_sidebar} 0 45%,${theme.surface_raised} 45% 75%,${theme.accent} 75%)`}}/><strong>{theme.name}</strong><span>{theme.base_mode==='dark'?'Mörk':'Ljus'} · {theme.density} · v{theme.version}</span><div className="claim-actions"><button className="mini-action" onClick={()=>handleActivateTheme(theme.id)} type="button">{theme.is_active?'Aktivt':'Använd'}</button><button className="mini-action" onClick={()=>handleDuplicateTheme(theme)} type="button">Duplicera</button><button className="mini-action" onClick={()=>handleExportTheme(theme)} type="button">Exportera</button></div></article>)}</div></section>
              <section className="work-item form-stack theme-form">
                <strong>Skapa eget tema</strong>
                <input value={themeDraft.name} onChange={e=>setThemeDraft({...themeDraft,name:e.target.value})} placeholder="Temanamn"/>
                <select value={themeDraft.base_mode} onChange={e=>setThemeDraft({...themeDraft,base_mode:e.target.value as 'light'|'dark'})}><option value="dark">Mörk bas</option><option value="light">Ljus bas</option></select>
                <div className="color-field-grid">{([['Accent','accent'],['Bakgrund','surface_main'],['Sidopanel','surface_sidebar'],['Panel','surface_raised'],['Text','text_primary'],['Sekundär text','text_secondary'],['Kantlinje','border_color']] as [string,keyof typeof themeDraft][]).map(([label,key])=><label key={key}>{label}<input type="color" value={String(themeDraft[key])} onChange={e=>setThemeDraft({...themeDraft,[key]:e.target.value})}/></label>)}</div>
                <label>Hörnradie {themeDraft.radius_px}px<input type="range" min="0" max="30" value={themeDraft.radius_px} onChange={e=>setThemeDraft({...themeDraft,radius_px:Number(e.target.value)})}/></label>
                <label>Textstorlek {Math.round(themeDraft.font_scale*100)} %<input type="range" min="0.8" max="1.4" step="0.05" value={themeDraft.font_scale} onChange={e=>setThemeDraft({...themeDraft,font_scale:Number(e.target.value)})}/></label>
                <label>Radavstånd {themeDraft.line_height.toFixed(2)}<input type="range" min="1" max="2.2" step=".05" value={themeDraft.line_height} onChange={e=>setThemeDraft({...themeDraft,line_height:Number(e.target.value)})}/></label>
                <label>Skuggstyrka {Math.round(themeDraft.shadow_strength*100)} %<input type="range" min="0" max="1" step=".05" value={themeDraft.shadow_strength} onChange={e=>setThemeDraft({...themeDraft,shadow_strength:Number(e.target.value)})}/></label>
                <label>Panelopacitet {Math.round(themeDraft.transparency*100)} %<input type="range" min=".55" max="1" step=".05" value={themeDraft.transparency} onChange={e=>setThemeDraft({...themeDraft,transparency:Number(e.target.value)})}/></label>
                <label>Blur {themeDraft.blur_px}px<input type="range" min="0" max="30" value={themeDraft.blur_px} onChange={e=>setThemeDraft({...themeDraft,blur_px:Number(e.target.value)})}/></label>
                <label>Animationshastighet {themeDraft.animation_speed.toFixed(2)}×<input type="range" min=".25" max="2" step=".05" value={themeDraft.animation_speed} onChange={e=>setThemeDraft({...themeDraft,animation_speed:Number(e.target.value)})}/></label>
                <select value={themeDraft.density} onChange={e=>setThemeDraft({...themeDraft,density:e.target.value as ThemeProfile['density']})}><option value="compact">Kompakt</option><option value="comfortable">Bekväm</option><option value="spacious">Rymlig</option></select>
                <select value={themeDraft.motion} onChange={e=>setThemeDraft({...themeDraft,motion:e.target.value as ThemeProfile['motion']})}><option value="normal">Normala animationer</option><option value="reduced">Minskade animationer</option><option value="off">Animationer av</option></select>
                <select value={themeDraft.font_family} onChange={e=>setThemeDraft({...themeDraft,font_family:e.target.value as ThemeProfile['font_family']})}><option value="system">Systemtypsnitt</option><option value="serif">Serif</option><option value="mono">Monospace</option></select>
                <select value={themeDraft.preview_ratio} onChange={e=>setThemeDraft({...themeDraft,preview_ratio:e.target.value as ThemeProfile['preview_ratio']})}><option value="document">Dokumentproportion</option><option value="square">Kvadratisk förhandsvisning</option><option value="wide">Bred förhandsvisning</option></select>
                <select value={themeDraft.schedule_mode} onChange={e=>setThemeDraft({...themeDraft,schedule_mode:e.target.value as ThemeProfile['schedule_mode']})}><option value="manual">Manuellt tema</option><option value="day_night">Ljust dagtid, mörkt kvällstid</option><option value="windows">Följ Windows</option></select>
                <label><input type="checkbox" checked={themeDraft.follow_windows} onChange={e=>setThemeDraft({...themeDraft,follow_windows:e.target.checked})}/> Följ Windows färgläge</label>
                <input placeholder="Lokal bakgrundsbild (sökväg, valfritt)" value={themeDraft.background_image} onChange={e=>setThemeDraft({...themeDraft,background_image:e.target.value})}/>
                <div className="theme-live-preview" style={{background:themeDraft.surface_main,color:themeDraft.text_primary,borderColor:themeDraft.border_color,borderRadius:themeDraft.radius_px,opacity:themeDraft.transparency,fontFamily:themeDraft.font_family==='serif'?'Georgia,serif':themeDraft.font_family==='mono'?'Consolas,monospace':'inherit',lineHeight:themeDraft.line_height,boxShadow:`0 10px 30px rgb(0 0 0 / ${themeDraft.shadow_strength})`}}><strong>Direkt förhandsgranskning</strong><span style={{color:themeDraft.text_secondary}}>Panel, text, hörn och skugga</span><button style={{background:themeDraft.accent}}>Exempelknapp</button></div>
                <button className="primary-action" onClick={handleSaveTheme} type="button">Spara eget tema</button>
                {themeMessage?<small className="path-value">{themeMessage}</small>:null}
              </section>
            </div>:null}

            {activeView === 'backup' ? (
              <div className="work-panel-grid">
                <div className="work-item form-stack restore-wizard">
                  <strong>Guide för säker återställning</strong>
                  <ol>
                    <li className={backups.length?'done':''}><b>1. Välj backup</b><span>{backups[0]?.backup_root??'Skapa eller välj en backup'}</span></li>
                    <li className={backupValidation?.ok?'done':''}><b>2. Verifiera manifest, SHA-256 och databas</b><button className="mini-action" disabled={!backups.length||isValidatingBackup} onClick={handleValidateLatestBackup} type="button">{isValidatingBackup?'Verifierar…':'Verifiera senaste'}</button></li>
                    <li className={restoreTest?.ok?'done':''}><b>3. Visa innehåll och prova isolerat</b><span>{backupValidation?`${backupValidation.document_count} dokument · ${backupValidation.file_count} filer · SQLite ${backupValidation.database_integrity}`:'Väntar på verifiering'}</span><button className="mini-action" disabled={!backupValidation?.ok} onClick={handleTestLatestRestore} type="button">Kör isolerat prov</button></li>
                    <li><b>4. Full restore med återställningspunkt</b><button className="primary-action" disabled={!backupValidation?.ok||!restoreTest?.ok||isRestoringBackup} onClick={handleRestoreLatestBackup} type="button">{isRestoringBackup?'Återställer…':'Genomför full restore'}</button></li>
                    <li className={lastRestore&&healthReport?.ok?'done':''}><b>5. Verifiera resultat och index</b><span>{lastRestore?`${lastRestore.document_count} dokument · ${lastRestore.file_count} filer · återställningspunkt ${lastRestore.pre_restore_backup_root}`:'Ingen restore körd i sessionen'}</span>{lastRestore?<button className="mini-action" onClick={handleReindexSearch} type="button">Bygg om sökindex</button>:null}</li>
                  </ol>
                  {backupValidation?.warnings.map(warning=><small key={warning}>{warning}</small>)}
                  {restoreError?<p className="inline-error">{restoreError}</p>:null}
                </div>
                <div className="work-item form-stack"><strong>Schemalagd lokal backup</strong><label><input type="checkbox" checked={backupPolicy.enabled} onChange={e=>setBackupPolicy({...backupPolicy,enabled:e.target.checked})}/> Kör automatiskt</label><label><input type="checkbox" checked={backupPolicy.paused} onChange={e=>setBackupPolicy({...backupPolicy,paused:e.target.checked})}/> Pausad</label><label>Intervall i timmar<input type="number" min="1" max="8760" value={backupPolicy.interval_hours} onChange={e=>setBackupPolicy({...backupPolicy,interval_hours:Number(e.target.value)})}/></label><label>Backuptyp<select value={backupPolicy.backup_mode} onChange={e=>setBackupPolicy({...backupPolicy,backup_mode:e.target.value})}><option value="incremental">Inkrementell, självständig</option><option value="full">Full kopia</option></select></label><label>Destination<input readOnly placeholder="Standard: Vaults lokala datamapp" value={backupPolicy.destination_path??''}/></label><div className="claim-actions"><button className="secondary-action" onClick={handleChooseBackupDestination} type="button">Välj disk eller nätverksmapp</button>{backupPolicy.destination_path?<button className="mini-action" onClick={()=>setBackupPolicy({...backupPolicy,destination_path:null})} type="button">Använd standardmapp</button>:null}</div><label>Undanta dokument-ID:n<input placeholder="12, 45" value={backupExclusions} onChange={e=>setBackupExclusions(e.target.value)}/></label><div className="claim-actions"><button className="primary-action" onClick={handleSaveBackupPolicy} type="button">Spara backuppolicy</button><button className="secondary-action" disabled={isCreatingBackup||backupPolicy.paused} onClick={handleRunConfiguredBackup} type="button">Kör enligt policy nu</button></div><small>{backupPolicy.last_run_epoch_seconds?`Senast körd ${new Date(backupPolicy.last_run_epoch_seconds*1000).toLocaleString('sv-SE')}`:'Ingen policykörning ännu'}</small><small>{backupPolicy.destination_path?`Extra kopia: ${backupPolicy.destination_path}`:'Backup lagras i Vaults lokala datamapp.'}</small><small>Inkrementella set hårdlänkar oförändrade filer när Windows-filsystemet tillåter det men kan återställas fristående.</small></div>
                <div className="work-item">
                  <strong>Backup</strong>
                  <span>{backups.length} lokala backup-set</span>
                  <div className="claim-actions">
                    <button className="mini-action" onClick={handleCreateBackup} type="button">
                      Skapa backup
                    </button>
                    <button className="mini-action" onClick={handleValidateLatestBackup} type="button">
                      Validera senaste
                    </button>
                    <button className="mini-action" onClick={handleRestoreLatestBackup} type="button">
                      Återställ senaste
                    </button>
                    <button className="mini-action" disabled={!backups.length} onClick={handleTestLatestRestore} type="button">Testa återställning isolerat</button>
                  </div>
                  <small className="path-value">{backups[0]?.backup_root ?? 'Ingen backup finns'}</small>
                  {backups[0]?<code>{backups[0].backup_mode} · {backups[0].copied_file_count} kopierade · {backups[0].linked_file_count} hårdlänkade</code>:null}
                  {restoreTest?<code>{restoreTest.ok?'Godkänt':'Fel'} · SQLite {restoreTest.database_integrity} · {restoreTest.document_count} dokument · {restoreTest.missing_file_count} saknade filer</code>:null}
                </div>
                <div className="work-item form-stack">
                  <strong>Lösenordsskyddad backup</strong>
                  <span>AES-256-krypterat lokalt arkiv med databas, originalfiler och manifest.</span>
                  <input type="password" minLength={8} placeholder="Minst 8 tecken" value={backupPassword} onChange={event=>setBackupPassword(event.target.value)}/>
                  <button className="mini-action" disabled={backupPassword.length<8} onClick={handleCreatePasswordBackup} type="button">Skapa skyddad backup</button>
                  {protectedBackup?<><small className="path-value">{protectedBackup.archive_path}</small><code>{protectedBackup.sha256.slice(0,24)}… · {protectedBackup.entry_count} poster</code></>:null}
                  <button className="secondary-action" onClick={handleChoosePasswordBackup} type="button">Välj krypterad backup för återställning</button>
                  {passwordRestorePath?<small className="path-value">{passwordRestorePath}</small>:null}
                  <button className="mini-action" disabled={!passwordRestorePath||backupPassword.length<8} onClick={handleRestorePasswordBackup} type="button">Återställ krypterad backup</button>
                </div>
                <div className="work-item form-stack">
                  <strong>Flyttbart Vault-arkiv</strong>
                  <span>Öppet dokumenterat paket med databas, originalfiler, JSON-manifest och CSV-lista. Kan återställas på en annan dator.</span>
                  <div className="claim-actions"><button className="mini-action" onClick={handleCreatePortableArchive} type="button">Exportera .vaultarchive</button><button className="mini-action" onClick={handleRestorePortableArchive} type="button">Importera på denna dator</button></div>
                  {portableArchive?<><small className="path-value">{portableArchive.archive_path}</small><code>{portableArchive.document_count} dokument · {portableArchive.file_count} filer · SHA-256 {portableArchive.sha256.slice(0,20)}…</code></>:null}
                  {restoreError?<p className="inline-error">{restoreError}</p>:null}
                </div>
                <div className="work-item form-stack">
                  <strong>Fristående dataexport</strong>
                  <span>Exporterar metadata, claims, relationer, historik, kategorier, regler, teman, kodord, sökningar och layout med stabila ID:n.</span>
                  <label>Format<select value={dataExportDraft.format} onChange={event=>setDataExportDraft({...dataExportDraft,format:event.target.value})}><option value="package">Paket (.vaultzip)</option><option value="json">JSON</option><option value="csv">CSV-mapp</option></select></label>
                  <label>Mappstruktur<select value={dataExportDraft.folderLayout} onChange={event=>setDataExportDraft({...dataExportDraft,folderLayout:event.target.value})}><option value="vault">Vault-struktur</option><option value="flat">Platt</option><option value="type">Efter dokumenttyp</option></select></label>
                  <label><input type="checkbox" checked={dataExportDraft.includeOriginals} onChange={event=>setDataExportDraft({...dataExportDraft,includeOriginals:event.target.checked})}/> Inkludera originaldokument</label>
                  <button className="primary-action" onClick={handleCreateDataExport} type="button">Skapa dataexport</button>
                  <div className="claim-actions"><button className="secondary-action" onClick={handleExportUserProfile} type="button">Exportera användarprofil</button><button className="secondary-action" onClick={handleImportUserProfile} type="button">Importera användarprofil</button></div>
                  {dataExport?<><small className="path-value">{dataExport.export_path}</small><code>{dataExport.row_count} rader · {dataExport.file_count} original · {dataExport.table_count} tabeller</code></>:null}
                </div>
                <div className="work-item">
                  <strong>Arkivhälsa</strong>
                  <span>
                    {healthReport
                      ? `${healthReport.document_count} dokument, ${healthReport.missing_file_count} saknade filer`
                      : 'Inte kontrollerad'}
                  </span>
                  <button className="mini-action" onClick={handleCheckHealth} type="button">
                    Kontrollera arkiv
                  </button>
                </div>
              </div>
            ) : null}

            {activeView==='integrity'?<div className="test-center-layout"><section className="work-item form-stack"><strong>Filintegritet och dubbletter</strong><p>Kontrollerar SHA-256 och hittar exakta filer, nästan identisk text, korsformat, dubbla sidor och liknande skanningar med perceptuell bildhash. Ingenting raderas automatiskt.</p><button className="primary-action" onClick={handleIntegrityScan} type="button">Kör full kontroll</button>{integrityReport?<div className="diagnostic-grid"><span>{integrityReport.checked_files} kontrollerade</span><span>{integrityReport.intact_files} intakta</span><span>{integrityReport.missing_files} saknas</span><span>{integrityReport.changed_files} ändrade</span><span>{integrityReport.duplicate_groups} förslag</span></div>:null}{healthError?<p className="inline-error">{healthError}</p>:null}</section><section className="work-panel-grid">{integrityReport?.candidates.map(candidate=><article className="work-item" key={`${candidate.primary_document_id}-${candidate.secondary_document_id}`}><strong>{candidate.primary_title}</strong><span>jämförd med {candidate.secondary_title}</span><code>{candidate.confidence}% · {candidate.match_kind}</code><p>{candidate.explanation}</p><div className="claim-actions"><button className="mini-action" onClick={()=>handleDuplicateDecision(candidate,'primary_selected')}>Välj första som primär</button><button className="mini-action" onClick={()=>handleDuplicateDecision(candidate,'keep_both')}>Behåll båda</button><button className="mini-action" onClick={()=>handleDuplicateDecision(candidate,'not_duplicate')}>Inte dubblett</button></div>{candidate.decision?<small>Beslut: {candidate.decision}</small>:null}</article>)}{integrityReport&&!integrityReport.candidates.length?<div className="empty-state">Inga dubblettkandidater hittades.</div>:null}</section></div>:null}

            {activeView==='plugins'?<div className="test-center-layout"><section className="work-item form-stack"><strong>Isolerade lokala plugins</strong><p>Vault kör bara deklarativa JSON-manifest. Plugins får aldrig köra program, använda nätverk, läsa godtyckliga filer eller anropa AI. Installation aktiverar ingenting förrän du granskat dataåtkomst och behörigheter.</p><button className="primary-action" onClick={handleInstallPlugin} type="button">Installera lokalt manifest</button><small>Adapter-API 1.0 stöder OCR, importör, metadataextraktor, sökparser, exportör, dashboard-widget och domänmodell som strikt deklarerade kontrakt. Varje manifest låses med SHA-256 och inaktiveras vid ändring.</small>{pluginError?<p className="inline-error">{pluginError}</p>:null}{pluginRun?<code>{pluginRun.explanation}</code>:null}</section><section className="work-panel-grid">{plugins.map(plugin=><article className="work-item form-stack" key={plugin.id}><strong>{plugin.name} <small>v{plugin.version}</small></strong><span>{plugin.id} · API {plugin.api_version} · {plugin.adapter_kind}</span><p>{plugin.description||'Ingen beskrivning'}</p><div><b>Behörigheter</b><div className="tag-list">{plugin.capabilities.map(value=><span className="tag-chip" key={value}>{value}</span>)}</div></div><div><b>Dataåtkomst</b><div className="tag-list">{plugin.data_access.map(value=><span className="tag-chip" key={value}>{value}</span>)}</div></div><small>Tillit: {plugin.signature_status} · SHA-256 {plugin.manifest_sha256.slice(0,12)}… · gräns {plugin.resource_limit}</small><span>{plugin.enabled&&plugin.approved?'Aktiv och uttryckligen godkänd':'Inaktiverad'}</span>{plugin.last_result?<small>{plugin.last_result}</small>:null}<div className="claim-actions"><button className="mini-action" onClick={()=>handleSetPluginEnabled(plugin)}>{plugin.enabled?'Inaktivera':'Godkänn och aktivera'}</button><button className="mini-action" disabled={!plugin.enabled} onClick={()=>handleRunPlugin(plugin)}>Kör lokalt</button><button className="mini-action danger" onClick={()=>handleRemovePlugin(plugin)}>Ta bort</button></div></article>)}{!plugins.length?<div className="empty-state">Inga plugins installerade. Kärnfunktionerna kräver aldrig plugins.</div>:null}</section><section className="work-item form-stack"><strong>Externa källor – lokal, frivillig överlämning</strong><p>OAuth-nycklar lagras inte i Vault. Välj i stället en lokal export- eller synkmapp från Google Drive, OneDrive, Gmail eller Outlook. Vault visar filer först och importerar bara exakt markerade filer efter bekräftelse.</p><div className="work-panel-grid">{externalImportSources.map(source=><article className="work-item form-stack" key={source.provider}><strong>{source.display_name}</strong><small>{source.local_staging_path||'Ingen mapp vald'} · {source.enabled?'aktiverad':'avstängd som standard'}</small>{source.last_result?<span>{source.last_result}</span>:null}<div className="claim-actions"><button className="mini-action" onClick={()=>handleConfigureExternalImport(source)}>Välj mapp och aktivera</button><button className="mini-action" disabled={!source.enabled} onClick={()=>handlePreviewExternalImport(source)}>Förhandsgranska</button></div></article>)}</div>{externalImportPreview?<div className="form-stack"><strong>{externalImportPreview.files.length} filer · {(externalImportPreview.total_size_bytes/1048576).toFixed(1)} MB</strong>{externalImportPreview.files.slice(0,100).map(file=><label className="checkbox-row" key={file.path}><input checked={externalImportSelection.includes(file.path)} onChange={event=>setExternalImportSelection(current=>event.target.checked?[...current,file.path]:current.filter(value=>value!==file.path))} type="checkbox"/><span>{file.relative_path}</span></label>)}{externalImportPreview.files.length>100?<small>De första 100 visas; resterande är valda som standard.</small>:null}<button className="primary-action" disabled={!externalImportSelection.length} onClick={handleImportExternalSelection}>Importera {externalImportSelection.length} valda</button></div>:null}</section></div>:null}

            {activeView === 'testcenter' ? (
              <div className="test-center-layout"><div className="testlab-banner">TESTMILJÖ – INGA RIKTIGA DOKUMENT · separat databas och filförvaring</div><section className="test-control-grid"><div className="work-item form-stack"><strong>Modultester</strong><select value={testModule} onChange={e=>setTestModule(e.target.value)}><option value="all">Alla moduler</option><option value="search">Sökning och rankning</option><option value="storage">Databas, filer och backup</option><option value="analysis">Claims, konflikter och relationer</option><option value="ocr">OCR-adapter</option></select><button className="primary-action" disabled={isRunningTestCenter} onClick={testModule==='all'?handleRunTestCenter:handleRunTestModule} type="button">{isRunningTestCenter?'Kör…':'Kör vald modul'}</button><span>{testCenterReport?`${testCenterReport.passed} godkända · ${testCenterReport.failed} misslyckade`:'Inte kört'}</span></div><div className="work-item form-stack"><strong>Skaltest</strong><p>Genererar endast tydligt märkt syntetisk metadata i Test Lab.</p><div className="claim-actions">{[100,1000,10000,50000].map(count=><button className="mini-action" disabled={isRunningTestCenter} key={count} onClick={()=>handleGenerateTestScale(count)} type="button">{count.toLocaleString('sv-SE')}</button>)}</div>{testLabScale?<code>{testLabScale.total_documents.toLocaleString('sv-SE')} dokument · import {testLabScale.elapsed_ms} ms · sök {testLabScale.search_elapsed_ms} ms · {(testLabScale.database_bytes/1048576).toFixed(1)} MB</code>:null}</div><div className="work-item form-stack"><strong>Felsimulering</strong><div className="claim-actions">{[['ocr','OCR-fel'],['database','Databasfel'],['file','Filfel'],['interrupt','Avbrott'],['restore','Återställning']].map(([value,label])=><button className="mini-action" key={value} onClick={()=>handleSimulateFailure(value)} type="button">{label}</button>)}</div>{simulatedCases.map(item=><small key={item.name}>{item.name}: {item.actual}</small>)}</div><div className="work-item form-stack"><strong>Säker rapport</strong><button className="secondary-action" onClick={handleExportTestReport} type="button">Exportera testrapport</button>{testReportExport?<><small className="path-value">{testReportExport.path}</small><code>{testReportExport.sha256.slice(0,24)}… · {testReportExport.case_count} fall</code></>:null}</div></section><section className="test-results-grid">{testCenterReport?.cases.map((testCase)=><article className={`test-case ${testCase.status}`} key={testCase.name}><strong>{testCase.name}</strong><span>{testCase.status}</span><small>Förväntat: {testCase.expected}</small><code>Faktiskt: {testCase.actual}</code></article>)}</section>{testCenterError?<p className="inline-error">{testCenterError}</p>:null}</div>
            ) : null}
          </section>
        ) : null}

        <div className={['documents', 'inbox', 'archive', 'favorites', 'recent', 'trash'].includes(activeView) ? `content-grid view-${documentViewMode}${!previewVisible?' preview-hidden':''}${!detailsVisible?' details-hidden':''}` : 'content-grid hidden'}>
          <section className="document-list" id="inbox" aria-labelledby="inbox-title">
            <div className="section-heading">
              <div>
                <h1 id="inbox-title">
                  {query.trim()
                    ? 'Sökresultat'
                    : activeView === 'inbox'
                      ? 'Inkorg'
                      : activeView === 'archive'
                        ? 'Arkiv'
                        : activeView === 'favorites'
                          ? 'Favoriter'
                          : activeView === 'recent'
                            ? 'Senaste dokument'
                        : activeView === 'trash'
                          ? 'Papperskorg'
                          : 'Alla dokument'}
                </h1>
                <p>{activeArchiveLabel}: dokumentlistan läses från lokal SQLite i EXE-läge.</p>
                <small className="search-help">Stöd: "exakt fras", type:, tag:, date:, status: och -uteslut</small>
              </div>
              <div className="view-switcher"><span className="badge">FTS5 + regler</span>{(['list','grid','table','gallery','kanban','split'] as DocumentViewMode[]).map(mode=><button aria-pressed={documentViewMode===mode} className={documentViewMode===mode?'active':''} key={mode} onClick={()=>{setDocumentViewMode(mode);if(mode==='split'){setPreviewVisible(true);setDetailsVisible(true)}}} type="button">{mode==='list'?'Lista':mode==='grid'?'Rutnät':mode==='table'?'Tabell':mode==='gallery'?'Galleri':mode==='kanban'?'Kanban':'Delad'}</button>)}<button aria-pressed={previewVisible} onClick={()=>setPreviewVisible(value=>!value)} type="button">Förhandsvisning</button><button aria-pressed={detailsVisible} onClick={()=>setDetailsVisible(value=>!value)} type="button">Detaljer</button>{detailsVisible?<label className="details-width-control">Panel <input aria-label="Detaljpanelens bredd" max="520" min="260" onChange={event=>setDetailsWidth(Number(event.target.value))} type="range" value={detailsWidth}/></label>:null}</div>
            </div>
            <div className="batch-toolbar"><button className={batchMode?'secondary-action active':'secondary-action'} onClick={()=>{setBatchMode(value=>!value);setSelectedDocumentIds([])}} type="button">{batchMode?'Avsluta batchläge':'Batchåtgärder'}</button>{batchMode?<><button className="mini-action" onClick={()=>setSelectedDocumentIds(renderedResults.map(result=>result.document.id))} type="button">Välj visade</button><span>{selectedDocumentIds.length} valda</span><select value={batchDraft.action} onChange={event=>setBatchDraft({...batchDraft,action:event.target.value})}><option value="add_tag">Lägg till tagg</option><option value="set_document_type">Sätt dokumenttyp</option><option value="archive">Flytta till arkiv</option><option value="review">Flytta till granskning</option></select>{['add_tag','set_document_type'].includes(batchDraft.action)?<input aria-label="Batchvärde" placeholder={batchDraft.action==='add_tag'?'Tagg':'Dokumenttyp'} value={batchDraft.value} onChange={event=>setBatchDraft({...batchDraft,value:event.target.value})}/>:null}<button className="primary-action" disabled={!selectedDocumentIds.length||(['add_tag','set_document_type'].includes(batchDraft.action)&&!batchDraft.value.trim())} onClick={handleApplyBatch} type="button">Kör på valda</button></>:null}</div>

            <div className="table-head">
              <span>Dokument</span>
              <span>Typ</span>
              <span>Datum</span>
              <span>Status</span>
            </div>

            {visibleResults.length === 0 ? (
              <div className="empty-state">
                {query.trim() ? 'Inga dokument matchar sökningen.' : 'Den här vyn är tom.'}
              </div>
            ) : (
              renderedResults.map((result, localIndex) => {
                const index=resultWindowStart+localIndex
                return (
                <button
                  aria-pressed={batchMode?selectedDocumentIds.includes(result.document.id):undefined}
                  className={`${index === selectedIndex ? 'document-row selected' : 'document-row'}${selectedDocumentIds.includes(result.document.id)?' batch-selected':''} status-${result.document.inbox_status}`}
                  key={result.document.id}
                  onClick={() => batchMode?setSelectedDocumentIds(ids=>ids.includes(result.document.id)?ids.filter(id=>id!==result.document.id):[...ids,result.document.id]):setSelectedIndex(index)}
                  type="button"
                >
                  <span>
                    <strong>{batchMode?(selectedDocumentIds.includes(result.document.id)?'☑ ':'☐ '):''}{result.document.title}</strong>
                    <small>{result.document.source_label}</small>
                  </span>
                  <span>{result.document.document_type}</span>
                  <span>{result.document.document_date ?? 'Datum saknas'}</span>
                  <span className="row-status">{formatInboxStatus(result.document.inbox_status)}</span>
                </button>
              )})
            )}
            {visibleResults.length>resultWindowSize?<nav className="result-pagination" aria-label="Dokumentsidor"><button disabled={resultWindowStart===0} onClick={()=>{const start=Math.max(0,resultWindowStart-resultWindowSize);setResultWindowStart(start);setSelectedIndex(start)}}>Föregående 100</button><span>Visar {resultWindowStart+1}–{Math.min(resultWindowStart+resultWindowSize,visibleResults.length)} av {visibleResults.length.toLocaleString('sv-SE')}</span><button disabled={resultWindowStart+resultWindowSize>=visibleResults.length} onClick={()=>{const start=resultWindowStart+resultWindowSize;setResultWindowStart(start);setSelectedIndex(start)}}> Nästa 100</button></nav>:null}
          </section>

          <section className={viewerFullscreen?'preview-pane viewer-fullscreen':'preview-pane'} aria-labelledby="preview-title" hidden={!previewVisible}>
            <div className="document-preview">
              <div className="page-toolbar">
                <span>Sida {selectedPageNo} av {documentPages.length || 1}</span>
                <div className="viewer-controls"><button onClick={()=>setSelectedPageNo(Math.max(1,selectedPageNo-1))} disabled={selectedPageNo<=1}>←</button><button onClick={()=>setSelectedPageNo(Math.min(documentPages.length||1,selectedPageNo+1))} disabled={selectedPageNo>=(documentPages.length||1)}>→</button><button onClick={()=>setViewerZoom(z=>Math.max(50,z-10))}>−</button><span>{viewerZoom}%</span><button onClick={()=>setViewerZoom(z=>Math.min(200,z+10))}>+</button><button onClick={()=>setViewerRotation(r=>(r+90)%360)}>Rotera</button><button onClick={()=>setViewerFullscreen(v=>!v)}>{viewerFullscreen?'Stäng helskärm':'Helskärm'}</button></div>
              </div>
              <div className="viewer-search"><input aria-label="Sök i dokument" placeholder="Sök i dokumentets OCR-text…" value={viewerSearch} onChange={e=>setViewerSearch(e.target.value)}/><button onClick={()=>jumpToViewerMatch(-1)}>Föregående</button><button onClick={()=>jumpToViewerMatch(1)}>Nästa</button><span>{viewerSearch?`${documentPages.filter(p=>p.text.toLocaleLowerCase('sv-SE').includes(viewerSearch.toLocaleLowerCase('sv-SE'))).length} sidor med träff`:''}</span></div>
              <div className="paper-stage" onMouseUp={captureViewerSelection}><div className="paper" style={{width:`${viewerZoom}%`,transform:`rotate(${viewerRotation}deg)`}}>
                <h2 id="preview-title">{selectedDocument.title}</h2>
                {displayedPreviewText ? (
                  <pre className="extracted-text">{displayedPreviewText}</pre>
                ) : (
                  <p>Ingen extraherbar text finns sparad för det här dokumentet ännu.</p>
                )}
                <div className="source-line">Källa: {selectedDocument.source_label}</div>
                <div className="highlight-line">{selectedDocument.match_explanation}</div>
              </div></div>
            </div>
          </section>

          <aside className="details-pane" aria-label="Dokumentinformation" hidden={!detailsVisible}>
            <h2>Detaljer</h2>
            <div className="detail-actions">
              <button className="primary-action" disabled={!selectedResult||!canUseStoredFile} onClick={()=>setInternalViewerOpen(true)} type="button">Öppna dokumentvisare</button>
              <button
                className="secondary-action"
                disabled={!canUseStoredFile}
                onClick={handleOpenStoredFile}
                type="button"
              >
                Öppna fil
              </button>
              <button
                className="secondary-action"
                disabled={!canUseStoredFile}
                onClick={handleRevealStoredFile}
                type="button"
              >
                Visa i Utforskaren
              </button>
              <button
                className="secondary-action"
                disabled={!canRunOcr || Boolean(selectedOcrJob)}
                onClick={handleRunOcrForSelectedDocument}
                type="button"
              >
                {selectedOcrJob
                  ? `OCR ${selectedOcrJob.progress_current}/${selectedOcrJob.progress_total || '?'} · ${selectedOcrJob.status}`
                  : 'Köa OCR'}
              </button>
              {selectedOcrJob ? (
                <button className="secondary-action" onClick={handleToggleOcrPause} type="button">
                  {selectedOcrJob.status === 'paused' ? 'Återuppta OCR' : 'Pausa OCR'}
                </button>
              ) : null}
              <button
                className="secondary-action danger-action"
                disabled={activeView === 'trash' || !hasProductionDocuments || !selectedResult || isTrashingDocument}
                onClick={handleTrashSelectedDocument}
                type="button"
              >
                {isTrashingDocument ? 'Tar bort...' : 'Ta bort'}
              </button>
              {activeView === 'trash' ? (
                <button
                  className="primary-action"
                  disabled={!selectedResult}
                  onClick={handleRestoreSelectedDocument}
                  type="button"
                >
                  Återställ dokument
                </button>
              ) : null}
              {activeView !== 'trash' ? (
                <button className="secondary-action" disabled={!canEditDocument || isAddingVersion} onClick={handleAddVersion} type="button">
                  {isAddingVersion ? 'Lägger till version...' : 'Lägg till ny version'}
                </button>
              ) : null}
              {activeView !== 'trash' && selectedDocumentId ? <>
                <button className="secondary-action" onClick={handleToggleFavorite} type="button">{favoriteDocumentIds.includes(selectedDocumentId)?'★ Ta bort favorit':'☆ Lägg till favorit'}</button>
                {collections.length?<select aria-label="Lägg i samling" defaultValue="" onChange={event=>{if(event.target.value)void handleAddToCollection(Number(event.target.value));event.target.value='' }}><option value="">Lägg i samling…</option>{collections.map(collection=><option key={collection.id} value={collection.id}>{collection.name}</option>)}</select>:null}
                {folders.length?<select aria-label="Växla mapp" defaultValue="" onChange={event=>{if(event.target.value)void handleAssignOrganizer('folder',Number(event.target.value));event.target.value=''}}><option value="">Mapp…</option>{folders.map(item=><option key={item.id} value={item.id}>{documentFolderIds.includes(item.id)?'✓ ':''}{item.name}</option>)}</select>:null}
                {categories.length?<select aria-label="Växla kategori" defaultValue="" onChange={event=>{if(event.target.value)void handleAssignOrganizer('category',Number(event.target.value));event.target.value=''}}><option value="">Kategori…</option>{categories.map(item=><option key={item.id} value={item.id}>{documentCategoryIds.includes(item.id)?'✓ ':''}{item.name}</option>)}</select>:null}
                <button className="secondary-action" onClick={handleToggleDocumentLock} type="button">{lockedDocumentIds.includes(selectedDocumentId)?'Lås upp dokumentstatus':'Lås dokument'}</button>
                <button className="secondary-action" onClick={handleTogglePrivateDocument} type="button">{privateDocumentIds.includes(selectedDocumentId)?'Ta ur privat sektion':'Kryptera i privat sektion'}</button>
                {privateDocumentIds.includes(selectedDocumentId)?<button className="secondary-action" onClick={handleOpenPrivateDocument} type="button">Öppna privat original</button>:null}
                <button className="secondary-action" onClick={()=>handleProtectedExport(false)} type="button">Exportera original säkert</button>
                <button className="secondary-action" onClick={()=>handleProtectedExport(true)} type="button">Exportera maskerad text</button>
              </>:null}
            </div>
            {workflowError ? <p className="inline-error">{workflowError}</p> : null}
            {activeView!=='trash'&&selectedDocumentId?<div className="viewer-annotations work-item form-stack"><h2>Markeringar och bokmärken</h2><div className="inline-control"><select value={annotationDraft.type} onChange={e=>setAnnotationDraft({...annotationDraft,type:e.target.value as 'bookmark'|'note'|'highlight'})}><option value="bookmark">Bokmärke</option><option value="highlight">Markering</option><option value="note">Kommentar</option></select><input type="color" aria-label="Markeringsfärg" value={annotationDraft.color} onChange={e=>setAnnotationDraft({...annotationDraft,color:e.target.value})}/></div><input placeholder="Markerad text (markera text i visaren eller skriv)" value={annotationDraft.selectedText} onChange={e=>setAnnotationDraft({...annotationDraft,selectedText:e.target.value})}/><textarea rows={3} placeholder="Kommentar…" value={annotationDraft.body} onChange={e=>setAnnotationDraft({...annotationDraft,body:e.target.value})}/><button className="secondary-action" disabled={!annotationDraft.body.trim()&&!annotationDraft.selectedText.trim()&&annotationDraft.type!=='bookmark'} onClick={handleSaveAnnotation}>Spara på sida {selectedPageNo}</button>{annotations.map(item=><div className="annotation-row" key={item.id} style={{borderLeftColor:item.color}}><button className="annotation-jump" onClick={()=>setSelectedPageNo(item.page_no)}><strong>{item.annotation_type} · sida {item.page_no}</strong><span>{item.selected_text||item.body||'Bokmärke'}</span></button><button aria-label="Ta bort markering" onClick={()=>handleDeleteAnnotation(item.id)}>×</button></div>)}</div>:null}
            {activeView!=='trash'&&selectedDocumentId?<div className="document-notes work-item form-stack"><h2>Dokumentanteckningar</h2><select value={noteDraft.kind} onChange={e=>setNoteDraft({...noteDraft,kind:e.target.value})}><option value="note">Anteckning</option><option value="todo">Att göra</option><option value="decision">Beslut</option></select><textarea rows={4} placeholder="Skriv en lokal anteckning…" value={noteDraft.body} onChange={e=>setNoteDraft({...noteDraft,body:e.target.value})}/><button className="secondary-action" disabled={!noteDraft.body.trim()} onClick={handleSaveNote}>{noteDraft.id?'Spara ny version':'Lägg till anteckning'}</button>{documentNotes.map(note=><button className="note-history-row" key={note.id} onClick={()=>setNoteDraft({id:note.id,body:note.body,kind:note.kind})} type="button"><strong>{note.kind} · version {note.version_no}</strong><span>{note.body}</span><small>{note.updated_at}</small></button>)}</div>:null}
            {versions.length ? (
              <div className="version-list">
                <h2>Versioner</h2>
                <input placeholder="Kommentar till nästa version…" value={versionComment} onChange={e=>setVersionComment(e.target.value)}/><small>Välj exakt två versioner för en deterministisk jämförelse.</small>
                {versions.map((version) => (
                  <div className="version-row" key={version.id}>
                    <label><input type="checkbox" checked={versionSelection.includes(version.id)} onChange={()=>setVersionSelection(current=>current.includes(version.id)?current.filter(id=>id!==version.id):current.length<2?[...current,version.id]:[current[1],version.id])}/><strong>v{version.version_no}{version.is_current_file ? ' · aktuell' : ''}</strong></label>
                    <span>{version.original_name}</span>
                    <small>{version.imported_at} · {(version.size_bytes / 1024).toFixed(1)} KB · {version.origin}{version.comment?` · ${version.comment}`:''}</small>
                    {!version.is_current_file?<button className="mini-action" onClick={()=>handleRestoreVersion(version.id)}>Återställ som aktuell</button>:null}
                  </div>
                ))}
                <button className="secondary-action" disabled={versionSelection.length!==2} onClick={handleCompareVersions}>Jämför valda versioner</button>
                {versionComparison?<div className="version-comparison"><strong>v{versionComparison.left_version_no} → v{versionComparison.right_version_no}{versionComparison.identical?' · identiska filer':''}</strong><small>Storlek {versionComparison.size_change_bytes>=0?'+':''}{versionComparison.size_change_bytes} byte · sidor {versionComparison.page_change>=0?'+':''}{versionComparison.page_change} · datum {versionComparison.date_changed?'ändrat':'oförändrat'}</small><code>SHA-256: {versionComparison.left_sha256.slice(0,12)}… → {versionComparison.right_sha256.slice(0,12)}…</code><div className="side-by-side-comparison"><section><b>Version {versionComparison.left_version_no}</b>{versionComparison.left_lines.map((line,i)=><span className={versionComparison.removed_lines.includes(line)?'diff-removed':''} key={`l-${i}`}>{line||' '}</span>)}</section><section><b>Version {versionComparison.right_version_no}</b>{versionComparison.right_lines.map((line,i)=><span className={versionComparison.added_lines.includes(line)?'diff-added':''} key={`r-${i}`}>{line||' '}</span>)}</section></div><details><summary>Sammanfattning av textskillnader</summary><div className="diff-columns"><div><b>Tillagd text ({versionComparison.added_lines.length})</b>{versionComparison.added_lines.slice(0,100).map((line,i)=><ins key={i}>{line}</ins>)}</div><div><b>Borttagen text ({versionComparison.removed_lines.length})</b>{versionComparison.removed_lines.slice(0,100).map((line,i)=><del key={i}>{line}</del>)}</div></div></details></div>:null}
              </div>
            ) : null}
            {documentPages.length ? (
              <div className="page-source-list">
                <h2>Källsidor</h2>
                <div className="tag-list">
                  {documentPages.map((page) => (
                    <button className={page.page_no === selectedPageNo ? 'tag-chip active' : 'tag-chip'} key={page.page_no} onClick={() => setSelectedPageNo(page.page_no)} type="button">
                      Sida {page.page_no} · {page.text_quality}
                    </button>
                  ))}
                </div>
                <small>{selectedPage?.source_kind} · {selectedPage?.text.length ?? 0} tecken</small>
              </div>
            ) : null}
            <div className="metadata-editor">
              <label>
                Titel
                <input
                  disabled={!canEditDocument}
                  onChange={(event) =>
                    setMetadataDraft((draft) => ({ ...draft, title: event.target.value }))
                  }
                  value={metadataDraft.title}
                />
              </label>
              <label>
                Typ
                <input
                  disabled={!canEditDocument}
                  onChange={(event) =>
                    setMetadataDraft((draft) => ({ ...draft, document_type: event.target.value }))
                  }
                  value={metadataDraft.document_type}
                />
              </label>
              <label>
                Datum
                <input
                  disabled={!canEditDocument}
                  onChange={(event) =>
                    setMetadataDraft((draft) => ({ ...draft, document_date: event.target.value }))
                  }
                  placeholder="YYYY-MM-DD"
                  value={metadataDraft.document_date}
                />
              </label>
              <label>
                Status
                <select
                  disabled={!canEditDocument}
                  onChange={(event) =>
                    setMetadataDraft((draft) => ({ ...draft, inbox_status: event.target.value }))
                  }
                  value={metadataDraft.inbox_status}
                >
                  <option value="inbox">Inkorg</option>
                  <option value="review">Granskning</option>
                  <option value="indexed">Indexerat</option>
                  <option value="archived">Arkiverat</option>
                </select>
              </label>
              <label>
                Källa
                <input
                  disabled={!canEditDocument}
                  onChange={(event) =>
                    setMetadataDraft((draft) => ({ ...draft, source_label: event.target.value }))
                  }
                  value={metadataDraft.source_label}
                />
              </label>
              <label>
                Anteckning
                <textarea
                  disabled={!canEditDocument}
                  onChange={(event) =>
                    setMetadataDraft((draft) => ({
                      ...draft,
                      match_explanation: event.target.value,
                    }))
                  }
                  rows={3}
                  value={metadataDraft.match_explanation}
                />
              </label>
              <button
                className="secondary-action"
                disabled={!canEditDocument || isSavingMetadata}
                onClick={handleSaveMetadata}
                type="button"
              >
                {isSavingMetadata ? 'Sparar...' : 'Spara metadata'}
              </button>
              {metadataError ? <div className="inline-error">{metadataError}</div> : null}
            </div>
            <dl>
              {fileActionError ? (
                <div>
                  <dt>Filåtgärd</dt>
                  <dd>{fileActionError}</dd>
                </div>
              ) : null}
              <div>
                <dt>Dokumenttyp</dt>
                <dd>{selectedDocument.document_type}</dd>
              </div>
              <div>
                <dt>Datum</dt>
                <dd>{selectedDocument.document_date ?? 'Datum saknas'}</dd>
              </div>
              <div>
                <dt>Matchorsak</dt>
                <dd>{selectedDocument.match_explanation}</dd>
              </div>
              <div>
                <dt>Sökpoäng</dt>
                <dd>{selectedResult ? selectedResult.score : 0}</dd>
              </div>
              <div>
                <dt>Rankningsförklaring</dt>
                <dd>{selectedResult?.reasons.join('; ') || 'Ingen träff vald'}</dd>
              </div>
              <div>
                <dt>Query plan</dt>
                <dd>{selectedResult?.query_plan.join(', ') || 'Ingen sökfråga'}</dd>
              </div>
              <div>
                <dt>Filtyp</dt>
                <dd>{documentDetail?.mime_type ?? 'Ingen fil kopplad'}</dd>
              </div>
              <div>
                <dt>Filstorlek</dt>
                <dd>
                  {documentDetail?.size_bytes == null
                    ? 'Saknas'
                    : `${documentDetail.size_bytes.toLocaleString('sv-SE')} byte`}
                </dd>
              </div>
              <div>
                <dt>Lagrad fil</dt>
                <dd className="path-value">{documentDetail?.stored_path ?? 'Ingen lokal filväg'}</dd>
              </div>
              <div>
                <dt>Taggar</dt>
                <dd>
                  {documentDetail?.tags.length ? (
                    <span className="tag-list">
                      {documentDetail.tags.map((tag) => (
                        <button
                          className="tag-chip"
                          disabled={!canEditDocument}
                          key={tag}
                          onClick={() => handleRemoveTag(tag)}
                          type="button"
                        >
                          {tag}
                        </button>
                      ))}
                    </span>
                  ) : (
                    'Inga taggar'
                  )}
                </dd>
              </div>
              <div>
                <dt>Claims</dt>
                <dd>
                  {documentDetail?.claims.length
                    ? `${documentDetail.claims.length} lokala claims`
                    : 'Inga claims'}
                </dd>
              </div>
              <div>
                <dt>Relationer</dt>
                <dd>
                  {documentDetail?.entities.length
                    ? documentDetail.entities
                        .map((entity) => `${entity.display_name} (${entity.entity_type})`)
                        .join(', ')
                    : 'Inga relationer hittade'}
                </dd>
              </div>
            </dl>

            <div className="inline-control">
              <input
                disabled={!canEditDocument}
                onChange={(event) => setTagInput(event.target.value)}
                placeholder="ny-tagg"
                value={tagInput}
              />
              <button
                className="secondary-action"
                disabled={!canEditDocument || isSavingTag || !tagInput.trim()}
                onClick={handleAddTag}
                type="button"
              >
                Lägg till tagg
              </button>
            </div>

            {documentDetail?.claims.length ? (
              <>
                <h2>Claims</h2>
                <dl>
                  {documentDetail.claims.slice(0, 6).map((claim) => (
                    <div key={claim.id}>
                      <dt>{claim.claim_type}</dt>
                      <dd className="path-value">
                        {claim.value_json} · {claim.status} · {claim.extraction_method}
                        <br />
                        {claim.source_page_no ? `Källa: sida ${claim.source_page_no}` : 'Källa: dokumentmetadata'}
                        {' · '}{claim.actuality_status}
                        {claim.effective_from ? ` från ${claim.effective_from}` : ''}
                        <br />
                        {claim.actuality_explanation}
                        {claim.source_page_no ? (
                          <>
                            <br />
                            <button className="mini-action" onClick={() => { setSelectedPageNo(claim.source_page_no ?? 1); setViewerSourceClaim(claim); setInternalViewerOpen(true) }} type="button">
                              Visa källa i dokumentet
                            </button>
                          </>
                        ) : null}
                      </dd>
                    </div>
                  ))}
                </dl>
              </>
            ) : null}

            {documentDetail?.claims.length ? (
              <div className="claim-review-list">
                {documentDetail.claims.slice(0, 6).map((claim) => (
                  <div className="claim-actions" key={`review-${claim.id}`}>
                    <span>{claim.claim_type}</span>
                    <button
                      className="mini-action"
                      disabled={!canEditDocument || claim.status === 'review_approved'}
                      onClick={() => handleSetClaimStatus(claim.id, 'review_approved')}
                      type="button"
                    >
                      Godkänn
                    </button>
                    <button className="mini-action" onClick={() => handleSetClaimActuality(claim.id, 'manual_current')} type="button">Aktuell</button>
                    <button className="mini-action" onClick={() => handleSetClaimActuality(claim.id, 'manual_historical')} type="button">Historisk</button>
                    <button
                      className="mini-action"
                      disabled={!canEditDocument || claim.status === 'review_rejected'}
                      onClick={() => handleSetClaimStatus(claim.id, 'review_rejected')}
                      type="button"
                    >
                      Avvisa
                    </button>
                  </div>
                ))}
              </div>
            ) : null}

            <h2>Kodord</h2>
            <div className="metadata-editor">
              <label>
                Kodord
                <input
                  onChange={(event) => setCodeWordInput(event.target.value)}
                  placeholder="privat-kategori"
                  value={codeWordInput}
                />
              </label>
              <label>
                Beskrivning
                <input
                  onChange={(event) => setCodeWordDescription(event.target.value)}
                  placeholder="valfri lokal beskrivning"
                  value={codeWordDescription}
                />
              </label>
              <button
                className="secondary-action"
                disabled={isSavingCodeWord || !codeWordInput.trim()}
                onClick={handleAddCodeWord}
                type="button"
              >
                {isSavingCodeWord ? 'Sparar...' : 'Spara kodord'}
              </button>
            </div>
            <dl>
              <div>
                <dt>Register</dt>
                <dd>
                  {codeWords.length
                    ? codeWords.map((codeWord) => codeWord.word).join(', ')
                    : 'Inga kodord sparade'}
                </dd>
              </div>
            </dl>

            <h2>Konflikter</h2>
            <dl>
              <div>
                <dt>Status</dt>
                <dd>
                  {conflicts.length
                    ? `${conflicts.length} saker kräver uppmärksamhet`
                    : 'Inga konflikter hittade'}
                </dd>
              </div>
              {conflicts.slice(0, 4).map((conflict) => (
                <div key={`${conflict.conflict_type}-${conflict.title}`}>
                  <dt>{conflict.severity} / {conflict.conflict_type}</dt>
                  <dd>{conflict.title}</dd>
                  <dd className="path-value">{conflict.detail}</dd>
                </div>
              ))}
            </dl>

            <h2>Senaste import</h2>
            <dl>
              <div>
                <dt>Status</dt>
                <dd>
                  {importError
                    ? importError
                    : lastImport
                      ? lastImport.duplicate
                        ? 'Importerad som dubblett'
                        : 'Importerad lokalt'
                      : 'Ingen import körd'}
                </dd>
              </div>
              <div>
                <dt>Fil</dt>
                <dd>{lastImport?.document.title ?? 'Ingen fil vald'}</dd>
              </div>
              <div>
                <dt>SHA-256</dt>
                <dd className="path-value">{lastImport?.sha256 ?? 'Inte beräknad'}</dd>
              </div>
              <div>
                <dt>Lokal kopia</dt>
                <dd className="path-value">{lastImport?.stored_path ?? 'Ingen lokal kopia ännu'}</dd>
              </div>
            </dl>

            <h2>Senaste backup</h2>
            <div className="detail-actions">
              <button
                className="secondary-action"
                disabled={!backups.length || isValidatingBackup}
                onClick={handleValidateLatestBackup}
                type="button"
              >
                {isValidatingBackup ? 'Validerar...' : 'Validera senaste'}
              </button>
              <button
                className="secondary-action"
                disabled={!backups.length || isRestoringBackup}
                onClick={handleRestoreLatestBackup}
                type="button"
              >
                {isRestoringBackup ? 'Återställer...' : 'Återställ senaste'}
              </button>
            </div>
            <dl>
              <div>
                <dt>Status</dt>
                <dd>
                  {backupError
                    ? backupError
                    : restoreError
                      ? restoreError
                      : lastRestore
                      ? `Återställd. Säkerhetskopia före restore: ${lastRestore.pre_restore_backup_root}`
                      : lastBackup
                        ? 'Backup skapad lokalt'
                        : 'Ingen backup skapad'}
                </dd>
              </div>
              <div>
                <dt>Validering</dt>
                <dd>
                  {backupValidation
                    ? backupValidation.ok
                      ? `OK: ${backupValidation.document_count} dokument, SHA matchar`
                      : `Varning: ${backupValidation.warnings.join('; ')}`
                    : 'Inte körd'}
                </dd>
              </div>
              <div>
                <dt>Backupmapp</dt>
                <dd className="path-value">{lastBackup?.backup_root ?? 'Saknas'}</dd>
              </div>
              <div>
                <dt>Databas</dt>
                <dd className="path-value">{lastBackup?.backup_path ?? 'Saknas'}</dd>
              </div>
              <div>
                <dt>Manifest</dt>
                <dd className="path-value">{lastBackup?.manifest_path ?? 'Saknas'}</dd>
              </div>
              <div>
                <dt>SHA-256</dt>
                <dd className="path-value">{lastBackup?.sha256 ?? 'Inte beräknad'}</dd>
              </div>
              <div>
                <dt>Innehåll</dt>
                <dd>
                  {lastBackup
                    ? `${lastBackup.document_count} dokument, ${lastBackup.copied_file_count}/${lastBackup.file_count} filer kopierade`
                    : 'Saknas'}
                </dd>
              </div>
            </dl>

            <h2>Lokal status</h2>
            <dl>
              <div>
                <dt>Test Center</dt>
                <dd>
                  {testCenterError
                    ? testCenterError
                    : testCenterReport
                      ? `${testCenterReport.passed} passed, ${testCenterReport.failed} failed${
                          testCenterReport.failed
                          ? ` · ${testCenterReport.cases.find((testCase) => testCase.status === 'failed')?.name}`
                            : ''
                        }`
                      : 'Inte kört'}
                </dd>
              </div>
              <div>
                <dt>OCR</dt>
                <dd>
                  {ocrError
                    ? ocrError
                    : ocrStatus.available
                      ? `${ocrStatus.version ? `${ocrStatus.engine} ${ocrStatus.version}` : ocrStatus.engine} · språk: ${
                          ocrStatus.languages.join(', ') || 'okända'
                        }`
                      : ocrStatus.notes[0]}
                </dd>
              </div>
              <div>
                <dt>OCR språkdata</dt>
                <dd className="path-value">{ocrStatus.tessdata_dir ?? 'Inte hittad'}</dd>
              </div>
              <div>
                <dt>Senaste OCR</dt>
                <dd>
                  {latestCompletedOcrJob?.result_summary ?? 'Ingen köad OCR slutförd'}
                </dd>
              </div>
              <div>
                <dt>Sökindexreparation</dt>
                <dd>
                  {reindexError
                    ? reindexError
                    : lastReindex
                      ? `${lastReindex.indexed_document_count} dokument indexerade`
                      : 'Inte körd'}
                </dd>
              </div>
              <div>
                <dt>Backuper</dt>
                <dd>
                  {backups.length > 0
                    ? `${backups.length} lokala backup-set, senaste: ${backups[0].backup_root}`
                    : 'Inga lokala backup-set hittade'}
                </dd>
              </div>
              <div>
                <dt>Audit-logg</dt>
                <dd>
                  {auditEvents.length > 0
                    ? `${auditEvents.length} senaste händelser, senaste: ${auditEvents[0].safe_summary}`
                    : 'Inga händelser ännu'}
                </dd>
              </div>
              <div>
                <dt>Arkivkontroll</dt>
                <dd>
                  {healthError
                    ? healthError
                    : healthReport
                      ? healthReport.ok
                        ? 'OK'
                        : `Varning: ${healthReport.warnings.join('; ')}`
                      : 'Inte körd'}
                </dd>
              </div>
              <div>
                <dt>SQLite integrity</dt>
                <dd>{healthReport?.database_integrity ?? 'Inte kontrollerad'}</dd>
              </div>
              <div>
                <dt>Arkivinnehåll</dt>
                <dd>
                  {healthReport
                    ? `${healthReport.document_count} dokument, ${healthReport.file_count} filer, ${healthReport.fts_entry_count} indexrader`
                    : 'Inte kontrollerat'}
                </dd>
              </div>
              <div>
                <dt>Saknade filer</dt>
                <dd>{healthReport?.missing_file_count ?? 'Inte kontrollerat'}</dd>
              </div>
              <div>
                <dt>App</dt>
                <dd>{status.app_mode}</dd>
              </div>
              <div>
                <dt>Lagring</dt>
                <dd>{status.storage}</dd>
              </div>
              <div>
                <dt>Databas</dt>
                <dd>{status.database}, schema v{vaultInit.schema_version}</dd>
              </div>
              <div>
                <dt>Sökindex</dt>
                <dd>{status.search_index}</dd>
              </div>
              <div>
                <dt>Nätverk</dt>
                <dd>{status.network}</dd>
              </div>
              <div>
                <dt>Policy</dt>
                <dd>{status.ai_policy}</dd>
              </div>
              <div>
                <dt>Data root</dt>
                <dd className="path-value">{vaultInit.data_root}</dd>
              </div>
              <div>
                <dt>Vault DB</dt>
                <dd className="path-value">{vaultInit.database_path}</dd>
              </div>
              <div>
                <dt>Test Lab DB</dt>
                <dd className="path-value">{vaultInit.testlab_database_path}</dd>
              </div>
              <div>
                <dt>Test Lab</dt>
                <dd>
                  {vaultInit.testlab_isolated
                    ? `${vaultInit.testlab_document_count} syntetiska dokument i separat databas`
                    : 'Endast i EXE-läge'}
                </dd>
              </div>
            </dl>
          </aside>
        </div>
      </section>
    </main>
  )
}

export default App
