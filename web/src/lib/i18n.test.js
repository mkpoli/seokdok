import { expect, test } from 'bun:test'

// Runes are compiled by Svelte; under bun, $state is shimmed as identity.
globalThis.$state = (v) => v
globalThis.localStorage = { getItem: () => null, setItem() {} }
globalThis.navigator = { language: 'ko' }
globalThis.document = { documentElement: {} }
const { isSino, r, plain, t, setLang } = await import('./i18n.svelte.js')
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
