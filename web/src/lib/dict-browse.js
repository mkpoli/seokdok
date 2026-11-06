// The /dict page's data: every row of the committed dictionary files, packed by
// scripts/bundle-dict.mjs into public/dict/browse.bin and searched in a worker.
//
// The first line lists the files. Each other line is
//   reading  spelling  file  pos  count  tag  note
// with file a base-36 index into that list and trailing empty fields dropped.
// Rows sort by reading, then spelling, so one word's rows from different files
// sit together. A reading or spelling equal to the row above is left blank, which
// saves 18% after gzip; expandRows() restores them for searching.

// One entry per committed dict/*.tsv, in the engine's priority order. A file
// missing here stays off the page; dict/user.tsv is a person's own and never ships.
export const FILES = [
  ['corpus', 'native', ['MIT']],
  ['editorial', 'native', ['MIT']],
  ['community', 'native', ['permission']],
  ['eonhae', 'native', ['CC BY-SA 4.0', 'CC BY-NC-SA 2.0 KR:namu']],
  ['kun-attested', 'native', ['CC BY-SA 4.0']],
  ['suggested', 'native', ['MIT', 'CC BY-SA 4.0:chat']],
  ['place-names', 'native', ['CC BY-SA 4.0']],
  ['wikt-native', 'native', ['CC BY-SA 4.0']],
  ['gloss', 'native', ['BSD-3-Clause']],
  ['kun-ja', 'native', ['CC BY-SA 4.0']],
  ['kun-ja-weak', 'native', ['CC BY-SA 4.0']],
  ['editorial-sino', 'sino', ['MIT']],
  ['fusion', 'sino', ['GPL-3.0']],
  ['sino', 'sino', ['BSD-3-Clause']],
  ['etym', 'sino', ['CC BY-SA 4.0']],
  ['loanwords', 'sino', ['CC BY-SA 4.0']],
  ['hun', 'sino', ['BSD-3-Clause']],
].map(([stem, kind, licences]) => ({ stem, kind, licences }))

const REPO = 'https://github.com/mkpoli/seokdok/blob/main'

/** Where each licence in FILES can be read; `permission` has no text of its own. */
export const LICENCE_URLS = {
  MIT: `${REPO}/LICENSE`,
  'CC BY-SA 4.0': 'https://creativecommons.org/licenses/by-sa/4.0/',
  'CC BY-NC-SA 2.0 KR': 'https://creativecommons.org/licenses/by-nc-sa/2.0/kr/',
  'BSD-3-Clause': 'https://github.com/libhangul/libhangul/blob/23d2cd9d10d516d523d60c562dbcc7ea08139a09/data/hanja/hanja.txt',
  'GPL-3.0': `${REPO}/dict/LICENSE-fusion.txt`,
  permission: '/dict/NOTICE.txt',
}

export const fileUrl = (stem) => `${REPO}/dict/${stem}.tsv`
export const README_URL = `${REPO}/dict/README.md#licences-and-provenance`

const dataLines = (text) =>
  text.split('\n').filter((l) => l && !l.startsWith('#') && !l.startsWith('lemma\t') && !l.startsWith('reading\t'))

// The source most of a file's rows carry says nothing beyond the file's name.
// The others mark rows that differ within it (a sino.tsv row the chat keeps in Hangul).
function commonest(values) {
  const seen = new Map()
  for (const v of values) seen.set(v, (seen.get(v) ?? 0) + 1)
  return [...seen].reduce((a, b) => (b[1] > a[1] ? b : a), ['', 0])[0]
}

/** @param {(stem: string) => string} read the text of dict/<stem>.tsv */
export function packRows(read) {
  const rows = []
  FILES.forEach(({ stem, kind }, i) => {
    const fields = dataLines(read(stem)).map((l) => l.split('\t'))
    const code = i.toString(36)
    const sourceAt = kind === 'native' ? 4 : 3
    const common = commonest(fields.map((f) => f[sourceAt] ?? ''))
    for (const f of fields) {
      const tag = (f[sourceAt] ?? '') === common ? '' : f[sourceAt]
      rows.push(
        kind === 'native'
          ? [f[0], f[1], code, f[2] ?? '', f[5] ?? '', tag, f[6] ?? '']
          : // loanwords.tsv has a fifth column, the source language.
            [f[0], f[1], code, '', f[2] ?? '', tag, f[4] ?? ''],
      )
    }
  })
  const order = (a, b) => (a < b ? -1 : a > b ? 1 : 0)
  rows.sort((a, b) => order(a[0], b[0]) || order(a[1], b[1]) || parseInt(a[2], 36) - parseInt(b[2], 36))
  let reading = ''
  let spelling = ''
  const lines = rows.map((r) => {
    let end = r.length
    while (r[end - 1] === '') end--
    const line = [r[0] === reading ? '' : r[0], r[1] === spelling ? '' : r[1], ...r.slice(2, end)].join('\t')
    reading = r[0]
    spelling = r[1]
    return line
  })
  return [FILES.map((f) => f.stem).join(','), ...lines].join('\n')
}

/** Fills in the readings and spellings packRows() left blank. */
export function expandRows(packed) {
  const lines = packed.split('\n')
  let reading = ''
  let spelling = ''
  for (let i = 1; i < lines.length; i++) {
    const line = lines[i]
    const first = line.indexOf('\t')
    const second = line.indexOf('\t', first + 1)
    if (first > 0) reading = line.slice(0, first)
    if (second > first + 1) spelling = line.slice(first + 1, second)
    lines[i] = `${reading}\t${spelling}${line.slice(second)}`
  }
  return lines.join('\n')
}

/** Query text as the rows hold it: composed Hangul (macOS sends jamo), unified Hanja. */
export const normalize = (q) => q.normalize('NFC').replace(/\s+/g, ' ').trim()

export class DictIndex {
  /** @param {string} text the output of expandRows() */
  constructor(text) {
    this.text = text
    const head = text.indexOf('\n')
    this.stems = text.slice(0, head === -1 ? text.length : head).split(',')
    let n = 0
    for (let at = head; at !== -1; at = text.indexOf('\n', at + 1)) n++
    this.size = n
    // starts[i] is where row i begins; starts[n] sits one past the last newline.
    this.starts = new Uint32Array(n + 1)
    this.files = new Uint8Array(n)
    this.fileRows = new Array(this.stems.length).fill(0)
    let at = head + 1
    for (let i = 0; i < n; i++) {
      this.starts[i] = at
      const file = parseInt(text[text.indexOf('\t', text.indexOf('\t', at) + 1) + 1], 36)
      this.files[i] = file
      this.fileRows[file]++
      const end = text.indexOf('\n', at)
      at = end === -1 ? text.length + 1 : end + 1
    }
    this.starts[n] = at
    // A Korean IME hands out compatibility Hanja, and a few rows hold them too.
    // A query is folded to unified forms, so those rows are compared folded.
    this.compat = []
    for (const m of text.matchAll(/[豈-﫿]/g)) {
      const row = this.rowAt(m.index)
      if (this.compat.at(-1)?.row === row) continue
      const [reading, spelling] = this.line(row).split('\t')
      if (normalize(reading) !== reading || normalize(spelling) !== spelling) {
        this.compat.push({ row, reading: normalize(reading), spelling: normalize(spelling) })
      }
    }
    this.all = Int32Array.from({ length: n }, (_, i) => i)
  }

  line(i) {
    return this.text.slice(this.starts[i], this.starts[i + 1] - 1)
  }

  /** The row holding text offset `at`. */
  rowAt(at) {
    let lo = 0
    let hi = this.size - 1
    while (lo < hi) {
      const mid = (lo + hi + 1) >> 1
      if (this.starts[mid] <= at) lo = mid
      else hi = mid - 1
    }
    return lo
  }

  row(i) {
    const f = this.line(i).split('\t')
    return {
      reading: f[0],
      spelling: f[1],
      file: this.stems[parseInt(f[2], 36)],
      pos: f[3] ?? '',
      count: f[4] ?? '',
      tag: f[5] ?? '',
      note: f[6] ?? '',
    }
  }

  /**
   * Rows whose reading or spelling holds the query: exact matches first, best
   * file first, then matches at the start, then the rest in dictionary order.
   */
  search(query) {
    const q = normalize(query)
    if (!q) return this.all
    const { text, starts } = this
    const exact = []
    const start = []
    const inside = []
    const place = (i, reading, spelling) => {
      if (reading === q || spelling === q) exact.push(i)
      else if (reading.startsWith(q) || spelling.startsWith(q)) start.push(i)
      else inside.push(i)
    }
    for (let at = text.indexOf(q, text.indexOf('\n') + 1); at !== -1; ) {
      const i = this.rowAt(at)
      const tab = text.indexOf('\t', starts[i])
      const keyEnd = text.indexOf('\t', tab + 1)
      // A hit past the spelling is in a note or a count.
      if (at + q.length <= keyEnd) place(i, text.slice(starts[i], tab), text.slice(tab + 1, keyEnd))
      at = i + 1 < this.size ? text.indexOf(q, starts[i + 1]) : -1
    }
    const found = new Set()
    for (const list of [exact, start, inside]) for (const i of list) found.add(i)
    for (const { row, reading, spelling } of this.compat) {
      if (!found.has(row) && (reading.includes(q) || spelling.includes(q))) place(row, reading, spelling)
    }
    const byRow = (a, b) => a - b
    exact.sort((a, b) => this.files[a] - this.files[b] || a - b)
    start.sort(byRow)
    inside.sort(byRow)
    return Int32Array.from([...exact, ...start, ...inside])
  }

  /** How many of `hits` each file holds, by file index. */
  tally(hits) {
    const counts = new Array(this.stems.length).fill(0)
    for (const i of hits) counts[this.files[i]]++
    return counts
  }
}

/** One page of `hits`, optionally from one file. Out-of-range pages land on the nearest. */
export function pageOf(index, hits, { file = '', page = 1, size = 50 } = {}) {
  const fileIndex = index.stems.indexOf(file)
  const counts = index.tally(hits)
  const total = fileIndex === -1 ? hits.length : counts[fileIndex]
  const pages = Math.max(1, Math.ceil(total / size))
  const current = Math.min(Math.max(1, Math.trunc(page) || 1), pages)
  const skip = (current - 1) * size
  const rows = []
  if (fileIndex === -1) {
    for (let k = skip; k < Math.min(skip + size, total); k++) rows.push(index.row(hits[k]))
  } else {
    let seen = 0
    for (const i of hits) {
      if (index.files[i] !== fileIndex) continue
      if (seen++ < skip) continue
      rows.push(index.row(i))
      if (rows.length === size) break
    }
  }
  return { rows, total, page: current, pages, counts }
}

// The part-of-speech names files use, in both spellings dict/README.md allows.
const POS = { 명사: 'noun', 대명사: 'pron', 수사: 'num', 의존명사: 'bound', 부사: 'adv', 관형사: 'det', 감탄사: 'interj', 동사: 'verb', 형용사: 'adj' }
const POS_KEYS = new Set(Object.values(POS))

/** The label key of a row's part of speech, or null to show it as written. */
export const posKey = (pos) => POS[pos] ?? (POS_KEYS.has(pos) ? pos : null)
