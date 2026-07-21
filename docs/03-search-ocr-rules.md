# Search, OCR and rules

## OCR-arkitektur

OCR ska vara lokal och explicit.

Pipeline:

1. Kontrollera om dokumentet redan har extraherbar text.
2. Rendera sidor lokalt vid behov.
3. Kör lokal OCR per sida.
4. Spara textspans med sidnummer, bounding boxes, confidence och OCR-profil.
5. Markera låg kvalitet för granskning.
6. Uppdatera fulltextindex.

OCR får pausas, återstartas och köras om. OCR-resultat är källtext för sök och regler, inte juridisk eller semantisk tolkning.

## Fulltextsökning

Första sökmotor: SQLite FTS5.

Indexera:

- dokumenttitel,
- originalfilnamn,
- kategorier,
- taggar,
- kodord,
- företag/person/objekt,
- dokumenttyp,
- textspans,
- utvalda claims.

Senare alternativ om FTS5 inte räcker:

- Tantivy som lokal Rust-sökmotor.
- Meilisearch endast som lokal, frivillig process.

## Datamodell för OCR-text och index

`text_spans` lagrar källtext per sida och ursprung:

- `embedded_pdf_text`
- `ocr_text`
- `manual_text`
- `office_extracted_text`

FTS-tabeller:

```text
document_fts(document_id, title, metadata_text, body_text)
span_fts(text_span_id, document_id, page_no, text)
```

Sökresultat ska kunna peka tillbaka till sida och textspan.

## Svensk sökning

Första versionen stödjer:

- normalisering till gemener,
- borttagning av enkel diakritisk variation där det är säkert,
- svenska månadsnamn,
- vanliga dokumenttypssynonymer,
- alias för företag och objekt,
- särskrivningsvänliga sökningar,
- fuzzy matching för metadata och titel.

Exempel:

- `kontrakt` matchar `avtal`.
- `lön juni` expanderar till lönerelaterade dokumenttyper och månad 6.
- `DAGAB` matchar definierade företagsalias.

## Datum- och månadstolkning

Datumparsern ska stödja:

- `2026-07-20`
- `20 juli 2026`
- `juli 2026`
- `juni`
- `v. 32`
- intervall med `från`, `till`, `gäller`, `period`.

Tolkade datum ska alltid markeras som tolkade och visa parserregel.

## Fuzzy matching

Fuzzy matching används försiktigt:

- titlar,
- företagsalias,
- personnamn,
- registreringsnummer med OCR-fel,
- dokumentnummer.

Fuzzy match får höja relevans men inte ensam skapa verifierad claim utan policy som tillåter det.

## Viktad rankning

Första rankningsmodell:

- exakt fras i titel: hög vikt,
- dokumenttypmatch: hög vikt,
- företag/person/objektmatch: hög vikt,
- metadatafält: medium/hög vikt,
- brödtext: medium vikt,
- OCR-text med låg kvalitet: lägre vikt,
- datumperiodmatch: hög vikt när sökningen innehåller månad/år,
- favorit/senaste kan påverka sortering men ska visas som sådan.

Varje resultat ska ha matchförklaring:

- vilka termer matchade,
- vilka synonymer eller alias användes,
- vilka fält matchade,
- varför resultatet rankades högt.

## Sökgränssnitt

Första UI:

- global sökruta,
- snabbfilter för typ, kategori, person/företag/objekt, datum och status,
- resultatrad med titel, typ, datum, källa, utdrag och matchorsak,
- öppna på sida,
- markera textspan,
- växla lista/rutnät.

## Sökoperatorer

Planerade operatorer:

- `"exakt fras"`
- `type:anställningsavtal`
- `company:DAGAB`
- `person:namn`
- `tag:kvitto`
- `date:2026`
- `month:juni`
- `status:current`
- `has:conflict`
- `-ord`

## Regelmotor

Regler versioneras och lagras som data.

Regeltyper:

- dokumenttypregel,
- metadataextraktor,
- datumparser,
- beloppsparser,
- registreringsnummerparser,
- MRZ-parser,
- aktualitetsregel,
- konfliktregel,
- relationsregel,
- påminnelseregel.

Godkännandepolicy:

- endast föreslå,
- spara automatiskt som automatiskt extraherad,
- spara automatiskt om validering lyckas,
- kräver manuell granskning vid fuzzy match,
- förbjud automatisk åtgärd i låst/privat sektion.

## Aktualitetsmotor

Aktualitetsmotorn läser claims, relationer, giltighetsperioder, domänpolicyer och användarverifieringar.

Den får:

- markera framtida och utgångna perioder,
- föreslå ersättning enligt tydlig regel,
- stödja parallellt aktuella relationer,
- skapa konflikt vid osäkerhet,
- respektera manuella låsningar.

Den får inte:

- anta att senaste dokument alltid gäller,
- avsluta äldre anställning automatiskt när ny anställning hittas,
- blanda ihop lönespecifikation med avtalad grundlön utan regel.

