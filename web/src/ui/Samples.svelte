<script>
  import { i18n, t, r } from '../lib/i18n.svelte.js'
  import { engine, status } from '../lib/engine.svelte.js'
  import { reveal } from '../lib/reveal.js'
  import { SAMPLES } from '../lib/samples.js'
  import { rubify } from '../lib/rubify.js'
  import { options } from '../lib/options.svelte.js'

  let { onopen } = $props()
  let kind = $state('all')
  /** id → segments ({ reading, text }) once converted. */
  let conv = $state({})
  /** id → how many segments have turned into 釋讀 so far. */
  let shown = $state({})

  const kinds = ['all', 'classic', 'poem', 'wiki', 'law']
  const list = $derived(kind === 'all' ? SAMPLES : SAMPLES.filter((s) => s.kind === kind))

  // Convert each sample when its card first scrolls into view, then turn the
  // segments over one by one.
  // Samples already shown are converted again, without the animation, when
  // the mode changes (釋讀 ⇄ 國漢文混用).
  let mode = options.sinoOnly
  $effect(() => {
    if (options.sinoOnly === mode) return
    mode = options.sinoOnly
    // The engine gets the new options first (ImePanel); then convert.
    setTimeout(() => {
      for (const id of Object.keys(conv)) {
        const sample = SAMPLES.find((x) => x.id === id)
        engine.segments(sample.text).then((segs) => {
          conv[id] = segs.map((s) => ({ reading: s.reading, text: s.candidates[0]?.text ?? s.reading }))
          shown[id] = conv[id].length
        })
      }
    }, 50)
  })

  async function play(sample) {
    if (conv[sample.id]) return
    const go = async () => {
      const segs = await engine.segments(sample.text)
      conv[sample.id] = segs.map((s) => ({ reading: s.reading, text: s.candidates[0]?.text ?? s.reading }))
      const n = conv[sample.id].length
      const still = matchMedia('(prefers-reduced-motion: reduce)').matches
      if (still) return (shown[sample.id] = n)
      shown[sample.id] = 0
      const step = () => {
        shown[sample.id] = (shown[sample.id] ?? 0) + 1
        if (shown[sample.id] < n) setTimeout(step, 45)
      }
      setTimeout(step, 120)
    }
    if (status.ready) go()
    else {
      const wait = setInterval(() => status.ready && (clearInterval(wait), go()), 200)
    }
  }
</script>

<section class="section" id="samples">
  <p class="eyebrow" use:reveal>{@html r(t().samples.eyebrow)}</p>
  <h2 class="h2" use:reveal={1}>{@html r(t().samples.title)}</h2>
  <p class="lede" use:reveal={2}>{@html r(t().samples.lede)}</p>
  <div class="seg kinds" role="group" aria-label={t().samples.eyebrow} use:reveal={3}>
    {#each kinds as k}
      <button aria-pressed={kind === k} onclick={() => (kind = k)}>{@html r(t().samples.kinds[k])}</button>
    {/each}
  </div>
  <div class="grid">
    {#each list as s, i (s.id)}
      <article class="card sample" data-kind={s.kind} use:reveal={i % 3} onrevealed={() => play(s)}>
        <header>
          <span class="kind">{@html r(t().samples.kinds[s.kind])}</span>
          <h3>{@html r(s.title)}</h3>
          <p class="by">{@html r(s.by)}</p>
        </header>
        <p class="out" lang="ko">
          {#if conv[s.id]}
            {#each conv[s.id] as g, j}{#if j < (shown[s.id] ?? 0) && g.text !== g.reading}{@const rb = i18n.lang === 'ko' ? rubify(g.text, g.reading) : null}{#if rb}<span class="turn"><ruby>{rb.base}<rt>{rb.rt}</rt></ruby>{rb.tail}</span>{:else}<span class="turn">{g.text}</span>{/if}{:else}<span class:pending={j >= (shown[s.id] ?? 0)}>{g.reading}</span>{/if}{/each}
          {:else}
            <span class="pending">{s.text}</span>
          {/if}
        </p>
        <details class="orig">
          <summary>{@html r(t().samples.original)}</summary>
          <p lang="ko">{s.text}</p>
        </details>
        <footer>
          <button class="btn" data-size="sm" onclick={() => onopen(s.text)}>{@html r(t().samples.open)}</button>
          <p class="src">
            <a href={s.source.url} target="_blank" rel="noopener">{s.source.label}</a>
            · {s.source.rev ?? ''} · {s.source.licence}{s.source.note ? ` · ${s.source.note}` : ''}
          </p>
        </footer>
      </article>
    {/each}
  </div>
</section>

<style>
  .kinds {
    margin-bottom: var(--s-5);
  }
  .grid {
    columns: 3 300px;
    column-gap: var(--s-4);
  }
  .sample {
    break-inside: avoid;
    display: grid;
    gap: var(--s-3);
    margin-bottom: var(--s-4);
    padding: var(--s-5);
    position: relative;
    overflow: hidden;
  }
  .sample::before {
    content: '';
    position: absolute;
    inset: 0 auto 0 0;
    width: 4px;
    background: var(--c-accent);
    opacity: 0.85;
  }
  .sample[data-kind='poem']::before {
    background: var(--c-ruby);
  }
  .sample[data-kind='wiki']::before,
  .sample[data-kind='law']::before {
    background: var(--c-border-strong);
  }
  header {
    display: grid;
    gap: 2px;
  }
  .kind {
    color: var(--c-accent);
    font: 700 11.5px/1.4 var(--f-sans);
    letter-spacing: 0.12em;
  }
  h3 {
    margin: 0;
    font: 800 24px/1.5 var(--f-serif);
  }
  .by {
    margin: 0;
    color: var(--c-muted);
    font-size: 13px;
  }
  .out {
    margin: 0;
    white-space: pre-wrap;
    font: 500 19px/2.2 var(--f-serif);
  }
  .pending {
    color: var(--c-muted);
  }
  .turn {
    display: inline-block;
    color: var(--c-ink);
    animation: turn 420ms var(--ease) both;
  }
  @keyframes turn {
    from {
      opacity: 0;
      filter: blur(3px);
      transform: translateY(6px);
    }
  }
  .orig summary {
    width: max-content;
    color: var(--c-muted);
    font-size: 13px;
    cursor: pointer;
  }
  .orig p {
    margin: var(--s-2) 0 0;
    color: var(--c-soft);
    font-size: 14.5px;
    white-space: pre-wrap;
  }
  footer {
    display: grid;
    gap: var(--s-2);
    justify-items: start;
  }
  .btn[data-size='sm'] {
    min-height: 32px;
    padding: 0 var(--s-3);
    font-size: 13px;
  }
  .src {
    margin: 0;
    color: var(--c-muted);
    font-size: 12px;
    line-height: 1.5;
  }
</style>
