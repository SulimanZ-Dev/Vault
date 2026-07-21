# Vault product definition

## Produktvision

Vault är ett lokalt och privat dokumentarkiv som hjälper användaren att hitta rätt originaldokument, rätt sida, rätt textstycke och rätt källbundna uppgift. Systemet ska kännas som en kombination av dokumenthanteringssystem, privat arkiv, lokal sökmotor, modern filutforskare och strukturerad kunskapsdatabas.

Vault får aldrig vara ett AI-system. All analys ska vara deterministisk, reproducerbar och förklarbar genom OCR, inbäddad text, fulltextsökning, metadata, regler, parsers, hashning, fuzzy matching och manuella beslut.

## AI-fri produktdefinition

Förbjudet i kärna, moduler, plugins och framtidsförberedelser:

- Språkmodeller, chatbotar, agenter, embeddings, vektordatabaser och RAG.
- AI-baserad OCR-analys, klassificering, metadataextraktion, taggning, rankning eller sammanfattning.
- Externa AI-API:er, telemetri till AI-tjänster eller molnbaserad dokumentbehandling som standard.
- Fri genererad textsammanfattning, rådgivning, juridisk tolkning och slutsatser utan definierad regel.

Tillåtet:

- Lokal OCR som endast omvandlar bild/skannad text till maskinläsbar text.
- Fulltextsökning, tokenisering, svenska språkregler, synonymer, alias och fuzzy matching.
- Regelbaserad dokumenttypning, metadataextraktion, claims, relationer och aktualitetsbedömning.
- Strukturerade översikter med definierade fält, källor, status, metod och klickbar väg till original.

## Viktigaste användarflöden

1. Importera dokument till inkorg, skapa lokal filpost, beräkna hash, extrahera text, köra eventuell OCR och visa granskningsbar metadata.
2. Söka efter dokument med fritext, svenska synonymer, datumtolkning, typfilter och rankningsförklaring.
3. Öppna dokument på relevant sida med markerad match och källbunden metadata bredvid.
4. Godkänna eller avvisa deterministiska metadataförslag i inkorg/granskningskö.
5. Koppla dokument till person, företag, fordon, anställning, utbildning, avtal eller produkt.
6. Bevara dokumentversioner, dubbletter, nästan-dubbletter och historiska uppgifter utan destruktiv överskrivning.
7. Visa aktuella, historiska, framtida, ersatta, osäkra och parallellt aktuella claims.
8. Skapa konflikt när regler inte säkert kan välja mellan flera dokumenterade uppgifter.
9. Låta användaren verifiera, låsa eller ångra beslut med full historik.
10. Köra Developer Test Lab i isolerad miljö med syntetiska dokument och samma produktionskodvägar.

## Produktprinciper

- Privat före bekvämt.
- Transparens före automatisering.
- Källor före påståenden.
- Konflikt före gissning.
- Historik före överskrivning.
- Manuell låsning före regelmotor.
- Progressiv komplexitet i UI.
- Offline-kärna före integrationer.

## Första fungerande version

Första versionen ska vara smal men verklig:

- Lokal SQLite-databas med migreringar.
- Lokal filförvaring med content hash och originalpath.
- Import av PDF, bild och textfil via filväljare eller drag/drop.
- Dokumentlista, inkorg, dokumentdetalj och enkel förhandsvisning.
- Grundmetadata, kategorier, taggar och kodord.
- Fulltextsökning mot titel, metadata och extraherad text.
- Matchorsak för sökresultat.
- Theme tokens och minst två teman.
- Developer Test Lab med separat databas och syntetisk seed-data.

