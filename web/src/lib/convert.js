// The converter section's handling of one line of text.

/** @typedef {{ text: string, note: string }} Candidate */
/** @typedef {{ reading: string, candidates: Candidate[], pick: number }} Seg */

/**
 * Segments of one line, Hangul or text already in Hanja alike: a word holding
 * Hanja comes back as one segment whose menu begins with the spelling as
 * typed. Only a blank line skips the engine.
 * @param {string} line
 * @param {{ segments: (text: string) => Promise<{ reading: string, candidates: Candidate[] }[]>, input: (text: string) => string, output: (cands: Candidate[]) => Candidate[] }} io
 * @returns {Promise<Seg[]>}
 */
export async function convertLine(line, { segments, input, output }) {
  if (!line.trim()) return [{ reading: line, candidates: [{ text: line, note: '' }], pick: 0 }]
  const segs = await segments(input(line))
  return segs.map((s) => ({ ...s, candidates: output(s.candidates), pick: 0 }))
}

/** A converted line's key: the text its segments read, which is the line as typed. */
export const lineKey = (/** @type {Seg[]} */ segs) => segs.map((s) => s.reading).join('')

/** The converted lines by key, so a line that did not change keeps its picks. */
export const byLine = (/** @type {Seg[][]} */ lines) => new Map(lines.map((segs) => [lineKey(segs), segs]))

/** The text the picks make up. */
export const picked = (/** @type {Seg[][]} */ lines) =>
  lines.map((segs) => segs.map((s) => s.candidates[s.pick]?.text ?? s.reading).join('')).join('\n')
