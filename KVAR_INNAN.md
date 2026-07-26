# Kvar innan hela originalprompten är uppfylld

Detta dokument spårar status för varje krav i originalprompten.
Statusar: ✅ Fullt implementerad | 🔷 Implementerad (kräver EXE-verifiering) | 🔶 Delvis | ❌ Saknas | ➖ Inte relevant

## 1. Dokumentvisaren
- [x] ✅ PDF-rendering internt (PDF.js är bundlat offline och verifierat i release-EXE)
- [x] ✅ Sidnavigation, zoom, passning, rotation och återställning av senaste läge
- [x] ✅ Miniatyrer med fönstrad lazy rendering
- [x] ✅ Valbart textlager och lokala sökträffar
- [x] ✅ Versions- och sidbundna markeringar med normaliserade koordinater
- [ ] 🔷 Claim till exakt källsida, söktext och koordinatmarkering
- [ ] 🔷 Dokumentjämförelse sida vid sida med ordningsbevarad text och markerade skillnader

## 2. Djupare områdesmodeller
- [x] ✅ Grundläggande domänregister (schema v11)
- [x] ✅ Boende-, rese- och myndighetsvyer (schema v19)
- [ ] 🔷 Deterministisk TD3-MRZ-parser för pass med kontrollsiffror och källfält
- [ ] 🔶 Fordon och service (schema samt lokalt utdrag av VIN, regnummer, miltal och verkstad)
- [ ] 🔶 Garanti och produkter (schema samt lokalt utdrag av garantiperiod, slut och reklamation)
- [ ] 🔶 Anställning (schema samt lokalt utdrag av alias, organisationsnummer, befattning och sysselsättningsgrad)
- [ ] 🔶 Lönehistorik och tilläggsavtal (separata historiktabeller och lokala källfält)
- [ ] 🔷 Konfliktgranskning där användaren väljer källa/båda/ingen och beslutet låses

## 3. Sökning och sparade sökningar
- [x] ✅ Fritextsökning med FTS5
- [x] ✅ Sökoperatorer (type:, tag:, date:, status:, -ord, exakt fras)
- [ ] 🔷 Deterministiskt språkstöd för vanliga svenska böjningsformer, sammansättningar, synonymer och OCR-felen 0/o samt 1/l
- [x] ✅ Kombinerbara filter för dokumenttyp, kategori, arbetsgivare/entitet, status, filformat och datumintervall
- [x] ✅ Sparade sökningar körs dynamiskt mot aktuellt lokalt arkiv och kan fästas i navigationen
- [ ] 🔷 Versionsvaliderad, transaktionell import/export av layout, sökningar, teman, kodord och regler

## 4. Analys, regler och bakgrundsjobb
- [x] ✅ Jobb-tabell med status (schema v9)
- [ ] 🔷 Enhetlig beständig jobbmodell med payload, checkpoints, beroenden, paus, avbrott och retry-policy för OCR, analys, index, backup, integritet och export
- [ ] 🔷 Incrementell aktualitet med beständiga dirty flags, orsak och selektiv omindexering/claim-/entitetsuppdatering
- [ ] 🔷 Regelbyggare UI med AND/OR/NOT, flera villkor, versionshantering, matchförklaringar och test/förslagsläge
- [ ] 🔷 Batchåtgärder för upp till 10 000 valda dokument: tagg, typ, arkiv och granskning

## 5. Import, export och portabilitet
- [x] ✅ .vaultarchive-format (schema v20)
- [ ] 🔷 Fristående export (metadata, claims, relationer, historik, regler, teman, kodord, sökningar och layout)
- [ ] 🔷 Exportformat (JSON, CSV och versionerat .vaultzip-paket)
- [ ] 🔷 Mappstruktur (platt, Vault-struktur eller dokumenttyp med kollisionssäkra namn)
- [x] ✅ .vaultarchive- och .vaultzip-format dokumenterade och versionsstyrda
- [ ] 🔶 Portabilitetstest dokumenterat och automatiserat där möjligt; fysisk ren Windows-profil återstår

## 6. Backup och temporär säkerhet
- [x] ✅ Lokal backup (full, inkrementell)
- [x] ✅ AES-256-krypterad backup
- [x] ✅ Backupvalidering
- [x] ✅ Schemalagd backup
- [ ] ❌ Backup till extern disk/nätverksmapp
- [ ] ❌ Säkert upplåsningsflöde (Credential Manager/DPAPI)
- [ ] 🔷 Restore-guide (välj → verifiera SHA/SQLite → innehåll → isolerat prov → bekräftad full restore → återställningspunkt → hälsa/index)
- [ ] 🔷 Temporära filer (kontrollerad katalog, städning vid start/efter operation och ZIP-import flyttad ur permanent index)

## 7. Säkerhet och hårdning
- [x] ✅ PIN-läge, sessionslåsning, dokumentlås
- [x] ✅ Krypterad privat sektion (schema v21)
- [x] ✅ Gästläge
- [ ] 🔷 Hotmodell och automatiska tester för traversal, ZIP-bomber, kompressionsgrad, symlänkar och storleksgränser
- [ ] ❌ Nyckelhantering (Credential Manager, DPAPI)
- [ ] ❌ Loggaudit (inga känsliga data i loggar)
- [ ] 🔷 Plugin- och exporträttigheter (backendvalidering och synlig pluginmatris klara; separat exportpolicy återstår)

## 8. Prestanda och skalverifiering
- [x] ✅ Testgenerator (100, 1 000, 10 000, 50 000 dokument)
- [ ] ❌ Virtuell rendering för listor
- [ ] ❌ Cache och lazy loading för PDF/bilder
- [ ] ❌ Prestandabudgetar och regressionstester
- [ ] ❌ Krasch- och avbrottstester

## 9. Plugins och externa importörer
- [x] ✅ Deklarativt sandboxat pluginsystem (schema v22)
- [ ] 🔷 Versionerade adapter-API:er (API 1.0, kompatibilitet, typer, resursgräns och dokumentation klara; fler körbara adapterkontrakt återstår)
- [x] ✅ Externa importörer (Google Drive, OneDrive, Gmail, Outlook via frivillig lokal export-/synkmapp med preview, exakt urval och bekräftelse)
- [ ] 🔷 Plugin-tillit (SHA-256, manipulationsavstängning och användargodkännande klara; kryptografisk utgivarsignering återstår)

## 10. Test Lab och slutacceptans
- [x] ✅ Isolerad Test Lab-databas
- [x] ✅ Test Center med godkända fall
- [x] ✅ Skaltest
- [ ] ❌ End-to-end-scenarier (DAGAB, lön juni, flera pass, parallella jobb, etc.)
- [ ] ❌ Felsökning i Test Lab
- [ ] ❌ Avbrottsscenarier
- [ ] ❌ UI-tester
- [ ] ❌ Tillgänglighetsaudit
