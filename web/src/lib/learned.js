// What the reader keeps picking: the input methods' own table (the wasm
// Learned, crates/seokdok-session/src/learned.rs) kept in localStorage.
// Nothing here leaves the browser.

export const STORAGE_KEY = 'seokdok:learned'
const SAVE_DELAY = 1000

export class Picks {
  #Table
  #table
  #storage
  #timer = null

  /**
   * @param {any} Table the wasm class
   * @param {Storage | null} [storage] null keeps the table in memory only
   */
  constructor(Table, storage = null) {
    this.#Table = Table
    this.#storage = storage
    this.#table = this.#read()
  }

  /** The stored table; one that is missing or does not parse is an empty one. */
  #read() {
    let text = ''
    try {
      text = this.#storage?.getItem(STORAGE_KEY) ?? ''
    } catch {}
    return new this.#Table(text)
  }

  get empty() {
    return this.#table.isEmpty()
  }

  /** @param {string} reading */
  promotes(reading) {
    return this.#table.promotes(reading)
  }

  /**
   * Move the learned form of this reading to the front of `candidates`.
   * @param {string} reading
   * @param {{ text: string }[]} candidates
   * @returns {boolean} a learned form now leads
   */
  rank(reading, candidates) {
    if (this.empty) return false
    const at = this.#table.rankIndex(reading, JSON.stringify(candidates.map((c) => c.text)))
    if (at < 0) return false
    if (at > 0) candidates.unshift(...candidates.splice(at, 1))
    return true
  }

  /**
   * The runs of adjacent segments to join.
   * @param {string[]} readings
   * @returns {[number, number][]}
   */
  joins(readings) {
    return this.empty ? [] : JSON.parse(this.#table.joins(JSON.stringify(readings)))
  }

  /**
   * @param {string} reading
   * @param {string} text the form committed
   * @param {boolean} first it was the first candidate shown
   */
  commit(reading, text, first) {
    if (this.#table.commit(reading, text, first)) this.#saveSoon()
  }

  /** Empty the table and the stored copy. */
  forget() {
    clearTimeout(this.#timer)
    this.#timer = null
    this.#table = new this.#Table('')
    try {
      this.#storage?.removeItem(STORAGE_KEY)
    } catch {}
  }

  /** Write what is waiting, as the page goes away. */
  flush() {
    if (this.#timer !== null) this.#save()
  }

  /** Another tab changed or cleared the stored table. */
  reload() {
    this.#table = this.#read()
  }

  #saveSoon() {
    if (this.#timer === null) this.#timer = setTimeout(() => this.#save(), SAVE_DELAY)
  }

  #save() {
    clearTimeout(this.#timer)
    this.#timer = null
    try {
      this.#storage?.setItem(STORAGE_KEY, this.#table.toText())
    } catch {}
  }
}
