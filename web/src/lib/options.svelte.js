// Spelling options, remembered per browser.
// script: 'seokdok' (釋讀), 'sino' (國漢文混用: Hanja for 音讀 words only) or
// 'hangul' (韓㐎 專用: nothing is converted).
// The 音讀 page (data-product on its <html>) keeps to its own key and script.
const EUMDOK = typeof document !== 'undefined' && document.documentElement?.dataset?.product === 'eumdok'
const PAGE = EUMDOK ? { key: 'eumdok:options', script: 'sino' } : { key: 'seokdok:options', script: 'seokdok' }
const DEFAULTS = { rubyPosition: 'above', spaceMode: 'convert', layout: 'vertical', oldClusters: false, learn: true, style: 'surface', fragment: 'absorb', codaAlways: false, resyllabify: false, hadaKun: false, itda: '有', oldHa: false, script: PAGE.script }

function load() {
  let stored = {}
  try {
    stored = JSON.parse(localStorage.getItem(PAGE.key) ?? '{}')
  } catch {}
  if (!stored || typeof stored !== 'object' || Array.isArray(stored)) stored = {}
  const o = { ...DEFAULTS }
  for (const k of Object.keys(DEFAULTS)) if (k in stored) o[k] = stored[k]
  if (!(EUMDOK ? ['sino', 'hangul'] : ['seokdok', 'sino', 'hangul']).includes(o.script)) o.script = DEFAULTS.script
  if (!['有', '在', '在ㅆ'].includes(o.itda)) o.itda = DEFAULTS.itda
  if (!['above', 'right'].includes(o.rubyPosition)) o.rubyPosition = DEFAULTS.rubyPosition
  // The engine's own flag, which the site's 釋讀 / 國漢文混用 switch sets.
  // Turning it off leaves 韓㐎 專用 alone.
  Object.defineProperty(o, 'sinoOnly', {
    enumerable: true,
    get() {
      return this.script === 'sino'
    },
    set(v) {
      if (v) this.script = 'sino'
      else if (this.script === 'sino') this.script = 'seokdok'
    },
  })
  return o
}

export const options = $state(load())

/** Where readings sit, on every page: `above` as furigana, `right` as 文書문서. */
export function showRubyPosition() {
  if (typeof document !== 'undefined') document.documentElement.dataset.ruby = options.rubyPosition
}
showRubyPosition()
export { DEFAULTS }

export function saveOptions() {
  try {
    const { sinoOnly, ...stored } = options
    localStorage.setItem(PAGE.key, JSON.stringify(stored))
  } catch {}
}
