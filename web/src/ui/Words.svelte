<script>
  import { t, r, p } from '../lib/i18n.svelte.js'
  import { engine, status } from '../lib/engine.svelte.js'

  let { onpick } = $props()
  let entries = $state([])
  let q = $state('')

  $effect(() => {
    if (status.ready && !entries.length) engine.entries(240).then((e) => (entries = e))
  })

  const shown = $derived(
    (q ? entries.filter((e) => e.lemma.includes(q) || e.written.includes(q)) : entries).slice(0, 120),
  )
  const max = $derived(Math.max(1, ...entries.map((e) => e.count)))
</script>

<section class="section" id="words">
  <p class="eyebrow">{@html r(t().words.eyebrow)}</p>
  <h2 class="h2">{@html r(t().words.title)}</h2>
  <p class="lede">{@html r(t().words.lede)}</p>
  <input class="filter" type="search" bind:value={q} placeholder={p(t().words.filter)} aria-label={p(t().words.filter)} />
  <div class="grid">
    {#each shown as e (e.lemma + e.written)}
      <button
        class="tile"
        onclick={() => onpick(e.lemma)}
        title="{e.lemma} · {e.count}{p(t().words.count)}"
        style:--w={0.55 + 0.45 * Math.sqrt(e.count / max)}
      >
        <small lang="ko">{e.lemma}</small>
        <span lang="ko">{e.written}</span>
      </button>
    {:else}
      {#each Array(24) as _}<span class="tile ghost"></span>{/each}
    {/each}
  </div>
</section>

<style>
  .filter {
    width: min(360px, 100%);
    height: 40px;
    margin-bottom: var(--s-4);
    padding: 0 var(--s-4);
    border: 1px solid var(--c-border-strong);
    border-radius: 999px;
    background: var(--c-surface);
    color: var(--c-ink);
    font: 15px var(--f-sans);
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(118px, 1fr));
    gap: var(--s-2);
  }
  .tile {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    min-height: 74px;
    padding: var(--s-2) var(--s-3);
    border: 1px solid var(--c-border);
    border-radius: var(--r-2);
    background: var(--c-surface);
    color: var(--c-ink);
    text-align: left;
    cursor: pointer;
    transition:
      transform var(--t-fast) var(--ease),
      border-color var(--t-fast);
  }
  .tile:hover {
    transform: translateY(-2px);
    border-color: var(--c-accent);
  }
  .tile small {
    color: var(--c-muted);
    font-size: 12px;
  }
  .tile span {
    font: 700 calc(16px + 12px * var(--w)) / 1.2 var(--f-serif);
    color: color-mix(in srgb, var(--c-accent) calc(var(--w) * 100%), var(--c-ink));
    white-space: nowrap;
  }
  @media (max-width: 520px) {
    .grid {
      grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
    }
    .tile span {
      font-size: calc(15px + 8px * var(--w));
    }
  }
  .ghost {
    background: var(--c-sunken);
    animation: pulse 1.4s ease-in-out infinite;
  }
  @keyframes pulse {
    50% {
      opacity: 0.5;
    }
  }
</style>
