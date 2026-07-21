# Technical architecture

## Rekommenderad stack

Rekommenderad första stack:

- Desktop shell: Tauri 2.
- UI: React, TypeScript och Vite.
- Styling: CSS variables med egen tokenstruktur.
- Backend/runtime: Rust via Tauri commands.
- Databas: SQLite lokalt.
- Migreringar: sqlx migrations eller refinery.
- Fulltextsökning: SQLite FTS5 i första versionen.
- OCR: lokal Tesseract via explicit installerad binär eller valfri lokal adapter.
- Textutvinning: PDF/text/bild-adaptrar bakom gemensamt gränssnitt.
- Tester: Vitest för UI/logik, Rust unit tests, Playwright för kritiska UI-flöden.

Motivering:

- Tauri ger liten lokal desktop-app utan inbyggd molnmodell.
- Rust passar filhantering, hashing, parsers, köer och säker lokal IO.
- SQLite räcker långt för privat arkiv, FTS5, transaktioner och portabel backup.
- React/TypeScript ger snabbt byggbart och testbart gränssnitt.

## Alternativa stackar

Electron + React + SQLite:

- Fördel: stor ekosystemyta och enkel desktopintegration.
- Nackdel: tyngre runtime, större app, större attackyta.

.NET MAUI/WPF + SQLite:

- Fördel: stark Windows-integration.
- Nackdel: sämre portabilitet och mindre webbkomponentekosystem.

Qt + C++/Rust:

- Fördel: robust native desktop.
- Nackdel: mer komplex UI-utveckling och långsammare iteration.

Web-only lokal PWA:

- Fördel: enkel distribution.
- Nackdel: begränsad filsystemsåtkomst, sämre lokal OCR/köhantering och svagare offline-filmodell.

## Systemarkitektur

Vault delas i tydliga moduler:

- `app-ui`: vyer, komponenter, teman, kommandopalett och interaktion.
- `app-shell`: Tauri, fönster, filväljare, säker local IPC.
- `vault-core`: domänmodeller, claims, statusar, konflikter och relationer.
- `vault-storage`: SQLite, migreringar, repositories och transaktioner.
- `vault-files`: import, hashning, filkopiering, filintegritet och karantän.
- `vault-text`: textutvinning från PDF, text, Office-exporter och OCR-resultat.
- `vault-ocr`: lokal OCR-kö, OCR-profiler, kvalitet och fel.
- `vault-search`: FTS-index, tokenisering, svenska regler, rankning och förklaringar.
- `vault-rules`: deterministiska regler, parsers, godkännandepolicyer och regelhistorik.
- `vault-activity`: audit log, verifieringshistorik och bakgrundsjobb.
- `vault-testlab`: isolerad testmiljö med syntetiska data.

## Dataflöde vid import

1. Användaren väljer eller släpper fil.
2. Filen kopieras till importstaging.
3. Systemet beräknar SHA-256, storlek, mimetype och filsignatur.
4. Dubblettkontroll görs mot content hash.
5. Dokumentpost och första version skapas i databas.
6. Filen flyttas atomiskt till vault storage.
7. Textutvinning körs om filtypen stöds.
8. OCR-jobb skapas om text saknas och OCR är aktiverat.
9. FTS-index uppdateras.
10. Regler körs endast enligt användarens analysläge.
11. Förslag, claims eller granskningsposter skapas enligt policy.

## Bakgrundsjobb

Jobb lagras i databas med status `queued`, `running`, `paused`, `failed`, `cancelled` eller `completed`.

Första jobbtyper:

- `extract_text`
- `ocr_document`
- `index_document`
- `run_rules`
- `rebuild_index`
- `backup_vault`
- `testlab_seed`

Jobb ska vara idempotenta där det är möjligt och aldrig logga känsligt dokumentinnehåll.

## Filstruktur för kodbas

Planerad struktur:

```text
Vault/
  docs/
  app/
    src/
      components/
      features/
      routes/
      styles/
      test/
    src-tauri/
      crates/
        vault-core/
        vault-storage/
        vault-files/
        vault-text/
        vault-ocr/
        vault-search/
        vault-rules/
        vault-testlab/
      migrations/
      tests/
  fixtures/
    testlab/
  scripts/
```

