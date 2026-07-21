# Security, storage and plugins

## Lokal filförvaring

Vault använder en lokal vault-root:

```text
VaultData/
  production/
    vault.db
    files/
      sha256-prefix/
    index/
    thumbnails/
    backups/
    logs/
  testlab/
    vault-test.db
    files/
    index/
    backups/
```

Filer namnges internt efter hash och metadata i databas. Originalfilnamn bevaras i metadata.

Import ska vara atomisk:

1. staging,
2. hashning,
3. databastransaktion,
4. flytt till content-addressed storage,
5. indexjobb.

## Säkerhetsmodell

Grundläge:

- ingen obligatorisk inloggning,
- lokal åtkomst,
- inga nätverksanrop för kärnfunktioner,
- ingen telemetri.

Frivilliga nivåer:

- PIN-läge,
- lösenord,
- Windows Hello,
- auto-lås,
- maskering av känsliga uppgifter,
- låsta dokument/mappar/kategorier,
- krypterad privat sektion.

## Krypterad sektion

Skjuts upp till senare fas men datamodellen ska inte blockera det.

Principer:

- separat krypterad filrot,
- separat nyckelhantering,
- indexering bara när upplåst,
- ingen läsbar OCR-text i vanlig databas för låst sektion,
- tydlig UI-status.

## Audit och loggning

Loggar får innehålla:

- jobbtyp,
- dokument-id,
- status,
- tidsåtgång,
- felkod,
- regel-id,
- säker sammanfattning.

Loggar får inte innehålla:

- full dokumenttext,
- personnummer,
- kontonummer,
- OCR-utdrag,
- känsligt innehåll.

## Backup och återställning

Första modell:

- SQLite backup snapshot.
- manifest över filer och hashes.
- verifiering av att filerna finns.
- återställning till ny lokal vault-root.

Senare:

- krypterade backupfiler,
- schemalagda lokala backups,
- export av testlab-rapport utan känsligt innehåll.

## Externa integrationer

Alla externa integrationer är framtida och frivilliga. De får inte vara del av kärnflödet.

Krav:

- explicit godkännande,
- tydlig lista över data som hämtas/skickas,
- avstängningsbart,
- begränsade behörigheter,
- överföringslogg utan känsligt innehåll,
- ingen AI-tjänst.

## Plugin-arkitektur

Plugins är senare fas. Första arkitekturbeslut:

- manifest krävs,
- explicit rättighetsmodell,
- inga AI-beroenden,
- ingen nätverksåtkomst utan godkännande,
- versionshanterade API-gränser,
- plugins kan inaktiveras och tas bort,
- plugins får inte kringgå säkerhetsmodell eller AI-förbud.

Möjliga plugin-typer:

- importkälla,
- dokumenttyp,
- metadataextraktor,
- sökparser,
- exportmetod,
- dashboard-widget,
- tema.

