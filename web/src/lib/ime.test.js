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

test('holding enter on a candidate does not go on to break the line', async () => {
  const m = ime()
  m.insertText('하늘을')
  await m.convert()
  m.key({ key: 'Enter' })
  m.key({ key: 'Enter', repeat: true })
  m.key({ key: 'Enter', repeat: true })
  expect(m.doc).toBe('天을')
  // Released and pressed again, or held with nothing to commit, it breaks the line.
  m.key({ key: 'Enter' })
  m.key({ key: 'Enter', repeat: true })
  expect(m.doc).toBe('天을\n\n')
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

test('a resized word keeps its full candidate list', async () => {
  // Whole-phrase conversion stops after one candidate, as the engine's does.
  const m = new Ime(composer(), segment, async (text) => cands(text).slice(0, 1))
  m.insertText('하늘')
  await m.convert()
  m.key({ key: 'ArrowLeft', shift: true })
  await tick()
  expect(m.segs[0].candidates.map((c) => c.text)).toEqual(['夏', '下', '하'])
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

test('韓㐎 專用 turned on while a conversion is in flight leaves the Hangul as typed', async () => {
  let release
  const slow = (t) => new Promise((r) => (release = () => r(segment(t))))
  const m = new Ime(composer(), slow, whole)
  m.insertText('하늘')
  m.key({ key: ' ' })
  expect(m.busy).toBe(true)
  m.setHangulOnly(true)
  expect(m.doc).toBe('하늘')
  release()
  await tick()
  await tick()
  expect(m.segs).toBe(null)
  expect(m.doc).toBe('하늘')
  expect(m.busy).toBe(false)
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

test('the 한/영 key switches mode and confirms the composition', async () => {
  const m = ime()
  m.insertText('하늘')
  m.key({ key: 'HangulMode' })
  expect(m.mode).toBe('ascii')
  expect(m.doc).toBe('하늘')
  m.key({ key: 'g' })
  expect(m.doc).toBe('하늘g')
  m.toggleMode()
  m.key({ key: 'g' })
  expect(m.pre).toBe('ㅎ')
})

test('selection: shift+arrows select, typing replaces, backspace deletes', async () => {
  const m = ime()
  m.setDoc('ABCD')
  m.key({ key: 'ArrowLeft', shift: true })
  m.key({ key: 'ArrowLeft', shift: true })
  expect(m.selection).toEqual([2, 4])
  expect(m.selectedText).toBe('CD')
  m.key({ key: '1' })
  expect(m.doc).toBe('AB1')
  m.selectAll()
  m.key({ key: 'Backspace' })
  expect(m.doc).toBe('')
  m.setDoc('ABCD')
  m.setSelection(1, 3)
  m.insertText('하늘')
  await m.convert()
  m.key({ key: 'Enter' })
  expect(m.doc).toBe('A天D')
  m.setSelection(0, 2)
  m.key({ key: 'ArrowRight' })
  expect(m.selection).toBe(null)
  expect(m.cursor).toBe(2)
})

test('horizontal layout: ← → walk the open candidate list', async () => {
  const m = ime()
  m.layout = 'horizontal'
  m.insertText('하늘')
  await m.convert()
  m.key({ key: ' ' })
  m.key({ key: 'ArrowRight' })
  expect(m.converted).toBe('하늘')
  m.key({ key: 'ArrowLeft' })
  expect(m.converted).toBe('乾')
})

test('undo and redo restore text, caret and conversions', async () => {
  const m = ime()
  m.insertText('하늘')
  await m.convert()
  m.key({ key: 'Enter' })
  m.key({ key: '1' })
  m.key({ key: '2' })
  expect(m.doc).toBe('天12')
  expect(m.undo()).toBe(true)
  expect(m.doc).toBe('天')
  expect(m.undo()).toBe(true)
  expect(m.doc).toBe('')
  expect(m.undo()).toBe(false)
  m.redo()
  expect(m.doc).toBe('天')
  m.redo()
  expect(m.doc).toBe('天12')
  m.setSelection(0, 1)
  m.key({ key: 'Backspace' })
  expect(m.doc).toBe('12')
  m.undo()
  expect(m.doc).toBe('天12')
  m.insertText('하')
  expect(m.undo()).toBe(true)
  expect(m.pre).toBe('')
  expect(m.doc).toBe('天12')
})

test('the caret steps over an old-Hangul syllable as one character', () => {
  const m = ime()
  m.setDoc('ᄒᆞ여') // ᄒᆞ여
  expect(m.cursor).toBe(2)
  m.key({ key: 'ArrowLeft' })
  m.key({ key: 'ArrowLeft' })
  expect(m.cursor).toBe(0)
  m.key({ key: 'Delete' })
  expect(m.doc).toBe('여')
})

test('韓㐎 專用: Space commits the Hangul and types a space, with no conversion', async () => {
  const m = ime()
  m.setHangulOnly(true)
  for (const k of 'gks') m.key({ key: k })
  expect(m.pre).toBe('ㅎㅏㄴ')
  m.key({ key: ' ' })
  await tick()
  expect(m.doc).toBe('ㅎㅏㄴ ')
  expect(m.pre).toBe('')
  expect(m.segs).toBe(null)
  expect(m.open).toBe(false)
  // Shift+Space makes no difference.
  for (const k of 'gks') m.key({ key: k })
  m.key({ key: ' ', shift: true })
  expect(m.doc).toBe('ㅎㅏㄴ ㅎㅏㄴ ')
})

test('韓㐎 專用: Enter commits, Tab and ↓ do nothing, punctuation commits then types', async () => {
  const m = ime()
  m.setHangulOnly(true)
  for (const k of 'gks') m.key({ key: k })
  for (const key of ['Tab', 'ArrowDown', 'Convert']) {
    expect(m.key({ key })).toBe(true)
    await tick()
    expect(m.segs).toBe(null)
    expect(m.pre).toBe('ㅎㅏㄴ')
  }
  m.key({ key: 'Enter' })
  expect(m.doc).toBe('ㅎㅏㄴ')
  for (const k of 'gks') m.key({ key: k })
  m.key({ key: '.' })
  expect(m.doc).toBe('ㅎㅏㄴㅎㅏㄴ.')
  expect(m.pre).toBe('')
  expect(await m.convert()).toBe(false)
})

test('韓㐎 專用 confirms a running conversion when it is switched on; off again converts', async () => {
  const m = ime()
  m.insertText('하늘')
  await m.convert()
  expect(m.segs).not.toBe(null)
  m.setHangulOnly(true)
  expect(m.doc).toBe('天')
  expect(m.segs).toBe(null)
  m.setHangulOnly(false)
  m.insertText('하늘')
  m.key({ key: ' ' })
  await tick()
  expect(m.converted).toBe('天')
})

// ---------------------------------------------------------------------------
// Every keyboard, through the real composer (`bun run wasm` builds it).
//
// A recording is the events a browser dispatches for one keystroke, played on a
// model of the textarea. The browser edits the text between beforeinput and
// input, as Input Events specifies. Provenance of the shapes:
//   spec      Input Events 2 (composition update order, cancelability),
//             UI Events (compositionstart/update/end, isComposing)
//   browsers  long-known engine differences: Firefox's extra input after
//             compositionend, WebKit's deleteCompositionText +
//             insertFromComposition, keyCode 229 on Android Chrome
//   inferred  no device here; from the iOS bug report (separate jamo, a doubled
//             ㅇ), and from how Gboard and Samsung use setComposingText. These
//             need checking on real phones.
// ---------------------------------------------------------------------------

const wasm = await import('../../pkg/seokdok.js')
wasm.initSync({ module: await Bun.file(new URL('../../pkg/seokdok_bg.wasm', import.meta.url)).arrayBuffer() })
const { Sink } = await import('./sink.js')
const { chars } = await import('./chars.js')

const dictionary = {
  안녕: ['安寧', '안녕'],
  아녀: ['아녀'],
  한글: ['韓㐎', '한글'],
  하늘: ['天', '하늘'],
}
const realSegment = async (text) =>
  text
    .split(/( )/)
    .filter(Boolean)
    .map((w) => ({ reading: w, candidates: (dictionary[w] ?? [w]).map((t) => ({ text: t, note: '' })) }))
const realWhole = async (text) => (await realSegment(text)).flatMap((s) => s.candidates)
const realIme = () => new Ime(new wasm.Composer(), realSegment, realWhole)

/** Physical 두벌식 keys on a fresh input method: what everything must match. */
function physical(keys) {
  const m = realIme()
  for (const key of keys) m.key({ key })
  return m.pre
}

/** The textarea as the browser keeps it, with the handlers ImePanel wires up. */
function page() {
  const ime = realIme()
  const el = { value: '', setSelectionRange() {} }
  const sink = new Sink(ime, el)
  let region = 0
  let prevented = false

  const edits = {
    insertText: (d) => (el.value += d),
    insertLineBreak: () => (el.value += '\n'),
    insertCompositionText: (d) => (el.value = el.value.slice(0, region) + d),
    deleteCompositionText: () => (el.value = el.value.slice(0, region)),
    insertFromComposition: (d) => (el.value = el.value.slice(0, region) + d),
    deleteContentBackward: () => (el.value = [...el.value].slice(0, -1).join('')),
    insertReplacementText: (d, n) => (el.value = el.value.slice(0, el.value.length - n) + d),
    insertDictationResult: (d) => (el.value += d),
  }

  const dispatch = {
    keydown(key, keyCode = 0) {
      prevented = false
      if (!sink.usable({ key, keyCode })) return
      if (ime.key({ key })) {
        prevented = true
        sink.reset()
      }
    },
    beforeinput() {},
    input(type, data, n) {
      if (prevented) return
      // Nothing to delete, nothing changes, and no input event follows.
      if (type === 'deleteContentBackward' && !el.value) return
      edits[type](data, n)
      sink.input({ inputType: type })
    },
    compositionstart(back = 0) {
      region = el.value.length - back
      sink.compositionstart()
    },
    compositionupdate() {},
    compositionend() {
      sink.compositionend()
    },
    keyup() {},
  }
  return {
    ime,
    el,
    sink,
    play(events) {
      for (const [name, ...args] of events) dispatch[name](...args)
    },
  }
}

// --- One keystroke, by keyboard ---------------------------------------------

/** A key press the page can read: the browser's default is prevented. */
const hardKey = (key) => [['keydown', key, key.length === 1 ? key.toUpperCase().charCodeAt(0) : 8], ['keyup']]

/** A keyboard that names no key and inserts text. */
const inserted = (c) => [
  ['keydown', 'Unidentified', 229],
  ['beforeinput', 'insertText', c],
  ['input', 'insertText', c],
  ['keyup'],
]
/** The same with no beforeinput (engines and keyboards that skip it). */
const insertedBare = (c) => [['keydown', 'Unidentified', 229], ['input', 'insertText', c]]
const deleted = () => [
  ['keydown', 'Unidentified', 229],
  ['beforeinput', 'deleteContentBackward', null],
  ['input', 'deleteContentBackward', null],
]
/** iOS replacing what it holds: delete, then the syllable as it now stands. */
const reinserted = (text, n = 1) => [...Array(n).fill(0).flatMap(deleted), ...[...text].flatMap((c) => inserted(c).slice(1, 3))]
const update = (text) => [
  ['keydown', 'Process', 229],
  ['compositionupdate', text],
  ['beforeinput', 'insertCompositionText', text],
  ['input', 'insertCompositionText', text],
]

/** How each engine ends a composition. */
const ends = {
  chrome: (text) => [['compositionend', text]],
  firefox: (text) => [
    ['compositionend', text],
    ['input', 'insertCompositionText', text],
  ],
  safari: (text) => [
    ['input', 'deleteCompositionText', null],
    ['input', 'insertFromComposition', text],
    ['compositionend', text],
  ],
}

/** One composition through to its end. */
const composition = (steps, browser = 'chrome', back = 0) => [
  ['compositionstart', back],
  ...steps.flatMap(update),
  ...ends[browser](steps.at(-1)),
]

async function space(p) {
  p.play(inserted(' '))
  await tick()
}

async function expectConverts(p, reading, text) {
  await space(p)
  expect(p.ime.segs?.map((s) => s.reading).join('')).toBe(reading)
  expect(p.ime.converted).toBe(text)
  p.play(inserted(' '))
  expect(p.ime.open).toBe(true)
  p.play(inserted('\n'))
  expect(p.ime.segs).toBe(null)
}

// --- Latin keyboards --------------------------------------------------------

test('the composer agrees with physical keys', () => {
  expect(physical('dkssud')).toBe('안녕')
  expect(physical('gKs')).toBe('\u1112\u119E\u11AB')
})

test('Latin, real key events (desktop; iOS English): keys, then Space converts', async () => {
  const p = page()
  p.play([...'dkssud'].flatMap(hardKey))
  expect(p.ime.pre).toBe('안녕')
  p.play(hardKey(' '))
  await tick()
  expect(p.ime.converted).toBe('安寧')
  p.play(hardKey(' '))
  expect(p.ime.open).toBe(true)
  p.play(hardKey('1'))
  expect(p.ime.doc).toBe('安寧')
})

test.each([
  ['with beforeinput', inserted],
  ['without', insertedBare],
])('Latin, input only (%s): letters map through 두벌식', async (_, send) => {
  const p = page()
  p.play([...'dkssud'].flatMap(send))
  expect(p.ime.pre).toBe(physical('dkssud'))
  expect(p.el.value).toBe('\u200B') // Nothing for the keyboard's autocorrect to work on.
  await expectConverts(p, '안녕', '安寧')
})

test.each(['chrome', 'firefox', 'safari'])('Gboard Latin word composition on %s: same as the keys', async (browser) => {
  const p = page()
  p.play(composition(['d', 'dk', 'dks', 'dkss', 'dkssu', 'dkssud'], browser))
  expect(p.ime.pre).toBe('안녕')
  await expectConverts(p, '안녕', '安寧')
  expect(p.ime.doc).toBe('안녕') // The window opened on the next candidate.
})

test('Gboard Latin: shown while composing, following backspace, then the space', async () => {
  const p = page()
  p.play([['compositionstart'], ...update('d'), ...update('dk')])
  expect(p.ime.pre).toBe('아')
  p.play(update('dks'))
  expect(p.ime.pre).toBe('안')
  p.play(update('dk'))
  expect(p.ime.pre).toBe('아')
  p.play(update('dks'))
  p.play([
    ['compositionend', 'dks'],
    ['input', 'insertText', ' '],
  ])
  await tick()
  expect(p.ime.converted).toBe('안')
})

test('Latin autocorrect replacing the letters leaves the jamo they typed', () => {
  const p = page()
  p.play(composition(['d', 'dk', 'dks']))
  p.play([
    ['beforeinput', 'insertReplacementText', 'desk'],
    ['input', 'insertReplacementText', 'desk', 3],
  ])
  expect(p.ime.pre).toBe('안')
  expect(p.el.value).toBe('\u200Bdks')
  p.play(inserted('u'))
  expect(p.ime.pre).toBe('아녀')
})

test('Latin autocorrect with the space that triggered it still converts', async () => {
  const p = page()
  p.play(composition(['d', 'dk', 'dks', 'dkss', 'dkssu', 'dkssud']))
  p.play([['input', 'insertReplacementText', 'disused ', 6]])
  await tick()
  expect(p.ime.converted).toBe('安寧')
})

test('Shift through input gives the old letter, as on the keys', () => {
  const p = page()
  p.play([...'gKs'].flatMap(inserted))
  expect(p.ime.pre).toBe(physical('gKs'))
})

test('a word swiped on a Latin keyboard passes through', () => {
  const p = page()
  p.play(composition(['hello']))
  expect(p.ime.pre).toBe('')
  expect(p.ime.doc).toBe('hello')
  expect(p.el.value).toBe('\u200B')
})

// --- iOS: compatibility jamo, one at a time ---------------------------------

test.each([
  ['with beforeinput', inserted],
  ['without', insertedBare],
])('iOS jamo (%s) compose: the reported 안녕', async (_, send) => {
  const p = page()
  p.play([...'ㅇㅏㄴㄴㅕㅇ'].flatMap(send))
  expect(p.ime.pre).toBe('안녕')
  await expectConverts(p, '안녕', '安寧')
  expect(p.ime.doc).toBe('안녕') // The window opened on the next candidate.
})

test('iOS names the jamo in keydown: it still takes the input path', () => {
  const p = page()
  p.play(
    [...'ㅇㅏㄴ'].flatMap((c) => [
      ['keydown', c, 0],
      ['input', 'insertText', c],
    ]),
  )
  expect(p.ime.pre).toBe('안')
})

test('iOS keeps its context: the textarea holds what it typed', () => {
  const p = page()
  p.play([...'ㅇㅏ'].flatMap(inserted))
  expect(p.el.value).toBe('\u200Bㅇㅏ')
})

test('iOS delete-then-reinsert composing by itself: nothing doubled', async () => {
  const p = page()
  p.play(inserted('ㅇ'))
  p.play(reinserted('아'))
  p.play(reinserted('안'))
  p.play(inserted('ㄴ'))
  p.play(reinserted('녀'))
  p.play(reinserted('녕'))
  expect(p.ime.pre).toBe('안녕')
  await expectConverts(p, '안녕', '安寧')
})

test('iOS delete-then-reinsert taking the final to the next syllable', () => {
  const p = page()
  p.play(inserted('ㅇ'))
  p.play(reinserted('아'))
  p.play(reinserted('안'))
  p.play(reinserted('아녀'))
  expect(p.ime.pre).toBe('아녀')
})

test('iOS jamo: a jamo after a finished syllable moves its final, as on the keys', () => {
  const p = page()
  p.play(inserted('안'))
  p.play(inserted('ㅕ'))
  expect(p.ime.pre).toBe(physical('dksu'))
})

test('iOS backspace removes letters, then earlier text, one press at a time', () => {
  const p = page()
  p.ime.setDoc('天')
  p.sink.reset()
  p.play([...'ㅇㅏㄴ'].flatMap(inserted))
  p.play(deleted())
  expect(p.ime.pre).toBe('아')
  p.play(deleted())
  expect(p.ime.pre).toBe('ㅇ')
  p.play(deleted())
  expect(p.ime.pre).toBe('')
  // The textarea holds only its mark, so this press still fires input.
  p.play(deleted())
  expect(p.ime.doc).toBe('')
  expect(p.el.value).toBe('\u200B')
  p.play(deleted())
  expect(p.el.value).toBe('\u200B')
})

test('iOS backspace with a real keydown leaves the deleting to the browser', () => {
  const p = page()
  p.play([...'ㅇㅏㄴ'].flatMap(inserted))
  expect(p.sink.usable({ key: 'Backspace', keyCode: 8 })).toBe(false)
  p.play([
    ['keydown', 'Backspace', 8],
    ['input', 'deleteContentBackward', null],
  ])
  expect(p.ime.pre).toBe('아')
})

test('backspace past the keyboard text reaches text it did not type', () => {
  const p = page()
  p.ime.insertText('하늘')
  p.sink.reset()
  p.play(deleted())
  expect(p.ime.pre).toBe('하느')
  p.play(inserted('ㄹ'))
  expect(p.ime.pre).toBe('하늘')
})

test('iOS: candidates work from the keyboard: Space cycles, a digit picks, typing goes on', async () => {
  const p = page()
  p.play([...'ㅎㅏㄴㄱㅡㄹ'].flatMap(inserted))
  await space(p)
  expect(p.ime.converted).toBe('韓㐎')
  p.play(inserted(' '))
  expect(p.ime.converted).toBe('한글')
  p.play(inserted('1'))
  expect(p.ime.doc).toBe('韓㐎')
  p.play([...'ㅎㅏ'].flatMap(inserted))
  expect(p.ime.pre).toBe('하')
})

test('iOS: backspace in a conversion goes back to the Hangul', async () => {
  const p = page()
  p.play([...'ㅇㅏㄴ'].flatMap(inserted))
  await space(p)
  expect(p.ime.segs).not.toBe(null)
  p.play(deleted())
  expect(p.ime.segs).toBe(null)
  expect(p.ime.pre).toBe('안')
})

test('iOS: typing on after a Space conversion puts the word break in', async () => {
  const p = page()
  p.play([...'ㅎㅏㄴㄱㅡㄹ'].flatMap(inserted))
  await space(p)
  p.play(inserted('ㅎ'))
  expect(p.ime.doc).toBe('韓㐎 ')
  expect(p.ime.pre).toBe('ㅎ')
  p.play(inserted('ㅏ'))
  expect(p.ime.doc).toBe('韓㐎 ')
  expect(p.ime.pre).toBe('하')
})

// --- Keyboards that compose by themselves -----------------------------------

test.each(['chrome', 'firefox', 'safari'])('a syllable at a time, each ended (Windows, macOS IME) on %s', async (browser) => {
  const p = page()
  p.play(composition(['ㅇ', '아', '안'], browser))
  expect(p.ime.pre).toBe('안')
  p.play(composition(['ㄴ', '녀', '녕'], browser))
  expect(p.ime.pre).toBe('안녕')
  await expectConverts(p, '안녕', '安寧')
  expect(p.ime.doc).toBe('안녕') // The window opened on the next candidate.
})

test.each(['chrome', 'firefox', 'safari'])('Windows IME moving the final out on %s', (browser) => {
  const p = page()
  p.play([['compositionstart'], ...update('ㅇ'), ...update('아'), ...update('안'), ...update('아'), ...ends[browser]('아')])
  p.play(composition(['녀'], browser))
  expect(p.ime.pre).toBe('아녀')
})

test.each(['chrome', 'firefox', 'safari'])('Gboard: one region replaced whole, final moving inside it, on %s', async (browser) => {
  const p = page()
  p.play(composition(['ㅇ', '아', '안', '아녀', '아녕'], browser))
  expect(p.ime.pre).toBe('아녕')
  await space(p)
  expect(p.ime.segs?.[0].reading).toBe('아녕')
})

test('Gboard: the shown text follows each update, nothing doubled', () => {
  const p = page()
  p.play([['compositionstart']])
  for (const [sent, shown] of [
    ['ㅇ', 'ㅇ'],
    ['아', '아'],
    ['안', '안'],
    ['안ㄴ', '안ㄴ'],
    ['아녀', '아녀'],
    ['아녕', '아녕'],
  ]) {
    p.play(update(sent))
    expect(p.ime.pre).toBe(shown)
  }
  p.play(ends.chrome('아녕'))
  expect(p.ime.pre).toBe('아녕')
  expect(p.ime.doc).toBe('')
})

test('Samsung: the keyboard reopens the syllable it just ended and replaces it', () => {
  const p = page()
  p.play(composition(['ㅇ', '아', '안']))
  // setComposingRegion over 안, then the two syllables in its place.
  p.play(composition(['아녀'], 'chrome', 1))
  expect(p.ime.pre).toBe('아녀')
  p.play(composition(['아녕'], 'chrome', 2))
  expect(p.ime.pre).toBe('아녕')
})

test('a keyboard that ends the syllable, deletes it and starts again', () => {
  const p = page()
  p.play(composition(['ㅇ', '아', '안']))
  p.play(deleted())
  p.play(composition(['아', '아녀']))
  expect(p.ime.pre).toBe('아녀')
})

test('a composition that shrinks away leaves nothing, and backspace goes on', () => {
  const p = page()
  p.ime.setDoc('天')
  p.sink.reset()
  p.play([['compositionstart'], ...update('ㅇ'), ...update(''), ['compositionend', '']])
  expect(p.ime.pre).toBe('')
  p.play(deleted())
  expect(p.ime.doc).toBe('')
})

test('backspace inside a composition stays with the keyboard', () => {
  const p = page()
  p.play([['compositionstart'], ...update('안')])
  p.play([['keydown', 'Backspace', 229], ...update('아')])
  expect(p.ime.pre).toBe('아')
  p.play(ends.chrome('아'))
  expect(p.ime.pre).toBe('아')
})

test('Space after a composed word ends it and converts, with no leftover in the textarea', async () => {
  const p = page()
  p.play(composition(['ㅎ', '하', '한']))
  p.play(composition(['ㄱ', '그', '글']))
  await space(p)
  expect(p.ime.converted).toBe('韓㐎')
  expect(p.el.value).toBe('\u200B')
})

test('a real Space key after an ended composition converts', async () => {
  const p = page()
  p.play(composition(['ㅎ', '하', '한']))
  p.play(composition(['ㄱ', '그', '글']))
  p.play(hardKey(' '))
  await tick()
  expect(p.ime.converted).toBe('韓㐎')
  expect(p.el.value).toBe('\u200B')
})

test('a real Enter key commits what the keyboard composed', () => {
  const p = page()
  p.play(composition(['ㅎ', '하', '한']))
  p.play(hardKey('Enter'))
  expect(p.ime.doc).toBe('한')
  expect(p.ime.pre).toBe('')
})

test('Enter from a keyboard with no key events commits, and then breaks the line', () => {
  const p = page()
  p.play([...'ㅇㅏ'].flatMap(inserted))
  p.play(inserted('\n'))
  expect(p.ime.doc).toBe('아')
  p.play(inserted('\n'))
  expect(p.ime.doc).toBe('아\n')
})

test('a composition continues the preedit already there', () => {
  const p = page()
  p.ime.insertText('하')
  p.sink.reset()
  p.play(composition(['ㄴ', '느', '늘']))
  expect(p.ime.pre).toBe('하늘')
})

test('an open conversion is confirmed by the next composition, with the word break', async () => {
  const p = page()
  p.play(composition(['ㅎ', '하']))
  p.play(composition(['ㄴ', '느', '늘']))
  await space(p)
  expect(p.ime.converted).toBe('天')
  p.play([['compositionstart'], ...update('ㅎ')])
  expect(p.ime.doc).toBe('天 ')
  expect(p.ime.pre).toBe('ㅎ')
  p.play(ends.chrome('ㅎ'))
  expect(p.ime.pre).toBe('ㅎ')
})

// --- Old Hangul --------------------------------------------------------------

const HA = '\u1112\u119E' // ᄒᆞ
const HAN = '\u1112\u119E\u11AB' // ᄒᆞᆫ
const noPua = (s) => expect(/[-]/.test(s)).toBe(false)

test.each([
  ['ㆍ', 'ㄱㆍ', 'rK'],
  ['ㅿ', 'ㅿㅏ', 'Ak'],
  ['ㆁ', 'ㆁㅏ', 'Dk'],
  ['ㆆ', 'ㆆㅏ', 'Gk'],
])('the old letter %s composes like its Shift key', (_, sent, keys) => {
  const p = page()
  p.play([...sent].flatMap(inserted))
  expect(p.ime.pre).toBe(physical(keys))
  expect(chars(p.ime.pre).length).toBe(1)
  noPua(p.ime.pre)
})

test('compatibility old letters one at a time, final included, are one unit', async () => {
  const p = page()
  p.play([...'ㅎㆍㄴ'].flatMap(inserted))
  expect(p.ime.pre).toBe(HAN)
  expect(chars(p.ime.pre).length).toBe(1)
  await space(p)
  expect(p.ime.segs).not.toBe(null)
})

test('compatibility old letters pasted are one unit per syllable', () => {
  const p = page()
  p.ime.insertText('ㅎㆍㄴ ㅅㅏㄹㆍㅁ')
  expect(chars(p.ime.pre)[0]).toBe(HAN)
  expect(chars(p.ime.pre)[1]).toBe(' ')
  noPua(p.ime.pre)
})

test('the 첫가끝 keyboard: conjoining jamo one at a time stay one unit', () => {
  const p = page()
  p.play([...HAN].flatMap(inserted))
  expect(p.ime.pre).toBe(HAN)
  expect(chars(p.ime.pre).length).toBe(1)
  noPua(p.ime.pre)
  p.play(deleted())
  expect(p.ime.pre).toBe(HA)
  // A key goes on from there.
  p.play(inserted('s'))
  expect(p.ime.pre).toBe(HAN)
})

test('the 첫가끝 keyboard: modern syllables come out precomposed', () => {
  const p = page()
  p.play([...'\u1112\u1161\u11AB'].flatMap(inserted))
  expect(p.ime.pre).toBe('한')
  p.play([...'\u1100\u1173\u11AF'].flatMap(inserted))
  expect(p.ime.pre).toBe('한글')
})

test('an old-Hangul keyboard composing in one region keeps each syllable whole', () => {
  const p = page()
  p.play(composition(['\u1112', HA, HAN, HAN + '\u1100', HAN + '\u1100\u119E']))
  expect(chars(p.ime.pre)).toEqual([HAN, '\u1100\u119E'])
  noPua(p.ime.pre)
})

test('Hangul Jamo Extended in a paste stays one unit', () => {
  for (const s of ['\uA960\u1161', '\u1100\uD7B0', '\u1100\u1161\uD7CB', '\uA97C\uD7C6\uD7FB']) {
    const p = page()
    p.ime.insertText(s)
    expect(p.ime.pre).toBe(s)
    expect(chars(p.ime.pre).length).toBe(1)
  }
})

test('Hangul Jamo Extended from a keyboard goes on as a syllable', () => {
  const p = page()
  p.play([...'\uA960\u1161'].flatMap(inserted))
  expect(p.ime.pre).toBe('\uA960\u1161')
  p.play(inserted('s'))
  expect(chars(p.ime.pre).length).toBe(1)
  noPua(p.ime.pre)
})

test('a decomposed paste (macOS) comes out as modern syllables', () => {
  const p = page()
  p.ime.insertText('안녕'.normalize('NFD'))
  expect(p.ime.pre).toBe('안녕')
})

test('a modern syllable with an old final is held in one form', () => {
  const p = page()
  p.ime.insertText('가\u11F0')
  expect(p.ime.pre).toBe(physical('rkD'))
})

test('pasted jamo with no old letter stay as they are', () => {
  const p = page()
  p.ime.insertText('하하ㅎㅎ ㅋㅋ')
  expect(p.ime.pre).toBe('하하ㅎㅎ ㅋㅋ')
})

// --- Finished text ----------------------------------------------------------

test('dictation: a finished run of Hangul waits to convert', async () => {
  const p = page()
  p.play([['input', 'insertDictationResult', '안녕']])
  expect(p.ime.pre).toBe('안녕')
  expect(p.ime.doc).toBe('')
  p.sink.now = () => 1e12 // dictation has stopped
  await expectConverts(p, '안녕', '安寧')
})

test('dictation: several words and punctuation, and the keyboard revising them', () => {
  const p = page()
  p.play([['input', 'insertDictationResult', '하늘 봤다.']])
  expect(p.ime.pre).toBe('하늘 봤다.')
  p.play([['input', 'insertReplacementText', '하늘을 봤다.', 6]])
  expect(p.ime.pre).toBe('하늘을 봤다.')
})

test('dictation rewriting its text is not doubled, and Space converts afterwards', async () => {
  const p = page()
  let now = 0
  p.sink.now = () => now
  // iOS sends the sentence in runs, ending some in a space, and revises them.
  p.play([['input', 'insertDictationResult', '안녕하세요 ']])
  now += 300
  p.play([['input', 'insertDictationResult', '음성으로 ']])
  now += 300
  p.play([['input', 'insertDictationResult', '.']])
  now += 300
  p.play([['input', 'insertReplacementText', '안녕하세요. 음성으로 입력해요. ', 12]])
  expect(p.ime.doc).toBe('')
  expect(p.ime.pre).toBe('안녕하세요. 음성으로 입력해요. ')
  // Once it has stopped, Space is Space again.
  now += 2000
  await space(p)
  expect(p.ime.segs).not.toBe(null)
})

test('Korean autocomplete replacing a word follows the keyboard', () => {
  const p = page()
  p.play([...'ㅇㅏㄴ'].flatMap(inserted))
  p.play([
    ['beforeinput', 'insertReplacementText', '안녕'],
    ['input', 'insertReplacementText', '안녕', 3],
  ])
  expect(p.ime.pre).toBe('안녕')
})

test('emoji passes through, into the preedit when one is open', () => {
  const p = page()
  p.play(inserted('😀'))
  expect(p.ime.doc).toBe('😀')
  expect(p.ime.pre).toBe('')
  p.play([...'ㅎㅏ'].flatMap(inserted))
  p.play(inserted('😀'))
  expect(p.ime.pre).toBe('하😀')
})

test('Latin text sent whole passes through, Latin keys after it still map', () => {
  const p = page()
  p.play(inserted('hello'))
  expect(p.ime.doc).toBe('hello')
  p.play(inserted('r'))
  p.play(inserted('k'))
  expect(p.ime.pre).toBe('가')
})

test('digits and punctuation from a keyboard with no key events act like keys', () => {
  const p = page()
  p.play(inserted('1'))
  expect(p.ime.doc).toBe('1')
  p.play([...'ㅎㅏ'].flatMap(inserted))
  p.play(inserted('.'))
  expect(p.ime.pre).toBe('하.')
  p.play(inserted('é'))
  expect(p.ime.pre).toBe('하.é')
})

test('paste: Hangul waits to convert, other text goes in', () => {
  const p = page()
  p.ime.insertText('hello ')
  p.sink.reset()
  expect(p.ime.doc).toBe('hello ')
  expect(p.ime.pre).toBe('')
  p.ime.insertText('하늘')
  p.sink.reset()
  expect(p.ime.pre).toBe('하늘')
})

test('a letter named in keydown and typed by input goes in once', () => {
  const p = page()
  p.play([
    ['keydown', 'é', 0],
    ['input', 'insertText', 'é'],
  ])
  expect(p.ime.doc).toBe('é')
})

// --- 韓 / A and the rest of the input method ---------------------------------

test('A mode: Latin stays Latin, typed or composed', () => {
  const p = page()
  p.ime.toggleMode()
  p.play([...'hi'].flatMap(inserted))
  expect(p.ime.doc).toBe('hi')
  p.play([['compositionstart'], ...update('w'), ...update('wo')])
  expect(p.ime.pre).toBe('wo')
  p.play(ends.chrome('wo'))
  expect(p.ime.doc).toBe('hiwo')
  expect(p.ime.pre).toBe('')
  p.play(hardKey('x'))
  expect(p.ime.doc).toBe('hiwox')
})

test.each(['chrome', 'firefox', 'safari'])('A mode: a Korean keyboard still composes and converts, on %s', async (browser) => {
  const p = page()
  p.ime.toggleMode()
  p.play(composition(['ㅇ', '아', '안'], browser))
  p.play(composition(['ㄴ', '녀', '녕'], browser))
  expect(p.ime.pre).toBe('안녕')
  await expectConverts(p, '안녕', '安寧')
})

test('A mode: iOS jamo compose too', () => {
  const p = page()
  p.ime.toggleMode()
  p.play([...'ㅇㅏㄴ'].flatMap(inserted))
  expect(p.ime.pre).toBe('안')
})

test('switching mode confirms what the keyboard holds', () => {
  const p = page()
  p.play([...'ㅇㅏㄴ'].flatMap(inserted))
  p.ime.toggleMode()
  expect(p.ime.doc).toBe('안')
  p.play(inserted('x'))
  expect(p.ime.doc).toBe('안x')
  expect(p.ime.pre).toBe('')
})

test('韓㐎 專用: Space commits and types a space from any keyboard', async () => {
  const p = page()
  p.ime.setHangulOnly(true)
  p.play(composition(['ㅎ', '하', '한']))
  await space(p)
  expect(p.ime.doc).toBe('한 ')
  p.play([...'ㅎㅏ'].flatMap(inserted))
  await space(p)
  expect(p.ime.doc).toBe('한 하 ')
  expect(p.ime.segs).toBe(null)
})

test("spaceMode 'space': Space from the keyboard stays in the preedit", () => {
  const p = page()
  p.ime.spaceMode = 'space'
  p.play([...'ㅎㅏ'].flatMap(inserted))
  p.play(inserted(' '))
  expect(p.ime.pre).toBe('하 ')
  p.play([...'ㄴㅡㄹ'].flatMap(inserted))
  expect(p.ime.pre).toBe('하 늘')
})

test('the on-screen keyboard between two typed letters: what the keyboard held is spent', () => {
  const p = page()
  p.play([...'ㅎㅏ'].flatMap(inserted))
  p.ime.key({ key: 's' })
  expect(p.ime.pre).toBe('한')
  p.play(inserted('ㄱ'))
  expect(p.ime.pre).toBe('한ㄱ')
  expect(p.el.value).toBe('\u200Bㄱ')
})

test('a button that commits leaves no trace of the keyboard text', () => {
  const p = page()
  p.play([...'ㅎㅏ'].flatMap(inserted))
  p.ime.commit()
  p.play(inserted('ㄴ'))
  expect(p.ime.doc).toBe('하')
  expect(p.ime.pre).toBe('ㄴ')
  p.play(deleted())
  expect(p.ime.pre).toBe('')
  p.ime.setDoc('')
  p.play(deleted())
  expect(p.el.value).toBe('\u200B')
})

test('a button that commits, then a composition: the old text does not come back', () => {
  const p = page()
  p.play(composition(['ㅎ', '하', '한']))
  p.ime.commit()
  p.play([['compositionstart'], ...update('ㄱ'), ...update('그')])
  expect(p.ime.doc).toBe('한')
  expect(p.ime.pre).toBe('그')
  p.play(update('글'))
  expect(p.ime.pre).toBe('글')
  p.play(ends.chrome('글'))
  expect(p.ime.pre).toBe('글')
  expect(p.el.value).toBe('\u200B글')
})

/** A page whose conversion stays in flight until released. */
function slowPage() {
  let release
  const slow = (t) => new Promise((r) => (release = () => r(realSegment(t))))
  const ime = new Ime(new wasm.Composer(), slow, realWhole)
  const el = { value: '', setSelectionRange() {} }
  const sink = new Sink(ime, el)
  const type = (text) => {
    for (const c of text) {
      el.value += c
      sink.input()
    }
  }
  const done = async () => {
    release()
    await tick()
    await tick()
  }
  return { ime, el, sink, type, done }
}

test('input during a conversion in flight waits its turn', async () => {
  const p = slowPage()
  p.type('ㅇㅏㄴㄴㅕㅇ ㅎㅏ')
  expect(p.ime.busy).toBe(true)
  expect(p.ime.doc).toBe('')
  expect(p.el.value).toBe('​ㅎㅏ')
  await p.done()
  expect(p.ime.doc).toBe('安寧 ')
  expect(p.ime.pre).toBe('하')
})

test('Latin letters during a conversion stay keys, one at a time', async () => {
  const p = slowPage()
  p.type('dkssud ab')
  expect(p.el.value).toBe('​')
  await p.done()
  expect(p.ime.doc).toBe('安寧 ')
  expect(p.ime.pre).toBe(physical('ab'))
})

test('A mode: text during a conversion goes in once', async () => {
  const p = slowPage()
  p.ime.toggleMode()
  p.type('ㅇㅏㄴㄴㅕㅇ ab')
  await p.done()
  expect(p.ime.doc).toBe('安寧ab')
})

test('punctuation during a conversion goes in once', async () => {
  const p = slowPage()
  p.type('ㅇㅏㄴㄴㅕㅇ ..')
  await p.done()
  expect(p.ime.doc).toBe('安寧..')
})

test('a second word and a composition during a conversion: nothing lost', async () => {
  const p = slowPage()
  p.type('ㅇㅏㄴㄴㅕㅇ ')
  // The next word is ended, spaced, and another one is begun, all in flight.
  p.sink.compositionstart()
  p.el.value += '글'
  p.sink.input()
  p.sink.compositionend()
  p.type(' ')
  p.sink.compositionstart()
  p.el.value += '하'
  p.sink.input()
  p.el.value = p.el.value.slice(0, -1) + '한'
  p.sink.input()
  await p.done()
  await p.done()
  p.sink.compositionend()
  expect(p.ime.doc).toBe('安寧 글 ')
  expect(p.ime.pre).toBe('한')
})

test('backspace during a conversion keeps the mark, so the next press still fires', async () => {
  const p = slowPage()
  p.type('ㅇㅏㄴㄴㅕㅇ ')
  p.el.value = ''
  p.sink.input()
  expect(p.el.value).toBe('​')
  await p.done()
  expect(p.ime.segs).toBe(null)
  expect(p.ime.pre).toBe('안녕')
})

test('a composition confirmed from outside while still open is not doubled', () => {
  const p = page()
  p.play([['compositionstart'], ...update('하')])
  p.ime.toggleMode()
  expect(p.ime.doc).toBe('하')
  p.play(update('한'))
  expect(p.ime.doc).toBe('하')
  expect(p.ime.pre).toBe('')
  p.play(update('한ㄱ'))
  expect(p.ime.pre).toBe('ㄱ')
  p.play(ends.chrome('한ㄱ'))
  expect(p.ime.pre).toBe('ㄱ')
  expect(p.el.value).toBe('​ㄱ')
})

test('the caret is put back at the end, but not during a composition', () => {
  const calls = []
  const m = realIme()
  const el = { value: '', selectionStart: 0, selectionEnd: 0, setSelectionRange: (a) => calls.push(a) }
  const s = new Sink(m, el)
  el.selectionStart = el.selectionEnd = 0
  calls.length = 0
  s.caretToEnd()
  expect(calls).toEqual([1])
  s.compositionstart()
  s.caretToEnd()
  expect(calls).toEqual([1])
})

test('the textarea never empties, and the caret stays past the mark', () => {
  const p = page()
  const at = []
  p.el.setSelectionRange = (a) => at.push(a)
  p.play(deleted())
  expect(p.el.value).toBe('\u200B')
  expect(at.at(-1)).toBe(1)
})

test('text already in Hanja converts like Hangul, pasted or typed', async () => {
  const hanja = {
    '天을': ['天을', '乾을', '하늘을'],
    '良心': ['良心', '兩心', '양심'],
  }
  const menu = (w) => (hanja[w] ?? [w]).map((t) => ({ text: t, note: '' }))
  const m = new Ime(
    composer(),
    async (text) => text.split(/( )/).filter(Boolean).map((w) => ({ reading: w, candidates: menu(w) })),
    async (text) => menu(text),
  )
  m.insertText('天을 良心')
  expect(await m.convert()).toBe(true)
  expect(m.converted).toBe('天을 良心')
  expect(m.segs.filter((s) => s.candidates.length > 1)).toHaveLength(2)
  m.key({ key: ' ' })
  expect(m.converted).toBe('乾을 良心')
  m.key({ key: 'Enter' })
  expect(m.doc).toBe('乾을 良心')
})

test('pasted Hanja is left as it is in 韓㐎 專用', async () => {
  const m = ime()
  m.setHangulOnly(true)
  m.insertText('天을 良心')
  expect(await m.convert()).toBe(false)
  expect(m.segs).toBe(null)
  m.commit()
  expect(m.doc).toBe('天을 良心')
})
