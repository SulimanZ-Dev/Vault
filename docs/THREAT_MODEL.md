# Hotmodell för lokala Vault

## Skyddsvärden

Originaldokument, OCR-text, claims, identitetsuppgifter, krypteringshemligheter, backup och verifieringshistorik. Vault är lokal-first; nätverk är inte en förutsättning för kärnan.

## Angripare och gränser

- Ett importerat dokument eller arkiv kan vara fientligt.
- En lokal användare utan Vaults PIN ska inte kunna exportera låsta original genom appen.
- Plugins är datafiler, inte körbar kod, och är avstängda tills behörigheter godkänts.
- En angripare med full kontroll över Windows-kontot ligger utanför appens fullständiga skyddsgräns; filsystemskryptering och Windows-kontosäkerhet krävs också.

## Implementerade kontroller

- ZIP-sökvägar måste vara inneslutna; symboliska länkar hoppas över.
- Antal poster, okomprimerad totalstorlek, storlek per post och extrem kompressionsgrad begränsas.
- Importformat tillåts explicit och original lagras under Vaults egen datarot med SHA-256.
- Restore extraherar till staging, validerar backup och skapar en återställningspunkt.
- Kraschrester i endast appägda `temp`, `portable-import` och äldre `zip-import-*` rensas vid start.
- Krypterad backup använder AES-256; lösenord skickas aldrig till auditloggen.
- Diagnostik och audit använder säkra summeringar, inte dokument- eller OCR-text.
- Pluginbackend nekar nätverk, processkörning, AI och fri filåtkomst oavsett UI.

## Testade angrepp

Automatiska Rust-prov täcker traversal via ZIP-bibliotekets inneslutna namn, ZIP-bombgränser, fel lösenord, pluginbehörigheter, tempstädning, filintegritet och isolerad restore. Fysisk testning av junctions, långa Windows-sökvägar och antivirusinteraktion dokumenteras i portabilitetstestet.
