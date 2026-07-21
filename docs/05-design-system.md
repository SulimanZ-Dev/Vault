# Design system

## Informationsarkitektur

Primära områden:

- Hem
- Inkorg
- Alla dokument
- Sök
- Kategorier
- Personer
- Företag
- Objekt
- Fordon
- Anställningar
- Utbildningar
- Avtal
- Tidslinje
- Konflikter
- Granskningskö
- Regler
- Importhistorik
- Backup
- Developer Test Lab
- Inställningar

Utvecklarfunktioner ska kunna döljas i normalt läge.

## Grundlayout

Vänster sidopanel:

- navigering,
- favoritsektioner,
- egna genvägar,
- minimerbar och justerbar.

Toppbar:

- global sökning,
- kommandopalett,
- import,
- vyval,
- sortering,
- bakgrundsjobb,
- lokal säkerhetsstatus.

Huvudyta:

- lista,
- rutnät,
- kompakt tabell,
- delad vy,
- tidslinje,
- objektvy,
- konfliktvy,
- Test Center.

Höger informationspanel:

- metadata,
- förhandsvisning,
- extraherade fakta,
- källor,
- matchade regler,
- konflikter,
- historik.

## Design tokens

Tokenkategorier:

- färg: bakgrund, panel, text, sekundärtext, accent, varning, konflikt, källa,
- spacing: tät, normal, rymlig,
- radius: 4, 6, 8,
- border,
- shadow,
- typography,
- focus ring,
- animation duration,
- transparency,
- document preview aspect ratios.

CSS-variabler är primär teknik. Teman versioneras som JSON.

## Första teman

Första version:

- Midnight
- Snow

Senare:

- Pure Black OLED
- Warm Paper
- Forest
- Ocean
- Nord
- Glass
- Minimal Gray
- High Contrast

## Komponentlista

Första komponenter:

- AppShell
- Sidebar
- TopBar
- SearchBox
- DocumentList
- DocumentRow
- DocumentCard
- DocumentPreview
- MetadataPanel
- SourceChip
- StatusBadge
- TagEditor
- CategoryPicker
- ImportDropzone
- InboxReviewPanel
- JobStatusIndicator
- EmptyState
- ErrorState
- ThemeProvider

Senare komponenter:

- RuleBuilder
- ConflictResolver
- Timeline
- GraphView
- ThemeEditor
- TestCenter

## Mockupbeskrivningar

Hem:

- kompakt översikt över senaste importer, aktiva jobb, konflikter och dokument som behöver granskning.
- ingen marknadshero, inget generiskt dashboard-brus.

Inkorg:

- vänster lista med importerade dokument,
- mitten förhandsvisning,
- höger metadata och förslag,
- tydlig knapp för godkänn, flytta till arkiv eller avvisa.

Sökresultat:

- sökfält överst,
- filterrad under,
- resultat med titel, typ, datum, utdrag, sida och matchorsak,
- höger panel visar vald träff och källspår.

Dokumentvisare:

- sidlista eller miniatyrer,
- stor dokumentyta,
- markerade textträffar,
- höger panel med metadata, claims och källor.

Konfliktvy:

- en konflikt per rad,
- berörda värden sida vid sida,
- källor och regel,
- beslut: välj A, välj B, välj båda, välj inget, lås beslut.

Test Center:

- permanent märkning `TESTMILJÖ - INGA RIKTIGA DOKUMENT`,
- modulfilter,
- kör tester,
- förväntat/faktiskt resultat,
- öppna felande dokument,
- exportera testrapport.

## Tillgänglighet

Krav från start:

- tangentbordsnavigering,
- synlig fokusmarkering,
- skärmläsbara labels,
- kontrastkontroll,
- stöd för reducerade animationer,
- responsiv layout,
- text som klarar långa svenska filnamn och 125-200 procent Windows-skalning.

