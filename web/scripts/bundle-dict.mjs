// Pack dict/*.tsv into two gzipped files for the browser:
//   public/dict/native.bin  訓讀 entries, in the engine's priority order
//   public/dict/sino.bin    音讀 entries (fusion.tsv + sino.tsv), one line per
//                              reading/Hanja pair with its largest count, then
//                              one with the rest of its counts if there are any
// Comments are dropped; licence notices travel in public/dict/NOTICE.txt.
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs'
import { gzipSync } from 'node:zlib'
import { join } from 'node:path'

const root = join(import.meta.dirname, '..', '..')
const dict = join(root, 'dict')
const out = join(import.meta.dirname, '..', 'public', 'dict')
mkdirSync(out, { recursive: true })

const lines = (f) =>
  readFileSync(join(dict, f), 'utf8')
    .split('\n')
    .filter((l) => l && !l.startsWith('#') && !l.startsWith('lemma\t') && !l.startsWith('reading\t'))

const native = ['corpus.tsv', 'editorial.tsv', 'community.tsv', 'eonhae.tsv', 'suggested.tsv', 'gloss.tsv', 'kun-ja.tsv', 'kun-ja-weak.tsv']
  .flatMap((f) => lines(f).map((l) => l.split('\t').slice(0, 6).join('\t')))
const sino = new Map()
for (const f of ['editorial-sino.tsv', 'fusion.tsv', 'sino.tsv', 'etym.tsv', 'loanwords.tsv', 'hun.tsv']) {
  for (const l of lines(f)) {
    const [reading, hanja, count = '0', source = ''] = l.split('\t')
    // Gloss keys are looked up apart from readings, so they never merge with one.
    const key = reading + '\t' + hanja + (source === 'hun' ? '\thun' : '')
    const n = Number(count) || 0
    const prev = sino.get(key)
    if (!prev) sino.set(key, { n, sum: n, source })
    else {
      prev.sum += n
      if (n > prev.n) Object.assign(prev, { n, source })
    }
  }
}
// The engine costs a pair by its largest count and breaks ties between readings
// by the sum over the files (선생 and 선상 both have 30 in sino.tsv, and 선생 has
// 16 more in fusion.tsv), so the rest of the sum travels in a second row.
const sinoText = [...sino]
  .flatMap(([key, v]) => {
    const k = key.split('\t').slice(0, 2).join('\t')
    return [`${k}\t${v.n || ''}\t${v.source}`, ...(v.sum > v.n ? [`${k}\t${v.sum - v.n}\t${v.source}`] : [])]
  })
  .join('\n')

const write = (name, text) => {
  const gz = gzipSync(Buffer.from(text, 'utf8'), { level: 9 })
  writeFileSync(join(out, name), gz)
  console.log(`${name}: ${(text.length / 1e6).toFixed(1)} M chars → ${(gz.length / 1e6).toFixed(2)} MB gz`)
}
write('native.bin', native.join('\n'))
write('sino.bin', sinoText)
writeFileSync(
  join(out, 'NOTICE.txt'),
  readFileSync(join(dict, 'README.md'), 'utf8') + '\n\n' + readFileSync(join(dict, 'LICENSE-fusion.txt'), 'utf8'),
)
