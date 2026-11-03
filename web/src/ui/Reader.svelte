<script>
  import { reveal } from '../lib/reveal.js'
  import { t, r, plain } from '../lib/i18n.svelte.js'
  import Ruby from './Ruby.svelte'
  import { engine, status } from '../lib/engine.svelte.js'

  let text = $state(t().reader.sample)
  let hangul = $state('')
  let copied = $state(false)
  let timer

  // The whole text read back into plain Hangul.
  $effect(() => {
    const src = text
    if (!status.ready) return
    clearTimeout(timer)
    timer = setTimeout(async () => {
      const out = await engine.read(src)
      if (text === src) hangul = out
    }, 150)
  })

  async function copy() {
    await navigator.clipboard.writeText(hangul)
    copied = true
    setTimeout(() => (copied = false), 1400)
  }
</script>

<section class="section" id="read">
  <p class="eyebrow" use:reveal>{@html r(t().reader.eyebrow)}</p>
  <h2 class="h2" use:reveal={1}>{@html r(t().reader.title)}</h2>
  <p class="lede" use:reveal={2}>{@html r(t().reader.lede)}</p>
  <div class="reader card" use:reveal={3}>
    <textarea bind:value={text} rows="3" lang="ko" aria-label={plain(t().reader.title)}></textarea>
    <div class="pane">
      <p class="label">{@html r(t().reader.ruby)}</p>
      <p class="out" lang="ko" aria-label={plain(t().reader.ruby)}><Ruby {text} /></p>
    </div>
    <div class="pane">
      <div class="head">
        <p class="label">{@html r(t().reader.plain)}</p>
        <button class="btn" data-size="sm" onclick={copy} disabled={!hangul}>{@html r(copied ? t().ime.copied : t().ime.copy)}</button>
      </div>
      <p class="hangul" lang="ko" aria-label={plain(t().reader.plain)}>{hangul}</p>
    </div>
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
  .pane {
    display: grid;
    gap: var(--s-1);
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-3);
  }
  .label {
    margin: 0;
    color: var(--c-muted);
    font: 700 13px/1.4 var(--f-sans);
    letter-spacing: 0.06em;
  }
  .hangul {
    margin: 0;
    padding: var(--s-2) var(--s-3);
    border-radius: var(--r-2);
    background: var(--c-sunken);
    font: 500 clamp(19px, 2.2vw, 22px) / 1.8 var(--f-sans);
    white-space: pre-wrap;
  }
  .out {
    margin: 0;
    padding: 0 var(--s-2);
    font: 500 clamp(22px, 2.6vw, 28px) / 2.3 var(--f-serif);
    white-space: pre-wrap;
  }
</style>
