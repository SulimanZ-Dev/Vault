# Vault

Vault 1.1.0 är ett lokalt, privat och helt AI-fritt dokumentarkiv för Windows.
Appen är byggd med Tauri, React, Rust, SQLite och FTS5 och fungerar utan konto,
molntjänst eller telemetri.

[Hämta senaste Windows-installationen](https://github.com/SulimanZ-Dev/Vault/releases/latest)

## Funktioner

- Import av filer, mappar, ZIP, EML, urklipp och lokalt sparade webbsidor.
- Svensk OCR med Tesseract samt PDF-text och OCR-fallback med Poppler.
- Lokal textutvinning ur PDF, DOCX, XLSX och PPTX.
- Intern PDF-visare med sidnavigation, zoom, rotation, miniatyrer, textlager,
  markeringar, versioner och dokumentjämförelse.
- Snabb lokal fulltextsökning med svenska regler, filter, operatorer, synonymer,
  sökhistorik och sparade sökningar.
- Claims, källor, granskningskö, relationer, graf, tidslinje, kalender,
  konflikter och låsta manuella beslut.
- Strukturerade lokala modeller för pass och identitet, fordon och service,
  anställningar, lönehistorik, löneperioder, tilläggsavtal och garantier.
- Regler med AND/OR/NOT, dokumentmallar, egna fält och batchåtgärder.
- Mappar, kategorier, samlingar, favoriter och länkade anteckningar.
- Lista, rutnät, tabell, galleri, kanban och delad dokumentvy.
- Fullständigt anpassningsbara lokala teman och layouter.
- Lokala notiser för granskning, utgående giltighet och misslyckade jobb.
- Full, inkrementell, schemalagd och AES-256-krypterad backup.
- Export till JSON, CSV, `.vaultzip` och `.vaultarchive`.
- PIN/lösenord, sessionslås, dokumentlås, gästläge, DPAPI-snabbupplåsning och
  krypterad privat sektion.
- SHA-256-integritetskontroll och detektering av exakta och nära dubbletter.
- Deklarativt lokalt plugin-API med sju adaptertyper och utan nätverks-,
  process- eller AI-behörighet.
- Isolerat Test Lab med 57 syntetiska grundfall och skaltest upp till
  50 000 dokument.

## Integritet och avgränsningar

Vault lagrar dokument, metadata, OCR-text, databas, index och historik lokalt.
Programmet använder inga språkmodeller, embeddings eller externa
dokumenttjänster.

Följande ingår avsiktligt inte:

- synkning mellan datorer eller enheter,
- OAuth-integrationer för Drive, OneDrive, Gmail eller Outlook,
- kameraimport eller komplett styrning av fysisk skanner,
- mobil companion,
- Windows Hello.

## Installera

Öppna [GitHub Releases](https://github.com/SulimanZ-Dev/Vault/releases/latest)
och hämta `Vault_1.1.0_x64-setup.exe`. Installern innehåller den färdiga
Windows-appen.

Den fristående `vault.exe` publiceras också för den som inte vill använda
installern.

## Utveckling

Krav:

- Node.js och npm
- aktuell stabil Rust toolchain
- Tauri-förutsättningar för Windows
- Tesseract med svensk språkdata
- Poppler

Starta utvecklingsversionen:

```powershell
cd app
npm install
npm run tauri:dev
```

Kör kontroller:

```powershell
cd app
npm run lint
npm run test:ui
npm run build

cd src-tauri
cargo test
```

Bygg optimerad EXE och NSIS-installer:

```powershell
cd app
npm run tauri build
```

## Dokumentation

- [Full kravstatus](KVAR_INNAN.md)
- [Produktdefinition](docs/00-product-definition.md)
- [Teknisk arkitektur](docs/01-technical-architecture.md)
- [Datamodell](docs/02-data-model.md)
- [Sökning, OCR och regler](docs/03-search-ocr-rules.md)
- [Säkerhet, lagring och plugins](docs/04-security-storage-plugins.md)
- [Test Lab och acceptanstester](docs/06-test-lab-and-acceptance.md)
- [Portabla format](docs/PORTABLE_FORMATS.md)
- [Plugin Adapter API](docs/PLUGIN_ADAPTER_API.md)
- [Implementationslogg](docs/10-implementation-log.md)

## Verifierad release

Version 1.1.0 är verifierad med:

- 35 Rust-tester,
- 22 UI- och tillgänglighetskontroller,
- TypeScript/Vite-produktionsbygge,
- faktisk start och funktionsprov i paketerad `vault.exe`,
- Test Center i release-EXE: 19 godkända och 0 misslyckade.

## Licens

[MIT](LICENSE)
