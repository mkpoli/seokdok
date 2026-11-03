<script>
  // Hangul text in, 釋讀 out: every word converted, each with its other
  // candidates in a menu that opens on hover, focus or tap.
  import { reveal } from '../lib/reveal.js'
  import { t, r, plain } from '../lib/i18n.svelte.js'
  import { engine, status } from '../lib/engine.svelte.js'
  import { input, output } from '../lib/custom.svelte.js'

  let text = $state(t().convert.sample)
  /** @type {{ reading: string, candidates: { text: string, note: string }[], pick: number }[][]} */
  let lines = $state([])
  let copied = $state(false)
  let timer

  // Re-convert after typing pauses; lines that did not change keep their picks.
  $effect(() => {
    const src = text
    if (!status.ready) return
    clearTimeout(timer)
    timer = setTimeout(async () => {
      const old = new Map(lines.map((segs) => [segs.map((s) => s.reading).join(''), segs]))
      lines = await Promise.all(
        src.split('\n').map(async (line) => {
          if (old.has(line)) return old.get(line)
          if (!/[가-힣]/.test(line)) return [{ reading: line, candidates: [{ text: line, note: '' }], pick: 0 }]
          const segs = await engine.segments(input(line))
          return segs.map((s) => ({ ...s, candidates: output(s.candidates), pick: 0 }))
        }),
      )
    }, 200)
  })

  const result = $derived(lines.map((segs) => segs.map((s) => s.candidates[s.pick]?.text ?? s.reading).join('')).join('\n'))

  /** Keep an opened menu inside the viewport. */
  function fit(e) {
    const menu = e.currentTarget.querySelector('.menu')
    if (!menu) return
    menu.style.left = '0px'
    requestAnimationFrame(() => {
      const over = menu.getBoundingClientRect().right - (document.documentElement.clientWidth - 8)
      if (over > 0) menu.style.left = `${-over}px`
    })
  }

  async function copy() {
    await navigator.clipboard.writeText(result)
    copied = true
    setTimeout(() => (copied = false), 1400)
  }
</script>

<section class="section" id="convert">
  <p class="eyebrow" use:reveal>{@html r(t().convert.eyebrow)}</p>
  <h2 class="h2" use:reveal={1}>{@html r(t().convert.title)}</h2>
  <p class="lede" use:reveal={2}>{@html r(t().convert.lede)}</p>
  <div class="box card" use:reveal={3}>
    <textarea bind:value={text} rows="3" lang="ko" aria-label={plain(t().convert.input)}></textarea>
    <div class="out" lang="ko" aria-label={plain(t().convert.result)}>
      {#each lines as segs, li}
        <p class="line">
          {#each segs as seg, si}
            {#if seg.candidates.length > 1}
              <span class="word" role="presentation" onpointerenter={fit} onfocusin={fit}>
                <button class="w" class:changed={seg.pick > 0} aria-haspopup="listbox" title={seg.reading}>{seg.candidates[seg.pick].text}</button>
                <span class="menu card" role="listbox" aria-label={seg.reading}>
                  {#each seg.candidates.slice(0, 30) as c, ci}
                    <button
                      role="option"
                      aria-selected={ci === seg.pick}
                      onclick={(e) => {
                        lines[li][si].pick = ci
                        e.currentTarget.closest('.word')?.querySelector('.w')?.focus()
                      }}
                    >
                      <span class="c">{c.text}</span>
                      {#if c.note}<small>{c.note}</small>{/if}
                    </button>
                  {/each}
                </span>
              </span>
            {:else}<span class="plain">{seg.candidates[0]?.text ?? seg.reading}</span>{/if}
          {/each}
        </p>
      {/each}
    </div>
    <div class="actions">
      <button class="btn" onclick={copy} disabled={!result}>{@html r(copied ? t().ime.copied : t().ime.copy)}</button>
    </div>
  </div>
</section>

<style>
  .box {
    display: grid;
    gap: var(--s-4);
    padding: var(--s-4);
  }
  textarea {
    width: 100%;
    padding: var(--s-3) var(--s-4);
    border: 1px solid var(--c-border);
    border-radius: var(--r-2);
    background: var(--c-sunken);
    color: var(--c-ink);
    font: 500 18px/1.7 var(--f-sans);
    resize: vertical;
  }
  .out {
    padding: 0 var(--s-2);
    font: 500 clamp(21px, 2.4vw, 26px) / 1.9 var(--f-serif);
  }
  .line {
    min-height: 1.9em;
    margin: 0;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .word {
    position: relative;
    display: inline-block;
  }
  .w {
    padding: 0 1px;
    border: 0;
    border-bottom: 2px dotted var(--c-border-strong);
    border-radius: 2px;
    background: none;
    color: inherit;
    font: inherit;
    cursor: pointer;
  }
  .w.changed {
    border-bottom-color: var(--c-accent);
    color: var(--c-accent);
  }
  .word:hover .w,
  .w:focus-visible {
    background: var(--c-accent-soft);
    outline: none;
  }
  /* The candidates open below the word on hover, focus or tap. */
  .menu {
    position: absolute;
    z-index: 20;
    top: 100%;
    left: 0;
    display: none;
    min-width: max(100%, 9em);
    max-height: 18em;
    padding: var(--s-1);
    overflow-y: auto;
    box-shadow: var(--shadow-2);
  }
  .word:hover .menu,
  .word:focus-within .menu {
    display: grid;
  }
  .menu button {
    display: flex;
    align-items: baseline;
    gap: var(--s-3);
    padding: 2px var(--s-2);
    border: 0;
    border-radius: var(--r-1);
    background: none;
    color: var(--c-ink);
    text-align: left;
    cursor: pointer;
  }
  .menu button:hover,
  .menu button:focus-visible {
    background: var(--c-hover);
    outline: none;
  }
  .menu button[aria-selected='true'] {
    background: var(--c-accent-soft);
  }
  .c {
    font: 500 20px/1.5 var(--f-serif);
    white-space: nowrap;
  }
  .menu small {
    color: var(--c-muted);
    font: 400 12px/1.3 var(--f-sans);
    white-space: nowrap;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
  }
</style>
