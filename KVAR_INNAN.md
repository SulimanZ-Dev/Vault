# Full kravstatus

Statusen avser originalprompten med de uttryckliga avgränsningar som beslutats
under utvecklingen. Vault är en lokal Tauri/React/Rust-app utan konto, molnkrav
eller AI-tjänst.

## Uttryckligen bortvalt

- Mobil companion.
- Synkning mellan egna datorer eller enheter.
- Riktiga OAuth-integrationer för Drive, OneDrive, Gmail och Outlook.
- Kameraimport och komplett styrning av fysisk skanner.
- Windows Hello. PIN, lösenord, DPAPI-snabbupplåsning och krypterad privat
  sektion finns kvar.

## Färdig funktionalitet

- Import av filer, mappar, ZIP, EML, urklipp och lokalt sparade webbsidor.
- Lokal OCR med Tesseract och svenska språkdata, PDF-text med Poppler och
  OCR-fallback för skannade PDF:er.
- Textutdrag ur DOCX, XLSX och PPTX.
- Intern PDF-visare med sidor, zoom, rotation, miniatyrer, textlager,
  markeringar, källkoordinater, versioner och jämförelse.
- SQLite/FTS5-sökning, svenska regler, synonymer, operatorer, kombinerade filter,
  förklarad rankning, historik och sparade sökningar.
- Claims, källor, granskningskö, entiteter, relationer, graf, tidslinje,
  kalender, konflikter, låsta beslut och aktualitet.
- Djupa lokala modeller för identitet/pass, fordon/service, arbete,
  avtalad lönehistorik, separata löneperioder, tilläggsavtal, produkter och
  garantier. Systemet gissar inte mellan motstridiga källor.
- Regler med AND/OR/NOT, mallar, egna fält, batchåtgärder och beständiga
  bakgrundsjobb med paus, återupptagning, avbrott och retry.
- Mappar, kategorier, samlingar, favoriter, anteckningar och dashboard.
- Lista, rutnät, tabell, galleri, kanban och delad dokumentvy samt sparad
  panelbredd och panelvisning.
- Fulla lokala temaprofiler med färger, typografi, radavstånd, radie, skuggor,
  transparens, blur, animation, bakgrund, schema, Windows-/dag-nattläge,
  import, export och duplicering.
- Lokala notiser för granskning, utgående giltighet och misslyckade jobb.
- Full, inkrementell och schemalagd backup; extern lokal destination;
  validering; isolerat återställningsprov; AES-256-skyddad backup.
- JSON-, CSV-, `.vaultzip`- och `.vaultarchive`-export med dokumenterade,
  versionsstyrda format.
- PIN/lösenord, sessionslås, dokumentlås, gästläge, DPAPI-snabbupplåsning,
  AES-256-krypterad privat sektion, audit och säker temporär filhantering.
- Integritetskontroll, SHA-256, exakta och nära dubbletter, manuella beslut,
  ZIP-bomb-, traversal-, symlink- och storleksgränser.
- Lokalt deklarativt plugin-API 1.0 med sju adaptertyper, körbara
  adapterartefakter, resursgränser, Ed25519, checksumma, explicit godkännande
  och förbud mot nätverk, processkörning, AI och fri filåtkomst.
- Isolerat Test Lab med 57 breda syntetiska grundfall, skaltest upp till
  50 000 dokument, modultester, felsimulering och säker rapport.

## Slutverifiering

- Rust: 35 automatiska tester.
- UI/tillgänglighet: 22 automatiska kontroller.
- TypeScript/Vite: lint, språkguard och produktionsbygge.
- Tauri: optimerad `vault.exe` och NSIS-installer.
- Release-EXE: start, schema, OCR/PDF/Office-diagnostik, navigation,
  dashboard, notiser och Test Center provas i den faktiska Windows-appen.
