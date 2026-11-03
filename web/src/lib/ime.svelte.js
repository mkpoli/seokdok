// The in-page input method, modelled on Japanese IMEs (MS-IME, Google 日本語入力).
//
// Composing (Hangul in the preedit):
//   letters        type 두벌식 (or text from the reader's own IME)
//   Space / Tab    convert
//   Enter          commit the Hangul as typed
//   Esc            clear the preedit
//   Shift+Space    a literal space inside the preedit
// Converting (segments, the focused one highlighted):
//   Space / ↓      open the candidate window, then next candidate
//   Shift+Space / ↑  previous candidate
//   Tab / Shift+Tab, PageDown / PageUp   next / previous page of candidates
//   1–9            pick that candidate and confirm up to this segment
//   ← / →          move between segments
//   Shift+← / →    shrink / extend the focused segment by one syllable
//   Enter          confirm everything
//   Esc            close the window; again, back to Hangul
//   Backspace      back to Hangul
//   any other key  confirm everything, then type it
// Editing (nothing composed):
//   ← → Home End   move the caret; a click places it
//   Backspace / Delete  delete before / after the caret
//
// Word spacing (띄어쓰기): a conversion started with Space marks a word break,
// so the next Hangul typed after it is preceded by a space. Enter and
// punctuation confirm without one. With spaceMode 'space', Space types a
// space and Tab, ↓ or the convert key convert.

/** @typedef {{ text: string, note: string }} Candidate */
/** @typedef {{ reading: string, candidates: Candidate[], pick: number }} Seg */

export const PAGE = 9
const HANGUL = /[가-힣]/

export class Ime {
  /** Committed text. */
  doc = $state('')
  /** Caret position in `doc`, in code points. */
  cursor = $state(0)
  /** 'convert': Space converts (Japanese style); 'space': Space types a space. */
  spaceMode = $state('convert')
  /** Hangul being composed. */
  pre = $state('')
  /** Text the reader's own IME is still composing. */
  osComposing = $state('')
  /** @type {Seg[] | null} */
  segs = $state(null)
  focus = $state(0)
  /** The candidate window is open. */
  open = $state(false)
  /** 'sim': QWERTY letters type 두벌식; 'os': letters pass through. */
  mode = $state('sim')
  /** Key last pressed, for the on-screen keyboard. */
  lastKey = $state('')
  busy = $state(false)

  /**
   * @param {object} composer 두벌식 buffer (wasm Composer)
   * @param {(text: string) => Promise<{ reading: string, candidates: Candidate[] }[]>} segment
   * @param {(text: string) => Promise<Candidate[]>} [whole] candidates for an exact span
   */
  constructor(composer, segment, whole) {
    this.composer = composer
    this.segment = segment
    this.whole = whole ?? (async (t) => (await segment(t)).flatMap((s) => s.candidates))
  }

  #wordBreak = false
  #spaced = false
  /** Keys pressed while a conversion is in flight, replayed in order. */
  #queue = []

  #drain() {
    while (this.#queue.length && !this.busy) this.key(this.#queue.shift())
  }

  get before() {
    return [...this.doc].slice(0, this.cursor).join('')
  }

  get after() {
    return [...this.doc].slice(this.cursor).join('')
  }

  /** Insert text at the caret. */
  #insert(text) {
    const a = [...this.doc]
    const t = [...text]
    a.splice(this.cursor, 0, ...t)
    this.doc = a.join('')
    this.cursor += t.length
  }

  /** Replace the whole document (clear, paste into the editor). */
  setDoc(text) {
    this.doc = text
    this.cursor = [...text].length
  }

  /** Place the caret (a click in the text). */
  setCursor(i) {
    if (this.segs || this.pre) this.commit()
    this.cursor = Math.max(0, Math.min([...this.doc].length, i))
    this.#wordBreak = false
  }

  #space() {
    const prev = [...this.before].pop()
    if (prev && !/\s/.test(prev)) this.#insert(' ')
  }

  get current() {
    return this.segs?.[this.focus] ?? null
  }

  /** The conversion as currently picked. */
  get converted() {
    return (this.segs ?? []).map((s) => s.candidates[s.pick]?.text ?? s.reading).join('')
  }

  /** First index of the page holding the focused segment's pick. */
  get pageStart() {
    const s = this.current
    return s ? Math.floor(s.pick / PAGE) * PAGE : 0
  }

  #sync() {
    this.pre = this.composer.text()
  }

  /** Segments the cursor can visit: those with a real choice. */
  #stops() {
    return (this.segs ?? []).map((s, i) => (s.candidates.length > 1 || HANGUL.test(s.reading) ? i : -1)).filter((i) => i >= 0)
  }

  /** @param {boolean} [spaced] started by Space: confirming implies a word break */
  async convert(spaced = false) {
    if (!this.pre || !HANGUL.test(this.pre)) return false
    this.#spaced = spaced
    this.busy = true
    try {
      const segs = await this.segment(this.pre)
      this.segs = segs.map((s) => ({ ...s, pick: 0 }))
      this.focus = this.#stops()[0] ?? 0
      this.open = false
      return true
    } finally {
      this.busy = false
      this.#drain()
    }
  }

  /** Confirm everything. */
  commit() {
    if (this.segs) this.#wordBreak = this.#spaced
    this.#insert(this.segs ? this.converted : this.pre)
    this.segs = null
    this.open = false
    this.composer.clear()
    this.#sync()
  }

  /** Confirm segments 0..i; the rest stay converting. */
  commitThrough(i) {
    if (!this.segs) return
    const done = this.segs.slice(0, i + 1)
    const rest = this.segs.slice(i + 1)
    this.#insert(done.map((s) => s.candidates[s.pick]?.text ?? s.reading).join(''))
    this.open = false
    const left = rest.map((s) => s.reading).join('')
    if (!rest.length || !HANGUL.test(left)) {
      this.#insert(left)
      this.#wordBreak = this.#spaced
      this.segs = null
      this.composer.clear()
      this.#sync()
      return
    }
    this.segs = rest
    this.composer.setText(left)
    this.#sync()
    this.focus = this.#stops()[0] ?? 0
  }

  cancel() {
    this.segs = null
    this.open = false
  }

  /** @param {number} d */
  cycle(d) {
    const s = this.current
    if (!s) return
    const n = s.candidates.length
    if (!this.open) {
      // The first press opens the window on the next candidate.
      this.open = true
      if (n > 1) s.pick = (s.pick + (d > 0 ? 1 : n - 1)) % n
      return
    }
    s.pick = (s.pick + d + n) % n
  }

  /** @param {number} d pages */
  page(d) {
    const s = this.current
    if (!s) return
    this.open = true
    const n = s.candidates.length
    const pages = Math.ceil(n / PAGE)
    const p = (Math.floor(s.pick / PAGE) + d + pages) % pages
    s.pick = Math.min(n - 1, p * PAGE)
  }

  /** @param {number} d */
  move(d) {
    const stops = this.#stops()
    if (!stops.length) return
    const at = stops.indexOf(this.focus)
    this.focus = stops[Math.max(0, Math.min(stops.length - 1, (at < 0 ? 0 : at) + d))]
    this.open = false
  }

  /** Pick candidate `i` of the focused segment and confirm through it. */
  pick(i) {
    const s = this.current
    if (!s || i < 0 || i >= s.candidates.length) return
    s.pick = i
    this.commitThrough(this.focus)
  }

  /**
   * Shrink (d = -1) or extend (d = +1) the focused segment by one syllable,
   * taking it from or giving it to the next segment.
   */
  async resize(d) {
    const segs = this.segs
    if (!segs) return
    const i = this.focus
    const cur = [...segs[i].reading]
    const next = segs[i + 1] ? [...segs[i + 1].reading] : []
    if (d < 0 && cur.length <= 1) return
    if (d > 0 && !next.length) return
    const a = d < 0 ? cur.slice(0, -1) : [...cur, next[0]]
    const b = d < 0 ? [cur[cur.length - 1], ...next] : next.slice(1)
    this.busy = true
    try {
      const [ca, cb] = await Promise.all([this.#span(a.join('')), b.length ? this.#span(b.join('')) : null])
      const fresh = [{ reading: a.join(''), candidates: ca, pick: 0 }]
      if (cb) fresh.push({ reading: b.join(''), candidates: cb, pick: 0 })
      this.segs = [...segs.slice(0, i), ...fresh, ...segs.slice(i + 2)]
      this.focus = i
      this.open = false
    } finally {
      this.busy = false
      this.#drain()
    }
  }

  async #span(text) {
    const c = await this.whole(text)
    return c.some((x) => x.text === text) ? c : [...c, { text, note: '' }]
  }

  /** Text from the reader's own IME (composition end, paste). */
  insertText(s) {
    if (this.segs) this.commit()
    if (this.#wordBreak && !this.pre && HANGUL.test(s)) this.#space()
    this.#wordBreak = false
    this.composer.setText(this.pre + s)
    this.#sync()
  }

  /**
   * One key press. Returns true when the key was used (the caller then
   * prevents the default action).
   * @param {{ key: string, shift?: boolean, ctrl?: boolean, alt?: boolean, meta?: boolean }} k
   */
  key(k) {
    if (k.ctrl || k.meta || k.alt) return false
    const key = k.key
    this.lastKey = key
    if (this.busy) {
      this.#queue.push(k)
      return true
    }
    if (this.segs) return this.#converting(k)
    const brk = this.#wordBreak
    this.#wordBreak = false
    switch (key) {
      case ' ':
        if (this.pre && (k.shift || this.spaceMode === 'space')) {
          this.composer.pushLiteral(' ')
          this.#sync()
        } else if (this.pre) {
          this.convert(true)
        } else {
          this.#insert(' ')
        }
        return true
      case 'Tab':
      case 'ArrowDown':
      case 'Convert':
        if (!this.pre) return false
        this.convert(false)
        return true
      case 'Backspace':
        if (this.pre) {
          this.composer.backspace()
          this.#sync()
        } else if (this.cursor > 0) {
          const a = [...this.doc]
          a.splice(this.cursor - 1, 1)
          this.doc = a.join('')
          this.cursor--
        } else return false
        return true
      case 'Delete':
        if (this.pre) return true
        if (this.cursor < [...this.doc].length) {
          const a = [...this.doc]
          a.splice(this.cursor, 1)
          this.doc = a.join('')
        }
        return true
      case 'ArrowLeft':
      case 'ArrowRight':
      case 'Home':
      case 'End': {
        if (this.pre) this.commit()
        const n = [...this.doc].length
        const b = this.before
        if (key === 'ArrowLeft') this.cursor = Math.max(0, this.cursor - 1)
        else if (key === 'ArrowRight') this.cursor = Math.min(n, this.cursor + 1)
        else if (key === 'Home') this.cursor = [...b.slice(0, b.lastIndexOf('\n') + 1)].length
        else {
          const nl = this.after.indexOf('\n')
          this.cursor = nl < 0 ? n : this.cursor + [...this.after.slice(0, nl)].length
        }
        return true
      }
      case 'Enter':
        if (this.pre) this.commit()
        else this.#insert('\n')
        return true
      case 'Escape':
        if (!this.pre) return false
        this.composer.clear()
        this.#sync()
        return true
    }
    if (key.length !== 1) return false
    if (this.mode === 'sim' && /[a-zA-Z]/.test(key)) {
      if (brk && !this.pre) this.#space()
      this.composer.input(key)
      this.#sync()
      return true
    }
    if (this.pre) {
      this.composer.pushLiteral(key)
      this.#sync()
    } else {
      if (brk && HANGUL.test(key)) this.#space()
      this.#insert(key)
    }
    return true
  }

  #converting(k) {
    const { key, shift } = k
    switch (key) {
      case ' ':
        if (this.spaceMode === 'space' && !this.open) {
          this.commit()
          this.#wordBreak = false
          this.#insert(' ')
          return true
        }
        this.cycle(shift ? -1 : 1)
        return true
      case 'Convert':
      case 'ArrowDown':
        this.cycle(1)
        return true
      case 'ArrowUp':
        this.cycle(-1)
        return true
      case 'Tab':
        this.page(shift ? -1 : 1)
        return true
      case 'PageDown':
        this.page(1)
        return true
      case 'PageUp':
        this.page(-1)
        return true
      case 'ArrowRight':
        if (shift) this.resize(1)
        else this.move(1)
        return true
      case 'ArrowLeft':
        if (shift) this.resize(-1)
        else this.move(-1)
        return true
      case 'Enter':
        this.commit()
        return true
      case 'Escape':
        if (this.open) this.open = false
        else this.cancel()
        return true
      case 'Backspace':
        this.cancel()
        return true
    }
    if (this.open && /^[1-9]$/.test(key)) {
      this.pick(this.pageStart + Number(key) - 1)
      return true
    }
    if (key.length === 1) {
      this.commit()
      return this.key(k)
    }
    return true
  }
}
