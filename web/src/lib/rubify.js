// Ruby over the Hanja part only: "國의" read "나라의" → 國 (나라) + 의.
const HANGUL = /[가-힣ㄱ-ㆎ]/
const HAN = /\p{Script=Han}/u

/** @returns {{ base: string, rt: string, tail: string } | null} */
export function rubify(text, reading) {
  if (!HAN.test(text) || !reading || text === reading) return null
  const a = [...text]
  const b = [...reading]
  let i = a.length
  let j = b.length
  while (i > 0 && j > 0 && a[i - 1] === b[j - 1] && HANGUL.test(a[i - 1])) {
    i--
    j--
  }
  if (i === 0 || j === 0) return { base: text, rt: reading, tail: '' }
  return { base: a.slice(0, i).join(''), rt: b.slice(0, j).join(''), tail: a.slice(i).join('') }
}
