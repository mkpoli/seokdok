// Text is edited by user-perceived character: an old-Hangul syllable written
// as conjoining jamo (ᄒᆞ = U+1112 U+119E) is one character, as are emoji.
const graphemes = new Intl.Segmenter('ko', { granularity: 'grapheme' })

/** @param {string} s */
export const chars = (s) => Array.from(graphemes.segment(s), (g) => g.segment)
