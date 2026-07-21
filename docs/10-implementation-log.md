# Implementation log

## 2026-07-21: Schema v9 för sidor och källtext

- `document_pages`, `text_spans` och separat `span_fts` har införts.
- varje sida lagrar sidnummer, källtyp och textkvalitet.
- importer och nya dokumentversioner skriver sid-/spanmodellen.
- manuell OCR ersätter aktuell sidas källa med `ocr_text` och indexerar textspanet.
- dokumentpanelen visar Källsidor, kvalitet, källtyp och antal tecken.
- preview kan visa vald sidas text på breda fönster.
- jobbschemat har progress-, paus- och resultfält som grund för nästa asynkrona OCR-del.

Verifierat i release-EXE:

- schema v9 laddades med 6/6 diagnostikkontroller.
- riktig PDF kördes genom OCR.
- UI visade `Sida 1 · good`, `ocr_text` och `1514 tecken`.
- audit-loggen visade slutförd OCR-händelse.

## 2026-07-21: Verklig EXE-runda och avancerad sökning

Release-EXE:n öppnades och styrdes som vanlig Windows-app. Följande verifierades i UI:

- startsida och lokal diagnostik laddar schema v8, Tesseract, svensk språkdata och Poppler.
- sökning från startsidan växlar nu automatiskt till synliga sökresultat.
- `type:Importerad` gav 18 verkliga PDF-träffar i produktionsarkivet.
- PDF valdes i listan och OCR-knappen var aktiverad.
- manuell PDF-OCR slutfördes i EXE:n och rapporterade 1 514 indexerade tecken.
- detaljpanelen var inte klickbar när tre paneler tvingades in vid 1322 px; responsiv layout återställdes till lista + detaljer vid den bredden.

Nästa sökblock implementerades samtidigt:

- `type:`, `tag:`, `date:` och `status:`.
- exakta fraser inom citattecken.
- negativa termer med `-ord`.
- filterorsaker visas i rankningsförklaringen.
- 14 Rust-tester, inklusive operator- och filtertest.

## 2026-07-21: Manuell PDF-OCR och bakgrundsjobb

- OCR-knappen stöder nu PNG, JPEG och skannade PDF-dokument.
- PDF-OCR använder lokalt `pdftoppm` och Tesseract, utan nätverksanrop.
- manuell OCR uppdaterar dokumenttyp och datum när deterministiska regler hittar säkra värden.
- efter OCR körs innehållsclaims och entitetskopplingar om; granskningskö och relationer uppdateras direkt i UI.
- beständig jobbstatus använder den befintliga `jobs`-tabellen.
- bevakade mappar kontrolleras automatiskt varje minut medan Vault är öppet.
- jobb kan också startas direkt från UI och senaste status/fel visas.
- automatisk mappskanning hoppar över kända SHA-256-filer och är testad som idempotent.

Verifierat med 13 Rust-tester, lint, språkguard och frontendbygge.

## 2026-07-21: Versioner, konfliktbeslut, påminnelser och bevakade mappar

Ett större workflow-block har implementerats end-to-end:

- schema v8 med beständiga konflikter, användarverifieringar, lokala påminnelser och bevakade mappar.
- nya filer kan läggas till som version 2, 3 och vidare på ett befintligt logiskt dokument.
- tidigare versioner bevaras, endast en filversion markeras aktuell och aktuell text indexeras om.
- versionshistoriken visas i dokumentpanelen med filnamn, importtid och storlek.
- upptäckta konflikter materialiseras med stabil identitet och kan lösas/låsas eller ignoreras.
- användarbeslut sparas separat i `user_verifications` och skrivs till audit-loggen.
- lokala, dokumentkopplade eller fristående påminnelser kan skapas, slutföras och återaktiveras.
- lokala mappar kan registreras och skannas på begäran med samma produktionsimport som vanlig import.
- återkommande mappskanning är idempotent: redan kända filhashar skapar inte nya dokument varje gång.
- nya vyer för Påminnelser och Bevakade mappar samt konflikthantering i Konflikter-vyn.
- integrationstest täcker versioner, påminnelser, bevakade mappar och idempotent skanning.

Verifierat:

```text
npm run build
npm run lint
cargo fmt
cargo test (13 godkända)
```

## 2026-07-21: Inkorg, arkiv och återställningsbar papperskorg

Dokumentens livscykel fungerar nu som ett sammanhängande vardagsflöde:

- separata vyer för alla dokument, inkorg, arkiv och papperskorg.
- statusändringar valideras i Rust-kärnan; godtyckliga statusvärden sparas inte längre.
- arkiverade dokument filtreras fram utan att historik eller sökdata skrivs över.
- borttagna dokument kan listas och återställas från papperskorgen.
- återställning bygger tillbaka dokumentets FTS5-post och skriver en säker audit-händelse.
- redigering och OCR är avstängda i papperskorgen, medan återställning är ett tydligt val.
- tomma vyer visar inte längre ett syntetiskt dokument som om det vore valt.
- Rust-testet för borttagning verifierar nu även papperskorg, återställning och återskapad sökträff.

Verifierat:

```text
npm run build
npm run lint
cargo fmt
cargo test (12 godkända)
```

## 2026-07-21: Vault 1.1.0 Office och relationer

Byggt vidare på kvarvarande premiumfunktioner:

- svensk Tesseract-språkdata paketeras som Tauri-resource.
- appen kopierar automatiskt `swe.traineddata` till Vaults AppData om den saknas.
- DOCX, XLSX och PPTX kan importeras och få lokal textutvinning från Office Open XML.
- importdialogen accepterar nu `.docx`, `.xlsx` och `.pptx`.
- schema v7 lägger till `entities` och `document_entities`.
- importen skapar första lokala relationerna/entities för organisationer och kontaktpunkter.
- detaljpanelen visar relationer per dokument.

Verifierat:

```text
npm run build
npm run lint
cargo test
```

## 2026-07-21: Vault 1.0.3 lokal OCR/PDF-runtime

Installerat och verifierat lokala dokumentverktyg:

- Tesseract OCR installerat via `winget` (`tesseract-ocr.tesseract` 5.5.0).
- svensk Tesseract-språkdata `swe.traineddata` nedladdad från officiella `tesseract-ocr/tessdata`.
- språkdata ligger i Vaults lokala AppData-mapp och i projektets `src-tauri/tessdata`.
- OCR-kommandot använder `--tessdata-dir`, så svensk OCR fungerar utan adminskrivning till Program Files.
- Poppler installerat via `winget` för robustare PDF-verktyg.
- PDF-textutvinning hittar nu `pdftotext` via PATH, Git, WinGet-länkar eller Poppler-installation.
- verifierat svensk OCR med genererad PNG: `Lön juni test DAGAB`.

Verifierat:

```text
tesseract --version
tesseract --list-langs --tessdata-dir %APPDATA%\local.vault.desktop\tessdata
svenskt OCR-röktest med PNG
npm run build
npm run lint
cargo test
```

## 2026-07-21: Vault 1.0.2 språkfix och språkguard

Fixat språk/encoding efter OCR-batchen:

- normaliserat synliga svenska texter i app och Rust-kommandon.
- ersatt ASCII-nödlösningar med riktiga svenska tecken där de syns för användaren.
- lagt till `npm run lint:language`.
- `npm run lint` kör nu både `oxlint` och språkguard.
- språkguarden stoppar mojibake och vanliga ASCII-regressioner som saknade svenska tecken.

Verifierat:

```text
npm run lint:language
```

## 2026-07-21: Vault 1.0.1 OCR-adapter

Byggt första robusta lokala OCR-steget utan att göra Tesseract till ett hårt krav:

- nytt Tauri-kommando `ocr_status`.
- nytt Tauri-kommando `run_ocr_for_production_document`.
- appen detekterar Tesseract via PATH och vanliga Windows-installationsmappar.
- importerade PNG/JPG-filer OCR-läsas automatiskt om Tesseract finns.
- vald importerad PNG/JPG kan OCR-läsas i efterhand och indexeras om.
- OCR-resultat sparas i `documents.extracted_text`, FTS5-indexeras och audit-loggas.
- OCR-status visas i toppbar och lokal statuspanel.
- Test Center kontrollerar att OCR-adaptern kan statuskontrolleras även när Tesseract saknas.

Verifierat:

```text
npm run build
npm run lint
cargo fmt
cargo test
```

## 2026-07-20: Vault 1.0.0 release candidate

Lyft appen till Vault 1.0.0 som komplett första desktoprelease:

- versionsnummer uppdaterat till `1.0.0` i npm, Cargo och Tauri.
- schema v6 lägger till kodordsregister och index for granskningsköer.
- metadata kan redigeras direkt i detaljpanelen for produktionsdokument.
- claims kan godkannas eller avvisas lokalt utan AI.
- taggar kan laggas till och tas bort per dokument.
- kodord kan sparas i ett lokalt register.
- konfliktmotorn listar dublettfiler, motstridiga claims och claims som väntar på granskning.
- Test Center kontrollerar nu aven kodordsregister och konfliktmotor.

Verifierat:

```text
npm run build
npm run lint
cargo fmt
cargo check
cargo test
```

## 2026-07-20: Fas 1 startpunkt

Skapat första körbara appgrund:

- befintligt Vite/React-skelett gjordes till Vault-app,
- Tauri 2 initierades i `app/src-tauri`,
- lokala npm-beroenden för Tauri lades till,
- appnamn, identifierare, fönsterstorlek och scripts uppdaterades,
- Rust-entrypoint bytte från generisk mall till `vault_lib`,
- Tauri command `environment_status` lades till,
- Vite-demot ersattes med en första Vault-arbetsyta,
- standard README i `app/` ersattes med Vault-kommandon,
- layouten verifierades i browser och justerades för att undvika horisontell overflow.
- desktopspåret markerades som primärt: Vite är endast utvecklingspreview, slutprodukten ska byggas som Tauri/Windows `.exe`.
- Tauri bundle-målet sattes till NSIS för Windows-installation.

Verifierat:

```text
npm run build
npm run lint
cargo fmt
cargo check
browser check: page loads, title Vault, lang sv, no Vite overlay, no console errors, no horizontal overflow
```

Kvar:

- Tesseract OCR är fortfarande inte installerat.
- Databas och lokal filförvaring är inte implementerade.
- Importknappen är avstängd tills importflödet finns.
- OCR-knappen är avstängd tills lokal OCR-adapter och Tesseract finns.

## 2026-07-20: Första Windows EXE-build

Förtydligat att Vaults slutprodukt är en Tauri-baserad Windows `.exe`, inte en webbläsarapp. Vite används bara för snabb UI-preview under utveckling.

Ändrat:

- `npm run exe:build` lades till.
- Tauri bundle target sattes till `nsis`.
- Tauri CSP skärptes från `null` till en lokal `self`-policy.
- README-filerna uppdaterades med desktop-kommandon.

Verifierat:

```text
npm run build
npm run lint
cargo check
npm run exe:build
vault.exe smoke test: process started and stopped successfully
```

Skapade native build-filer:

```text
app/src-tauri/target/release/vault.exe
app/src-tauri/target/release/bundle/nsis/Vault_0.1.0_x64-setup.exe
```

Build-output ligger i ignorerade mappar och ska inte versioneras.

## 2026-07-20: Lokal SQLite-grund

Skapat första lokala databasgrunden i Tauri/Rust:

- `rusqlite` med bundled SQLite lades till.
- första migreringen skapades i `app/src-tauri/migrations/0001_initial.sql`.
- `initialize_vault` Tauri command skapades.
- appen skapar lokal AppData-root, `vault.db`, `files`, `index`, `backups` och isolerad `testlab`-mapp.
- UI:t visar schema-version, databasstatus, lokal data-root och DB-sökväg i EXE-läge.

Verifierat:

```text
npm run build
npm run lint
cargo fmt
cargo check
cargo test
npm run exe:build
vault.exe smoke test: process started, initialized local vault data, and stopped successfully
sqlite3 vault.db ".tables"
sqlite3 vault.db "SELECT version, name FROM schema_migrations; SELECT id, name, mode FROM vaults;"
```

Skapad lokal runtime-data vid smoke test:

```text
C:\Users\troll\AppData\Roaming\local.vault.desktop\vault.db
C:\Users\troll\AppData\Roaming\local.vault.desktop\files
C:\Users\troll\AppData\Roaming\local.vault.desktop\index
C:\Users\troll\AppData\Roaming\local.vault.desktop\backups
C:\Users\troll\AppData\Roaming\local.vault.desktop\testlab
```

## 2026-07-20: Isolerad Test Lab-databas

Byggt första riktiga Test Lab-grunden:

- schema v2 lades till via `app/src-tauri/migrations/0002_document_display_fields.sql`.
- migreringsköraren uppgraderar befintlig lokal databas från v1 till v2.
- separat Test Lab-databas skapas på `testlab/vault-test.db`.
- fyra syntetiska dokument seedas i Test Lab med samma `documents`-tabell som produktion.
- UI:t läser dokumentlistan via Tauri-kommandot `list_testlab_documents` i EXE-läge.
- web-preview har bara fallback-data för utveckling.

Verifierat:

```text
npm run build
npm run lint
cargo fmt
cargo check
cargo test
npm run exe:build
vault.exe smoke test
sqlite3 production vault.db schema_migrations
sqlite3 testlab vault-test.db documents
```

Bekräftad lokal data:

```text
production schema migrations: 1, 2
testlab schema migrations: 1, 2
testlab documents: 4
```

## 2026-07-20: Första lokala sökversionen

Byggt första deterministiska Test Lab-sökningen:

- Test Lab-dokumenten skrivs till `document_fts`.
- nytt Tauri-kommando `search_testlab_documents` söker i den separata Test Lab-databasen.
- sökfrågan normaliseras lokalt och expanderas med definierade regler, till exempel `kontrakt -> avtal` och `lön juni -> löneperiod, månad, 06`.
- resultat rankas deterministiskt efter titel, dokumenttyp, datum/period, matchförklaring, metadata och FTS5-träff.
- UI-sökfältet använder Tauri-kommandot i EXE-läge och visar sökpoäng, rankningsförklaring och query plan.

Verifierat:

```text
npm run build
npm run lint
cargo fmt
cargo check
cargo test
npm run exe:build
vault.exe smoke test
sqlite3 testlab vault-test.db FTS query for dagab/kontrakt/avtal
sqlite3 testlab vault-test.db FTS query for juni/06
```

Söktester:

```text
DAGAB kontrakt -> DAGAB anställningsavtal
lön juni -> Lönespecifikation juni
```

## 2026-07-20: Första lokala importflödet

Byggt första verkliga produktionsimporten i EXE-appen:

- Tauri dialog-plugin lades till för lokal filväljare.
- importknappen i UI:t är nu aktiv i EXE-läge.
- `import_document` Tauri-kommandot tar emot vald lokal fil.
- filen hashberäknas med SHA-256.
- filen kopieras till lokal content-addressed storage under `files/<prefix>/`.
- `files`, `documents` och `document_versions` skrivs i production-databasen.
- exakt dubblett upptäcks via SHA-256 och återanvänder befintlig `file_id`.
- importerade dokument indexeras i `document_fts`.
- UI:t visar senaste import, hash, lokal kopia och dubblettstatus.

Verifierat:

```text
npm run build
npm run lint
cargo fmt
cargo check
cargo test
npm run exe:build
vault.exe smoke test
```

Nytt test:

```text
imports_local_file_into_production_vault
```

## 2026-07-20: Produktionsdokument i appytan

Kopplat ihop importflödet med den faktiska dokumentlistan:

- nytt Tauri-kommando `list_production_documents` läser importerade dokument från `vault.db`.
- nytt Tauri-kommando `search_production_documents` söker i produktionsdatabasens FTS-index.
- UI:t visar produktionsdokument som aktiv lista när minst en verklig import finns.
- Test Lab fortsätter vara fallback och isolerad utvecklingsmiljö när produktionen är tom.
- import efter filväljare lägger direkt in dokumentet överst i aktiv lista.
- metrikraden skiljer nu på syntetiska Test Lab-dokument och importerade produktionsdokument.

Verifierat:

```text
npm run build
npm run lint
cargo fmt
cargo check
cargo test
```

## 2026-07-20: Inbyggt Test Center

Byggt ett lokalt Test Center for att kunna kontrollera appens viktigaste floden direkt i EXE-appen:

- nytt Tauri-kommando `run_test_center`.
- kontrollerar produktionsdatabasens integritet och saknade importerade filer.
- kontrollerar att Developer Test Lab har sina fyra syntetiska dokument.
- kör acceptanstester for sökning pa DAGAB-kontrakt och lön juni.
- verifierar att backuplistan och auditloggen kan lasas.
- UI:t har en `Test Center`-knapp i toppbaren.
- lokal status visar antal passed/failed och första felande test om något faller.

Verifierat:

```text
npm run build
npm run lint
cargo check
cargo test
```

## 2026-07-20: Backupvalidering och restore

Byggt säkrare restoreflöde för lokala backup-set:

- nytt Tauri-kommando `validate_local_backup`.
- validering kontrollerar manifest, SHA-256, SQLite `integrity_check`, dokumentantal och filantal.
- nytt Tauri-kommando `restore_local_backup`.
- restore kräver godkänd validering innan något skrivs.
- restore skapar alltid en pre-restore-backup först.
- restore kopierar tillbaka `vault.db` och backupens `files/`-träd.
- backupnamn görs unika även om flera backupoperationer sker samma sekund.
- UI:t har knappar för att validera och återställa senaste backup.

Verifierat:

```text
npm run build
npm run lint
cargo fmt
cargo check
cargo test
```

## 2026-07-20: Tags, claims och PDF-textadapter

Större funktionsbatch mot fullare app:

- schema v5 lägger till `tags`, `document_tags` och `claims`.
- import skapar automatiska lokala taggar baserat på dokumenttyp.
- import skapar claims för dokumenttyp, SHA-256 och tolkat dokumentdatum.
- claims markeras med status, value kind, confidence kind och extraktionsmetod.
- dokumentdetalj hämtar och visar taggar och claims.
- PDF-import försöker extrahera text via lokal `pdftotext` om verktyget finns.
- PDF-import faller tillbaka säkert till tom text om verktyget saknas eller filen inte kan läsas.

Verifierat:

```text
npm run build
npm run lint
cargo fmt
cargo check
cargo test
```

## 2026-07-20: Audit, backuplista och sökindexreparation

Större stabilitetsbatch:

- schema v4 lägger till `audit_events`.
- import, backup och sökindexreparation skriver lokala audit-händelser.
- nytt Tauri-kommando `list_audit_events` visar senaste händelser.
- nytt Tauri-kommando `list_local_backups` läser backupmanifest från backupmappar.
- nytt Tauri-kommando `rebuild_production_search_index` bygger om FTS-index från dokumentmetadata och extraherad text.
- UI:t har nu `Reparera sök`.
- lokal status visar senaste reindex-resultat, antal backup-set och senaste audit-händelse.

Verifierat:

```text
npm run build
npm run lint
cargo fmt
cargo check
cargo test
```

## 2026-07-20: Deterministisk datumtolkning vid import

Utökat den lokala importanalysen med datumregler:

- import försöker sätta `document_date` när tydliga datum hittas i filnamn eller extraherad text.
- stöder `YYYY-MM-DD`.
- stöder kompakt `YYYYMMDD`.
- stöder svenskt månadsnamn med år, exempelvis `juni 2024`, som sparas som första dagen i månaden.
- fristående månad utan år lämnar datum tomt för att undvika gissning.
- ogiltiga datum, exempelvis `2023-02-29`, ignoreras.
- datumregeln läggs in i matchorsaken och datumet indexeras som metadata.

Verifierat:

```text
npm run build
npm run lint
cargo fmt
cargo check
cargo test
```

## 2026-07-20: Deterministisk dokumenttypning vid import

Lagt till första lokala regelbaserade analyssteget:

- importerade dokument klassificeras deterministiskt vid import.
- regler matchar filnamn och extraherad text, inte AI eller externa tjänster.
- första dokumenttyperna: `Lönespecifikation`, `Anställningsavtal`, `Identitetshandling`, `Kvitto`.
- dokument som inte matchar regler sparas fortsatt som `Importerad fil` för manuell granskning.
- matchorsaken visar vilken lokal regel som slog.
- FTS-indexet får dokumenttypen som metadata så sökningen rankar bättre.

Verifierat:

```text
npm run build
npm run lint
cargo fmt
cargo check
cargo test
```

## 2026-07-20: Lokal arkivkontroll

Lagt till icke-destruktiv hälsokontroll för produktionsarkivet:

- nytt Tauri-kommando `check_vault_health`.
- kör SQLite `PRAGMA integrity_check`.
- räknar schema-version, aktiva dokument, tillgängliga filer och FTS-indexrader.
- varnar om importerade filer saknas på disk.
- varnar om FTS-index har färre rader än aktiva dokument.
- UI:t har nu `Kontrollera arkiv` i toppbaren.
- lokal status visar senaste kontrollresultat, integritet, innehåll och saknade filer.

Verifierat:

```text
npm run build
npm run lint
cargo fmt
cargo check
cargo test
```

## 2026-07-20: Lokal verifierbar backup

Byggt första lokala backupflödet för produktionsdatabasen:

- nytt Tauri-kommando `create_local_backup`.
- backup skapas med SQLite `VACUUM INTO` för konsistent `.db`-kopia även med WAL-läge.
- varje backup blir en egen timestampad mapp under lokal `backups`.
- backupmappen innehåller `vault.db`, `manifest.json` och kopior av importerade originalfiler.
- originalfiler kopieras med samma relativa `files/<prefix>/...`-struktur som produktionen.
- manifestet innehåller SHA-256, storlek, dokumentantal, filantal och lokal policy.
- UI:t har nu `Skapa backup` i toppbaren.
- detaljpanelen visar senaste backupmapp, databasbackup, manifest, hash och kopierade filer.

Verifierat:

```text
npm run build
npm run lint
cargo fmt
cargo check
cargo test
```

## 2026-07-20: Återställbar Developer Test Lab

Gjort Test Lab-miljön återställbar utan att påverka produktion:

- nytt Tauri-kommando `reset_testlab` tömmer syntetiska dokument, versioner, filer och FTS-index.
- samma seedfunktion bygger därefter upp de fyra syntetiska fixtures igen.
- produktionsdatabasen och importerade originalfiler lämnas orörda.
- sidomenyn har nu `Återställ Test Lab`.
- UI:t uppdaterar Test Lab-listan direkt efter reset.

Verifierat:

```text
npm run build
npm run lint
cargo fmt
cargo check
cargo test
```

## 2026-07-20: Öppna lagrade originalfiler

Gjort dokumentflödet mer komplett i Windows-appen:

- nytt Tauri-kommando `open_production_document_file` öppnar lagrad import med Windows standardapp.
- nytt Tauri-kommando `reveal_production_document_file` markerar lagrad import i Utforskaren.
- filupplösningen går via SQLite och content-addressed storage, inte via osäker UI-state.
- detaljpanelen har knappar för `Öppna fil` och `Visa i Utforskaren`.
- knapparna är automatiskt avstängda för Test Lab-dokument och dokument utan lagrad fil.
- fel vid filåtgärder visas i detaljpanelen.

Verifierat:

```text
npm run build
npm run lint
cargo fmt
cargo check
cargo test
```

## 2026-07-20: Textutvinning, dokumentdetalj och teman

Förbättrat import och visning så appen beter sig mer som ett faktiskt lokalt arkiv:

- schema v3 lägger till `documents.extracted_text`.
- importerade `.txt`, `.md`, `.csv` och `.json` läses deterministiskt lokalt och indexeras i FTS5.
- dokumentdetalj-kommandon hämtar extraherad text, lokal filväg, filtyp och storlek.
- previewytan visar sparad extraherad text när den finns.
- produktionssökning hittar nu innehåll inuti importerade textfiler.
- dokumentlistan sorteras nyast först.
- explicit temaväxling `Auto`, `Ljus`, `Mörk` lades till ovanpå design tokens.

Verifierat:

```text
npm run build
npm run lint
cargo fmt
cargo check
cargo test
```
# 2026-07-21 — OCR-jobb, dokumentsidor och källbundna claims

- Lagt till beständiga dokumentsidor och textspans med sidnummer, källtyp och kvalitetsstatus.
- OCR körs som lokalt bakgrundsjobb med progress, pausbegäran och återupptagning sida för sida.
- PDF-resultat sparas löpande per sida och indexeras först deterministiskt när jobbet är klart.
- Claims har nu exakt textspan-/sidkälla, giltighetsfält, aktualitetsstatus och regelbaserad förklaring.
- Befintliga dokument får källreferenser backfillade när analys eller OCR körs igen.
- Release-EXE verifierad med verklig PDF: köad OCR, running-status, sidprogress, indexerat slutresultat och schema v10.

## Aktualitet, tidslinje och domänöversikter

- Deterministisk aktualitetsmotor skiljer på aktuell, historisk, framtida och osäker utifrån uttryckliga giltighetsdatum.
- Importdatum används aldrig som bevis för aktualitet; varje automatisk status visar sin regelbaserade förklaring.
- Manuella beslut kan låsa en claim som aktuell, historisk eller osäker och sparas i verifieringshistoriken.
- Ny källbunden tidslinje visar dokumentdatum, dokumenttyp, kopplade entiteter, källsida och förklaring.
- Separata vyer för personer, företag samt objekt/fordon använder det lokala entitetsregistret.
- Release-EXE klicktestad: tidslinje med åtta daterade poster, företagsvy med DAGAB/WILLYS och manuell `manual_current`-låsning.
- Schema v11 inför ett beständigt, generellt domänregister för arbete/lön, utbildning, fordon, identitet, avtal och produkter.
- Domänposter bär uttrycklig period, ämne, strukturerade claims, källdokument, källsida, extraktionsmetod och förklarad aktualitetsstatus.
- Lönebesked/kvitton bevaras som historikposter; öppna anställningsperioder markeras utan antagande om att de fortfarande pågår.
- Arkivanalys backfillar sidmodellen för äldre textdokument så uppgraderade arkiv får samma källspårning som nya importer.
- Release-EXE verifierad med DAGAB-anställning, WILLYS-löneperioder och backfillad `källa sida 1`.
# 2026-07-21 — Avancerad import, regler och lokal säkerhet

- Schema v12 ger beständig importhistorik, säker ZIP-import, EML-textutvinning, urklippsimport och drag-and-drop.
- Schema v13 ger versionshanterade automationsregler, dokumentmallar, egna metadatafält och sparade sökningar med förklarad regelhistorik.
- Schema v14 ger frivilliga säkerhetslägen, saltad iterativ PIN-hashning, automatisk sessionslåsning, dokumentlås och maskerad säker export.
- Backend nekar öppning, visning och originalexport av låsta dokument utan korrekt sessions-PIN.
- Lösenordsskyddad backup använder AES-256, innehåller databas, originalfiler och manifest samt får SHA-256 och innehållsfri revisionshändelse.
- Release-EXE verifierad med schema v14, fungerande säkerhetsvy och skapad `.vaultzip` med 25 krypterade poster.
- Frontend build/lint och 16 Rust-tester godkända; NSIS-installationspaket byggt.

## 2026-07-21 — Teman och avancerade arkivvyer

- Schema v15 innehåller elva beständiga teman, egen theme editor, design tokens, täthet, rörelseinställning, duplicering samt lokal import/export.
- Warm Paper aktiverades i release-EXE:n och låg kvar efter full omstart av processen.
- Schema v16 lägger till beständiga favoriter, färgkodade/fästa samlingar och dokumentmedlemskap.
- Dokumentytan stöder lista, rutnät och kompakt tabell samt särskilda vyer för Favoriter och Senaste.
- Ctrl+K öppnar en körbar kommandopalett för navigation, import, analys, teman, säkerhet och Test Center.
- Release-EXE verifierad med schema v16, Ctrl+K → Visa senaste, rutnätsvy, beständig favorit och skapad samling `Viktiga avtal`.

## 2026-07-21 — Analysomfattning, skanner och webbkopia

- Schema v17 lagrar åtta analyslägen, separata val för OCR/klassificering/claims/relationer och explicit opt-in för låsta dokument.
- Dokument, mappar, kategorier och dokumenttyper kan undantas med lokal förklaring.
- Windows WIA-skannerguiden detekteras lokalt och kan öppnas från ett tydligt skannerflöde; sparade skanningar importeras genom ordinarie produktionskod.
- Webbsidor kan sparas som lokala dokument från inklistrad HTML/text med käll-URL, tidpunkt och deterministisk borttagning av markup.
- Release-EXE verifierad med schema v17, upptäckt `wiaacmgr.exe`, synlig analyseditor och importerad syntetisk webbkopia; produktionsarkivet ökade från 28 till 29 dokument.

## 2026-07-21 — Utökat Developer Test Lab

- Test Center kör hela sviten eller moduler för sök, lagring, analys och OCR.
- Kontrollerade simuleringar täcker OCR-, databas- och filfel, avbrott samt återställning utan att ändra produktion.
- Skaltest genererar 100, 1 000, 10 000 eller 50 000 tydligt märkta syntetiska dokument och mäter importtid, söktid och databasstorlek.
- Säker testrapport exporteras som innehållsfri JSON med miljömärkning och SHA-256.
- Ett EXE-upptäckt regressionsfel i fixture-räkningen rättades så grundfixtures verifieras även när skaldata finns.
- Release-EXE verifierad med 100 genererade dokument (104 totalt), 5 ms import, 0 ms testsökning, kontrollerad `FILE_NOT_FOUND`, exporterad rapport och 11/11 godkända Test Center-fall.
- Rust-sviten omfattar nu 18 tester, inklusive bevis att skaldata stannar i Test Lab och lämnar produktionsdatabasen tom.
## 2026-07-21 — Organisationsblock (schema v18)

- Riktiga lokala mappar och kategorier med färg, fästning, dokumentantal och dokumentkopplingar.
- Dokumentanteckningar av typerna anteckning, att göra och beslut, med oföränderlig versionshistorik.
- Beständig lokal sökhistorik som registreras med Enter, dedupliceras och kan återanvändas.
- Nya navigationsvyer och kommandopalettkommandon för mappar, kategorier och sökhistorik.
- Dokumentpanelen kan växla mapp/kategori och skapa eller redigera anteckningar.
- Verifierat med 18/18 Rust-tester, frontend-lint, språkguard, TypeScript/Vite-build och paketerad release-EXE.
- Release-EXE startad och visuellt smoke-testad med schema v18 och Ctrl+K-kommandopaletten.

## 2026-07-21 — Kalender, graf och utökade domäner (schema v19)

- Lokal kalender sammanför manuella händelser, påminnelser och uttryckliga domändatum utan extern kalender eller antaganden.
- Sökbar relationsgraf visar dokument, personer, organisationer, objekt och kategorier samt källförklarade kanter.
- Boende-, rese- och myndighetsvyer extraherar endast uttryckliga fält; myndighetsvyn gör ingen juridisk tolkning.
- Release-EXE byggd och startad med schema v19; frontend och 20 Rust-tester godkända.

## 2026-07-21 — Portabilitet, dubbletter och filintegritet (schema v20)

- Flyttbar `.vaultarchive` innehåller SQLite-databas, originalfiler, manifest och en läsbar CSV-dokumentlista.
- Säker import avvisar fel filtyp, saknat manifest/databas och ZIP-sökvägar som försöker lämna stagingmappen.
- Återställning skapar alltid en lokal säkerhetskopia av det befintliga arkivet innan ersättning.
- Integritetskontroll räknar intakta, saknade och ändrade original genom jämförelse med importerad SHA-256.
- Dubblettförslag förklaras med exakt filhash eller identisk normaliserad text och kräver alltid manuellt beslut; ingenting raderas automatiskt.

## 2026-07-21 — Krypterad privat sektion och gästläge (schema v21)

- Fjärde säkerhetsnivån, Krypterad privat sektion, kompletterar bekvämt läge, PIN-läge och fullt låst läge.
- Privata original flyttas till en AES-256-krypterad `.vaultprivate`-behållare först efter lyckad dekrypterings- och SHA-256-verifiering.
- Originalets okrypterade lagringsfil tas bort först när den krypterade kopian verifierats; återställning kräver rätt PIN/lösenord och kontrollerar ursprunglig hash.
- Privata dokument är samtidigt låsta och dolda. Tillfällig öppning dekrypterar endast till en separat förhandsvisningskatalog och loggar en innehållsfri säkerhetshändelse.
- Gästläge döljer privata dokument från dokumentlistor, sökresultat och förhandsvisning utan att ändra data.
- Säker originalexport stöder även krypterade privata behållare; privat behållare visas aldrig direkt i Utforskaren.

## 2026-07-21 — Isolerade lokala plugins (schema v22)

- Plugins är deklarativa JSON-manifest och får inte innehålla eller starta exekverbar kod.
- Tillåtelselistan begränsar åtkomst till metadata eller dokumenttext och utdata till taggar eller dokumenttyp; nätverk, processer, AI och fri filåtkomst avvisas.
- `ai_free: true`, giltigt versionsmanifest och explicit deklarerade behörigheter krävs före installation.
- Installerade plugins är avstängda tills användaren granskar dataåtkomsten och uttryckligen godkänner aktivering.
- Låsta och privata dokument hoppas alltid över vid plugin-körning; varje installation, aktivering och körning får en innehållsfri lokal audit-händelse.
- Plugins kan inaktiveras eller tas bort utan att kärnfunktioner eller originaldokument påverkas.
# 2026-07-21 – storskalighet, tillgänglighet och arkivdashboard

- Test Lab-generatorn använder nu cachelagrade SQLite-satser och direkt FTS-inmatning i en transaktion. Release-EXE:n skapade och indexerade 50 000 syntetiska dokument; den isolerade databasen verifierades till 50 004 dokument och 50 004 FTS-poster medan appen svarade normalt och använde cirka 28 MB arbetsminne efter körningen.
- Dokumentresultat fönstras i grupper om 100 med föregående/nästa-navigering så att tusentals träffar inte skapar tusentals DOM-rader.
- Tillgänglighet: överhoppningslänk till arbetsytan, synliga `focus-visible`-markeringar och respekt för Windows/OS-inställningen för reducerad rörelse.
- Startsidan har fått ett verkligt arkivdashboard med granskningskö, senaste dokument, saknade datum/kategorier, kommande datum, favoriter, arkivstorlek, konflikter, backup och sökindex. Widgets kan flyttas, döljas, dupliceras och återställas; layouten sparas lokalt och kan exporteras som versionsmärkt JSON.
- Verifiering: `npm run lint`, TypeScript/Vite-produktionsbygge och samtliga 23 Rust-tester godkända. Ny `vault.exe` och NSIS-installer byggda. Dashboarden öppnades i den paketerade release-EXE:n och dess verkliga arkivvärden samt anpassningskontroller verifierades via Windows accessibility-trädet.

## Kontrollerbar sökhistorik

- Sökhistorik kan nu stängas av helt, begränsas till 1–3650 dagars lagring, fästas per post och rensas utan att fästa poster försvinner.
- Policyn sparas i den lokala Vault-databasen. Avstängt läge gör att nya sökningar inte registreras.
- Historiken kan exporteras till ett lokalt, versionsmärkt JSON-format utan nätverksanrop.
- Release-EXE-test: historiken stängdes av och sparades, tomläget ändrades till ”Historiken är avstängd”, JSON-export skapades i Vaults lokala exportmapp och standardpolicyn återställdes till aktiverad/90 dagar. Databasen verifierades efteråt.

## Schema v23 – länkade anteckningar

- Ny separat anteckningsmodell för egna arbetsdata, med länkar till hela Vault, dokument, personer, företag, objekt, kategorier, händelser, claims och konflikter.
- Markdown-källa, checklistor, interna `vault://`-länkar, taggar, kodord, sökning, två lokala mallar och oförstörande versionshistorik stöds.
- Anteckningar är tekniskt och visuellt separerade från källbundna claims och påverkar aldrig aktualitetsmotorn.
- Backendtest verifierar länkat objekt, två versioner och installerade mallar. Totalt 24/24 Rust-tester passerar.
- Release-EXE-test: anteckning skapades med checklista, intern dokumentlänk, två taggar och kodord; posten redigerades till version 2 och raderades därefter. Navigeringsräknaren gick 0 → 1 → 0 och produktion lämnades ren.

## Avancerad lokal backup

- Beständig policy för automatisk lokal backup med intervall 1–8760 timmar, paus/återupptagning, full eller inkrementell körning och dokumentundantag.
- Bakgrundsarbetaren kontrollerar policyn lokalt varje minut. Standardläget är avstängt så ingen schemaläggning aktiveras utan användarens val.
- Inkrementella set är fristående återställningspunkter: oförändrade original hårdlänkas från föregående set på samma filsystem och kopieras om hårdlänkning inte går. Backupdatabasen är alltid en full SQLite-ögonblicksbild.
- Undantagna dokument och original tas bort endast ur backupkopian; produktionsarkivet ändras aldrig.
- Backendtest verifierar full kopia, inkrementell hårdlänkning, fristående validering och dokumentundantag. 25/25 Rust-tester passerar.
- Release-EXE-test på det riktiga arkivet skapade ett inkrementellt set med 29 dokument och 24 original: 1 fil kopierades och 23 hårdlänkades. SHA-256 matchade manifestet exakt, SQLite `integrity_check` var `ok` och backupen innehöll 29 dokument/24 filer.

## Avancerad dubblettanalys och låsning efter vila

- Integritetskontrollen kombinerar nu SHA-256, exakt normaliserad text, SimHash för nästan identisk text, korsformat, exakta sidkopior och perceptuell bildhash för skanningar/foton.
- Varje förslag visar konfidens och förklaring. Inget raderas automatiskt och tidigare användarbeslut respekteras.
- En skyddad session låses när appen återkommer efter Windows-låsning eller vila på minst 15 sekunder, utöver tidsgräns och manuell låsning.
- Verifierat med språkgranskning, frontendbygge, 26/26 Rust-tester, ny release-EXE och nytt NSIS-installationspaket.
## Låsta organisatörer, Windows-inloggning och komplett dokumentarbetsyta

- Databasschema v24 lade till PIN-skyddade mappar och kategorier. Dokument i en låst organisatör omfattas nu av samma åtkomstkontroll vid öppning, visning i Utforskaren och säker export.
- Windows Hello-status och lokal verifiering byggdes som en frivillig upplåsningsväg. Vanlig PIN-användning är fortsatt huvudflödet och funktionen är inte ett krav.
- Databasschema v25 lade till permanenta sidbundna bokmärken, markeringar och kommentarer samt kommentar och ursprung för dokumentversioner.
- Dokumentvisaren har sidnavigering, dokumentsökning, zoom 50–200 %, rotation och en responsiv helskärmsvy som är nåbar även när den normala trepanelsvyn inte ryms.
- Versionshistoriken skriver aldrig över äldre versioner. Två versioner kan jämföras deterministiskt via tillagd/borttagen text, SHA-256, filstorlek, sidantal och dokumentdatum. En äldre version kan återställas som aktuell och dokumentets claims sätts samtidigt tillbaka i granskningsflödet.
- Verifierat med lint, språkguard, TypeScript/Vite-build, 26/26 Rusttester, releasebygge och NSIS-installer.
- Verifierat i den faktiska release-EXE:n mot produktionsarkivet: schema v25, 29 dokument, OCR/PDF/Office redo samt att knappen **Öppna dokumentvisare** öppnar en fungerande helskärmsvy med OCR-text och visarkontroller.
## Dashboardlayouter och kommandopalett

- Kommandopaletten innehåller nu navigation, import, analys, hälsokontroll, indexreparation, dubblettkontroll och granskningsflöden.
- Egna tangentkombinationer kan registreras lokalt per kommando. Dubbletter blockeras, Backspace/Delete rensar en kombination och genvägar körs inte medan användaren skriver i formulärfält.
- Dashboarden har 18 datadrivna widgettyper, tre storlekar, flyttning, döljning, duplicering och möjlighet att lägga tillbaka en widget.
- Flera namngivna layouter kan sparas, växlas och tas bort lokalt. Exportformatet v2 inkluderar widgetordning, storlekar och layoutnamn.
- Verifierat med lint, språkguard, TypeScript/Vite-build, 26/26 Rusttester, releasebygge och NSIS-installer.
- Verifierat i release-EXE:n: dashboardens **Anpassa** visar layoutnamn, sparning, widgetväljare och storlekskontroller; Ctrl+K öppnar kommandopaletten och **Kortkommandon** visar redigeringsfält för varje kommando.
