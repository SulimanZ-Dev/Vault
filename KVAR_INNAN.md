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
- [ ] ❌ Dokumentjämförelse sida vid sida

## 2. Djupare områdesmodeller
- [x] ✅ Grundläggande domänregister (schema v11)
- [x] ✅ Boende-, rese- och myndighetsvyer (schema v19)
- [ ] ❌ MRZ-parser för pass/ID
- [ ] ❌ Fordon och service (VIN, regnummer, servicehistorik)
- [ ] ❌ Garanti och produkter (garantiperiod, beräknat slut, reklamation)
- [ ] ❌ Anställning (arbetsgivaralias, organisationsnummer, befattning, sysselsättningsgrad)
- [ ] ❌ Lönehistorik och tilläggsavtal
- [ ] ❌ Konfliktgranskning (användare väljer källa, sparar beslut)

## 3. Sökning och sparade sökningar
- [x] ✅ Fritextsökning med FTS5
- [x] ✅ Sökoperatorer (type:, tag:, date:, status:, -ord, exakt fras)
- [ ] ❌ Språkstöd (svenska böjningsformer, sammansättningar, OCR-fel)
- [ ] ❌ Filter (kombinerbara: dokumenttyp, kategori, arbetsgivare, år, datumintervall, etc.)
- [ ] ❌ Sparade sökningar som dynamiska samlingar
- [ ] ❌ Import av layout, sökningar, teman, kodord, regler

## 4. Analys, regler och bakgrundsjobb
- [x] ✅ Jobb-tabell med status (schema v9)
- [ ] ❌ Enhetlig beständig jobbmodell för alla tunga operationer
- [ ] ❌ Incrementell aktualitet (dependency tracking, dirty flags)
- [ ] ❌ Regelbyggare UI (flera villkor, AND/OR/NOT, testkörning)
- [ ] ❌ Batchåtgärder

## 5. Import, export och portabilitet
- [x] ✅ .vaultarchive-format (schema v20)
- [ ] ❌ Fristående export (metadata, claims, relationer, historik)
- [ ] ❌ Exportformat (JSON, CSV, paketformat)
- [ ] ❌ Mappstruktur (platt, Vault-struktur, användarvald)
- [ ] ❌ .vaultzip-format dokumentation
- [ ] ❌ Portabilitetstest på ren Windows-profil

## 6. Backup och temporär säkerhet
- [x] ✅ Lokal backup (full, inkrementell)
- [x] ✅ AES-256-krypterad backup
- [x] ✅ Backupvalidering
- [x] ✅ Schemalagd backup
- [ ] ❌ Backup till extern disk/nätverksmapp
- [ ] ❌ Säkert upplåsningsflöde (Credential Manager/DPAPI)
- [ ] ❌ Restore-guide (välj backup → verifiera → lösenord → innehåll → restore)
- [ ] ❌ Temporära filer (kontrollerad katalog, rensning efter krasch)

## 7. Säkerhet och hårdning
- [x] ✅ PIN-läge, sessionslåsning, dokumentlås
- [x] ✅ Krypterad privat sektion (schema v21)
- [x] ✅ Gästläge
- [ ] ❌ Hotmodell och tester för filattacker (path traversal, ZIP-bomber, etc.)
- [ ] ❌ Nyckelhantering (Credential Manager, DPAPI)
- [ ] ❌ Loggaudit (inga känsliga data i loggar)
- [ ] ❌ Plugin- och exporträttigheter (behörighetsmatris)

## 8. Prestanda och skalverifiering
- [x] ✅ Testgenerator (100, 1 000, 10 000, 50 000 dokument)
- [ ] ❌ Virtuell rendering för listor
- [ ] ❌ Cache och lazy loading för PDF/bilder
- [ ] ❌ Prestandabudgetar och regressionstester
- [ ] ❌ Krasch- och avbrottstester

## 9. Plugins och externa importörer
- [x] ✅ Deklarativt sandboxat pluginsystem (schema v22)
- [ ] ❌ Versionerade adapter-API:er
- [ ] ❌ Externa importörer (Google Drive, OneDrive, Gmail, Outlook)
- [ ] ❌ Plugin-tillit (signering, checksummor, användargodkännande)

## 10. Test Lab och slutacceptans
- [x] ✅ Isolerad Test Lab-databas
- [x] ✅ Test Center med godkända fall
- [x] ✅ Skaltest
- [ ] ❌ End-to-end-scenarier (DAGAB, lön juni, flera pass, parallella jobb, etc.)
- [ ] ❌ Felsökning i Test Lab
- [ ] ❌ Avbrottsscenarier
- [ ] ❌ UI-tester
- [ ] ❌ Tillgänglighetsaudit
