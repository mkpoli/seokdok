import { describe, expect, test } from 'bun:test'
import { readdirSync, readFileSync } from 'node:fs'
import { join } from 'node:path'
import { DictIndex, expandRows, FILES, LICENCE_URLS, normalize, packRows, pageOf, posKey } from './dict-browse.js'
import text from './dict-text.js'

const dict = join(import.meta.dirname, '..', '..', '..', 'dict')
const read = (stem) => readFileSync(join(dict, `${stem}.tsv`), 'utf8')
const isRow = (l) => l && !l.startsWith('#') && !l.startsWith('lemma\t') && !l.startsWith('reading\t')

function fixture(tables) {
  return new DictIndex(expandRows(packRows((stem) => tables[stem] ?? '')))
}

describe('the packed file', () => {
  const index = new DictIndex(expandRows(packRows(read)))

  test('covers every committed file and no other', () => {
    const onDisk = readdirSync(dict)
      .filter((f) => f.endsWith('.tsv') && !f.startsWith('user'))
      .map((f) => f.replace(/\.tsv$/, ''))
    expect(FILES.map((f) => f.stem).sort()).toEqual(onDisk.sort())
  })

  test('has as many rows per file as the file has', () => {
    for (const { stem } of FILES) {
      const expected = read(stem).split('\n').filter(isRow).length
      expect(index.fileRows[index.stems.indexOf(stem)]).toBe(expected)
    }
    expect(index.size).toBe(FILES.reduce((n, { stem }) => n + read(stem).split('\n').filter(isRow).length, 0))
  })

  test('keeps a row of each format whole', () => {
    const rows = [...index.search('있다')].map((i) => index.row(i))
    expect(rows.find((r) => r.spelling === '在ㅆ다' && r.file === 'editorial')).toMatchObject({ reading: '있다', pos: 'adj', count: '20' })
    expect(pageOf(index, index.search('韓國語'), { file: 'sino' }).rows[0]).toMatchObject({ reading: '한국어', spelling: '韓國語', file: 'sino', pos: '' })
  })

  test('finds a word by reading and by Hanja, exact rows first', () => {
    expect(index.row(index.search('하늘')[0]).reading).toBe('하늘')
    expect(index.row(index.search('天')[0]).spelling).toBe('天')
    const inside = pageOf(index, index.search('讀'), { file: 'community' })
    expect(inside.rows.every((r) => r.spelling.includes('讀') && r.file === 'community')).toBe(true)
    expect(inside.total).toBeGreaterThan(0)
  })

  test('ignores hits in notes', () => {
    const note = read('editorial').split('\n').filter(isRow).map((l) => l.split('\t')[6]).find((n) => n?.includes('mkpoli'))
    expect(note).toBeTruthy()
    expect(index.search('mkpoli').length).toBe(0)
  })

  test('every source has text in both languages and a licence that resolves', () => {
    for (const { stem, licences } of FILES) {
      for (const lang of ['ko', 'en']) {
        expect(text[lang].files[stem].name).toBeTruthy()
        expect(text[lang].files[stem].about).toBeTruthy()
      }
      expect(licences.length).toBeGreaterThan(0)
      for (const l of licences) {
        const [id, scope] = l.split(':')
        expect(LICENCE_URLS[id]).toBeTruthy()
        if (scope) expect(text.ko.scopes[scope] && text.en.scopes[scope]).toBeTruthy()
      }
    }
  })

  test('both languages name every part of speech and row tag', () => {
    expect(Object.keys(text.ko.pos).sort()).toEqual(Object.keys(text.en.pos).sort())
    expect(Object.keys(text.ko.tags).sort()).toEqual(Object.keys(text.en.tags).sort())
    const tags = new Set()
    for (let i = 0; i < index.size; i++) {
      const line = index.line(i)
      const tag = line.split('\t')[5]
      if (tag) tags.add(tag)
    }
    for (const tag of tags) expect(text.en.tags[tag]).toBeTruthy()
    for (const pos of ['noun', '명사', '동사', 'adj', '형용사']) expect(text.en.pos[posKey(pos)]).toBeTruthy()
    expect(posKey('조수사')).toBeNull()
  })
})

test('marks in the Korean text hold Hanja over a Hangul reading', () => {
  const strings = (v) => (typeof v === 'string' ? [v] : Object.values(v).flatMap(strings))
  const same = (a, b) => {
    if (typeof a === 'string') return expect(typeof b).toBe('string')
    expect(Object.keys(a).sort()).toEqual(Object.keys(b).sort())
    for (const k of Object.keys(a)) same(a[k], b[k])
  }
  same(text.ko, text.en)
  for (const s of strings(text.ko)) {
    expect(s.replace(/\{[^|}]+\|[^}]+\}/g, '')).not.toMatch(/[{}|]/)
    for (const [, h, k] of s.matchAll(/\{([^|}]*)\|([^}]*)\}/g)) {
      expect(h).toMatch(/^[㐀-䶿一-鿿豈-﫿]+$/)
      expect(k).toMatch(/^[가-힣]+$/)
    }
  }
})

describe('search', () => {
  const index = fixture({
    community: '가을\t秋\t명사\t\tcommunity\t3\t\n하늘\t天\t명사\t\tcommunity\t7\t\n',
    gloss: '하늘\t天\tnoun\t\tgloss\t\t하늘 천\n밤하늘\t夜天\tnoun\t\tgloss\t\t\n하늘빛\t天色\tnoun\t\tgloss\t\t\n',
    sino: '가\t金\t1\tlexicon\n가\t金\t1\tlexicon\n',
    loanwords: '피자\t피자\t\tloanword\tit\n',
    etym: '가게\t假家\t\tnativised\n명사화\t名詞化\t\tform\n유종\t油種\t\tform\n',
  })

  test('ranks exact, then start, then inside', () => {
    const rows = [...index.search('하늘')].map((i) => index.row(i))
    expect(rows.map((r) => `${r.reading} ${r.file}`)).toEqual(['하늘 community', '하늘 gloss', '하늘빛 gloss', '밤하늘 gloss'])
  })

  test('an empty query lists everything in dictionary order', () => {
    expect(index.search('  ').length).toBe(index.size)
    const readings = [...index.search('')].map((i) => index.row(i).reading)
    expect(readings).toEqual([...readings].sort())
  })

  test('compatibility Hanja match both ways, and show as the file has them', () => {
    expect(normalize('金')).toBe('金')
    const rows = [...index.search('金')].map((i) => index.row(i).spelling)
    expect(rows).toContain('金')
    expect(rows).toContain('金')
    expect([...index.search('金')].map((i) => index.row(i).spelling)).toContain('金')
  })

  test('decomposed Hangul finds the composed row', () => {
    expect(index.search('하늘'.normalize('NFD')).length).toBe(4)
  })

  test('a word in only the note or count column is no match', () => {
    expect(index.search('천').length).toBe(0)
    expect(index.search('it').length).toBe(0)
    expect(index.search('7').length).toBe(0)
  })

  test('a row source only differs from its file when rows within it differ', () => {
    expect(index.row(index.search('假家')[0]).tag).toBe('nativised')
    expect(index.row(index.search('油種')[0]).tag).toBe('')
    expect(index.row(index.search('피자')[0]).note).toBe('it')
  })

  test('pages and filters stay in range', () => {
    const hits = index.search('')
    expect(pageOf(index, hits, { page: 99, size: 3 })).toMatchObject({ page: Math.ceil(index.size / 3), total: index.size })
    const gloss = pageOf(index, hits, { file: 'gloss', page: 1, size: 2 })
    expect(gloss).toMatchObject({ total: 3, pages: 2 })
    expect(gloss.rows.length).toBe(2)
    expect(pageOf(index, hits, { file: 'gloss', page: 2, size: 2 }).rows.length).toBe(1)
    expect(pageOf(index, index.search('없는말'), { page: 1 })).toMatchObject({ rows: [], total: 0, page: 1, pages: 1 })
  })
})
