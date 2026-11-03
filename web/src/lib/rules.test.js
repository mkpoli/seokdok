import { test, expect } from 'bun:test'
import { compile, rewriteInput, rewriteOutput, PRESETS } from './rules.js'

const c = (...t) => t.map((text) => ({ text, note: '' }))

test('xform replaces, derive adds after the original', () => {
  const k = compile([
    { on: true, stage: 'output', mode: 'xform', regex: false, pattern: '爲', replace: '為' },
    { on: true, stage: 'output', mode: 'derive', regex: true, pattern: '(\\p{Script=Han})았', replace: '$1ㅏㅆ', flags: 'gu' },
  ])
  expect(rewriteOutput(c('爲ㅆ다', '見았다', '봤다'), k).map((x) => x.text)).toEqual(['為ㅆ다', '見았다', '見ㅏㅆ다', '봤다'])
})

test('literal patterns are escaped and literal replacements keep $', () => {
  const k = compile([{ on: true, stage: 'output', mode: 'xform', regex: false, pattern: 'a.b', replace: '$1' }])
  expect(rewriteOutput(c('a.b axb'), k)[0].text).toBe('$1 axb')
})

test('input rules rewrite before conversion; disabled rules do nothing', () => {
  const k = compile([
    { on: true, stage: 'input', mode: 'xform', regex: false, pattern: '읍니다', replace: '습니다' },
    { on: false, stage: 'input', mode: 'xform', regex: false, pattern: '습', replace: 'X' },
  ])
  expect(rewriteInput('있읍니다', k)).toBe('있습니다')
})

test('invalid regexes are reported, not thrown', () => {
  const k = compile([{ on: true, stage: 'output', mode: 'xform', regex: true, pattern: '(', replace: '' }])
  expect(k.errors.length).toBe(1)
  expect(k.rules.length).toBe(0)
})

test('duplicates collapse, order kept', () => {
  const k = compile([{ on: true, stage: 'output', mode: 'xform', regex: false, pattern: '乾', replace: '天' }])
  expect(rewriteOutput(c('天', '乾', '하늘'), k).map((x) => x.text)).toEqual(['天', '하늘'])
})

test('every preset compiles', () => {
  expect(compile(PRESETS.map((p) => p.rule)).errors).toEqual([])
})
