<script>
  import { reveal } from '../lib/reveal.js'
  import { t, r, plain } from '../lib/i18n.svelte.js'
  import { engine, status } from '../lib/engine.svelte.js'
  import { options, DEFAULTS } from '../lib/options.svelte.js'

  const SAMPLE = '그래서 있으면 봤어요'
  const VARIANTS = [
    ['surface', {}],
    ['morpheme', { style: 'morpheme' }],
    ['letter', { fragment: 'letter' }],
    ['coda', { codaAlways: true }],
    ['resyllabify', { codaAlways: true, resyllabify: true }],
  ]
  let rows = $state([])

  $effect(() => {
    if (!status.ready) return
    const restore = { ...options }
    Promise.all(VARIANTS.map(([k, o]) => engine.convertWith(SAMPLE, { ...DEFAULTS, ...o }, restore))).then(
      (rs) => (rows = rs.map((r, i) => [VARIANTS[i][0], r[0]?.text ?? ''])),
    )
  })

  const cards = $derived([t().rules.sino, t().rules.kun, t().rules.okuri])
</script>

<section class="section" id="rules">
  <p class="eyebrow" use:reveal>{@html r(t().rules.eyebrow)}</p>
  <h2 class="h2" use:reveal={1}>{@html r(t().rules.title)}</h2>
  <p class="lede" use:reveal={2}>{@html r(t().rules.lede)}</p>
  <div class="cards">
    {#each cards as [head, sub, ex], i}
      <article class="card kind" use:reveal={i}>
        <span class="n">{i + 1}</span>
        <h3>{@html r(head)}</h3>
        <p class="sub">{@html r(sub)}</p>
        <p class="ex" lang="ko">{ex}</p>
      </article>
    {/each}
  </div>
  <h3 class="h3">{@html r(t().rules.variants)}</h3>
  <p class="lede small">{@html r(t().rules.variantsLede)}</p>
  <div class="variants card" use:reveal>
    <p class="src" lang="ko">{SAMPLE}</p>
    {#if rows.length}
      <dl>
        {#each rows as [k, text]}
          <dt>{@html r(t().rules.variantNames[k])}</dt>
          <dd lang="ko">{text}</dd>
        {/each}
      </dl>
    {:else}
      <p class="wait">…</p>
    {/if}
  </div>
</section>

<style>
  .cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: var(--s-4);
  }
  .kind {
    position: relative;
    padding: var(--s-5);
    overflow: hidden;
  }
  .n {
    position: absolute;
    right: var(--s-4);
    top: var(--s-2);
    color: var(--c-accent-soft);
    font: 900 72px/1 var(--f-serif);
  }
  h3 {
    position: relative;
    margin: 0;
    color: var(--c-accent);
    font: 900 34px/1.2 var(--f-serif);
  }
  .sub {
    position: relative;
    margin: var(--s-1) 0 var(--s-3);
    font-weight: 700;
  }
  .ex {
    position: relative;
    margin: 0;
    color: var(--c-soft);
    font: 500 18px/1.6 var(--f-serif);
  }
  .h3 {
    margin: var(--s-7) 0 var(--s-2);
    font: 800 24px/1.3 var(--f-serif);
  }
  .small {
    margin-bottom: var(--s-4);
    font-size: 15px;
  }
  .variants {
    padding: var(--s-5);
  }
  .src {
    margin: 0 0 var(--s-3);
    color: var(--c-muted);
    font: 500 17px/1.5 var(--f-sans);
  }
  dl {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: var(--s-2) var(--s-5);
    margin: 0;
  }
  dt {
    color: var(--c-muted);
    font-weight: 700;
    font-size: 13px;
    align-self: center;
  }
  dd {
    margin: 0;
    font: 500 24px/1.4 var(--f-serif);
  }
  @media (max-width: 520px) {
    dl {
      grid-template-columns: 1fr;
      gap: 0;
    }
    dd {
      margin-bottom: var(--s-3);
      font-size: 21px;
    }
  }
  .wait {
    color: var(--c-muted);
  }
</style>
