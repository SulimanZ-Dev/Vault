# Kvar innan hela originalprompten är uppfylld

Detta dokument spårar status för varje krav i originalprompten.
Statusar: ✅ Fullt implementerad | 🔷 Implementerad (kräver EXE-verifiering) | 🔶 Delvis | ❌ Saknas | ➖ Inte relevant

## 1. Dokumentvisaren
- [x] ✅ PDF-rendering internt (PDF.js är bundlat offline och verifierat i release-EXE)
- [x] ✅ Sidnavigation, zoom, passning, rotation och återställning av senaste läge
- [x] ✅ Miniatyrer med fönstrad lazy rendering
- [x] ✅ Valbart textlager och lokala sökträffar
- [x] ✅ Versions- och sidbundna markeringar med normaliserade koordinater
- [x] ✅ Claim till exakt källsida, söktext och koordinatmarkering
- [x] ✅ Dokumentjämförelse sida vid sida med ordningsbevarad text och markerade skillnader

## 2. Djupare områdesmodeller
- [x] ✅ Grundläggande domänregister (schema v11)
- [x] ✅ Boende-, rese- och myndighetsvyer (schema v19)
- [x] ✅ Deterministisk TD3-MRZ-parser för pass med kontrollsiffror och källfält
- [x] ✅ Fordon och service med register, historik och lokalt källbundet utdrag
- [x] ✅ Garanti och produkter med garantiperiod, slut, reklamation och källfält
- [x] ✅ Anställning med alias, organisationsnummer, befattning och sysselsättningsgrad
- [x] ✅ Lönehistorik och tilläggsavtal i separata historiktabeller med lokala källfält
- [x] ✅ Konfliktgranskning där användaren väljer källa/båda/ingen och beslutet låses

## 3. Sökning och sparade sökningar
- [x] ✅ Fritextsökning med FTS5
- [x] ✅ Sökoperatorer (type:, tag:, date:, status:, -ord, exakt fras)
- [x] ✅ Deterministiskt språkstöd för vanliga svenska böjningsformer, sammansättningar, synonymer och OCR-felen 0/o samt 1/l
- [x] ✅ Kombinerbara filter för dokumenttyp, kategori, arbetsgivare/entitet, status, filformat och datumintervall
- [x] ✅ Sparade sökningar körs dynamiskt mot aktuellt lokalt arkiv och kan fästas i navigationen
- [x] ✅ Versionsvaliderad, transaktionell import/export av layout, sökningar, teman, kodord och regler

## 4. Analys, regler och bakgrundsjobb
- [x] ✅ Jobb-tabell med status (schema v9)
- [x] ✅ Enhetlig beständig jobbmodell med payload, checkpoints, beroenden, paus, avbrott och retry-policy för OCR, analys, index, backup, integritet och export
- [x] ✅ Incrementell aktualitet med beständiga dirty flags, orsak och selektiv omindexering/claim-/entitetsuppdatering
- [x] ✅ Regelbyggare UI med AND/OR/NOT, flera villkor, versionshantering, matchförklaringar och test/förslagsläge
- [x] ✅ Batchåtgärder för upp till 10 000 valda dokument: tagg, typ, arkiv och granskning

## 5. Import, export och portabilitet
- [x] ✅ .vaultarchive-format (schema v20)
- [x] ✅ Fristående export (metadata, claims, relationer, historik, regler, teman, kodord, sökningar och layout)
- [x] ✅ Exportformat (JSON, CSV och versionerat .vaultzip-paket)
- [x] ✅ Mappstruktur (platt, Vault-struktur eller dokumenttyp med kollisionssäkra namn)
- [x] ✅ .vaultarchive- och .vaultzip-format dokumenterade och versionsstyrda
- [x] ✅ Portabilitetstest dokumenterat och automatiserat med isolerade datarötter samt verifierad release-EXE och NSIS-installer

## 6. Backup och temporär säkerhet
- [x] ✅ Lokal backup (full, inkrementell)
- [x] ✅ AES-256-krypterad backup
- [x] ✅ Backupvalidering
- [x] ✅ Schemalagd backup
- [x] ✅ Backup till extern disk/nätverksmapp med validerad destination och lokal spegling
- [x] ✅ Säkert upplåsningsflöde med användarbunden Windows DPAPI och vanlig PIN-hashverifiering
- [x] ✅ Restore-guide (välj → verifiera SHA/SQLite → innehåll → isolerat prov → bekräftad full restore → återställningspunkt → hälsa/index)
- [x] ✅ Temporära filer (kontrollerad katalog, städning vid start/efter operation och ZIP-import flyttad ur permanent index)

## 7. Säkerhet och hårdning
- [x] ✅ PIN-läge, sessionslåsning, dokumentlås
- [x] ✅ Krypterad privat sektion (schema v21)
- [x] ✅ Gästläge
- [x] ✅ Hotmodell och automatiska tester för traversal, ZIP-bomber, kompressionsgrad, symlänkar och storleksgränser
- [x] ✅ Nyckelhantering med Windows DPAPI för snabb upplåsning; privata original och backup använder AES-256
- [x] ✅ Loggaudit: releasebygget skriver inga applikationsloggar och säkerhetshändelser använder innehållsfria sammanfattningar
- [x] ✅ Plugin- och exporträttigheter med backendvalidering, synlig behörighetsmatris, explicit aktivering och skyddad privatdata

## 8. Prestanda och skalverifiering
- [x] ✅ Testgenerator (100, 1 000, 10 000, 50 000 dokument)
- [x] ✅ Fönstrad virtuell rendering för dokumentlistor och lazy miniatyrfönster
- [x] ✅ Sidvis lazy PDF-rendering, textcache och koddelad dokumentvisare
- [x] ✅ Prestandabudgetar för 100–50 000 syntetiska dokument med tydligt regressionsutfall
- [x] ✅ Verkliga isolerade rollback-, saknad-fil-, checkpoint/avbrotts- och restore-integritetstester

## 9. Plugins och externa importörer
- [x] ✅ Deklarativt sandboxat pluginsystem (schema v22)
- [x] ✅ Versionerade adapter-API:er 1.0 med kompatibilitet, sju adaptertyper, resursgräns, felisolering, exempel och dokumentation
- [x] ✅ Externa importörer (Google Drive, OneDrive, Gmail, Outlook via frivillig lokal export-/synkmapp med preview, exakt urval och bekräftelse)
- [x] ✅ Plugin-tillit med Ed25519-utgivarsignering, SHA-256, manipulationsavstängning och användargodkännande

## 10. Test Lab och slutacceptans
- [x] ✅ Isolerad Test Lab-databas
- [x] ✅ Test Center med godkända fall
- [x] ✅ Skaltest
- [x] ✅ End-to-end-scenarier för DAGAB, lön juni, flera pass, parallella jobb, backup, audit, OCR och konflikter
- [x] ✅ Felsökning i Test Lab med isolerade verkliga felprov och säker exporterbar rapport
- [x] ✅ Avbrottsscenarier med beständig checkpoint, paus, resume, cancel och retry
- [x] ✅ UI-tester med 12 automatiska kontroller samt direkt release-EXE-prov av navigation, säkerhet, plugins och Test Center
- [x] ✅ Tillgänglighetsaudit för svenska, namn, semantik, fokus, tangentbord, Escape, reducerad rörelse, teman och låsskärm
