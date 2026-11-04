<script>
  // A text page: the privacy policy or the support page (see lib/pages.js).
  import { t, r } from './lib/i18n.svelte.js'
  import TopBar from './ui/TopBar.svelte'
  import Footer from './ui/Footer.svelte'

  let { page } = $props()

  const doc = $derived(t()[page])

  // Rich text, with `code` and bare links.
  const rich = (s) =>
    r(s)
      .replace(/`([^`]+)`/g, '<code>$1</code>')
      .replace(/(https?:\/\/[^\s)<]+)/g, '<a href="$1" rel="noopener">$1</a>')
      .replace(/([\w.-]+@[\w.-]+\.\w+)/g, '<a href="mailto:$1">$1</a>')

  const steps = (ol) => (typeof ol === 'string' ? t().install[ol] : ol)
</script>

<div class="progress" aria-hidden="true"></div>
<TopBar home="/" />
<main class="section doc">
  <p class="eyebrow">{@html r(doc.eyebrow)}</p>
  <h1 class="h2">{@html r(doc.title)}</h1>
  {#if doc.updated}<p class="date">{doc.updated}</p>{/if}
  <p class="lede">{@html rich(doc.lede)}</p>
  {#if doc.note}<p class="note">{@html rich(doc.note)}</p>{/if}
  {#each doc.sections as s (s.id)}
    <section id={s.id}>
      <h2>{@html r(s.h)}</h2>
      {#each s.body as b}
        {#if typeof b === 'string'}
          <p>{@html rich(b)}</p>
        {:else if Array.isArray(b)}
          <ul>
            {#each b as li}<li>{@html rich(li)}</li>{/each}
          </ul>
        {:else}
          <article class="card item">
            <h3>{@html r(b.h)}</h3>
            {#if b.ol}
              <ol>
                {#each steps(b.ol) as li}<li>{@html rich(li)}</li>{/each}
              </ol>
            {:else if b.ul}
              <ul>
                {#each b.ul as li}<li>{@html rich(li)}</li>{/each}
              </ul>
            {:else}
              <p>{@html rich(b.p)}</p>
            {/if}
          </article>
        {/if}
      {/each}
    </section>
  {/each}
</main>
<Footer />

<style>
  .doc {
    max-width: 820px;
    min-height: 70vh;
  }
  .date {
    margin: calc(var(--s-2) * -1) 0 var(--s-4);
    color: var(--c-muted);
    font-size: 14px;
  }
  .lede {
    margin-bottom: var(--s-4);
  }
  .note {
    margin: 0 0 var(--s-6);
    padding: var(--s-3) var(--s-4);
    border-left: 3px solid var(--c-accent);
    border-radius: 0 var(--r-2) var(--r-2) 0;
    background: var(--c-accent-soft);
    color: var(--c-soft);
    font-size: 15px;
    line-height: 2;
  }
  section {
    margin-top: var(--s-7);
  }
  h2 {
    margin: 0 0 var(--s-3);
    font: 800 clamp(22px, 3vw, 28px) / 1.4 var(--f-serif);
  }
  .doc :global(.h2) {
    font: 800 clamp(28px, 4vw, 40px) / 1.25 var(--f-serif);
  }
  p,
  li {
    line-height: 2;
    overflow-wrap: anywhere;
  }
  p {
    margin: 0 0 var(--s-3);
  }
  ul,
  ol {
    margin: 0 0 var(--s-3);
    padding-left: 1.3em;
  }
  li + li {
    margin-top: var(--s-1);
  }
  .item {
    margin-bottom: var(--s-3);
    padding: var(--s-4) var(--s-5);
  }
  .item h3 {
    margin: 0 0 var(--s-2);
    font: 700 18px/1.6 var(--f-sans);
  }
  .item p,
  .item ol {
    margin-bottom: 0;
  }
  .doc :global(code) {
    padding: 1px 6px;
    border-radius: var(--r-1);
    background: var(--c-sunken);
    font: 0.88em var(--f-mono);
    overflow-wrap: anywhere;
  }
  @media (max-width: 640px) {
    .item {
      padding: var(--s-3) var(--s-4);
    }
  }
</style>
