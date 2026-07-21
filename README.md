# Vault

Vault 1.0.0 byggs som en Tauri-baserad Windows EXE. Vite/webbläsarläget ar bara en utvecklingspreview, inte slutprodukten.

Vault är ett lokalt, privat och helt AI-fritt dokumentarkiv för personligt bruk.

Målet är ett professionellt dokumenthanteringssystem som kan importera, organisera,
söka, indexera och strukturera dokument utan molnkrav, telemetri, språkmodeller,
embeddings, AI-klassificering eller externa dokumenttjänster.

Projektet byggs iterativt. Innan applikationskod skrivs finns den första
arkitektur- och produktleveransen i `docs/`.

## Innehåll i 1.0.0

- Windows EXE och NSIS-installer.
- Lokal SQLite-databas med migreringar.
- Lokal filimport med SHA-256, dubblettdetektion och content-addressed storage.
- FTS5-sök, svensk regelbaserad query expansion och förklarad rankning.
- Deterministisk dokumentklassificering och datumutvinning.
- Taggar, kodord, claims och manuell claim-granskning.
- Metadataredigering for produktionsdokument.
- Lokal backup, validering, restore och pre-restore-sakerhetskopia.
- Audit-logg, hälsokontroll, sökindexreparation och inbyggt Test Center.
- Developer Test Lab i separat databas med syntetiska dokument.

## Första leveransen

- `docs/00-product-definition.md` - AI-fri produktdefinition, flöden och principer.
- `docs/01-technical-architecture.md` - rekommenderad stack, alternativ och systemarkitektur.
- `docs/02-data-model.md` - databastabeller, relationer, claims, källor, aktualitet och konflikter.
- `docs/03-search-ocr-rules.md` - lokal OCR, fulltextsökning, svensk sökning, rankning och regelmotor.
- `docs/04-security-storage-plugins.md` - lokal filförvaring, säkerhetsmodell, backup och pluginarkitektur.
- `docs/05-design-system.md` - informationsarkitektur, navigering, design tokens och mockupbeskrivningar.
- `docs/06-test-lab-and-acceptance.md` - Developer Test Lab och acceptanstester.
- `docs/07-roadmap.md` - små milstolpar, första fungerande version, risker och uppskjutna funktioner.
- `docs/08-pre-implementation-checklist.md` - spårning av alla 47 punkter som ska levereras före implementation.
- `docs/09-dev-environment.md` - installerade utvecklingsverktyg och kvarvarande OCR-punkt.
- `docs/10-implementation-log.md` - genomförda implementationsteg och verifiering.

## Kör appen lokalt som desktop-app

```powershell
cd app
npm install
npm run tauri:dev
```

Skapa Windows `.exe`/installer:

```powershell
cd app
npm run exe:build
```

`npm run dev` används bara som snabb UI-preview under utveckling. Vaults slutprodukt ska vara en Tauri-baserad Windows `.exe`, inte en webbläsarapp.

## Absoluta principer

- Local-first och offline som standard.
- Ingen AI-funktionalitet, inga AI-beroenden och ingen AI-förberedande kärnarkitektur.
- Alla automatiska beslut ska vara deterministiska, testbara och förklarbara.
- Originaldokument, databas, metadata, index, OCR-text och verifieringshistorik lagras lokalt.
- Historik skrivs inte över av nya dokument.
- Osäkerhet visas som osäkerhet eller konflikt, inte som gissning.
- Manuellt låsta användarbeslut går före automatiska regler.
