# Vault app

Tauri + React + TypeScript-startpunkt för Vault.

## Kommandon

```powershell
npm install
npm run lint
npm run build
npm run tauri:dev
npm run exe:build
```

## Nuvarande status

Det här är första fungerande desktop-grunden. `npm run dev` är endast en Vite-preview för snabb UI-utveckling. Den slutliga produkten ska köras och distribueras som Tauri/Windows `.exe`.

- lokal Tauri-konfiguration,
- React/Vite frontend,
- Vault shell med sidopanel, sökfält, inkorg, detaljpanel och Test Lab-underlag,
- ett Rust/Tauri-kommando för lokal runtime-status,
- framtida funktioner är avstängda eller märkta som planerade.

Ingen import, databas, OCR eller sökindex är inkopplat ännu.
