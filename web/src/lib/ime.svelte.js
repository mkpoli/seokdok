// The in-page input method: the same behaviour as the Windows IME.
//
// Typing builds Hangul in the preedit (두벌식 from QWERTY keys, or text from
// the reader's own Korean IME). Space converts it into segments; Space or ↓
// cycles the focused segment's candidates, ←/→ move between segments, a digit
// picks, Enter commits, Esc returns to Hangul. Typing on commits and goes on.

/** @typedef {{ text: string, note: string }} Candidate */
/** @typedef {{ reading: string, candidates: Candidate[], pick: number }} Seg */

const HANGUL = /[가-힣]/

export class Ime {
  /** Committed text. */
  doc = $state('')
  /** Hangul being composed. */
  pre = $state('')
  /** Text the reader's own IME is still composing. */
  osComposing = $state('')
  /** @type {Seg[] | null} */
  segs = $state(null)
  focus = $state(0)
  open = $state(false)
  /** 'sim': QWERTY letters type 두벌식; 'os': letters pass through. */
  mode = $state('sim')
  /** Key last pressed, for the on-screen keyboard. */
  lastKey = $state('')
  busy = $state(false)

  /**
   * @param {{ input(k: string): boolean, backspace(): boolean, pushLiteral(s: string): void,
   *           text(): string, setText(s: string): void, clear(): void }} composer
   * @param {(text: string) => Promise<{ reading: string, candidates: Candidate[] }[]>} segment
   */
  constructor(composer, segment) {
    this.composer = composer
    this.segment = segment
  }

  /** Index of the focusable segments (more than one candidate). */
  get focusable() {
    return (this.segs ?? []).map((s, i) => (s.candidates.length > 1 ? i : -1)).filter((i) => i >= 0)
  }

  get current() {
    return this.segs?.[this.focus] ?? null
  }

  /** The text of the conversion as currently picked. */
  get converted() {
    return (this.segs ?? []).map((s) => s.candidates[s.pick]?.text ?? s.reading).join('')
  }

  #sync() {
    this.pre = this.composer.text()
  }

  async convert() {
    if (!this.pre || !HANGUL.test(this.pre)) return false
    this.busy = true
    try {
      const segs = await this.segment(this.pre)
      this.segs = segs.map((s) => ({ ...s, pick: 0 }))
      this.focus = this.focusable[0] ?? 0
      this.open = false
      return true
    } finally {
      this.busy = false
    }
  }

  commit() {
    if (this.segs) this.doc += this.converted
    else this.doc += this.pre
    this.segs = null
    this.composer.clear()
    this.#sync()
    this.open = false
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
    s.pick = (s.pick + d + n) % n
    this.open = true
  }

  /** @param {number} d */
  move(d) {
    const f = this.focusable
    if (!f.length) return
    const i = f.indexOf(this.focus)
    this.focus = f[Math.max(0, Math.min(f.length - 1, i + d))]
    this.open = false
  }

  /** Pick candidate `i` of the focused segment and go on to the next one. */
  pick(i) {
    const s = this.current
    if (!s || i >= s.candidates.length) return
    s.pick = i
    const f = this.focusable
    const at = f.indexOf(this.focus)
    if (at < f.length - 1) {
      this.focus = f[at + 1]
      this.open = false
    } else {
      this.commit()
    }
  }

  /** Text from the reader's own IME (composition end, paste). */
  insertText(s) {
    if (this.segs) this.commit()
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
    if (this.segs) {
      switch (key) {
        case ' ':
          if (k.shift) {
            this.commit()
            this.doc += ' '
          } else this.cycle(1)
          return true
        case 'ArrowDown':
        case 'Tab':
          this.cycle(1)
          return true
        case 'ArrowUp':
          this.cycle(-1)
          return true
        case 'ArrowRight':
          this.move(1)
          return true
        case 'ArrowLeft':
          this.move(-1)
          return true
        case 'Enter':
          this.commit()
          return true
        case 'Escape':
        case 'Backspace':
          this.cancel()
          return true
      }
      if (this.open && /^[1-9]$/.test(key)) {
        const s = this.current
        const page = Math.floor(s.pick / 9)
        this.pick(page * 9 + Number(key) - 1)
        return true
      }
      if (key.length === 1) {
        this.commit()
        return this.key(k)
      }
      return false
    }
    switch (key) {
      case ' ':
        if (k.shift && this.pre) {
          this.composer.pushLiteral(' ')
          this.#sync()
        } else if (this.pre) {
          this.convert()
        } else {
          this.doc += ' '
        }
        return true
      case 'Backspace':
        if (this.pre) {
          this.composer.backspace()
          this.#sync()
        } else if (this.doc) {
          this.doc = [...this.doc].slice(0, -1).join('')
        } else return false
        return true
      case 'Enter':
        if (this.pre) this.commit()
        else this.doc += '\n'
        return true
      case 'Escape':
        if (!this.pre) return false
        this.composer.clear()
        this.#sync()
        return true
      case 'Tab':
        if (!this.pre) return false
        this.convert()
        return true
    }
    if (key.length !== 1) return false
    if (this.mode === 'sim' && /[a-zA-Z]/.test(key)) {
      this.composer.input(key)
      this.#sync()
      return true
    }
    if (this.pre) {
      this.composer.pushLiteral(key)
      this.#sync()
    } else {
      this.doc += key
    }
    return true
  }
}
