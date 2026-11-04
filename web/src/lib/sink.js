// What the reader's own keyboard types, however it arrives.
//
// A keyboard with usable key events goes to Ime.key from keydown. All the
// others reach the page only as changes to this textarea, in shapes that differ
// by keyboard and browser: one jamo at a time, a syllable replaced by the next,
// a delete and a reinsert, a composition, a word. So the handlers never read an
// event's data. They read the textarea and compare it with what it held:
//
//   - a single character no Korean keyboard composes is the same as a key press;
//   - a Space or line break added at the end is that key;
//   - Hangul, and whatever is being composed, stays in the textarea and the
//     preedit is recomposed from it on every change, so the keyboard keeps the
//     context it edits and any edit it makes simply shows;
//   - anything else is finished text.
//
// MARK keeps the textarea from ever being empty, because Backspace in an empty
// textarea fires no input event on phones.

export const KOREAN = /[\u1100-\u11FF\u3131-\u318E\uA960-\uA97C\uD7B0-\uD7FB가-힣]/
export const JAMO = /[\u3131-\u318E]/
// A run of compatibility letters holding an old one (ㅎㆍㄴ): no finished text
// spells a syllable that way, so it is still to be composed. Others (ㅋㅋ) stay.
export const OLD_RUN = /([\u3131-\u318E]*[\u3165-\u318E][\u3131-\u318E]*)/
// A syllable written in conjoining jamo, and a modern one so written.
const CONJOINING = /[\u1100-\u115F\uA960-\uA97C]+[\u1160-\u11A7\uD7B0-\uD7C6]+[\u11A8-\u11FF\uD7CB-\uD7FB]*/g
const MODERN = /^[\u1100-\u1112][\u1161-\u1175][\u11A8-\u11C2]?$/

/**
 * One form for each syllable: modern ones precomposed, old ones in conjoining
 * jamo, so none is left half of each (가 + an old final).
 * @param {string} s
 */
export const syllables = (s) =>
  s
    .replace(/[가-힣][\u1160-\u11FF\uD7B0-\uD7FB]+/g, (m) => m.normalize('NFD'))
    .replace(CONJOINING, (m) => (MODERN.test(m) ? m.normalize('NFC') : m))

const MARK = '\u200B'
const LATIN = /^[a-zA-Z]$/
const BREAK = /^[ \n]$/

/** @typedef {{ c: string, finished: boolean }} Unit one code point, sent alone or as part of finished text */

export class Sink {
  /** @type {Unit[]} */
  #units = []
  /** The preedit the units continue. */
  #base = ''
  /** The preedit as the units left it; anything else means it moved on without them. */
  #shown = ''
  #composing = false
  /** Characters at the start of the textarea that are spent but cannot be
   *  removed yet: writing to it would break the composition. */
  #stale = 0

  /**
   * @param {import('./ime.svelte.js').Ime} ime
   * @param {HTMLTextAreaElement} el
   */
  constructor(ime, el) {
    this.ime = ime
    this.el = el
    this.reset()
  }

  /**
   * Whether a keydown means what it says. Otherwise the change it makes to the
   * textarea, if any, is what counts.
   * @param {{ key: string, keyCode?: number, isComposing?: boolean }} e
   */
  usable(e) {
    if (e.isComposing || e.key === 'Process' || e.keyCode === 229) return false
    // Hangul and accented letters: iOS may name the jamo without typing it yet.
    if (e.key.length === 1 && e.key > '~') return false
    // The browser deletes what the keyboard typed.
    return !(e.key === 'Backspace' && this.#units.length)
  }

  /** Run `f` now, or in turn behind a conversion in flight. The textarea is
   *  kept up to date at once; only the input method waits. */
  #then(f) {
    if (this.ime.busy) this.ime.later(f)
    else f()
  }

  /** Nothing is held any more: go on from the preedit as it then stands. */
  #rebase() {
    this.#units = []
    this.#then(() => {
      this.#base = this.#shown = this.ime.pre
    })
  }

  /** Start over from the preedit as it is now. */
  reset() {
    this.#rebase()
    if (this.#composing) this.#stale = [...this.el.value.replaceAll(MARK, '')].length
    else this.#write(MARK)
  }

  caretToEnd() {
    this.el.setSelectionRange(this.el.value.length, this.el.value.length)
  }

  compositionstart() {
    this.#composing = true
  }

  compositionend() {
    this.#composing = false
    this.input()
  }

  #write(value) {
    this.#stale = 0
    if (this.el.value === value) return
    this.el.value = value
    this.caretToEnd()
  }

  /** @param {Unit[]} units */
  #fill(units) {
    this.#units = units
    this.#then(() => {
      const ime = this.ime
      if (units.length && ime.segs) ime.commit()
      if (ime.pre !== this.#shown) this.#base = ime.pre
      ime.retype(this.#base, units)
      this.#shown = ime.pre
    })
  }

  #isKorean(u) {
    return KOREAN.test(u.c) || (this.ime.mode === 'hangul' && !u.finished && LATIN.test(u.c))
  }

  /** @param {{ inputType?: string }} [e] */
  input(e) {
    // WebKit empties the composition and fills it again as it ends. The text
    // that counts is what stands at compositionend.
    if (e?.inputType === 'deleteCompositionText') return
    const ime = this.ime
    const composing = this.#composing
    const value = this.el.value
    const next = [...value.replaceAll(MARK, '')].slice(this.#stale)
    const prev = this.#units
    let at = 0
    while (at < next.length && at < prev.length && next[at] === prev[at].c) at++
    const removed = prev.length - at
    let added = next.slice(at)
    // Something else moved the preedit on (a button, the on-screen keyboard).
    const spent = !ime.busy && (ime.segs !== null || ime.pre !== this.#shown)
    if (spent) this.#units = []
    if (spent && composing) this.#stale += at

    // Deleting past what the keyboard holds, or text it no longer owns.
    if (!composing && !added.length && ((spent && removed > 0) || (!prev.length && !value.includes(MARK)))) {
      ime.key({ key: 'Backspace' })
      return this.reset()
    }

    // Autocorrect on letters that stand for jamo: keep the letters typed.
    const typed = (u) => !u.finished && LATIN.test(u.c)
    if (!composing && removed && prev.slice(at).every(typed) && added.some((c) => LATIN.test(c))) {
      at = prev.length
      added = added.filter((c) => BREAK.test(c)).slice(-1)
    }

    let press = null
    if (!composing && BREAK.test(added.at(-1)) && !added.slice(0, -1).some((c) => BREAK.test(c))) press = added.pop()

    const kept = spent ? [] : prev.slice(0, at)
    if (!composing && added.length === 1 && added[0].length === 1 && !KOREAN.test(added[0])) {
      if (kept.length !== prev.length && !spent) this.#fill(kept)
      ime.key({ key: added[0] })
      this.#rebase()
    } else if (added.length) {
      const finished = added.length > 1 && !(composing && at > 0 && !removed)
      this.#fill([...kept, ...added.map((c) => ({ c, finished }))])
    } else if (kept.length !== prev.length && !spent) {
      this.#fill(kept)
    }

    // Finished text with no Hangul goes straight in, as other keys do.
    if (!composing && this.#units.length && !this.#units.some((u) => this.#isKorean(u))) {
      const text = this.#units.map((u) => u.c).join('')
      this.#fill([])
      this.#then(() => this.ime.insertText(text))
      this.#rebase()
    }

    if (press) ime.key({ key: press === '\n' ? 'Enter' : ' ' })
    if (press || !this.#units.length) return this.reset()
    if (!composing) this.#write(MARK + this.#units.map((u) => u.c).join(''))
  }
}
