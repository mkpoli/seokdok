<script>
  // The 音讀 input method's own page. It says nothing of the other product.
  import { i18n, setLang, t, r, plain } from './lib/i18n.svelte.js'
  import { engine } from './lib/engine.svelte.js'
  import { options, saveOptions } from './lib/options.svelte.js'
  import downloads from './lib/downloads.json'
  import Convert from './ui/Convert.svelte'

  const page = $derived(t().eumdokPage)
  const mb = (n) => `${(n / 1048576).toFixed(1)} MB`
  const files = downloads.eumdok ?? {}
  const platforms = [
    { key: 'windows', name: 'Windows', req: 'Windows 10 · 11, x64' },
    { key: 'linux', name: 'Linux', req: 'fcitx5 5.1+, x86_64' },
    { key: 'android', name: 'Android', req: 'Android 8.0+, arm64 · arm32 · x86_64' },
  ].filter((p) => files[p.key])
  const langs = [
    ['ko', '한', 'ko'],
    ['en', 'EN', 'en'],
  ]

  let theme = $state(document.documentElement.dataset.theme ?? 'auto')
  function cycleTheme() {
    theme = theme === 'auto' ? 'light' : theme === 'light' ? 'dark' : 'auto'
    if (theme === 'auto') delete document.documentElement.dataset.theme
    else document.documentElement.dataset.theme = theme
    try {
      localStorage.setItem('seokdok:theme', theme)
    } catch {}
  }

  $effect(() => {
    const o = { ...options }
    saveOptions()
    engine.options(o)
  })
</script>

<header class="bar">
  <a class="mark" href="#top" aria-label="Eumdok">
    <img src="/favicon-eumdok.svg" alt="" width="28" height="28" />
    <span class="mark-han">音讀</span><span class="mark-latn">Eumdok</span>
  </a>
  <div class="tools">
    <div class="seg" role="group" aria-label="language">
      {#each langs as [code, label, lang]}
        <button aria-pressed={i18n.lang === code || (code === 'ko' && i18n.lang === 'kore')} onclick={() => setLang(code)} {lang}>{label}</button>
      {/each}
    </div>
    <button class="theme" onclick={cycleTheme} title={plain(t().theme[theme])} aria-label={plain(t().theme[theme])}>
      <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="8" /><path d="M12 4a8 8 0 0 1 0 16z" class="fill" /></svg>
    </button>
  </div>
</header>

<main id="top">
  <section class="section lead">
    <p class="eyebrow">{@html r(page.eyebrow)}</p>
    <h1>音讀</h1>
    <p class="lede">{@html r(page.lede)}</p>
    <div class="compare">
      <figure>
        <figcaption>{@html r(page.typed)}</figcaption>
        <p lang="ko">선생님께서 하늘을 보셨다</p>
      </figure>
      <span class="arrow" aria-hidden="true">→</span>
      <figure class="own">
        <figcaption>{@html r(page.written)}</figcaption>
        <p lang="ko">先生님께서 하늘을 보셨다</p>
      </figure>
    </div>
    <a class="btn" data-variant="primary" data-size="lg" href="#install">{@html r(page.download)}</a>
  </section>

  <Convert strings={page.convert} />

  <section class="section" id="install">
    <p class="eyebrow">{@html r(page.installEyebrow)}</p>
    <h2 class="h2">{@html r(page.installTitle)}</h2>
    {#if platforms.length}
      <div class="cards">
        {#each platforms as p (p.key)}
          <article class="card plat">
            <header>
              <h3>{p.name}</h3>
              <span class="req">{p.req}</span>
            </header>
            <a class="btn dl" data-variant="primary" href="/downloads/{files[p.key].file}" download>
              <span>{@html r(page.download)}</span>
              <small>{mb(files[p.key].size)}</small>
            </a>
            <p class="file">{files[p.key].file}</p>
            <p>{@html r(page[p.key])}</p>
          </article>
        {/each}
      </div>
    {:else}
      <p class="note">{@html r(page.soon)}</p>
    {/if}
    <p class="note">{@html r(page.keys)}</p>
  </section>
</main>

<footer class="section foot">
  <p>{@html r(page.sources)} <a href="/dict/NOTICE.txt">NOTICE.txt</a></p>
  <p>{@html r(page.licence)} <a href="https://github.com/mkpoli/seokdok" rel="noopener">GitHub</a> · <a href="https://mkpo.li/">mkpoli</a></p>
</footer>

<style>
  :global(:root) {
    --c-accent: light-dark(#7c2d23, #eda99b);
    --c-accent-hover: light-dark(#5f2019, #f5c4ba);
    --c-accent-soft: light-dark(#f7e6e2, #3b1a14);
    --c-on-accent: light-dark(#ffffff, #1d0e0b);
    --c-ruby: light-dark(#8a4a40, #e0b0a6);
  }
  .bar {
    position: sticky;
    top: env(safe-area-inset-top, 0px);
    z-index: 20;
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 56px;
    padding: 0 max(var(--s-4), calc((100vw - var(--w-page)) / 2));
    background: color-mix(in srgb, var(--c-canvas) 82%, transparent);
    backdrop-filter: blur(12px);
    border-bottom: 1px solid var(--c-border);
  }
  .mark {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    color: var(--c-accent);
    text-decoration: none;
  }
  .mark img {
    border-radius: 18%;
  }
  .mark-han {
    font: 900 22px/1 var(--f-serif);
  }
  .mark-latn {
    font: 700 13px/1 var(--f-sans);
    letter-spacing: 0.08em;
    color: var(--c-muted);
  }
  .tools {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }
  .theme {
    display: grid;
    place-items: center;
    width: 36px;
    height: 36px;
    border: 1px solid var(--c-border);
    border-radius: 999px;
    background: var(--c-surface);
    color: var(--c-soft);
    cursor: pointer;
  }
  .theme svg {
    width: 18px;
    height: 18px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
  }
  .theme .fill {
    fill: currentColor;
  }
  .lead h1 {
    margin: 0;
    color: var(--c-accent);
    font: 900 clamp(84px, 16vw, 170px) / 1.05 var(--f-serif);
    margin-bottom: var(--s-5);
  }
  .compare {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--s-3);
    margin: var(--s-5) 0;
  }
  .arrow {
    color: var(--c-muted);
    font-size: 24px;
  }
  figure {
    flex: 1 1 260px;
    margin: 0;
    padding: var(--s-4);
    border: 1px solid var(--c-border);
    border-radius: 12px;
    background: var(--c-surface);
  }
  figure.own {
    border-color: var(--c-accent);
    box-shadow: inset 0 0 0 1px var(--c-accent);
  }
  figcaption {
    margin-bottom: var(--s-2);
    color: var(--c-muted);
    font-size: 14px;
  }
  figure p {
    margin: 0;
    font: 500 22px/1.6 var(--f-serif);
  }
  .cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(min(100%, 300px), 1fr));
    gap: var(--s-4);
    margin: var(--s-4) 0;
  }
  .plat {
    display: grid;
    gap: var(--s-2);
    align-content: start;
    padding: var(--s-5);
  }
  .plat header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--s-2);
  }
  .plat h3 {
    margin: 0;
    font: 700 24px/1.2 var(--f-serif);
  }
  .req,
  .file {
    color: var(--c-muted);
    font: 500 13px/1.4 var(--f-mono, monospace);
  }
  .file {
    margin: 0;
    overflow-wrap: anywhere;
  }
  .dl {
    justify-self: start;
  }
  .dl small {
    margin-left: var(--s-2);
    opacity: 0.85;
  }
  .plat p {
    margin: 0;
  }
  .note {
    color: var(--c-soft);
  }
  .foot p {
    margin: var(--s-1) 0;
    color: var(--c-muted);
    font-size: 14px;
  }
</style>
