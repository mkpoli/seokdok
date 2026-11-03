import { test, expect } from 'bun:test'

// A stand-in composer and segmenter, so the state machine runs without wasm.
// Runes are compiled by Svelte; under bun, $state is shimmed as identity.
globalThis.$state = (v) => v
const { Ime } = await import('./ime.svelte.js')

function composer() {
  let t = ''
  const map = { g: 'ㅎ', k: 'ㅏ', s: 'ㄴ', f: 'ㄹ', r: 'ㄱ', m: 'ㅡ', d: 'ㅇ' }
  return {
    input: (k) => (map[k] ? ((t += map[k]), true) : false),
    backspace: () => ((t = [...t].slice(0, -1).join('')), true),
    pushLiteral: (s) => (t += s),
    text: () => t,
    setText: (s) => (t = s),
    clear: () => (t = ''),
  }
}

const seg = async (text) => [
  { reading: text, candidates: [{ text: '天', note: '' }, { text: '乾', note: '' }, { text: text, note: '' }] },
]

test('space converts, cycles, enter commits', async () => {
  const ime = new Ime(composer(), seg)
  ime.insertText('하늘')
  expect(ime.pre).toBe('하늘')
  ime.key({ key: ' ' })
  await new Promise((r) => setTimeout(r))
  expect(ime.converted).toBe('天')
  ime.key({ key: ' ' })
  expect(ime.converted).toBe('乾')
  expect(ime.open).toBe(true)
  ime.key({ key: 'Enter' })
  expect(ime.doc).toBe('乾')
  expect(ime.pre).toBe('')
})

test('escape returns to hangul, digit picks and commits', async () => {
  const ime = new Ime(composer(), seg)
  ime.insertText('하늘')
  await ime.convert()
  ime.key({ key: 'Escape' })
  expect(ime.segs).toBe(null)
  expect(ime.pre).toBe('하늘')
  await ime.convert()
  ime.key({ key: 'ArrowDown' })
  ime.key({ key: '3' })
  expect(ime.doc).toBe('하늘')
})

test('typing while converted commits and continues', async () => {
  const ime = new Ime(composer(), seg)
  ime.insertText('하늘')
  await ime.convert()
  ime.key({ key: 'g' })
  expect(ime.doc).toBe('天')
  expect(ime.pre).toBe('ㅎ')
})

test('backspace edits preedit, then the document', () => {
  const ime = new Ime(composer(), seg)
  ime.doc = 'ab'
  ime.key({ key: 'g' })
  ime.key({ key: 'Backspace' })
  expect(ime.pre).toBe('')
  ime.key({ key: 'Backspace' })
  expect(ime.doc).toBe('a')
})

test('os mode passes letters through', () => {
  const ime = new Ime(composer(), seg)
  ime.mode = 'os'
  ime.key({ key: 'g' })
  expect(ime.doc).toBe('g')
})
