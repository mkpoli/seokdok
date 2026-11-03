<script>
  import { t, r, plain } from '../lib/i18n.svelte.js'
  import { engine, status } from '../lib/engine.svelte.js'
  import { custom, compiled, save, userTsv, input, output, exportJson, importJson } from '../lib/custom.svelte.js'
  import { PRESETS } from '../lib/rules.js'
  import { reveal } from '../lib/reveal.js'

  let tab = $state('rules')
  let sample = $state('봤다 했다 있는 하늘')
  let result = $state([])
  let userErrors = $state([])
  let importText = $state('')
  let importError = $state('')
  let copied = $state(false)

  // Persist on every change; rebuild the engine when words change.
  let lastUser = ''
  let timer
  $effect(() => {
    void JSON.stringify(custom)
    save()
    const [n, s] = userTsv()
    const key = n + '\u0000' + s
    if (!status.ready || key === lastUser) return
    clearTimeout(timer)
    timer = setTimeout(async () => {
      lastUser = key
      userErrors = await engine.user(n, s)
    }, 250)
  })

  // The test box: the sample converted with the rules applied.
  $effect(() => {
    void compiled.value
    void lastUser
    const text = sample
    if (!status.ready || !text.trim()) return
    engine.convert(input(text)).then((c) => (result = output(c).slice(0, 8)))
  })

  const errorAt = (i) => compiled.value.errors.find((e) => e.index === i)?.message

  function addRule(rule) {
    custom.rules.push(rule ?? { on: true, stage: 'output', mode: 'xform', regex: false, pattern: '', replace: '', flags: 'g', note: '' })
  }
  function move(list, i, d) {
    const j = i + d
    if (j < 0 || j >= list.length) return
    ;[list[i], list[j]] = [list[j], list[i]]
  }
  async function copyJson() {
    await navigator.clipboard.writeText(exportJson())
    copied = true
    setTimeout(() => (copied = false), 1400)
  }
  function download() {
    const a = document.createElement('a')
    a.href = URL.createObjectURL(new Blob([exportJson()], { type: 'application/json' }))
    a.download = 'seokdok-custom.json'
    a.click()
    URL.revokeObjectURL(a.href)
  }
  function doImport() {
    try {
      importJson(importText)
      importError = ''
      importText = ''
    } catch (e) {
      importError = String(e.message ?? e)
    }
  }
  async function readFile(e) {
    const f = e.currentTarget.files?.[0]
    if (f) importText = await f.text()
  }

  const tabs = ['rules', 'phrases', 'words', 'share']
  const POS = ['noun', 'pron', 'adv', 'det', 'verb', 'adj']
</script>

<section class="section" id="custom">
  <p class="eyebrow" use:reveal>{@html r(t().custom.eyebrow)}</p>
  <h2 class="h2" use:reveal={1}>{@html r(t().custom.title)}</h2>
  <p class="lede" use:reveal={2}>{@html r(t().custom.lede)}</p>

  <div class="panel card" use:reveal={3}>
    <div class="seg tabs" role="tablist" aria-label={plain(t().custom.title)}>
      {#each tabs as k}
        <button role="tab" aria-selected={tab === k} aria-pressed={tab === k} onclick={() => (tab = k)}>
          {@html r(t().custom.tabs[k])}
          {#if k === 'rules' && custom.rules.length}<span class="count">{custom.rules.length}</span>{/if}
          {#if k === 'phrases' && custom.phrases.length}<span class="count">{custom.phrases.length}</span>{/if}
          {#if k === 'words' && custom.words.length}<span class="count">{custom.words.length}</span>{/if}
        </button>
      {/each}
    </div>

    {#if tab === 'rules'}
      <p class="help">{@html r(t().custom.rulesHelp)}</p>
      <div class="presets">
        <span class="label">{@html r(t().custom.presets)}</span>
        {#each PRESETS as p}
          <button class="chip" onclick={() => addRule(structuredClone(p.rule))}>+ {p.rule.note}</button>
        {/each}
      </div>
      <ol class="rows">
        {#each custom.rules as rule, i (i)}
          <li class="row rule" class:off={!rule.on}>
            <label class="ui-field" title={plain(t().custom.on)}>
              <input class="ui-switch" type="checkbox" role="switch" bind:checked={rule.on} aria-label={plain(t().custom.on)} />
            </label>
            <span class="ui-select-wrap">
              <select class="ui-select" bind:value={rule.stage} aria-label={plain(t().custom.stage)}>
                <option value="input">{plain(t().custom.stageInput)}</option>
                <option value="output">{plain(t().custom.stageOutput)}</option>
              </select>
            </span>
            <span class="ui-select-wrap">
              <select class="ui-select" bind:value={rule.mode} aria-label={plain(t().custom.mode)}>
                <option value="xform">{plain(t().custom.xform)}</option>
                <option value="derive" disabled={rule.stage === 'input'}>{plain(t().custom.derive)}</option>
              </select>
            </span>
            <input class="ui-input pat" bind:value={rule.pattern} placeholder={plain(t().custom.pattern)} aria-label={plain(t().custom.pattern)} spellcheck="false" />
            <span class="arrow" aria-hidden="true">→</span>
            <input class="ui-input pat" bind:value={rule.replace} placeholder={plain(t().custom.replace)} aria-label={plain(t().custom.replace)} spellcheck="false" />
            <label class="ui-field small"><input class="ui-check" type="checkbox" bind:checked={rule.regex} />regex</label>
            {#if rule.regex}
              <input class="ui-input flags" bind:value={rule.flags} placeholder="gu" aria-label="flags" spellcheck="false" />
            {/if}
            <span class="tools">
              <button class="icon" onclick={() => move(custom.rules, i, -1)} aria-label="↑">↑</button>
              <button class="icon" onclick={() => move(custom.rules, i, 1)} aria-label="↓">↓</button>
              <button class="icon del" onclick={() => custom.rules.splice(i, 1)} aria-label={plain(t().custom.remove)}>×</button>
            </span>
            {#if errorAt(i)}<p class="err">{errorAt(i)}</p>{/if}
          </li>
        {:else}
          <li class="empty">{@html r(t().custom.noRules)}</li>
        {/each}
      </ol>
      <button class="btn" onclick={() => addRule()}>+ {@html r(t().custom.addRule)}</button>

      <div class="test">
        <label for="rule-test">{@html r(t().custom.test)}</label>
        <input id="rule-test" class="ui-input" bind:value={sample} lang="ko" />
        <div class="result" lang="ko">
          {#each result as c, i}<span class="cand" class:first={i === 0}>{c.text}</span>{/each}
        </div>
      </div>
    {:else if tab === 'phrases'}
      <p class="help">{@html r(t().custom.phrasesHelp)}</p>
      <ol class="rows">
        {#each custom.phrases as ph, i (i)}
          <li class="row" class:off={ph.on === false}>
            <label class="ui-field"><input class="ui-switch" type="checkbox" role="switch" checked={ph.on !== false} onchange={(e) => (ph.on = e.currentTarget.checked)} aria-label={plain(t().custom.on)} /></label>
            <input class="ui-input" bind:value={ph.reading} placeholder="석독회" aria-label={plain(t().custom.reading)} lang="ko" />
            <span class="arrow" aria-hidden="true">→</span>
            <input class="ui-input" bind:value={ph.text} placeholder="釋讀會" aria-label={plain(t().custom.text)} lang="ko" />
            <span class="tools"><button class="icon del" onclick={() => custom.phrases.splice(i, 1)} aria-label={plain(t().custom.remove)}>×</button></span>
          </li>
        {:else}
          <li class="empty">{@html r(t().custom.noPhrases)}</li>
        {/each}
      </ol>
      <button class="btn" onclick={() => custom.phrases.push({ on: true, reading: '', text: '' })}>+ {@html r(t().custom.addPhrase)}</button>
    {:else if tab === 'words'}
      <p class="help">{@html r(t().custom.wordsHelp)}</p>
      <ol class="rows">
        {#each custom.words as w, i (i)}
          <li class="row" class:off={w.on === false}>
            <label class="ui-field"><input class="ui-switch" type="checkbox" role="switch" checked={w.on !== false} onchange={(e) => (w.on = e.currentTarget.checked)} aria-label={plain(t().custom.on)} /></label>
            <input class="ui-input" bind:value={w.lemma} placeholder="짓다" aria-label={plain(t().custom.lemma)} lang="ko" />
            <span class="arrow" aria-hidden="true">→</span>
            <input class="ui-input" bind:value={w.written} placeholder="作다" aria-label={plain(t().custom.written)} lang="ko" />
            <span class="ui-select-wrap">
              <select class="ui-select" bind:value={w.pos} aria-label={plain(t().custom.pos)}>
                {#each POS as p}<option value={p}>{plain(t().custom.posNames[p])}</option>{/each}
              </select>
            </span>
            <span class="tools"><button class="icon del" onclick={() => custom.words.splice(i, 1)} aria-label={plain(t().custom.remove)}>×</button></span>
          </li>
        {:else}
          <li class="empty">{@html r(t().custom.noWords)}</li>
        {/each}
      </ol>
      <button class="btn" onclick={() => custom.words.push({ on: true, lemma: '', written: '', pos: 'verb' })}>+ {@html r(t().custom.addWord)}</button>
    {:else}
      <p class="help">{@html r(t().custom.shareHelp)}</p>
      <div class="share">
        <div class="actions">
          <button class="btn" data-variant="primary" onclick={download}>{@html r(t().custom.download)}</button>
          <button class="btn" onclick={copyJson}>{@html r(copied ? t().ime.copied : t().custom.copyJson)}</button>
        </div>
        <label class="file btn">
          {@html r(t().custom.openFile)}
          <input type="file" accept="application/json,.json" onchange={readFile} class="visually-hidden" />
        </label>
        <textarea bind:value={importText} rows="6" placeholder={'{ "format": "seokdok-custom", … }'} aria-label={plain(t().custom.import)} spellcheck="false"></textarea>
        <button class="btn" onclick={doImport} disabled={!importText.trim()}>{@html r(t().custom.import)}</button>
        {#if importError}<p class="err">{importError}</p>{/if}
      </div>
    {/if}

    {#if userErrors.length && (tab === 'words' || tab === 'phrases')}
      <ul class="err-list">
        {#each userErrors as e}<li>{e}</li>{/each}
      </ul>
    {/if}
  </div>
</section>

<style>
  .panel {
    display: grid;
    gap: var(--s-4);
    padding: var(--s-5);
  }
  .tabs {
    justify-self: start;
    flex-wrap: wrap;
  }
  .tabs button {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .count {
    min-width: 18px;
    padding: 1px 5px;
    border-radius: 999px;
    background: var(--c-accent-soft);
    color: var(--c-accent);
    font-size: 11px;
  }
  .help {
    margin: 0;
    color: var(--c-soft);
    font-size: 14.5px;
    max-width: 75ch;
  }
  .presets {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--s-2);
  }
  .label {
    color: var(--c-muted);
    font-size: 13px;
    font-weight: 700;
  }
  .chip {
    padding: 4px 10px;
    border: 1px dashed var(--c-border-strong);
    border-radius: 999px;
    background: var(--c-surface);
    color: var(--c-soft);
    font: 500 13px/1.4 var(--f-sans);
    cursor: pointer;
  }
  .chip:hover {
    border-style: solid;
    border-color: var(--c-accent);
    color: var(--c-accent);
  }
  .rows {
    display: grid;
    gap: var(--s-2);
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-2) var(--s-3);
    border: 1px solid var(--c-border);
    border-radius: var(--r-2);
    background: var(--c-sunken);
    transition: opacity var(--t-fast);
  }
  .row.off {
    opacity: 0.55;
  }
  .row .ui-input {
    height: 34px;
    border-radius: var(--r-1);
    min-width: 0;
    flex: 1 1 120px;
  }
  .row .pat {
    font-family: var(--f-mono), var(--f-serif);
    font-size: 14px;
  }
  .row .flags {
    flex: 0 0 56px;
    font-family: var(--f-mono);
  }
  .small {
    font-size: 12.5px;
    min-height: 0;
  }
  .arrow {
    color: var(--c-muted);
  }
  .tools {
    display: inline-flex;
    gap: 2px;
    margin-left: auto;
  }
  .icon {
    width: 28px;
    height: 28px;
    border: 1px solid transparent;
    border-radius: var(--r-1);
    background: none;
    color: var(--c-muted);
    cursor: pointer;
  }
  .icon:hover {
    border-color: var(--c-border);
    color: var(--c-ink);
  }
  .del:hover {
    color: var(--c-preedit);
  }
  .empty {
    color: var(--c-muted);
    font-size: 14px;
    padding: var(--s-2) 0;
  }
  .err {
    flex-basis: 100%;
    margin: 0;
    color: var(--c-preedit);
    font: 12.5px var(--f-mono);
  }
  .err-list {
    margin: 0;
    padding-left: 1.2em;
    color: var(--c-preedit);
    font: 12.5px var(--f-mono);
  }
  .test {
    display: grid;
    gap: var(--s-2);
    padding-top: var(--s-4);
    border-top: 1px solid var(--c-border);
  }
  .test label {
    color: var(--c-muted);
    font-size: 13px;
    font-weight: 700;
  }
  .test .ui-input {
    max-width: 420px;
  }
  .result {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
  }
  .result .cand {
    padding: 2px var(--s-3);
    border: 1px solid var(--c-border);
    border-radius: var(--r-1);
    background: var(--c-surface);
    font: 500 19px/1.6 var(--f-serif);
  }
  .result .first {
    border-color: var(--c-accent);
    color: var(--c-accent);
  }
  .share {
    display: grid;
    gap: var(--s-3);
    justify-items: start;
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
  }
  .file {
    cursor: pointer;
  }
  textarea {
    width: 100%;
    padding: var(--s-3);
    border: 1px solid var(--c-border);
    border-radius: var(--r-2);
    background: var(--c-sunken);
    color: var(--c-ink);
    font: 13px/1.5 var(--f-mono);
  }
</style>
