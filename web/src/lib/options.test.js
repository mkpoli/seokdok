import { test, expect } from 'bun:test'

globalThis.$state = (v) => v
const { options, DEFAULTS } = await import('./options.svelte.js')

test('the script setting has three values and drives the engine flag', () => {
  expect(DEFAULTS.script).toBe('seokdok')
  expect('sinoOnly' in DEFAULTS).toBe(false)
  expect(options.sinoOnly).toBe(false)
  options.script = 'sino'
  expect(options.sinoOnly).toBe(true)
  expect({ ...options }.sinoOnly).toBe(true)
  options.sinoOnly = false
  expect(options.script).toBe('seokdok')
  options.sinoOnly = true
  expect(options.script).toBe('sino')
})

for (const [name, raw] of [['null', 'null'], ['a number', '3'], ['a string', '"x"'], ['an array', '[1]'], ['broken JSON', '{']]) {
  test(`stored options that are ${name} fall back to the defaults`, async () => {
    globalThis.localStorage = { getItem: () => raw, setItem() {} }
    const mod = await import(`./options.svelte.js?stored=${name}`)
    expect(mod.options.script).toBe('seokdok')
    expect(mod.options.style).toBe('surface')
    expect(mod.options.sinoOnly).toBe(false)
  })
}

test('the site switch leaves 韓㐎 專用 alone when it turns 國漢文混用 off', () => {
  options.script = 'hangul'
  expect(options.sinoOnly).toBe(false)
  options.sinoOnly = false
  expect(options.script).toBe('hangul')
  options.sinoOnly = true
  expect(options.script).toBe('sino')
})
