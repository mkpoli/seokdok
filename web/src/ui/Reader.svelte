<script>
  import { t, r, p } from '../lib/i18n.svelte.js'
  import Ruby from './Ruby.svelte'

  let text = $state(t().reader.sample)
</script>

<section class="section" id="read">
  <p class="eyebrow">{@html r(t().reader.eyebrow)}</p>
  <h2 class="h2">{@html r(t().reader.title)}</h2>
  <p class="lede">{@html r(t().reader.lede)}</p>
  <div class="reader card">
    <textarea bind:value={text} rows="3" lang="ko" aria-label={p(t().reader.title)}></textarea>
    <p class="out" lang="ko" aria-label={p(t().reader.reading)}><Ruby {text} /></p>
  </div>
</section>

<style>
  .reader {
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
    font: 500 18px/1.7 var(--f-serif);
    resize: vertical;
  }
  .out {
    margin: 0;
    padding: 0 var(--s-2);
    font: 500 clamp(22px, 2.6vw, 28px) / 2.3 var(--f-serif);
    white-space: pre-wrap;
  }
</style>
