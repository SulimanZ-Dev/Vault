# Portabilitetstest på ren Windows-profil

1. Skapa en lokal Windows-testanvändare utan tidigare Vault-data.
2. Installera den signerade/byggda NSIS-installern från `app/src-tauri/target/release/bundle/nsis/`.
3. Starta Vault och kontrollera att diagnostiken visar aktuell schemaversion och tomt produktionsarkiv.
4. Importera ett syntetiskt PDF-, bild- och Office-dokument.
5. Kör OCR, sök efter en känd fras och öppna träffen på rätt sida.
6. Skapa full backup, validera den och kör isolerat restore-test.
7. Exportera `.vaultarchive`, JSON, CSV och `.vaultzip`.
8. Återställ `.vaultarchive` i profilen och kontrollera dokumentantal, originalfiler, sökning, teman, regler och sparade sökningar.
9. Kör indexreparation och arkivhälsa.
10. Avinstallera appen. Radera testprofilen först när resultatrapporten är sparad.

Använd bara syntetiska dokument. Produktionsdata får aldrig kopieras till testprofilen.
