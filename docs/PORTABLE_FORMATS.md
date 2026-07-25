# Vaults portabla format

Alla format skapas lokalt utan nätverk eller AI. Okända huvudversioner ska avvisas i stället för att tolkas tyst.

## `.vaultarchive`

`vault-portable-v1` är ett komplett återställningsarkiv (ZIP-container) med:

- `portable-format.json`: format, schema, antal dokument/filer och policy.
- `vault.db`: konsekvent SQLite-snapshot.
- `manifest.json`: checksummor och backupmetadata.
- `files/`: originalfiler med sina interna Vault-sökvägar.
- `documents.csv`: läsbar dokumentförteckning.

Import sker till en kontrollerad stagingkatalog, använder endast ZIP-poster med säkra relativa sökvägar och skapar en återställningspunkt innan den aktiva databasen ersätts.

## `.vaultzip`

Två varianter finns och identifieras av sitt manifest:

- Lösenordsskyddad backup: AES-256-krypterade poster och `vault-local-backup-v1`.
- Fristående dataexport: `vault-data-export`, version 1.

Dataexporten innehåller `manifest.json`, CSV-filer per tabell och, om användaren väljer det, `documents/`. `folder_layout` är `flat`, `vault` eller `type`. Stabilt databas-ID bevaras så relationer kan återskapas.

## Kompatibilitet

- Läsare får lägga till frivilliga fält inom samma huvudversion.
- Obligatoriska manifestfält får inte byta betydelse.
- Ny inkompatibel struktur kräver en ny huvudversion.
- Databasmigreringar körs av Vault efter verifierad restore.
- SHA-256 i backupmanifest och exportresultat används för integritetskontroll.
- ZIP traversal avvisas; arkiv extraheras aldrig direkt över aktiv data.

## Ren profil

Se `docs/PORTABILITY_TEST.md` för ett reproducerbart test av installer, första start, import, sökning, backup, restore och portabelt arkiv i en tom Windows-profil.
