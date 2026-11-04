// The reader's customisation: own words and rewrite rules, kept in the browser.
import { compile, rewriteInput, rewriteOutput } from './rules.js'

/**
 * phrases: fixed text for a reading, like RIME's custom_phrase (풀어읽기모임 → 釋讀會)
 * words:   訓讀 entries that inflect (짓다 → 作다), in dict/user.tsv form
 * rules:   rewrite rules, see rules.js
 */
const EMPTY = { phrases: [], words: [], rules: [] }

function load() {
  try {
    return { ...EMPTY, ...JSON.parse(localStorage.getItem('seokdok:custom') ?? '{}') }
  } catch {
    return structuredClone(EMPTY)
  }
}

export const custom = $state(load())
export const compiled = $state({ value: compile(custom.rules) })

export function save() {
  try {
    localStorage.setItem('seokdok:custom', JSON.stringify(custom))
  } catch {}
  compiled.value = compile(custom.rules)
}

/** TSV texts for the engine: [user.tsv, user-sino.tsv]. */
export function userTsv() {
  const clean = (s) => String(s ?? '').replace(/[\t\n]/g, ' ').trim()
  const native = custom.words
    .filter((w) => w.on !== false && w.lemma && w.written)
    .map((w) => [clean(w.lemma), clean(w.written), w.pos || 'noun', '', 'user', '', ''].join('\t'))
    .join('\n')
  const sino = custom.phrases
    .filter((p) => p.on !== false && p.reading && p.text)
    .map((p) => [clean(p.reading), clean(p.text), '', 'user'].join('\t'))
    .join('\n')
  return [native, sino]
}

export const input = (text) => rewriteInput(text, compiled.value)
export const output = (cands) => rewriteOutput(cands, compiled.value)

/** The whole customisation as JSON, for sharing. */
export function exportJson() {
  return JSON.stringify({ format: 'seokdok-custom', version: 1, ...custom }, null, 2)
}

/** Replace the customisation from JSON; throws on a bad file. */
export function importJson(text) {
  const d = JSON.parse(text)
  if (d.format !== 'seokdok-custom') throw new Error('not a seokdok-custom file')
  custom.phrases = d.phrases ?? []
  custom.words = d.words ?? []
  custom.rules = d.rules ?? []
  save()
}
