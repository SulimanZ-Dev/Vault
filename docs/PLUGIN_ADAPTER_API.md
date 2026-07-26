# Vault Plugin Adapter API 1.0

Vault plugins are local, declarative JSON manifests. They cannot execute binaries,
open sockets, call AI services, or read arbitrary paths. Installation validates the
manifest, stores a normalized copy, records its SHA-256 checksum, and leaves it
disabled. The user must approve the displayed capabilities before it can run. A
changed manifest is automatically disabled.

Publishers may add `publisher_public_key` and `publisher_signature` as standard
Base64-encoded Ed25519 values. The signature covers compact JSON serialization of
the parsed manifest with `publisher_signature` removed. Vault rejects malformed or
invalid signatures and displays `ed25519_verified` only after cryptographic
verification. Unsigned local manifests remain allowed but are clearly marked
`unsigned_local` and still require explicit user approval.

Supported `adapter_kind` values are `ocr_engine`, `importer`,
`metadata_extractor`, `search_parser`, `exporter`, `dashboard_widget`, and
`domain_model`. API 1.0 executes the safe `metadata_extractor` action contract;
other kinds are versioned declarations for compatible adapters and never gain
implicit file, process, or network access.

Required fields:

- `id`, `name`, `version`, `api_version: "1.0"`
- `adapter_kind`
- `ai_free: true`
- declared `capabilities` and `data_access`
- `resource_limit` between 100 and 100000

Executable 1.0 actions are `add_tag` and `set_document_type`. They require
`write_tags` and `write_document_type` respectively. `match_text` is evaluated
against the title unless `read_text` was approved. Locked and private documents
are always excluded. Failures are isolated to the plugin run and logged without
document content.

External sources use a separate local handoff adapter. Google Drive, OneDrive,
Gmail, and Outlook are disabled by default. The user selects an existing export
or sync folder, previews supported files, selects exact paths, and confirms the
import. Every selected canonical path must remain inside the approved folder.
OAuth tokens and provider credentials are neither requested nor stored.
