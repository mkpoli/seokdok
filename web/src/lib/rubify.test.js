import { test, expect } from 'bun:test'
import { rubify } from './rubify.js'

test('ruby covers only the Hanja part', () => {
  expect(rubify('國의', '나라의')).toEqual({ base: '國', rt: '나라', tail: '의' })
  expect(rubify('見ㅆ다', '봤다')).toEqual({ base: '見ㅆ', rt: '봤', tail: '다' })
  expect(rubify('規則을', '규칙을')).toEqual({ base: '規則', rt: '규칙', tail: '을' })
  expect(rubify('하늘', '하늘')).toBe(null)
  expect(rubify('天', '하늘')).toEqual({ base: '天', rt: '하늘', tail: '' })
})
