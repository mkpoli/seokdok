import { test, expect } from 'bun:test'
import { chars, splitsChar, HANGUL } from './chars.js'
import { rubify, WORD } from './rubify.js'

// Runes are compiled by Svelte; under bun, $state is shimmed as identity.
globalThis.$state = (v) => v
const { Ime } = await import('./ime.svelte.js')

// Old Hangul in escapes, so that no editor can normalise it.
const HOLL = 'ᄒᆞᆯ' // ᄒᆞᆯ
const SA = 'ᄉᆞ' // ᄉᆞ
const SSAM = 'ᄊᆞᆷ' // ᄊᆞᆷ
const STTO = 'ᄯᅩ' // ᄯᅩ
const MIXED = [
  [`追加${HOLL}까요`, ['追', '加', HOLL, '까', '요']],
  ['나랏말ᄊᆞ미', ['나', '랏', '말', 'ᄊᆞ', '미']],
  [HOLL, [HOLL]],
  [`${HOLL}${SA}${SSAM}${STTO}`, [HOLL, SA, SSAM, STTO]],
  [`${STTO} ${HOLL}까요`, [STTO, ' ', HOLL, '까', '요']],
  ['ꥠힰퟋᅟᆞ', ['ꥠힰퟋ', 'ᅟᆞ']],
]

function composer() {
  let t = ''
  return {
    input: () => false,
    backspace: () => ((t = [...t].slice(0, -1).join('')), true),
    pushLiteral: (s) => (t += s),
    text: () => t,
    setText: (s) => (t = s),
    clear: () => (t = ''),
  }
}

const one = (text) => [{ text, note: '' }]
// One segment per space-separated word, its reading as the only candidate.
const byWord = async (text) => text.split(/( )/).filter(Boolean).map((w) => ({ reading: w, candidates: one(w) }))
// A careless converter: one segment for every code point.
const byCodePoint = async (text) => [...text].map((c) => ({ reading: c, candidates: one(c) }))
const whole = async (text) => one(text)
const tick = () => new Promise((r) => setTimeout(r))

function assertWhole(m, text) {
  const segs = m.segs.map((s) => s.reading)
  expect(segs.join('')).toBe(text)
  segs.slice(1).forEach((s, i) => expect(splitsChar(segs.slice(0, i + 1).join(''), s)).toBe(false))
  expect(segs.flatMap(chars)).toEqual(chars(text))
}

test('chars() keeps a conjoining syllable whole', () => {
  for (const [text, units] of MIXED) expect(chars(text)).toEqual(units)
})

test('every kind of Hangul counts as Hangul', () => {
  for (const c of ['가', 'ㅎ', 'ㆍ', 'ᄒ', 'ᆞ', 'ᆯ', 'ꥠ', 'ힰ', 'ퟋ']) expect(HANGUL.test(c)).toBe(true)
  for (const c of ['追', 'a', ' ', '1', '。']) expect(HANGUL.test(c)).toBe(false)
})

for (const [name, segment] of [
  ['by word', byWord],
  ['by code point', byCodePoint],
]) {
  test(`resize walks every width without cutting a syllable (${name})`, async () => {
    for (const [text, units] of MIXED) {
      const m = new Ime(composer(), segment, whole)
      m.insertText(text)
      expect(await m.convert()).toBe(true)
      assertWhole(m, text)
      const focus = m.focus
      const from = chars(m.segs.slice(0, focus).map((s) => s.reading).join('')).length
      const span = units.slice(from)
      while (m.segs.length > focus + 1) {
        await m.resize(1)
        assertWhole(m, text)
      }
      expect(m.segs[focus].reading).toBe(span.join(''))
      for (let w = span.length - 1; w >= 1; w--) {
        await m.resize(-1)
        assertWhole(m, text)
        expect(m.segs[focus].reading).toBe(span.slice(0, w).join(''))
      }
      await m.resize(-1)
      expect(m.segs[focus].reading).toBe(span[0])
      for (let w = 2; w <= span.length; w++) {
        await m.resize(1)
        assertWhole(m, text)
        expect(m.segs[focus].reading).toBe(span.slice(0, w).join(''))
      }
      m.key({ key: 'Enter' })
      expect(m.doc).toBe(text)
      expect(m.cursor).toBe(units.length)
    }
  })
}

test('an old-only reading converts, takes the focus and stays after a pick', async () => {
  const m = new Ime(composer(), byWord, whole)
  m.insertText(`1 ${HOLL}`)
  expect(await m.convert()).toBe(true)
  expect(m.current.reading).toBe(HOLL)

  const n = new Ime(composer(), async (t) => t.split(/( )/).filter(Boolean).map((w) => ({ reading: w, candidates: [...one(w), ...one(`${w}!`)] })), whole)
  n.insertText(`가 ${HOLL}`)
  await n.convert()
  n.key({ key: 'ArrowDown' })
  n.key({ key: '1' })
  expect(n.doc).toBe('가')
  expect(n.segs?.map((s) => s.reading)).toEqual([' ', HOLL])
  expect(n.pre).toBe(` ${HOLL}`)
})

test('backspace and delete take a whole syllable from the text', () => {
  const m = new Ime(composer(), byWord, whole)
  m.setDoc(`追加${HOLL}까요`)
  m.key({ key: 'Backspace' })
  m.key({ key: 'Backspace' })
  expect(m.doc).toBe(`追加${HOLL}`)
  m.key({ key: 'Backspace' })
  expect(m.doc).toBe('追加')
  expect(m.cursor).toBe(2)

  m.setDoc(`나${SSAM}미`)
  m.setCursor(1)
  m.key({ key: 'Delete' })
  expect(m.doc).toBe('나미')
  expect(m.cursor).toBe(1)
})

test('backspace in a conversion returns to the same text', async () => {
  const m = new Ime(composer(), byWord, whole)
  const text = `追加${HOLL}까요`
  m.insertText(text)
  await m.convert()
  await m.resize(-1)
  m.key({ key: 'Backspace' })
  await tick()
  expect(m.segs).toBe(null)
  expect(m.pre).toBe(text)
})

test('the caret and the selection move by whole syllables', () => {
  const m = new Ime(composer(), byWord, whole)
  m.setDoc(`가${HOLL}나`)
  expect(m.cursor).toBe(3)
  m.key({ key: 'ArrowLeft' })
  expect(m.after).toBe('나')
  m.key({ key: 'ArrowLeft' })
  expect(m.before).toBe('가')
  expect(m.after).toBe(`${HOLL}나`)
  m.key({ key: 'ArrowRight', shift: true })
  expect(m.selectedText).toBe(HOLL)
  m.key({ key: 'Backspace' })
  expect(m.doc).toBe('가나')
})

test('a letter that joins the character before it does not overrun the caret', () => {
  const m = new Ime(composer(), byWord, whole)
  m.insertText('ᄒ')
  m.key({ key: 'Enter' })
  m.insertText('ᆞ')
  m.key({ key: 'Enter' })
  expect(m.doc).toBe(SA.replace('ᄉ', 'ᄒ'))
  expect(m.cursor).toBe(1)
  m.key({ key: 'Backspace' })
  expect(m.doc).toBe('')
  expect(m.cursor).toBe(0)
})

test('ruby leaves a trailing old syllable whole', () => {
  expect(rubify(`國${HOLL}`, `나라${HOLL}`)).toEqual({ base: '國', rt: '나라', tail: HOLL })
  expect(rubify(`國${HOLL}`, `나라ᄉᆞᆯ`)).toEqual({ base: `國${HOLL}`, rt: '나라ᄉᆞᆯ', tail: '' })
  expect(`앞 國${HOLL} 뒤`.split(WORD)).toEqual(['앞 ', `國${HOLL}`, ' 뒤'])
})
