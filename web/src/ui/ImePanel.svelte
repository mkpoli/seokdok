<script>
  import { t, r, p } from '../lib/i18n.svelte.js'
  import { engine, status } from '../lib/engine.svelte.js'
  import { options, saveOptions } from '../lib/options.svelte.js'
  import Keyboard from './Keyboard.svelte'
  import Ruby from './Ruby.svelte'

  /** @type {{ ime: import('../lib/ime.svelte.js').Ime }} */
  let { ime } = $props()

  let sink = $state()
  let focused = $state(false)
  let showRuby = $state(false)
  let showKeys = $state(matchMedia('(pointer: coarse)').matches)
  let preview = $state('')
  let reading = $state('')
  let copied = $state(false)

  // Live preview of the preedit, like a live-conversion IME.
  let timer
  $effect(() => {
    const pre = ime.pre
    const ready = status.ready
    clearTimeout(timer)
    if (!pre || ime.segs || !ready) {
      preview = ''
      return
    }
    timer = setTimeout(async () => {
      const r = await engine.convert(pre)
      if (ime.pre === pre) preview = r[0]?.text ?? ''
    }, 90)
  })

  // Options go to the engine; an open conversion is redone under them.
  $effect(() => {
    const o = { ...options }
    saveOptions()
    engine.options(o).then(() => {
      if (ime.segs) ime.convert()
    })
  })

  function onkeydown(e) {
    if (e.isComposing || e.key === 'Process' || e.keyCode === 229) return
    if (ime.key({ key: e.key, shift: e.shiftKey, ctrl: e.ctrlKey, alt: e.altKey, meta: e.metaKey })) {
      e.preventDefault()
    }
  }

  function oncompositionupdate(e) {
    ime.osComposing = e.data ?? ''
  }

  function oncompositionend(e) {
    ime.osComposing = ''
    if (e.data) ime.insertText(e.data)
    sink.value = ''
  }

  // Text that arrives without a composition (mobile keyboards, dictation).
  function oninput(e) {
    if (e.isComposing) return
    const v = sink.value
    sink.value = ''
    if (!v) return
    if (/[가-힣ㄱ-ㆎ]/.test(v)) ime.insertText(v)
    else for (const c of v) ime.key({ key: c })
  }

  function onpaste(e) {
    e.preventDefault()
    const text = e.clipboardData?.getData('text') ?? ''
    if (text) ime.insertText(text)
  }

  async function copy() {
    if (ime.segs || ime.pre) ime.commit()
    await navigator.clipboard.writeText(ime.doc)
    copied = true
    setTimeout(() => (copied = false), 1400)
  }

  async function readBack() {
    if (ime.segs || ime.pre) ime.commit()
    reading = await engine.read(ime.doc)
  }

  function clear() {
    ime.cancel()
    ime.composer.clear()
    ime.insertText('')
    ime.doc = ''
    reading = ''
  }

  const pct = $derived(Math.min(100, Math.round((status.loaded / status.total) * 100)))
  const empty = $derived(!ime.doc && !ime.pre && !ime.segs && !ime.osComposing)
</script>

<section class="section" id="ime">
  <p class="eyebrow">{@html r(t().ime.eyebrow)}</p>
  <h2 class="h2">{@html r(t().ime.title)}</h2>
  <p class="lede">{@html r(t().ime.lede)}</p>

  <div class="panel card">
    <div class="toolbar">
      <div class="seg" role="group" aria-label="input">
        <button aria-pressed={ime.mode === 'sim'} onclick={() => (ime.mode = 'sim')}>{@html r(t().ime.sim)}</button>
        <button aria-pressed={ime.mode === 'os'} onclick={() => (ime.mode = 'os')}>{@html r(t().ime.os)}</button>
      </div>
      <label class="switch"><input type="checkbox" bind:checked={showRuby} />{@html r(t().ime.ruby)}</label>
      <label class="switch"><input type="checkbox" bind:checked={showKeys} />{@html r(t().ime.keyboard)}</label>
      <details class="opts">
        <summary class="btn" data-size="sm">{@html r(t().options.title)}</summary>
        <div class="opts-body card">
          <fieldset>
            <legend>{@html r(t().options.style)}</legend>
            <label><input type="radio" bind:group={options.style} value="surface" />{@html r(t().options.surface)}</label>
            <label><input type="radio" bind:group={options.style} value="morpheme" />{@html r(t().options.morpheme)}</label>
          </fieldset>
          <fieldset>
            <legend>{@html r(t().options.fragment)}</legend>
            <label><input type="radio" bind:group={options.fragment} value="absorb" />{@html r(t().options.absorb)}</label>
            <label><input type="radio" bind:group={options.fragment} value="letter" />{@html r(t().options.letter)}</label>
          </fieldset>
          <label class="switch"><input type="checkbox" bind:checked={options.codaAlways} />{@html r(t().options.coda)}</label>
          <label class="switch"><input type="checkbox" bind:checked={options.resyllabify} />{@html r(t().options.resyllabify)}</label>
          <label class="switch"><input type="checkbox" bind:checked={options.hadaKun} />{@html r(t().options.hada)}</label>
        </div>
      </details>
    </div>

    <div class="editor" class:focused role="presentation" onclick={() => sink.focus()}>
      <textarea
        class="sink"
        bind:this={sink}
        aria-label={p(t().ime.title)}
        autocapitalize="off"
        autocomplete="off"
        spellcheck="false"
        {onkeydown}
        {oncompositionupdate}
        {oncompositionend}
        {oninput}
        {onpaste}
        onfocus={() => (focused = true)}
        onblur={() => (focused = false)}
      ></textarea>
      <div class="text" lang="ko">
        {#if empty}
          <span class="placeholder">{@html r(t().ime.placeholder)}</span>
        {/if}
        {#if showRuby}<Ruby text={ime.doc} />{:else}{ime.doc}{/if}
        {#if ime.segs}
          {#each ime.segs as s, i}
            <span class="segment" class:focus={i === ime.focus && s.candidates.length > 1} class:multi={s.candidates.length > 1}
              >{s.candidates[s.pick]?.text ?? s.reading}</span
            >
          {/each}
        {:else}
          <span class="pre">{ime.pre}{ime.osComposing}</span>
        {/if}
        {#if focused}<span class="caret"></span>{/if}
      </div>
    </div>

    {#if ime.segs && ime.current}
      <div class="cands" role="listbox" aria-label={ime.current.reading}>
        {#each ime.current.candidates.slice(Math.floor(ime.current.pick / 9) * 9, Math.floor(ime.current.pick / 9) * 9 + 9) as c, i}
          {@const idx = Math.floor(ime.current.pick / 9) * 9 + i}
          <button
            class="cand"
            role="option"
            aria-selected={idx === ime.current.pick}
            onclick={() => {
              ime.pick(idx)
              sink.focus()
            }}
          >
            <span class="num">{i + 1}</span>{c.text}
          </button>
        {/each}
      </div>
      <p class="note">{ime.current.candidates[ime.current.pick]?.note}</p>
    {:else if preview && preview !== ime.pre}
      <p class="preview"><span>{@html r(t().ime.preview)}</span> {preview}</p>
    {/if}

    <div class="foot">
      <ul class="hints">
        <li>{@html r(t().ime.keys.space)}</li>
        <li>{@html r(t().ime.keys.arrows)}</li>
        <li>{@html r(t().ime.keys.enter)}</li>
        <li>{@html r(t().ime.keys.esc)}</li>
        <li>{@html r(t().ime.keys.shiftSpace)}</li>
      </ul>
      <div class="actions">
        {#if !status.ready}
          <span class="loading" title={status.failed || p(t().ime.loading)}>
            <span>{@html r(status.failed ? t().ime.failed : t().ime.loading)}{status.failed ? '' : ` ${pct}%`}</span>
            <span class="bar"><span style:width="{pct}%"></span></span>
          </span>
        {/if}
        <button class="btn" onclick={readBack} disabled={!status.ready}>{@html r(t().ime.read)}</button>
        <button class="btn" onclick={copy}>{@html r(copied ? t().ime.copied : t().ime.copy)}</button>
        <button class="btn" onclick={clear}>{@html r(t().ime.clear)}</button>
      </div>
    </div>
    {#if reading}
      <p class="reading" lang="ko">{reading}</p>
    {/if}
  </div>

  {#if showKeys}
    <Keyboard {ime} focus={() => sink.focus()} />
  {/if}
</section>

<style>
  .panel {
    padding: var(--s-4);
  }
  .toolbar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--s-3) var(--s-4);
    margin-bottom: var(--s-3);
  }
  .opts {
    position: relative;
    margin-left: auto;
  }
  .opts summary {
    list-style: none;
  }
  .opts summary::-webkit-details-marker {
    display: none;
  }
  .opts-body {
    position: absolute;
    right: 0;
    top: calc(100% + 6px);
    z-index: 10;
    display: grid;
    gap: var(--s-3);
    width: min(340px, 86vw);
    padding: var(--s-4);
    box-shadow: var(--shadow-2);
    font-size: 14px;
  }
  fieldset {
    display: grid;
    gap: var(--s-1);
    margin: 0;
    padding: 0;
    border: 0;
  }
  legend {
    margin-bottom: var(--s-1);
    color: var(--c-muted);
    font-weight: 700;
    font-size: 12px;
  }
  fieldset label {
    display: flex;
    gap: var(--s-2);
    align-items: center;
    cursor: pointer;
  }
  input[type='radio'] {
    accent-color: var(--c-accent);
  }

  .editor {
    position: relative;
    min-height: 168px;
    padding: var(--s-4) var(--s-5);
    border: 1px solid var(--c-border);
    border-radius: var(--r-2);
    background: var(--c-sunken);
    cursor: text;
    transition: border-color var(--t-fast);
  }
  .editor.focused {
    border-color: var(--c-accent);
  }
  .sink {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    opacity: 0;
    resize: none;
    border: 0;
    caret-color: transparent;
    font-size: 16px;
  }
  .text {
    position: relative;
    pointer-events: none;
    white-space: pre-wrap;
    word-break: break-word;
    font: 500 clamp(22px, 2.6vw, 28px) / 2.1 var(--f-serif);
  }
  .placeholder {
    color: var(--c-muted);
    font: 400 15px/1.7 var(--f-sans);
  }
  .pre {
    text-decoration: underline dotted var(--c-preedit);
    text-decoration-thickness: 2px;
    text-underline-offset: 6px;
  }
  .segment {
    border-bottom: 2px solid var(--c-border-strong);
    padding-inline: 1px;
  }
  .segment.multi {
    border-bottom-color: var(--c-accent);
  }
  .segment.focus {
    background: var(--c-accent);
    color: var(--c-on-accent);
    border-radius: 4px;
  }
  .caret {
    display: inline-block;
    width: 2px;
    height: 1.1em;
    margin-left: 1px;
    vertical-align: -0.15em;
    background: var(--c-accent);
    animation: blink 1s steps(1) infinite;
  }
  @keyframes blink {
    50% {
      opacity: 0;
    }
  }

  .cands {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-1);
    margin-top: var(--s-3);
  }
  .cand {
    display: inline-flex;
    align-items: baseline;
    gap: var(--s-2);
    padding: var(--s-1) var(--s-3);
    border: 1px solid var(--c-border);
    border-radius: var(--r-1);
    background: var(--c-surface);
    color: var(--c-ink);
    font: 500 20px/1.5 var(--f-serif);
    cursor: pointer;
  }
  .cand[aria-selected='true'] {
    border-color: var(--c-accent);
    background: var(--c-accent-soft);
    color: var(--c-accent);
  }
  .num {
    color: var(--c-muted);
    font: 700 12px/1 var(--f-sans);
  }
  .note {
    min-height: 1.5em;
    margin: var(--s-2) 0 0;
    color: var(--c-muted);
    font-size: 13px;
  }
  .preview {
    margin: var(--s-3) 0 0;
    color: var(--c-ruby);
    font: 500 20px/1.5 var(--f-serif);
  }
  .preview span {
    margin-right: var(--s-2);
    color: var(--c-muted);
    font: 700 12px/1 var(--f-sans);
  }
  .foot {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-3);
    margin-top: var(--s-4);
  }
  .hints {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-1) var(--s-3);
    margin: 0;
    padding: 0;
    list-style: none;
    color: var(--c-muted);
    font-size: 12.5px;
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--s-2);
  }
  .actions .btn:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .loading {
    display: inline-flex;
    flex-direction: column;
    gap: 4px;
    color: var(--c-muted);
    font-size: 12px;
  }
  .bar {
    width: 140px;
    height: 4px;
    border-radius: 2px;
    background: var(--c-border);
    overflow: hidden;
  }
  .bar span {
    display: block;
    height: 100%;
    background: var(--c-accent);
    transition: width var(--t-med);
  }
  .reading {
    margin: var(--s-3) 0 0;
    padding: var(--s-3) var(--s-4);
    border-left: 3px solid var(--c-accent);
    background: var(--c-accent-soft);
    border-radius: 0 var(--r-1) var(--r-1) 0;
    white-space: pre-wrap;
    font-size: 17px;
  }
</style>
