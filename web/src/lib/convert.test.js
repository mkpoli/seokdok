import { test, expect } from 'bun:test'
import { convertLine, byLine, lineKey, picked } from './convert.js'

const c = (...texts) => texts.map((text) => ({ text, note: '' }))

// What the engine answers for the converter's examples, one segment per word
// and one per run of whitespace; a word holding Hanja has its own spelling first.
const MENUS = {
  '初음': c('初음', '始', '처음'),
  '來신': c('來신', '倒신', '오신'),
  '事ㅅ을': c('事ㅅ을', '件을', '것을'),
  '歡迎합니다': c('歡迎합니다', '歡迎ㅂ니다', '환영합니다'),
  '天을': c('天을', '乾을', '하늘을'),
  '見ㅆ다': c('見ㅆ다', '見았다', '봤다'),
  '先生님께서': c('先生님께서', '先生主께서', '선생님께서'),
  '良心': c('良心', '兩心', '양심'),
  '天': c('天', '乾', '하늘'),
  '超ㅜ': c('超ㅜ', '超무', '너무'),
  '過무': c('過무', '超무', '너무'),
  '此處서': c('此處서', '此所서', '여기서'),
  봤다: c('見ㅆ다', '見았다', '봤다'),
  하늘을: c('天을', '乾을', '하늘을'),
}
const engine = {
  calls: [],
  segments: async (text) => {
    engine.calls.push(text)
    return text.split(/(\s+)/).filter(Boolean).map((w) => ({ reading: w, candidates: MENUS[w] ?? c(w) }))
  },
}
const io = { segments: engine.segments, input: (t) => t, output: (x) => x }

const EXAMPLES = ['初음 來신 事ㅅ을 歡迎합니다', '天을 見ㅆ다', '先生님께서', '良心', '天', '超ㅜ', '過무', '此處서']

test('a line already in 釋讀 keeps its text and every word has a menu', async () => {
  for (const line of EXAMPLES) {
    const segs = await convertLine(line, io)
    expect(picked([segs])).toBe(line)
    for (const s of segs.filter((s) => /\S/.test(s.reading))) expect(s.candidates.length).toBeGreaterThan(1)
    for (const s of segs.filter((s) => !/\S/.test(s.reading))) expect(s.candidates).toHaveLength(1)
  }
})

test('a line with no Hangul reaches the engine; only a blank one does not', async () => {
  engine.calls.length = 0
  const segs = await convertLine('良心', io)
  expect(engine.calls).toEqual(['良心'])
  expect(segs[0].candidates.map((x) => x.text)).toEqual(['良心', '兩心', '양심'])
  for (const blank of ['', '  ']) {
    const segs = await convertLine(blank, io)
    expect(segs).toEqual([{ reading: blank, candidates: [{ text: blank, note: '' }], pick: 0 }])
  }
  expect(engine.calls).toEqual(['良心'])
})

test('picking another candidate changes only that word', async () => {
  const segs = await convertLine('天을 見ㅆ다', io)
  segs[0].pick = 2
  expect(picked([segs])).toBe('하늘을 見ㅆ다')
  segs[2].pick = 1
  expect(picked([segs])).toBe('하늘을 見았다')
})

test('Hangul words next to Hanja words convert, whichever side they stand on', async () => {
  expect(picked([await convertLine('天을 봤다', io)])).toBe('天을 見ㅆ다')
  expect(picked([await convertLine('하늘을 見ㅆ다', io)])).toBe('天을 見ㅆ다')
})

test('a line that did not change keeps its picks, also when the reading is the typed text', async () => {
  const lines = [await convertLine('天을 見ㅆ다', io), await convertLine('良心', io)]
  lines[0][0].pick = 2
  lines[1][0].pick = 1
  expect(lines.map(lineKey)).toEqual(['天을 見ㅆ다', '良心'])
  const old = byLine(lines)
  expect(old.get('天을 見ㅆ다')).toBe(lines[0])
  expect(picked([old.get('良心')])).toBe('兩心')
  expect(old.has('天을')).toBe(false)
})

test('input rules run before the engine and output rules after it', async () => {
  const seen = []
  const segs = await convertLine('天을 見ㅆ다', {
    segments: async (t) => (seen.push(t), engine.segments(t)),
    input: (t) => t.replace('見ㅆ다', '見았다'),
    output: (x) => x.filter((k) => k.text !== '乾을'),
  })
  expect(seen).toEqual(['天을 見았다'])
  expect(segs[0].candidates.map((k) => k.text)).toEqual(['天을', '하늘을'])
})
