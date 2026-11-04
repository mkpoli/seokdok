// Spelling options, remembered per browser.
// script: 'seokdok' (釋讀), 'sino' (國漢文混用: Hanja for 音讀 words only) or
// 'hangul' (韓㐎 專用: nothing is converted).
const DEFAULTS = { spaceMode: 'convert', layout: 'vertical', oldClusters: false, style: 'surface', fragment: 'absorb', codaAlways: false, resyllabify: false, hadaKun: false, itda: '有', oldHa: false, script: 'seokdok' }

function load() {
  let stored = {}
  try {
    stored = JSON.parse(localStorage.getItem('seokdok:options') ?? '{}')
  } catch {}
  if (!stored || typeof stored !== 'object' || Array.isArray(stored)) stored = {}
  const o = { ...DEFAULTS }
  for (const k of Object.keys(DEFAULTS)) if (k in stored) o[k] = stored[k]
  if (!['seokdok', 'sino', 'hangul'].includes(o.script)) o.script = DEFAULTS.script
  if (!['有', '在', '在ㅆ'].includes(o.itda)) o.itda = DEFAULTS.itda
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
export { DEFAULTS }

export function saveOptions() {
  try {
    const { sinoOnly, ...stored } = options
    localStorage.setItem('seokdok:options', JSON.stringify(stored))
  } catch {}
}
