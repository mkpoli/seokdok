import { expect, test } from 'bun:test'

// Runes are compiled by Svelte; under bun, $state is shimmed as identity.
globalThis.$state = (v) => v
globalThis.localStorage = { getItem: () => null, setItem() {} }
globalThis.navigator = { language: 'ko' }
globalThis.document = { documentElement: {} }
const { isSino, r, plain, t, setLang, reading } = await import('./i18n.svelte.js')
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

test('the 하 → ᄒᆞ setting is labelled in both languages', () => {
  for (const lang of ['ko', 'en']) {
    setLang(lang)
    for (const key of ['oldHa', 'oldHaTip']) {
      const text = plain(t().options[key])
      expect(text).toContain('\u1112\u119e')
      expect(text).not.toMatch(/[{}|]/)
    }
  }
  setLang('ko')
  expect(plain(t().options.oldHa)).toBe('하 → \u1112\u119e 우선')
})

test('a final or vowel jamo after a mark joins the reading', () => {
  expect(reading('{古|예}ㅅ{韓㐎|한글}')).toBe('옛한글')
  expect(reading('{不|안}ㅎ았다')).toBe('않았다')
  expect(reading('{事|거}ㅅ은')).toBe('것은')
  expect(reading('{自|저}ㅣ 컴퓨터')).toBe('제 컴퓨터')
  expect(reading('{見|보}ㄹ {手|수}')).toBe('볼 수')
  expect(reading('{異|다르}ㄴ')).toBe('다른')
  expect(reading('{打|치}ㅂ니다')).toBe('칩니다')
  expect(reading('{打|치}ㅣ')).toBe('치ㅣ')
  options.sinoOnly = true
  expect(r('{不|안}ㅎ지')).toBe('<span>않지</span>')
  expect(r('{自|저}ㅣ')).toBe('<span>제</span>')
  options.sinoOnly = false
  expect(r('{不|안}ㅎ지')).toBe('<span><ruby>不<rt>안</rt></ruby>ㅎ지</span>')
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

test('手 for the bound noun 수 is Hanja only in 釋讀', () => {
  expect(isSino('手', '수')).toBe(false)
  expect(isSino('手', '손')).toBe(false)
})
