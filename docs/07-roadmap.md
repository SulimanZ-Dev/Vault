# Roadmap

## Fas 0: Plan och teknisk grund

Mål:

- Produktdefinition.
- Arkitektur.
- Datamodell.
- Sök/OCR/regler-plan.
- Designsystem.
- Test Lab-plan.
- Acceptanskriterier.

Status: påbörjad med denna dokumentation.

## Fas 1: Dokumentgrund

Små milstolpar:

1. Skapa Tauri/React/Rust-projektstruktur.
2. Lägg SQLite och migreringssystem.
3. Implementera vault settings och lokal storage root.
4. Implementera import av PDF, bild och textfil.
5. Skapa dokumentlista och dokumentdetalj.
6. Lägg kategorier, taggar och kodord.
7. Lägg papperskorg utan permanent automatisk radering.
8. Skapa Midnight och Snow-teman.
9. Skapa grundläggande Test Lab med separat databas.

## Fas 2: OCR och fulltextsökning

Milstolpar:

1. Textutvinning från text/PDF.
2. Lokal OCR-adapter och OCR-kö.
3. FTS5-index.
4. Sökresultat med utdrag och sida.
5. Matchmarkering i dokumentvisare.
6. Indexreparation.

## Fas 3: Regelbaserad sökning

Milstolpar:

1. Svenska månader och datumparser.
2. Synonymer och alias.
3. Fuzzy matching för titel/metadata.
4. Sökoperatorer.
5. Rankningsförklaring.
6. Sparade sökningar.

## Fas 4: Strukturerad analys

Milstolpar:

1. Claims och claim sources.
2. Dokumenttypsregler.
3. Datum-, belopp- och identifierarparsning.
4. Granskningskö.
5. Relationer till person/företag/objekt.
6. Områdesscheman.

## Fas 5: Aktualitet och konflikter

Milstolpar:

1. Aktualitetsstatusar.
2. Parallellt aktuella relationer.
3. Konfliktmotor.
4. Användarverifiering och låsningar.
5. Historikvy.
6. Aktuellt läge per område.

## Fas 6: Automatisering utan AI

Milstolpar:

1. Regelhistorik.
2. Bevakade mappar.
3. Bakgrundsanalys.
4. Påminnelseförslag.
5. Omprövning när regler ändras.

## Fas 7: Avancerat arkiv

Milstolpar:

1. Versionshantering.
2. Dubblett och nästan-dubblett.
3. Grafvy.
4. Krypterad privat sektion.
5. Avancerad backup.
6. Plugin-system.
7. Frivilliga externa importer.

## Tekniska risker och kompromisser

- OCR-kvalitet varierar. Första versionen ska visa kvalitet och kräva granskning vid låg confidence.
- FTS5 kan räcka länge, men mycket stora arkiv kan kräva Tantivy senare.
- Office-dokument kan kräva lokala konverteringsverktyg. Första versionen bör börja med PDF, bild och text.
- Krypterad sektion påverkar sök, OCR och previews. Skjuts upp tills grundmodellen är stabil.
- Plugin-system är kraftfullt men riskabelt för integritet. Skjuts upp.
- Perceptuell jämförelse för dokumentbilder är mer komplex än content hash. Bör komma efter exakt dubblettkontroll.

## Funktioner som skjuts upp

- Mobil companion-app.
- Synkning mellan egna enheter.
- Externa integrationer.
- Skannerintegration.
- Krypterad privat sektion.
- Grafvy.
- Avancerad theme editor.
- Plugin-system.
- 50 000-dokumentgenerator.

## Nästa konkreta implementation

När denna plan är godkänd börjar implementation med Fas 1, milstolpe 1:

- skapa Tauri + React + TypeScript-projekt,
- skapa Rust workspace för core/storage/files/search,
- lägga första SQLite-migreringarna,
- skapa minimal AppShell utan döda knappar,
- lägga tester som säkerställer att appen startar och att Test Lab använder separat konfiguration.

