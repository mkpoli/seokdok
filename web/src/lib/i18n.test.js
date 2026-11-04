import { expect, test } from 'bun:test'

// Runes are compiled by Svelte; under bun, $state is shimmed as identity.
globalThis.$state = (v) => v
globalThis.localStorage = { getItem: () => null, setItem() {} }
globalThis.navigator = { language: 'ko' }
const { isSino, r, plain } = await import('./i18n.svelte.js')
const { options } = await import('./options.svelte.js')

test('marks read by their own Sino-Korean readings are 音讀', () => {
  expect(isSino('漢字', '한자')).toBe(true)
  expect(isSino('例', '예')).toBe(true)
  expect(isSino('韓㐎', '한글')).toBe(true)
  expect(isSino('打', '치')).toBe(false)
  expect(isSino('會', '모임')).toBe(false)
  expect(isSino('個言', '낱말')).toBe(false)
})

test('國漢文混用 mode shows 訓讀 marks in Hangul', () => {
  options.sinoOnly = true
  expect(plain('{漢字|한자}로 {打|치}고 {成|되}ㅂ니다')).toBe('한자로 치고 됩니다')
  expect(r('{古|예}ㅅ{韓㐎|한글}')).toBe('<span>옛<ruby>韓㐎<rt>한글</rt></ruby></span>')
  options.sinoOnly = false
  expect(r('{古|예}ㅅ{韓㐎|한글}')).toBe('<span><ruby>古<rt>예</rt></ruby>ㅅ<ruby>韓㐎<rt>한글</rt></ruby></span>')
})

test('page texts have well-formed marks and both languages have the same sections', async () => {
  const pages = (await import('./pages.js')).default
  const strings = (v) => (typeof v === 'string' ? [v] : v && typeof v === 'object' ? Object.values(v).flatMap(strings) : [])
  for (const s of strings(pages.ko)) {
    expect(s.replace(/\{[^|}]+\|[^}]+\}/g, '')).not.toMatch(/[{}|]/)
  }
  for (const page of ['privacy', 'support']) {
    expect(pages.ko[page].sections.map((s) => s.id)).toEqual(pages.en[page].sections.map((s) => s.id))
  }
})

test('page marks hold Hanja over a Hangul reading and leave nothing unmatched in either mode', async () => {
  const pages = (await import('./pages.js')).default
  const strings = (v) => (typeof v === 'string' ? [v] : v && typeof v === 'object' ? Object.values(v).flatMap(strings) : [])
  const hanja = /^[㐀-䶿一-鿿豈-﫿]+$/
  const hangul = /^[가-힣]+$/
  for (const s of strings(pages.ko)) {
    for (const [, h, k] of s.matchAll(/\{([^|}]*)\|([^}]*)\}/g)) {
      expect(h).toMatch(hanja)
      expect(k).toMatch(hangul)
    }
    for (const sinoOnly of [false, true]) {
      options.sinoOnly = sinoOnly
      expect(plain(s)).not.toMatch(/[{}|]/)
    }
  }
  options.sinoOnly = false
})
