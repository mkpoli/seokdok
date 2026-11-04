// Text is edited by user-perceived character: an old-Hangul syllable written
// as conjoining jamo (ᄒᆞ = U+1112 U+119E) is one character, as are emoji.
const graphemes = new Intl.Segmenter('ko', { granularity: 'grapheme' })

/** @param {string} s */
export const chars = (s) => Array.from(graphemes.segment(s), (g) => g.segment)

/** Would a boundary between these two strings fall inside one character? */
export const splitsChar = (a, b) => chars(a + b).length < chars(a).length + chars(b).length

// Hangul of every kind: conjoining jamo (modern and old), compatibility jamo
// and syllables, for use inside a regex character class.
export const HANGUL_CLASS = '\\u1100-\\u11FF\\u3131-\\u318E\\uA960-\\uA97C\\uAC00-\\uD7A3\\uD7B0-\\uD7FB'
export const HANGUL = new RegExp(`[${HANGUL_CLASS}]`)

// Text the converter works on: Hangul of any kind, or Hanja (words already in 釋讀).
export const WORD = new RegExp(`[${HANGUL_CLASS}\\p{Script=Han}]`, 'u')
