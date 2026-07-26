# Vault 1.1.0

Första kompletta publika Windows-releasen av Vault: ett lokalt, privat och
AI-fritt dokumentarkiv.

## Höjdpunkter

- Lokal import, OCR, PDF-/Office-textutvinning och intern dokumentvisare.
- SQLite/FTS5-sökning med svenska regler och förklarade träffar.
- Claims, granskning, relationer, graf, tidslinje, kalender och konflikter.
- Domänflöden för identitet, fordon, arbete, lön, avtal och garantier.
- Anpassningsbara dokumentvyer, paneler, dashboard och teman.
- Lokal säkerhet, krypterad privat sektion, backup, restore och portabilitet.
- Lokala notiser, plugin-adaptrar och isolerat Test Lab.

## Verifiering

- 35 godkända Rust-tester.
- 22 godkända UI- och tillgänglighetskontroller.
- Test Center i release-EXE: 19 godkända, 0 misslyckade.
- Optimerad `vault.exe` och NSIS-installer byggda och startprovade på Windows.

## Nedladdningar

- `Vault_1.1.0_x64-setup.exe` – rekommenderad Windows-installer.
- `vault-1.1.0-windows-x64.exe` – fristående körbar app.
- SHA-256-filer med kontrollsummor för båda binärerna.

## Avsiktliga avgränsningar

Releasen innehåller inte enhetssynkning, OAuth-integrationer, kamera/fysisk
skannerstyrning, mobil companion eller Windows Hello.
