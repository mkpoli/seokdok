// Ruby over the Hanja part only: "國의" read "나라의" → 國 (나라) + 의.
import { chars, HANGUL, HANGUL_CLASS } from './chars.js'

const HAN = /\p{Script=Han}/u

/** A 釋讀 word in running text: Hanja and the Hangul that follows them. */
export const WORD = new RegExp(`(\\p{Script=Han}+[${HANGUL_CLASS}]*)`, 'u')

/** @returns {{ base: string, rt: string, tail: string } | null} */
export function rubify(text, reading) {
  if (!HAN.test(text) || !reading || text === reading) return null
  const a = chars(text)
  const b = chars(reading)
  let i = a.length
  let j = b.length
  while (i > 0 && j > 0 && a[i - 1] === b[j - 1] && HANGUL.test(a[i - 1])) {
    i--
    j--
  }
  if (i === 0 || j === 0) return { base: text, rt: reading, tail: '' }
  return { base: a.slice(0, i).join(''), rt: b.slice(0, j).join(''), tail: a.slice(i).join('') }
}
