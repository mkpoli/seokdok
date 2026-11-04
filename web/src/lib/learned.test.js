import { test, expect } from 'bun:test'

globalThis.$state = (v) => v
const { Ime } = await import('./ime.svelte.js')
const { Picks, STORAGE_KEY } = await import('./learned.js')
const wasm = await import('../../pkg/seokdok.js')
wasm.initSync({ module: await Bun.file(new URL('../../pkg/seokdok_bg.wasm', import.meta.url)).arrayBuffer() })

function memoryStorage(initial = {}) {
  const items = new Map(Object.entries(initial))
  return {
    items,
    getItem: (k) => items.get(k) ?? null,
    setItem: (k, v) => items.set(k, v),
    removeItem: (k) => items.delete(k),
  }
}

const words = { 나: ['吾', '出', '我'], 나라: ['國', '我라'], 한: ['韓', '恨'], 가: ['家', '歌'], 한가: ['閑暇', '韓家'], 나는: ['吾는', '我는'] }
const list = (w) => (words[w] ?? [w]).map((text) => ({ text, note: '' }))
// One segment per space-separated word; 한가 comes as two, as 한 and 가.
const segment = async (text) =>
  text
    .split(/( )/)
    .filter(Boolean)
    .flatMap((w) => (w === '한가' ? ['한', '가'] : [w]))
    .map((w) => ({ reading: w, candidates: list(w) }))
const whole = async (text) => list(text)

function setup(storage = memoryStorage()) {
  const picks = new Picks(wasm.Learned, storage)
  const ime = new Ime(new wasm.Composer(), segment, whole)
  ime.learner = picks
  return { ime, picks, storage }
}

/** Convert `text` and return what shows first. */
async function first(ime, text) {
  ime.insertText(text)
  await ime.convert()
  const shown = ime.converted
  ime.cancel()
  ime.composer.clear()
  ime.pre = ''
  return shown
}

/** Convert `text`, step to candidate `n` of the focused segment and confirm. */
async function pick(ime, text, n) {
  ime.insertText(text)
  await ime.convert()
  for (let i = 0; i < n; i++) ime.key({ key: 'ArrowDown' })
  ime.key({ key: 'Enter' })
}

test('one pick changes nothing; two put the form first', async () => {
  const { ime } = setup()
  expect(await first(ime, '나')).toBe('吾')
  await pick(ime, '나', 2)
  expect(ime.doc).toBe('我')
  expect(await first(ime, '나')).toBe('吾')
  await pick(ime, '나', 2)
  expect(await first(ime, '나')).toBe('我')
  // The rest of the list keeps its order.
  ime.insertText('나')
  await ime.convert()
  expect(ime.segs[0].candidates.map((c) => c.text)).toEqual(['我', '吾', '出'])
})

test('a recent form outweighs an old one, and one more pick does not', async () => {
  const { ime } = setup()
  for (let i = 0; i < 5; i++) await pick(ime, '나', i < 2 ? 2 : 0)
  expect(await first(ime, '나')).toBe('我')
  await pick(ime, '나', 2)
  expect(await first(ime, '나')).toBe('我')
  await pick(ime, '나', 2)
  expect(await first(ime, '나')).toBe('出')
})

test('a longer word and an unpicked reading keep their order', async () => {
  const { ime } = setup()
  await pick(ime, '나', 2)
  await pick(ime, '나', 2)
  expect(await first(ime, '나라')).toBe('國')
  expect(await first(ime, '나는')).toBe('吾는')
  expect(await first(ime, '한')).toBe('韓')
})

test('a piece the engine left in Hangul inside a run stays as it is', async () => {
  const fragmented = async (text) =>
    text === '나봐요'
      ? [
          { reading: '나', candidates: [{ text: '나', note: '' }, ...list('나')] },
          { reading: '봐요', candidates: [{ text: '見요', note: '' }] },
        ]
      : segment(text)
  const picks = new Picks(wasm.Learned, null)
  picks.commit('나', '我', false)
  picks.commit('나', '我', false)
  const ime = new Ime(new wasm.Composer(), fragmented, whole)
  ime.learner = picks
  expect(await first(ime, '나봐요')).toBe('나見요')
  expect(await first(ime, '나')).toBe('我')
})

test('a span learned by resizing joins its segments, with the pick first', async () => {
  const { ime, picks } = setup()
  expect(await first(ime, '한가')).toBe('韓家')
  picks.commit('한가', '韓家', false)
  picks.commit('한가', '韓家', false)
  ime.insertText('한가')
  await ime.convert()
  expect(ime.segs.map((s) => s.reading)).toEqual(['한가'])
  expect(ime.converted).toBe('韓家')
})

test('without a learner, and with learning off, nothing is counted or used', async () => {
  const { ime, picks } = setup()
  ime.learner = null
  await pick(ime, '나', 2)
  await pick(ime, '나', 2)
  expect(picks.empty).toBe(true)
  ime.learner = picks
  await pick(ime, '나', 2)
  await pick(ime, '나', 2)
  ime.learner = null
  expect(await first(ime, '나')).toBe('吾')
})

test('taking the first candidate of an unpicked reading stores nothing', async () => {
  const { ime, picks } = setup()
  await pick(ime, '나', 0)
  expect(picks.empty).toBe(true)
})

test('an empty table leaves every conversion as it was', async () => {
  const { ime } = setup()
  const plain = new Ime(new wasm.Composer(), segment, whole)
  for (const t of ['나', '나라', '한가', '나 한']) {
    ime.insertText(t)
    plain.insertText(t)
    await ime.convert()
    await plain.convert()
    expect(ime.segs).toEqual(plain.segs)
    ime.cancel()
    plain.cancel()
    ime.composer.clear()
    plain.composer.clear()
    ime.pre = plain.pre = ''
  }
})

test('the table survives a reload, is stored late, and can be forgotten', async () => {
  const storage = memoryStorage()
  const { ime, picks } = setup(storage)
  await pick(ime, '나', 2)
  await pick(ime, '나', 2)
  // Saving waits, so typing does not write on each key.
  expect(storage.items.has(STORAGE_KEY)).toBe(false)
  picks.flush()
  expect(storage.items.get(STORAGE_KEY)).toContain('我')

  const again = setup(storage)
  expect(await first(again.ime, '나')).toBe('我')

  again.picks.forget()
  expect(storage.items.has(STORAGE_KEY)).toBe(false)
  expect(await first(again.ime, '나')).toBe('吾')
  again.picks.flush()
  expect(storage.items.has(STORAGE_KEY)).toBe(false)
})

test('a forgotten table does not come back from a save already due', async () => {
  const storage = memoryStorage()
  const { ime, picks } = setup(storage)
  await pick(ime, '나', 2)
  picks.forget()
  await Bun.sleep(1100)
  expect(storage.items.has(STORAGE_KEY)).toBe(false)
})

for (const [name, raw] of [['garbage', 'not a table'], ['a truncated row', '# seokdok learned picks 1\n나\t我'], ['a bad count', '# seokdok learned picks 1\n나\t我\tx\t1\t2\n']]) {
  test(`a stored table with ${name} is ignored`, async () => {
    const { ime, picks } = setup(memoryStorage({ [STORAGE_KEY]: raw }))
    expect(picks.empty).toBe(true)
    expect(await first(ime, '나')).toBe('吾')
  })
}

test('another tab saving keeps what waits here; another tab forgetting does not', async () => {
  const storage = memoryStorage()
  const here = setup(storage).picks
  const there = setup(storage).picks
  here.commit('나', '我', false)
  there.commit('한', '恨', false)
  there.flush()
  here.reload()
  here.flush()
  expect(storage.items.get(STORAGE_KEY)).toContain('我')
  expect(storage.items.get(STORAGE_KEY)).toContain('恨')

  here.commit('가', '歌', false)
  there.reload()
  there.forget()
  here.reload()
  here.flush()
  expect(storage.items.has(STORAGE_KEY)).toBe(false)
  expect(here.empty).toBe(true)
})

test('storage that throws or is absent still learns for the page', async () => {
  const broken = {
    getItem() {
      throw new Error('blocked')
    },
    setItem() {
      throw new Error('blocked')
    },
    removeItem() {
      throw new Error('blocked')
    },
  }
  for (const storage of [broken, null]) {
    const picks = new Picks(wasm.Learned, storage)
    const ime = new Ime(new wasm.Composer(), segment, whole)
    ime.learner = picks
    await pick(ime, '나', 2)
    await pick(ime, '나', 2)
    picks.flush()
    expect(await first(ime, '나')).toBe('我')
    picks.forget()
    expect(picks.empty).toBe(true)
  }
})
