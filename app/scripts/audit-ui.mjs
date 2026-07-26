import { readFileSync } from 'node:fs'

const app = readFileSync(new URL('../src/App.tsx', import.meta.url), 'utf8')
const viewer = readFileSync(new URL('../src/DocumentViewer.tsx', import.meta.url), 'utf8')
const css = readFileSync(new URL('../src/App.css', import.meta.url), 'utf8')
const combined = `${app}\n${viewer}`

const checks = [
  ['svenskt dokument', combined.includes('lang="sv"') || readFileSync(new URL('../index.html', import.meta.url), 'utf8').includes('lang="sv"')],
  ['tangentbordsgenväg', combined.includes("event.key.toLowerCase()==='k'") || combined.includes("event.key === 'k'")],
  ['Escape-stängning', combined.includes("event.key === 'Escape'") || combined.includes("event.key==='Escape'")],
  ['namngiven sökning', /aria-label="[^"]*(Sök|sök)/.test(combined) || /placeholder="Sök/.test(combined)],
  ['namngiven PDF-yta', /aria-label="[^"]*(PDF|Dokument|Sida)/.test(viewer)],
  ['semantiska knappar', (combined.match(/<button/g) ?? []).length > 20],
  ['fokusmarkering', /:focus-visible/.test(css)],
  ['reducerad rörelse', /prefers-reduced-motion/.test(css)],
  ['kontrastteman', css.includes('--text-primary') && css.includes('--surface-main')],
  ['Test Lab-varning', app.includes('TESTMILJÖ – INGA RIKTIGA DOKUMENT')],
  ['låsskärm med lösenordsfält', /type="password"/.test(app) && app.includes('Vault är låst')],
  ['inga tomma knappar', !/<button[^>]*>\s*<\/button>/.test(combined)],
]

const failed = checks.filter(([, passed]) => !passed)
for (const [name, passed] of checks) console.log(`${passed ? 'PASS' : 'FAIL'} ${name}`)
if (failed.length) {
  console.error(`${failed.length} UI-/tillgänglighetskontroller misslyckades`)
  process.exit(1)
}
console.log(`${checks.length}/${checks.length} UI-/tillgänglighetskontroller godkända`)
