import { test, expect } from 'bun:test'

// Runes are compiled by Svelte; under bun, $state is shimmed as identity.
globalThis.$state = (v) => v
const { Ime } = await import('./ime.svelte.js')

function composer() {
  let t = ''
  const map = { g: 'ㅎ', k: 'ㅏ', s: 'ㄴ' }
  return {
    input: (k) => (map[k] ? ((t += map[k]), true) : false),
    backspace: () => ((t = [...t].slice(0, -1).join('')), true),
    pushLiteral: (s) => (t += s),
    text: () => t,
    setText: (s) => (t = s),
    clear: () => (t = ''),
  }
}

const cands = (text) =>
  ({ 하늘: ['天', '乾', '하늘'], 하: ['夏', '下', '하'], 늘: ['늘'], 하늘을: ['天을', '하늘을'], 봤다: ['見ㅆ다', '봤다'] })[text]?.map(
    (t) => ({ text: t, note: '' }),
  ) ?? [{ text, note: '' }]
// One segment per space-separated word.
const segment = async (text) => text.split(/( )/).filter(Boolean).map((w) => ({ reading: w, candidates: cands(w) }))
const whole = async (text) => cands(text)
const tick = () => new Promise((r) => setTimeout(r))

function ime() {
  return new Ime(composer(), segment, whole)
}

test('space converts; a second space opens the window on the next candidate', async () => {
  const m = ime()
  m.insertText('하늘')
  m.key({ key: ' ' })
  await tick()
  expect(m.converted).toBe('天')
  expect(m.open).toBe(false)
  m.key({ key: ' ' })
  expect(m.open).toBe(true)
  expect(m.converted).toBe('乾')
  m.key({ key: ' ', shift: true })
  expect(m.converted).toBe('天')
})

test('digits only act with the window open, and pick and confirm', async () => {
  const m = ime()
  m.insertText('하늘')
  await m.convert()
  m.key({ key: '2' })
  expect(m.doc).toBe('天2')
  m.insertText('하늘')
  await m.convert()
  m.key({ key: ' ' })
  m.key({ key: '2' })
  expect(m.doc).toBe('天2乾')
  expect(m.segs).toBe(null)
})

test('a digit confirms through the focused segment and leaves the rest converting', async () => {
  const m = ime()
  m.insertText('하늘을 봤다')
  await m.convert()
  m.key({ key: 'ArrowDown' })
  m.key({ key: '1' })
  expect(m.doc).toBe('天을')
  expect(m.segs.map((s) => s.reading).join('')).toBe(' 봤다')
  expect(m.current.reading).toBe('봤다')
  m.key({ key: 'Enter' })
  expect(m.doc).toBe('天을 見ㅆ다')
})

test('enter confirms everything; esc closes the window, then returns to Hangul', async () => {
  const m = ime()
  m.insertText('하늘')
  await m.convert()
  m.key({ key: ' ' })
  m.key({ key: 'Escape' })
  expect(m.open).toBe(false)
  expect(m.segs).not.toBe(null)
  m.key({ key: 'Escape' })
  expect(m.segs).toBe(null)
  expect(m.pre).toBe('하늘')
  await m.convert()
  m.key({ key: 'Enter' })
  expect(m.doc).toBe('天')
})

test('tab pages through candidates', async () => {
  const many = Array.from({ length: 20 }, (_, i) => ({ text: 'c' + i, note: '' }))
  const m = new Ime(composer(), async (t) => [{ reading: t, candidates: many }], async () => many)
  m.insertText('하')
  await m.convert()
  m.key({ key: 'Tab' })
  expect(m.open).toBe(true)
  expect(m.current.pick).toBe(9)
  m.key({ key: '3' })
  expect(m.doc).toBe('c11')
})

test('shift+arrows resize the focused segment', async () => {
  const m = ime()
  m.insertText('하늘')
  await m.convert()
  m.key({ key: 'ArrowLeft', shift: true })
  await tick()
  expect(m.segs.map((s) => s.reading)).toEqual(['하', '늘'])
  expect(m.converted).toBe('夏늘')
  m.key({ key: 'ArrowRight', shift: true })
  await tick()
  expect(m.segs.map((s) => s.reading)).toEqual(['하늘'])
})

test('typing on after a Space conversion inserts the word break', async () => {
  const m = ime()
  m.insertText('하늘')
  m.key({ key: ' ' })
  await tick()
  m.key({ key: 'g' })
  expect(m.doc).toBe('天 ')
  expect(m.pre).toBe('ㅎ')
})

test('a conversion started with Tab, or confirmed by punctuation, adds no space', async () => {
  const m = ime()
  m.insertText('하늘')
  m.key({ key: 'Tab' })
  await tick()
  m.key({ key: 'g' })
  expect(m.doc).toBe('天')
  const n = ime()
  n.insertText('하늘')
  n.key({ key: ' ' })
  await tick()
  n.key({ key: '.' })
  expect(n.doc).toBe('天.')
})

test('the caret moves, and text goes in at the caret', async () => {
  const m = ime()
  m.setDoc('AB')
  m.key({ key: 'ArrowLeft' })
  expect(m.cursor).toBe(1)
  m.insertText('하늘')
  await m.convert()
  m.key({ key: 'Enter' })
  expect(m.doc).toBe('A天B')
  expect(m.cursor).toBe(2)
  m.key({ key: 'Home' })
  expect(m.cursor).toBe(0)
  m.key({ key: 'Delete' })
  expect(m.doc).toBe('天B')
  m.key({ key: 'End' })
  m.key({ key: 'Backspace' })
  expect(m.doc).toBe('天')
  m.setCursor(0)
  m.key({ key: '1' })
  expect(m.doc).toBe('1天')
})

test("spaceMode 'space': Space types spaces, Tab converts", async () => {
  const m = ime()
  m.spaceMode = 'space'
  m.insertText('하늘')
  m.key({ key: ' ' })
  expect(m.pre).toBe('하늘 ')
  m.key({ key: 'Backspace' })
  m.key({ key: 'Tab' })
  await tick()
  m.key({ key: ' ' })
  expect(m.doc).toBe('天 ')
})

test('backspace edits the preedit, then the document', () => {
  const m = ime()
  m.setDoc('ab')
  m.key({ key: 'g' })
  m.key({ key: 'Backspace' })
  expect(m.pre).toBe('')
  m.key({ key: 'Backspace' })
  expect(m.doc).toBe('a')
})

test('keys typed while converting are queued, not lost', async () => {
  let release
  const slow = (t) => new Promise((r) => (release = () => r(segment(t))))
  const m = new Ime(composer(), (t) => slow(t), whole)
  m.insertText('하늘')
  m.key({ key: ' ' })
  m.key({ key: 'g' })
  m.key({ key: 'k' })
  expect(m.doc).toBe('')
  release()
  await tick()
  await tick()
  expect(m.doc).toBe('天 ')
  expect(m.pre).toBe('ㅎㅏ')
})
