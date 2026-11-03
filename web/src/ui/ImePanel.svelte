<script>
  import { reveal } from '../lib/reveal.js'
  import { t, r, plain } from '../lib/i18n.svelte.js'
  import { engine, status } from '../lib/engine.svelte.js'
  import { options, saveOptions } from '../lib/options.svelte.js'
  import Keyboard from './Keyboard.svelte'
  import Ruby from './Ruby.svelte'
  import { PAGE } from '../lib/ime.svelte.js'
  import { input, output } from '../lib/custom.svelte.js'

  /** @type {{ ime: import('../lib/ime.svelte.js').Ime }} */
  let { ime } = $props()

  let sink = $state()
  let textEl = $state()
  let pos = $state({ left: 0, top: 0 })

  // The candidate window sits under the focused segment.
  $effect(() => {
    if (!ime.open || !ime.segs) return
    void ime.focus
    void ime.converted
    queueMicrotask(() => {
      const el = textEl?.querySelector(`[data-seg="${ime.focus}"]`)
      if (!el || !textEl) return
      const host = textEl.parentElement.getBoundingClientRect()
      const r = el.getBoundingClientRect()
      const left = Math.min(r.left - host.left, host.width - 240)
      pos = { left: Math.max(8, left), top: r.bottom - host.top + 6 }
    })
  })

  /** Doc offset (code points) of a DOM point inside a [data-off] span, or null. */
  function offsetOf(node, offset) {
    const el = (node?.nodeType === Node.TEXT_NODE ? node.parentElement : node)?.closest?.('[data-off]')
    if (!el || !textEl?.contains(el)) return null
    let n = Number(el.dataset.off)
    if (node === el) {
      for (const c of [...el.childNodes].slice(0, offset)) n += [...(c.textContent ?? '')].length
      return n
    }
    for (const c of el.childNodes) {
      if (c === node) break
      n += [...(c.textContent ?? '')].length
    }
    return n + [...(node.textContent ?? '').slice(0, offset)].length
  }

  function pointOffset(x, y) {
    const hit = document.caretPositionFromPoint?.(x, y)
    if (hit) return offsetOf(hit.offsetNode, hit.offset)
    const range = document.caretRangeFromPoint?.(x, y)
    return range ? offsetOf(range.startContainer, range.startOffset) : null
  }

  // A drag selects; a click places the caret. Readings (ruby) add text the
  // offsets cannot see, so with them on a click goes to the end.
  function onmouseup(e) {
    if (e.button !== 0) return
    const sel = window.getSelection()
    if (!showRuby && sel && !sel.isCollapsed && textEl?.contains(sel.anchorNode)) {
      const a = offsetOf(sel.anchorNode, sel.anchorOffset)
      const f = offsetOf(sel.focusNode, sel.focusOffset)
      if (a !== null && f !== null) ime.setSelection(a, f)
      sel.removeAllRanges()
    } else if (!ime.segs && !ime.pre) {
      const at = showRuby ? null : pointOffset(e.clientX, e.clientY)
      if (at !== null) ime.setCursor(at)
      else if (e.target === e.currentTarget || e.target === textEl) ime.setCursor([...ime.doc].length)
    }
    sink.focus({ preventScroll: true })
  }

  function ondblclick(e) {
    if (showRuby || ime.segs || ime.pre) return
    const at = pointOffset(e.clientX, e.clientY)
    if (at === null) return
    const chars = [...ime.doc]
    const word = (c) => c && !/[\s.,!?、。，！？]/.test(c)
    let a = at
    let b = at
    while (a > 0 && word(chars[a - 1])) a--
    while (b < chars.length && word(chars[b])) b++
    ime.setSelection(a, b)
    sink.focus({ preventScroll: true })
  }

  // The committed text in pieces around the selection and the caret.
  const pieces = $derived.by(() => {
    const chars = [...ime.doc]
    const n = chars.length
    const sel = ime.selection
    const cut = (a, b, selected = false) => ({ text: chars.slice(a, b).join(''), off: a, selected })
    if (!sel) return { head: [cut(0, ime.cursor)], tail: [cut(ime.cursor, n)] }
    const [a, b] = sel
    const mid = cut(a, b, true)
    return ime.cursor === a
      ? { head: [cut(0, a)], tail: [mid, cut(b, n)] }
      : { head: [cut(0, a), mid], tail: [cut(b, n)] }
  })

  /** "讀 읽다 +아/어 · 訓讀 community" → readable, with the source named. */
  function noteHtml(note) {
    const m = note.match(/^(.*) · (訓讀|音讀)(?: ([\w-]+))?$/)
    if (!m) return r(note)
    const [, what, kind, src] = m
    const k = kind === '音讀' ? t().ime.src.sino : t().ime.src.kun
    const from = src ? ' · ' + r(t().ime.src[src] ?? src) : ''
    return `<b>${r(what)}</b> · ${r(k)}${from}`
  }
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
      const r = output(await engine.convert(input(pre)))
      if (ime.pre === pre) preview = r[0]?.text ?? ''
    }, 90)
  })

  $effect(() => {
    ime.spaceMode = options.spaceMode
    ime.layout = options.layout
  })

  // Options go to the engine; an open conversion is redone under them.
  $effect(() => {
    const o = { ...options }
    saveOptions()
    engine.options(o).then(() => {
      if (ime.segs) ime.convert()
    })
  })

  // Alt pressed and released on its own switches 韓 ↔ A, like a 한/영 key.
  let altAlone = false
  function onkeyup(e) {
    if (e.key === 'Alt' && altAlone) {
      e.preventDefault()
      ime.toggleMode()
    }
    altAlone = false
  }

  function onkeydown(e) {
    if (e.key === 'Alt') {
      altAlone = !e.repeat
      e.preventDefault()
      return
    }
    altAlone = false
    if (e.isComposing || e.key === 'Process' || e.keyCode === 229) return
    const mod = e.ctrlKey || e.metaKey
    if (mod && !e.altKey && (e.key === 'a' || e.key === 'A')) {
      e.preventDefault()
      ime.selectAll()
      return
    }
    if (mod && (e.key === 'c' || e.key === 'x') && ime.selection) {
      e.preventDefault()
      navigator.clipboard.writeText(ime.selectedText)
      if (e.key === 'x') ime.deleteSelection()
      return
    }
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
    ime.setDoc('')
    reading = ''
  }

  const pct = $derived(Math.min(100, Math.round((status.loaded / status.total) * 100)))
  const empty = $derived(!ime.doc && !ime.pre && !ime.segs && !ime.osComposing)
</script>

<section class="section" id="ime">
  <p class="eyebrow" use:reveal>{@html r(t().ime.eyebrow)}</p>
  <h2 class="h2" use:reveal={1}>{@html r(t().ime.title)}</h2>
  <p class="lede" use:reveal={2}>{@html r(t().ime.lede)}</p>

  <div class="panel card" use:reveal={3}>
    <div class="toolbar">
      <div class="seg" role="group" aria-label="input">
        <button aria-pressed={ime.mode === 'hangul'} title={plain(t().ime.hangulTip)} onclick={() => ime.mode !== 'hangul' && ime.toggleMode()}>{@html r(t().ime.hangul)}</button>
        <button aria-pressed={ime.mode === 'ascii'} title={plain(t().ime.asciiTip)} onclick={() => ime.mode !== 'ascii' && ime.toggleMode()}>{@html r(t().ime.ascii)}</button>
      </div>
      <label class="ui-field"><input class="ui-switch" type="checkbox" role="switch" bind:checked={showRuby} />{@html r(t().ime.ruby)}</label>
      <label class="ui-field"><input class="ui-switch" type="checkbox" role="switch" bind:checked={showKeys} />{@html r(t().ime.keyboard)}</label>
      <div class="seg" role="group" aria-label={plain(t().options.layout)} title={plain(t().options.layoutTip)}>
        <button aria-pressed={options.layout === 'vertical'} onclick={() => (options.layout = 'vertical')}>{@html r(t().options.vertical)}</button>
        <button aria-pressed={options.layout === 'horizontal'} onclick={() => (options.layout = 'horizontal')}>{@html r(t().options.horizontal)}</button>
      </div>
      <label class="ui-field" title={plain(t().options.spaceTip)}>
        <input class="ui-switch" type="checkbox" role="switch" checked={options.spaceMode === 'convert'} onchange={(e) => (options.spaceMode = e.currentTarget.checked ? 'convert' : 'space')} />{@html r(t().options.spaceSwitch)}
      </label>
      <details class="opts">
        <summary class="btn" data-size="sm">{@html r(t().options.title)}</summary>
        <div class="opts-body card">
          <fieldset>
            <legend>{@html r(t().options.style)}</legend>
            <label class="ui-field"><input class="ui-radio" type="radio" bind:group={options.style} value="surface" />{@html r(t().options.surface)}</label>
            <label class="ui-field"><input class="ui-radio" type="radio" bind:group={options.style} value="morpheme" />{@html r(t().options.morpheme)}</label>
          </fieldset>
          <fieldset>
            <legend>{@html r(t().options.fragment)}</legend>
            <label class="ui-field"><input class="ui-radio" type="radio" bind:group={options.fragment} value="absorb" />{@html r(t().options.absorb)}</label>
            <label class="ui-field"><input class="ui-radio" type="radio" bind:group={options.fragment} value="letter" />{@html r(t().options.letter)}</label>
          </fieldset>
          <label class="ui-field"><input class="ui-check" type="checkbox" bind:checked={options.codaAlways} />{@html r(t().options.coda)}</label>
          <label class="ui-field"><input class="ui-check" type="checkbox" bind:checked={options.resyllabify} />{@html r(t().options.resyllabify)}</label>
          <label class="ui-field"><input class="ui-check" type="checkbox" bind:checked={options.hadaKun} />{@html r(t().options.hada)}</label>
        </div>
      </details>
    </div>

    <div class="editor" class:focused role="presentation" {onmouseup} {ondblclick}>
      <textarea
        class="sink"
        bind:this={sink}
        aria-label={plain(t().ime.title)}
        autocapitalize="off"
        autocomplete="off"
        spellcheck="false"
        {onkeydown}
        {onkeyup}
        {oncompositionupdate}
        {oncompositionend}
        {oninput}
        {onpaste}
        onfocus={() => (focused = true)}
        onblur={() => (focused = false)}
      ></textarea>
      <span class="mode" aria-live="polite" title={plain(ime.mode === 'hangul' ? t().ime.hangulTip : t().ime.asciiTip)}>{ime.mode === 'hangul' ? '韓' : 'A'}</span><div class="text" lang="ko" bind:this={textEl}>{#if empty}<span class="placeholder">{@html r(t().ime.placeholder)}</span>{/if}{#each pieces.head as pc}<span class="doc" class:selected={pc.selected} data-off={pc.off}>{#if showRuby}<Ruby text={pc.text} />{:else}{pc.text}{/if}</span>{/each}{#if ime.segs}{#each ime.segs as s, i}<span
              class="segment"
              class:focus={i === ime.focus}
              class:multi={s.candidates.length > 1}
              data-seg={i}>{s.candidates[s.pick]?.text ?? s.reading}</span>{/each}{:else}<span class="pre">{ime.pre}{ime.osComposing}</span>{/if}{#if focused}{#key ime.cursor}<span class="caret" aria-hidden="true"></span>{/key}{/if}{#each pieces.tail as pc}<span class="doc" class:selected={pc.selected} data-off={pc.off}>{#if showRuby}<Ruby text={pc.text} />{:else}{pc.text}{/if}</span>{/each}</div>
      {#if ime.segs && ime.current && ime.open}
        {@const start = ime.pageStart}
        {@const all = ime.current.candidates}
        <div class="win card" class:horizontal={options.layout === 'horizontal'} role="listbox" aria-orientation={options.layout} aria-label={ime.current.reading} style:left="{pos.left}px" style:top="{pos.top}px">
          {#each all.slice(start, start + PAGE) as c, i (start + i)}
            <button
              class="cand"
              role="option"
              aria-selected={start + i === ime.current.pick}
              onmousedown={(e) => e.preventDefault()}
              onclick={() => {
                ime.pick(start + i)
                sink.focus()
              }}
            >
              <span class="num">{i + 1}</span><span class="ct">{c.text}</span>
            </button>
          {/each}
          <div class="win-foot">
            <span class="why">{@html noteHtml(all[ime.current.pick]?.note ?? '')}</span>
            <span class="count">{ime.current.pick + 1}/{all.length}</span>
          </div>
        </div>
      {/if}
    </div>

    {#if ime.segs && ime.current && !ime.open}
      <p class="note">{@html noteHtml(ime.current.candidates[ime.current.pick]?.note ?? '')}</p>
    {:else if !ime.segs && preview && preview !== ime.pre}
      <p class="preview"><span>{@html r(t().ime.preview)}</span> {preview}</p>
    {/if}

    <div class="foot">
      <ul class="hints">
        {#each t().ime.hints as [keys, label]}
          <li>
            {#each keys.split('|') as alt, ai}{#if ai}<span class="or">/</span>{/if}{#each alt.split('+') as k, ki}{#if ki}<span class="plus">+</span>{/if}<kbd>{k}</kbd>{/each}{/each}
            <span>{@html r(label)}</span>
          </li>
        {/each}
      </ul>
      <div class="actions">
        {#if !status.ready}
          <span class="loading" title={status.failed || plain(t().ime.loading)}>
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
    /* Clicks reach the text (to place the caret); the handler focuses this. */
    pointer-events: none;
  }
  .text {
    position: relative;
    min-height: 2.1em;
    word-break: break-word;
    font: 500 clamp(22px, 2.6vw, 28px) / 2.1 var(--f-serif);
  }
  .doc,
  .pre {
    white-space: pre-wrap;
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
    white-space: pre-wrap;
    border-bottom: 2px solid var(--c-border-strong);
    margin-right: 2px;
    transition:
      background var(--t-fast),
      color var(--t-fast);
  }
  .segment.multi {
    border-bottom-color: var(--c-accent);
  }
  .segment.focus {
    background: var(--c-accent);
    color: var(--c-on-accent);
    border-radius: 4px 4px 0 0;
  }
  /* The caret takes no width, so moving it never shifts the text. */
  .caret {
    position: relative;
    display: inline-block;
    width: 0;
    height: 1.1em;
    vertical-align: -0.15em;
  }
  .caret::after {
    content: '';
    position: absolute;
    left: -1px;
    top: 0;
    width: 2px;
    height: 100%;
    background: var(--c-accent);
    animation: blink 1.06s steps(1) infinite;
  }
  .doc.selected {
    background: var(--c-select);
    border-radius: 2px;
  }
  @keyframes blink {
    50% {
      opacity: 0;
    }
  }

  /* The candidate window: a numbered vertical list under the focused segment. */
  .win {
    position: absolute;
    z-index: 15;
    display: grid;
    min-width: 200px;
    max-width: min(360px, 90%);
    padding: 4px;
    box-shadow: var(--shadow-2);
    animation: pop 140ms var(--ease);
    transform-origin: top left;
  }
  @keyframes pop {
    from {
      opacity: 0;
      transform: translateY(-4px) scale(0.98);
    }
  }
  .win.horizontal {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    max-width: min(640px, 92%);
  }
  .win.horizontal .win-foot {
    flex-basis: 100%;
  }
  .cand {
    display: flex;
    align-items: baseline;
    gap: var(--s-3);
    padding: 3px var(--s-3);
    border: 0;
    border-radius: var(--r-1);
    background: none;
    color: var(--c-ink);
    text-align: left;
    cursor: pointer;
  }
  .cand:hover {
    background: var(--c-hover);
  }
  .cand[aria-selected='true'] {
    background: var(--c-accent);
    color: var(--c-on-accent);
  }
  .cand[aria-selected='true'] .num {
    color: inherit;
    opacity: 0.8;
  }
  .num {
    width: 1ch;
    color: var(--c-muted);
    font: 700 12px/1 var(--f-sans);
  }
  .ct {
    font: 500 20px/1.5 var(--f-serif);
    white-space: nowrap;
  }
  .win-foot {
    display: flex;
    justify-content: space-between;
    gap: var(--s-3);
    margin-top: 4px;
    padding: 4px var(--s-2) 2px;
    border-top: 1px solid var(--c-border);
    color: var(--c-muted);
    font-size: 11.5px;
    line-height: 1.5;
  }
  .count {
    flex: none;
    font-variant-numeric: tabular-nums;
  }
  .note {
    min-height: 1.5em;
    margin: var(--s-2) 0 0;
    color: var(--c-muted);
    font-size: 13px;
  }
  .note :global(b),
  .why :global(b) {
    color: var(--c-soft);
    font-weight: 600;
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
  .mode {
    position: absolute;
    top: var(--s-2);
    right: var(--s-2);
    z-index: 2;
    display: grid;
    place-items: center;
    min-width: 28px;
    height: 28px;
    padding: 0 6px;
    border-radius: var(--r-1);
    background: var(--c-accent);
    color: var(--c-on-accent);
    font: 800 14px/1 var(--f-serif);
    transition: background var(--t-fast);
  }
  .hints li {
    display: inline-flex;
    align-items: center;
    gap: 3px;
  }
  .hints li > span:last-child {
    margin-left: 4px;
  }
  .or,
  .plus {
    color: var(--c-muted);
    font-size: 11px;
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
