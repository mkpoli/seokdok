// Custom rewrite rules, after RIME's spelling algebra and filters.
//
// A rule is { on, stage, mode, regex, pattern, replace, flags }:
//   stage 'input'   rewrites the Hangul before it is converted (RIME speller)
//   stage 'output'  rewrites every candidate after conversion (RIME filter)
//   mode  'xform'   replaces the text
//   mode  'derive'  keeps the original and adds the rewritten text after it
//   regex false     pattern is literal text; true: a JavaScript RegExp source
//                   with `flags` (default "g"), and $1… in `replace`
// Rules run top to bottom; each sees the output of the ones above.

/** @typedef {{ on: boolean, stage: 'input'|'output', mode: 'xform'|'derive', regex: boolean, pattern: string, replace: string, flags?: string, note?: string }} Rule */
/** @typedef {{ text: string, note: string }} Candidate */

const escape = (s) => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')

/**
 * Compile rules; invalid ones are reported and skipped.
 * @param {Rule[]} rules
 * @returns {{ rules: (Rule & { re: RegExp })[], errors: { index: number, message: string }[] }}
 */
export function compile(rules) {
  const out = []
  const errors = []
  rules.forEach((r, index) => {
    if (!r.on || !r.pattern) return
    try {
      const flags = r.regex ? (r.flags ?? 'g') : 'g'
      const re = new RegExp(r.regex ? r.pattern : escape(r.pattern), flags.includes('g') ? flags : flags + 'g')
      out.push({ ...r, re })
    } catch (e) {
      errors.push({ index, message: String(e.message ?? e) })
    }
  })
  return { rules: out, errors }
}

function apply(text, rule) {
  // Literal replacements must not read $1 in the replacement.
  return rule.regex ? text.replace(rule.re, rule.replace) : text.replace(rule.re, () => rule.replace)
}

/** Rewrite the input Hangul with the input-stage xform rules. */
export function rewriteInput(text, compiled) {
  let t = text
  for (const r of compiled.rules) if (r.stage === 'input' && r.mode === 'xform') t = apply(t, r)
  return t
}

/**
 * Rewrite a candidate list with the output-stage rules. `xform` changes a
 * candidate in place; `derive` adds the changed text right after it. The list
 * stays free of duplicates, in order.
 * @param {Candidate[]} cands
 */
export function rewriteOutput(cands, compiled) {
  const rules = compiled.rules.filter((r) => r.stage === 'output')
  if (!rules.length) return cands
  let list = cands.map((c) => ({ ...c }))
  for (const r of rules) {
    const next = []
    for (const c of list) {
      const t = apply(c.text, r)
      if (r.mode === 'xform') next.push(t === c.text ? c : { ...c, text: t })
      else {
        next.push(c)
        if (t !== c.text) next.push({ text: t, note: c.note ? `${c.note} · ${r.note || r.pattern}` : r.note || r.pattern })
      }
    }
    list = next
  }
  const seen = new Set()
  return list.filter((c) => !seen.has(c.text) && seen.add(c.text))
}

/** Rules people asked for or that show the format; added with one click. */
export const PRESETS = [
  {
    key: 'okuri-letter',
    rule: { on: true, stage: 'output', mode: 'derive', regex: true, pattern: '([\\p{Script=Han}])았', replace: '$1ㅏㅆ', flags: 'gu', note: '았 → ㅏㅆ' },
  },
  {
    key: 'okuri-at',
    rule: { on: true, stage: 'output', mode: 'derive', regex: true, pattern: '([\\p{Script=Han}])ㅆ', replace: '$1았', flags: 'gu', note: 'ㅆ → 았' },
  },
  { key: 'wi', rule: { on: true, stage: 'output', mode: 'xform', regex: false, pattern: '爲', replace: '為', note: '爲 → 為' } },
  {
    key: 'arae-a',
    rule: { on: true, stage: 'output', mode: 'derive', regex: false, pattern: '하', replace: 'ᄒᆞ', note: '하 → ᄒᆞ' },
  },
  {
    key: 'iss',
    rule: { on: true, stage: 'output', mode: 'xform', regex: true, pattern: '有(?=[ㅆ쓰써])', replace: '在', flags: 'g', note: '有ㅆ → 在ㅆ' },
  },
  { key: 'typo', rule: { on: true, stage: 'input', mode: 'xform', regex: false, pattern: '읍니다', replace: '습니다', note: '읍니다 → 습니다' } },
]
