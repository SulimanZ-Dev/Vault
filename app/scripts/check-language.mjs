import { readFileSync, readdirSync, statSync } from 'node:fs'
import { join, relative } from 'node:path'

const roots = ['src', 'src-tauri/src', '../README.md', '../docs/10-implementation-log.md']
const blockedPatterns = [
  /Ã/g,
  /Â/g,
  /\bwebblas/g,
  /\bInnehall\b/g,
  /\bsok(index|ning)?\b/g,
  /\bforklar/g,
  /\bhalsok/g,
  /\bvantar\b/g,
  /\bkraver\b/g,
  /\buppmarksamhet\b/g,
  /\bGodkann\b/g,
  /\bforsta\b/g,
  /\bgora\b/g,
  /\bhart\b/g,
  /\boven nar\b/g,
]

function collectFiles(path) {
  const stats = statSync(path)
  if (stats.isFile()) {
    return [path]
  }

  return readdirSync(path).flatMap((entry) => {
    const child = join(path, entry)
    const childStats = statSync(child)
    if (childStats.isDirectory()) {
      return collectFiles(child)
    }
    return [child]
  })
}

const files = roots.flatMap(collectFiles).filter((file) =>
  /\.(css|md|mjs|rs|tsx?)$/.test(file),
)
const failures = []

for (const file of files) {
  const text = readFileSync(file, 'utf8')
  for (const pattern of blockedPatterns) {
    pattern.lastIndex = 0
    if (pattern.test(text)) {
      failures.push(`${relative(process.cwd(), file)} matches ${pattern}`)
    }
  }
}

if (failures.length) {
  console.error('Language guard failed:')
  console.error(failures.join('\n'))
  process.exit(1)
}

console.log(`Language guard passed for ${files.length} files`)
