# Vault 1.2.0

Den här releasen lägger till en säker funktion för att radera hela den lokala
Vault-installationen och börja om från ett tomt arkiv.

## Nytt

- Fullständig nollställning från Säkerhet.
- Kräver den exakta bekräftelsefrasen `RADERA ALLT`.
- Kräver befintlig PIN om en PIN är konfigurerad.
- Raderar dokument, metadata, databas, sökindex, lokala backuper, Test Lab-data,
  plugins och lokala inställningar.
- Externa backupkopior utanför Vaults datakatalog påverkas inte.
- Återskapar automatiskt en frisk tom databas, katalogstruktur och Test Lab
  efter nollställningen.
- Atomisk flytt och återställning skyddar den gamla installationen om
  nyinitialiseringen skulle misslyckas.

## Verifiering

- 36 Rust-tester.
- 22 UI- och tillgänglighetskontroller.
- TypeScript/Vite-produktionsbygge.
- Fullt purge-flöde verifierat i en isolerad paketerad Windows-EXE: demoarkiv
  skapades, raderades och appen startade om med 0 importerade dokument.
