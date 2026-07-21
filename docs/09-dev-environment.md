# Development environment

Senast kontrollerat: 2026-07-20.

## Installerat och verifierat

```text
rustc 1.97.1
cargo 1.97.1
node v24.13.0
npm 11.6.2
tauri-cli 2.11.4
sqlite3 3.53.3
Microsoft C/C++ Optimizing Compiler 19.44.35222
Microsoft Edge WebView2 Runtime 150.0.4078.83
git 2.47.0.windows.1
```

## Installerat under detta steg

- Rustup med stable MSVC toolchain.
- Tauri CLI via npm.
- SQLite command line tools via winget.

## Redan installerat

- Node.js och npm.
- Git.
- Visual Studio Build Tools 2022 med C++ compiler.
- Microsoft Edge WebView2 Runtime.

## Kvarvarande punkt

Tesseract OCR är inte installerat. Två winget-paket testades:

- `tesseract-ocr.tesseract`
- `UB-Mannheim.TesseractOCR`

Båda laddades ner och verifierades, men installeraren avbröts med Windows-felkod `0x800704c7` (`operation was canceled by the user`). OCR kan därför tas i nästa steg antingen genom en manuell installer eller genom att senare göra OCR-adaptern valfri tills Tesseract finns lokalt.

## PATH-notering

Om ett nytt terminalfönster inte hittar nyinstallerade verktyg direkt, starta om terminalen så Windows läser om PATH. Under verifieringen användes även dessa tillfälliga PATH-tillägg:

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:LOCALAPPDATA\Microsoft\WinGet\Links;$(npm prefix -g);$env:Path"
```

