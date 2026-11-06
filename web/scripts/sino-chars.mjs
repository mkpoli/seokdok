// Sino-Korean readings of every Hanja used in the site's {漢字|한자} marks,
// from dict/sino.tsv and fusion.tsv. In 國漢文混用 mode a mark whose reading
// is its characters' own readings stays in Hanja; the others (訓讀) show
// their Hangul. Writes src/lib/sino-chars.json.
import { readFileSync, readdirSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'

const web = join(import.meta.dirname, '..')
const dict = join(web, '..', 'dict')
const sources = [join(web, 'src/lib/i18n.svelte.js'), join(web, 'src/lib/pages.js'), join(web, 'src/lib/dict-text.js'), ...readdirSync(join(web, 'src/ui')).map((f) => join(web, 'src/ui', f))]
const used = new Set()
for (const f of sources) {
  for (const [, h] of readFileSync(f, 'utf8').matchAll(/\{([^|}]+)\|[^}]+\}/g)) for (const c of h) used.add(c)
}
const readings = {}
for (const f of ['sino.tsv', 'fusion.tsv']) {
  for (const l of readFileSync(join(dict, f), 'utf8').split('\n')) {
    const [reading, hanja] = l.split('\t')
    if (hanja && [...hanja].length === 1 && used.has(hanja) && [...reading].length === 1) {
      ;(readings[hanja] ??= []).includes(reading) || readings[hanja].push(reading)
    }
  }
}
const sorted = Object.fromEntries(Object.keys(readings).sort().map((k) => [k, readings[k].sort()]))
writeFileSync(join(web, 'src/lib/sino-chars.json'), JSON.stringify(sorted, null, 0).replace(/\],/g, '],\n') + '\n')
console.log(`${Object.keys(sorted).length} characters`)
