// Spelling options, remembered per browser.
const DEFAULTS = { spaceMode: 'convert', layout: 'vertical', style: 'surface', fragment: 'absorb', codaAlways: false, resyllabify: false, hadaKun: false }

function load() {
  try {
    return { ...DEFAULTS, ...JSON.parse(localStorage.getItem('seokdok:options') ?? '{}') }
  } catch {
    return { ...DEFAULTS }
  }
}

export const options = $state(load())
export { DEFAULTS }

export function saveOptions() {
  try {
    localStorage.setItem('seokdok:options', JSON.stringify(options))
  } catch {}
}
