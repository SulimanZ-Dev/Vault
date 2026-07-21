# Developer Test Lab and acceptance tests

## Test Lab-arkitektur

Developer Test Lab är en separat intern miljö för utveckling, inte demo och inte onboarding.

Isolering:

- separat databas,
- separat filrot,
- separat sökindex,
- separat backup,
- separata inställningar,
- separat bakgrundskö,
- permanent UI-märkning: `TESTMILJÖ - INGA RIKTIGA DOKUMENT`,
- samma produktionskodvägar som vanlig vault.

Test Lab får aldrig innehålla riktiga personuppgifter.

## Syntetiskt testarkiv

Första seed:

- anställningsavtal DAGAB,
- lönespecifikation juni,
- två parallella jobb,
- ursprunglig timlön och tilläggsavtal,
- flera pass för samma syntetiska person,
- makulerat nyare pass,
- bilservicekvitto,
- motstridiga startdatum,
- låg OCR-kvalitet,
- exakt dubblett,
- nästan-dubblett,
- trasig filpost.

Alla identifierare ska märkas som syntetiska och ligga i testnamnrymder.

## Test Center

Första funktioner:

- kör alla tester,
- kör per modul,
- visa passed/failed/skipped,
- visa förväntat och faktiskt resultat,
- öppna berört syntetiskt dokument,
- återställ Test Lab.

Senare:

- simulera OCR-fel,
- simulera databasfel,
- simulera filfel,
- mäta import/OCR/sök/analys/indexering,
- generera 100, 1 000, 10 000 och 50 000 dokument,
- exportera säker rapport.

## Acceptanstest: DAGAB kontrakt

Fixture:

- syntetiskt anställningsavtal med arbetsgivare DAGAB,
- dokumenttyp `anställningsavtal`,
- text innehåller `avtal`, men sökning använder `kontrakt`.

Krav:

- sökningen hittar rätt dokument,
- `kontrakt` expanderas till `avtal`,
- dokumentet rankas högst när företag och dokumenttyp matchar,
- matchorsak visas,
- dokumentet öppnas på rätt sida,
- matchat textspan markeras.

## Acceptanstest: lön juni

Fixture:

- lönespecifikation för juni,
- separat anställningsavtal med avtalad lön.

Krav:

- `juni` tolkas som månad 6,
- löneperiod prioriteras över lös brödtext,
- filtrering på arbetsgivare och år fungerar,
- faktisk utbetald lön blandas inte ihop med avtalad grundlön.

## Acceptanstest: flera pass

Fixture:

- flera passhandlingar för samma syntetiska person,
- flera bilder av samma pass,
- ett makulerat nyare pass,
- ett äldre giltigt pass.

Krav:

- samma person identifieras genom strukturerade uppgifter,
- dubblettbilder skiljs från olika pass,
- senast giltiga pass föreslås som aktuellt,
- makulerat pass väljs inte,
- historik bevaras,
- flera giltiga pass kan visas parallellt.

## Acceptanstest: parallella jobb

Fixture:

- två anställningsavtal utan slutdatum hos olika arbetsgivare.

Krav:

- första jobbet avslutas inte automatiskt,
- båda visas som aktuella,
- ett huvudjobb kan väljas manuellt,
- löneposter hålls separerade per anställning.

## Acceptanstest: ändrad timlön

Fixture:

- ursprungligt avtal med timlön,
- senare tilläggsavtal med ny timlön och startdatum,
- lönespecifikationer efter ändringen.

Krav:

- tidigare timlön bevaras historiskt,
- ny timlön aktiveras från rätt datum,
- tilläggsavtalet visas som källa,
- lönespecifikation används inte som avtalad grundlön utan regel.

## Acceptanstest: konflikt

Fixture:

- två dokument med olika startdatum för samma relation.

Krav:

- konflikt skapas,
- båda värden visas,
- båda källor visas,
- systemet gissar inte,
- användarens beslut sparas,
- låst värde skrivs inte över automatiskt.

## Acceptanskriterier för första sökversion

- Indexering av titel, metadata och text fungerar lokalt.
- Sökresultat visas under 300 ms för Test Lab 1 000 dokument på normal utvecklingsmaskin.
- Resultat visar matchorsak.
- Svenska månader och minst tio synonymer fungerar.
- Sökning kan öppna rätt dokument och sida.
- Tester täcker DAGAB kontrakt och lön juni.

## Acceptanskriterier för första analysversion

- Regler kan köras manuellt på valt dokument.
- Minst tre dokumenttyper kan föreslås deterministiskt.
- Datum, belopp och företag kan extraheras med källor.
- Claims får status och källa.
- Osäkra fuzzy matchningar hamnar i granskningskö.
- Ingen fri textsammanfattning genereras.

## Acceptanskriterier för första aktualitetsversion

- Claims kan markeras som aktuell, historisk, framtida, utgången och konflikt.
- Parallella anställningar stöds.
- Ändrad timlön bevarar historik.
- Konfliktmotor visar båda källor.
- Manuellt låst beslut vinner över automatisk regel.

## Acceptanskriterier för första programversion

- Appen startar lokalt utan konto.
- Import, lista, sök, öppna dokument och metadata fungerar.
- Databas och filer skapas lokalt.
- Minst två teman fungerar via tokens.
- Test Lab är isolerat och återställningsbart.
- Automatiska jobb kan pausas eller misslyckas utan datakorruption.

