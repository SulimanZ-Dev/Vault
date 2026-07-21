# Data model

## Databasprinciper

- SQLite är system of record.
- Originalfiler lagras på disk, inte som stora blobs i databasen.
- Alla viktiga statusändringar är auditerbara.
- Claims är källbundna och historikbevarande.
- Manuella verifieringar och låsningar vinner över automatiska regler.
- Test Lab använder separat databas, separat filrot och separat index.

## Centrala tabeller

```text
vaults(id, name, mode, storage_root, created_at)
documents(id, vault_id, title, document_type_id, lifecycle_status, inbox_status, created_at, updated_at, trashed_at)
document_versions(id, document_id, version_no, file_id, imported_at, document_date, source_path, is_current_file)
files(id, vault_id, sha256, perceptual_hash, storage_path, original_name, mime_type, size_bytes, file_state)
document_pages(id, document_version_id, page_no, width, height, rotation, text_quality)
text_spans(id, document_page_id, source_kind, text, bbox_json, confidence, language, created_at)
categories(id, vault_id, name, parent_id, color_token)
tags(id, vault_id, name, color_token)
code_words(id, vault_id, name, description)
document_tags(document_id, tag_id)
document_code_words(document_id, code_word_id)
entities(id, vault_id, entity_type, display_name, sort_name, created_at)
entity_aliases(id, entity_id, alias, normalized_alias)
objects(id, vault_id, object_type, display_name, identifiers_json)
relationships(id, vault_id, subject_type, subject_id, relation_type, object_type, object_id, valid_from, valid_to, status)
claims(id, vault_id, subject_type, subject_id, claim_type, value_json, value_kind, status, confidence_kind, created_at)
claim_sources(id, claim_id, document_id, document_version_id, page_no, text_span_id, extraction_method, rule_id)
claim_validity(id, claim_id, valid_from, valid_to, observed_at, effective_status)
conflicts(id, vault_id, conflict_type, subject_type, subject_id, status, created_at, resolved_at)
conflict_items(id, conflict_id, claim_id, role, explanation)
user_verifications(id, vault_id, target_type, target_id, decision, note, locked, created_at)
rules(id, vault_id, rule_type, name, version, enabled, config_json, approval_policy)
rule_runs(id, rule_id, document_id, started_at, finished_at, outcome, explanation_json)
jobs(id, vault_id, job_type, target_type, target_id, status, priority, attempts, error_code, created_at, updated_at)
audit_events(id, vault_id, actor_kind, event_type, target_type, target_id, safe_summary, created_at)
settings(id, vault_id, key, value_json)
themes(id, vault_id, name, version, tokens_json, is_active)
```

## Dokument och versioner

Ett `document` är den logiska posten användaren arbetar med. En `document_version` pekar på en faktisk importerad filversion. Flera filer kan vara:

- exakt samma dokumentbild,
- ny version av samma dokument,
- separat dokument med liknande innehåll,
- bilaga eller relaterad handling.

Ingen ny import får automatiskt radera eller skriva över historisk information.

## Claims och källor

En claim är ett strukturerat påstående som alltid har status och källa.

Exempel:

```json
{
  "subject_type": "employment",
  "claim_type": "hourly_wage",
  "value_json": { "amount": 172.50, "currency": "SEK" },
  "value_kind": "explicit_document_value",
  "status": "auto_extracted_pending_review"
}
```

`value_kind`:

- `explicit_document_value`
- `calculated_value`
- `user_entered_value`
- `rule_inferred_status`

`confidence_kind`:

- `exact_rule_match`
- `validated_parse`
- `fuzzy_match`
- `ambiguous`
- `manual`

Varje claim ska kunna visa:

- dokument,
- version,
- sida,
- textstycke,
- extraktionsmetod,
- regelversion,
- om värdet är uttryckligt eller beräknat.

## Aktualitetsstatus

Aktualitet sparas separat från claimvärdet.

Statusar:

- `current`
- `historical`
- `future`
- `expired`
- `replaced`
- `cancelled`
- `parallel_current`
- `uncertain`
- `conflicted`
- `excluded`

Regler ska kunna uttrycka att flera relationer får vara aktuella samtidigt, till exempel parallella anställningar eller flera giltiga pass.

## Konflikter

En konflikt skapas när två eller flera claims inte kan samexistera enligt domänens regler och systemet inte deterministiskt kan välja.

Konflikten innehåller:

- berörda claims,
- källor,
- regel som upptäckte konflikten,
- förklaring,
- användarbeslut,
- låsningsstatus.

Systemet får aldrig lösa konflikt genom gissning.

## Parallella relationer

Relationstyper har en policy:

- `single_current`: högst en aktuell relation.
- `parallel_allowed`: flera aktuella relationer är giltiga.
- `date_partitioned`: flera relationer över tid, högst en per period.
- `manual_only`: systemet föreslår men sätter inte aktuell status.

Exempel: anställning har `parallel_allowed`, medan primär bostadsadress kan börja som `manual_only`.

## Användarverifiering

Verifieringar är egna poster, inte ändringar direkt i extraherade claims.

Beslut:

- godkänn claim,
- avvisa claim,
- markera claim som aktuell,
- markera claim som historisk,
- lås värde,
- välj flera parallella värden,
- välj inget värde,
- skapa manuell korrigering.

