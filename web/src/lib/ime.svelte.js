import { chars, splitsChar, HANGUL, WORD } from './chars.js'
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
// 韓㐎 專用 (hangulOnly): nothing is converted. Space commits the Hangul and
// types a space, Enter commits it, punctuation commits it and then types.
// Tab, ↓ and the convert key do nothing; there are no candidates.
//
// Word spacing (띄어쓰기): a conversion started with Space marks a word break,
// so the next Hangul typed after it is preceded by a space. Enter and
// punctuation confirm without one. With spaceMode 'space', Space types a
// space and Tab, ↓ or the convert key convert.

/** @typedef {{ text: string, note: string }} Candidate */
/** @typedef {{ reading: string, candidates: Candidate[], pick: number }} Seg */

export const PAGE = 9

export class Ime {
  /** Committed text. */
  doc = $state('')
  /** Caret position in `doc`, in characters (grapheme clusters). */
  cursor = $state(0)
  /** The other end of the selection, or null when nothing is selected. */
  anchor = $state(null)
  /** 'convert': Space converts (Japanese style); 'space': Space types a space. */
  spaceMode = $state('convert')
  /** 韓㐎 專用: Hangul is typed and committed as it is, never converted. */
  hangulOnly = $state(false)
  /** Candidate window layout; horizontal lets ← → walk the candidates. */
  layout = $state('vertical')
  /** Hangul being composed. */
  pre = $state('')
  /** Text the reader's own IME is still composing. */
  osComposing = $state('')
  /** @type {Seg[] | null} */
  segs = $state(null)
  focus = $state(0)
  /** The candidate window is open. */
  open = $state(false)
  /** 'hangul': QWERTY letters type 두벌식; 'ascii': keys pass through as typed
   *  (Latin text, or the reader's own Korean IME). Alt tapped alone or the
   *  한/영 key switches. */
  mode = $state('hangul')
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
    return chars(this.doc).slice(0, this.cursor).join('')
  }

  get after() {
    return chars(this.doc).slice(this.cursor).join('')
  }

  // Edit history: snapshots of the committed text, caret and selection.
  #past = []
  #future = []
  #lastKind = ''
  #lastAt = 0

  /** Save the state before an edit. Runs of typing (or of erasing) within a
   *  second count as one step, as in text editors. */
  #record(kind) {
    const now = Date.now()
    const run = (kind === 'type' || kind === 'erase') && kind === this.#lastKind && now - this.#lastAt < 1000
    this.#lastKind = kind
    this.#lastAt = now
    if (run) return
    this.#past.push({ doc: this.doc, cursor: this.cursor, anchor: this.anchor })
    if (this.#past.length > 300) this.#past.shift()
    this.#future = []
  }

  #restore(st) {
    this.doc = st.doc
    this.cursor = st.cursor
    this.anchor = st.anchor
    this.#lastKind = ''
  }

  get canUndo() {
    return this.#past.length > 0
  }

  get canRedo() {
    return this.#future.length > 0
  }

  /** Undo: first back out of a conversion or composition, then edits. */
  undo() {
    if (this.segs) return this.cancel(), true
    if (this.pre) {
      this.composer.clear()
      this.#sync()
      return true
    }
    const st = this.#past.pop()
    if (!st) return false
    this.#future.push({ doc: this.doc, cursor: this.cursor, anchor: this.anchor })
    this.#restore(st)
    return true
  }

  redo() {
    if (this.segs || this.pre) return false
    const st = this.#future.pop()
    if (!st) return false
    this.#past.push({ doc: this.doc, cursor: this.cursor, anchor: this.anchor })
    this.#restore(st)
    return true
  }

  /** The selected range [start, end), or null. */
  get selection() {
    if (this.anchor === null || this.anchor === this.cursor) return null
    return [Math.min(this.anchor, this.cursor), Math.max(this.anchor, this.cursor)]
  }

  get selectedText() {
    const s = this.selection
    return s ? chars(this.doc).slice(s[0], s[1]).join('') : ''
  }

  /** Select [a, b) with the caret at b. */
  setSelection(a, b) {
    if (this.segs || this.pre) this.commit()
    const n = chars(this.doc).length
    this.anchor = Math.max(0, Math.min(n, a))
    this.cursor = Math.max(0, Math.min(n, b))
  }

  selectAll() {
    this.setSelection(0, chars(this.doc).length)
  }

  /** Delete the selection; true when there was one. */
  deleteSelection(record = true) {
    const s = this.selection
    this.anchor = null
    if (!s) return false
    if (record) this.#record('delete')
    const a = chars(this.doc)
    a.splice(s[0], s[1] - s[0])
    this.doc = a.join('')
    this.cursor = s[0]
    return true
  }

  /** Insert text at the caret. */
  /** @param {string} text @param {string} [kind] history step kind */
  #insert(text, kind) {
    if (!text) return
    this.#record(kind ?? (chars(text).length === 1 ? 'type' : 'insert'))
    this.deleteSelection(false)
    // Counted again: the text can join the character before it (ᄒ + ᆞ).
    const head = this.before + text
    this.doc = head + this.after
    this.cursor = chars(head).length
  }

  /** Replace the whole document (clear, paste into the editor). */
  setDoc(text) {
    if (text !== this.doc) this.#record('set')
    this.doc = text
    this.anchor = null
    this.cursor = chars(text).length
  }

  /** Place the caret (a click in the text). */
  setCursor(i) {
    if (this.segs || this.pre) this.commit()
    this.cursor = Math.max(0, Math.min(chars(this.doc).length, i))
    this.anchor = null
    this.#wordBreak = false
  }

  #space() {
    const prev = chars(this.before).pop()
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
    return (this.segs ?? []).map((s, i) => (s.candidates.length > 1 || WORD.test(s.reading) ? i : -1)).filter((i) => i >= 0)
  }

  /** @param {boolean} [spaced] started by Space: confirming implies a word break */
  async convert(spaced = false) {
    if (this.hangulOnly || !this.pre || !WORD.test(this.pre)) return false
    this.#spaced = spaced
    this.busy = true
    try {
      const text = this.pre
      const segs = await this.#wholeChars(await this.segment(text))
      // 韓㐎 專用 may have been turned on, or the text committed, meanwhile.
      if (this.hangulOnly || this.pre !== text) return false
      this.segs = segs.map((s) => ({ ...s, pick: 0 }))
      this.focus = this.#stops()[0] ?? 0
      this.open = false
      return true
    } finally {
      this.busy = false
      this.#drain()
    }
  }

  /** A character never spans two segments: rejoin those a boundary cuts. */
  async #wholeChars(segs) {
    const out = []
    for (const s of segs) {
      const last = out.at(-1)
      if (last && splitsChar(last.reading, s.reading)) {
        const reading = last.reading + s.reading
        out[out.length - 1] = { reading, candidates: await this.#span(reading) }
      } else out.push(s)
    }
    return out
  }

  /** Confirm everything. */
  commit() {
    if (this.segs) this.#wordBreak = this.#spaced
    this.#insert(this.segs ? this.converted : this.pre, 'commit')
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
    this.#insert(done.map((s) => s.candidates[s.pick]?.text ?? s.reading).join(''), 'commit')
    this.open = false
    const left = rest.map((s) => s.reading).join('')
    if (!rest.length || !WORD.test(left)) {
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
    const cur = chars(segs[i].reading)
    const next = segs[i + 1] ? chars(segs[i + 1].reading) : []
    if (d < 0 && cur.length <= 1) return
    if (d > 0 && !next.length) return
    const a = d < 0 ? cur.slice(0, -1) : [...cur, next[0]]
    const b = d < 0 ? [cur[cur.length - 1], ...next] : next.slice(1)
    this.busy = true
    try {
      const [ca, cb] = await Promise.all([this.#span(a.join('')), b.length ? this.#span(b.join('')) : null])
      if (this.segs !== segs) return
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
    // Whole-phrase readings first; a single word also brings its full
    // candidate list, which the phrase list cuts short.
    const [c, segs] = await Promise.all([this.whole(text), this.segment(text)])
    if (segs.length === 1) for (const k of segs[0].candidates) if (!c.some((x) => x.text === k.text)) c.push(k)
    return c.some((x) => x.text === text) ? c : [...c, { text, note: '' }]
  }

  /** Turn 韓㐎 專用 on or off, confirming whatever is being composed. */
  setHangulOnly(v) {
    if (v === this.hangulOnly) return
    if (this.segs || this.pre) this.commit()
    this.hangulOnly = v
  }

  /** Switch 韓 ↔ A, confirming whatever is being composed. */
  toggleMode() {
    if (this.segs || this.pre) this.commit()
    this.mode = this.mode === 'hangul' ? 'ascii' : 'hangul'
  }

  /** Text from the reader's own IME (composition end, paste). */
  insertText(s) {
    if (this.segs) this.commit()
    if (this.#wordBreak && !this.pre && WORD.test(s)) this.#space()
    this.#wordBreak = false
    this.composer.setText(this.pre + s)
    this.#sync()
  }

  /**
   * One key press. Returns true when the key was used (the caller then
   * prevents the default action).
   * A held key that just committed goes no further: holding Enter on a
   * candidate must not go on to break the line.
   * @param {{ key: string, repeat?: boolean, shift?: boolean, ctrl?: boolean, alt?: boolean, meta?: boolean }} k
   */
  key(k) {
    if (k.repeat && this.#committedBy === k.key) return true
    const picks = k.key === 'Enter' ? this.pre || this.segs : this.segs && /^[1-9]$/.test(k.key)
    const handled = this.#dispatch(k)
    this.#committedBy = picks ? k.key : null
    return handled
  }

  #committedBy = null

  #dispatch(k) {
    if (k.ctrl || k.meta || k.alt) return false
    const key = k.key
    this.lastKey = key
    if (this.busy) {
      this.#queue.push(k)
      return true
    }
    if (key === 'HangulMode' || key === 'Lang1' || key === 'KanaMode') {
      this.toggleMode()
      return true
    }
    if (this.segs) return this.#converting(k)
    const brk = this.#wordBreak
    this.#wordBreak = false
    switch (key) {
      case ' ':
        if (this.pre && this.hangulOnly) {
          this.commit()
          this.#insert(' ')
        } else if (this.pre && (k.shift || this.spaceMode === 'space')) {
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
        if (!this.hangulOnly) this.convert(false)
        return true
      case 'Backspace':
        if (this.pre) {
          this.composer.backspace()
          this.#sync()
        } else if (this.deleteSelection()) {
          // removed the selection
        } else if (this.cursor > 0) {
          this.#record('erase')
          const a = chars(this.doc)
          a.splice(this.cursor - 1, 1)
          this.doc = a.join('')
          this.cursor--
        } else return false
        return true
      case 'Delete':
        if (this.pre) return true
        if (this.deleteSelection()) return true
        if (this.cursor < chars(this.doc).length) {
          this.#record('erase')
          const a = chars(this.doc)
          a.splice(this.cursor, 1)
          this.doc = a.join('')
        }
        return true
      case 'ArrowLeft':
      case 'ArrowRight':
      case 'Home':
      case 'End': {
        if (this.pre) this.commit()
        const n = chars(this.doc).length
        const b = this.before
        const sel = this.selection
        if (k.shift) {
          if (this.anchor === null) this.anchor = this.cursor
        } else {
          this.anchor = null
          // Without Shift, ← → collapse a selection to its edge.
          if (sel && key === 'ArrowLeft') return (this.cursor = sel[0]), true
          if (sel && key === 'ArrowRight') return (this.cursor = sel[1]), true
        }
        if (key === 'ArrowLeft') this.cursor = Math.max(0, this.cursor - 1)
        else if (key === 'ArrowRight') this.cursor = Math.min(n, this.cursor + 1)
        else if (key === 'Home') this.cursor = chars(b.slice(0, b.lastIndexOf('\n') + 1)).length
        else {
          const nl = this.after.indexOf('\n')
          this.cursor = nl < 0 ? n : this.cursor + chars(this.after.slice(0, nl)).length
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
    if (this.mode === 'hangul' && /[a-zA-Z]/.test(key)) {
      if (brk && !this.pre) this.#space()
      this.composer.input(key)
      this.#sync()
      return true
    }
    if (this.pre && this.hangulOnly) {
      this.commit()
      this.#insert(key)
    } else if (this.pre) {
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
        else if (this.open && this.layout === 'horizontal') this.cycle(1)
        else this.move(1)
        return true
      case 'ArrowLeft':
        if (shift) this.resize(-1)
        else if (this.open && this.layout === 'horizontal') this.cycle(-1)
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
