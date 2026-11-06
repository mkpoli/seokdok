<script>
  // The 音讀 site's front page.
  import { i18n, setLang, t, r, plain } from './lib/i18n.svelte.js'
  import { engine } from './lib/engine.svelte.js'
  import downloads from './lib/downloads.json'
  import { builds } from './lib/builds.js'
  import { Composer, Learned } from '../pkg/seokdok.js'
  import { Ime } from './lib/ime.svelte.js'
  import { Picks, STORAGE_KEY } from './lib/learned.js'
  import { input, output } from './lib/custom.svelte.js'
  import ImePanel from './ui/ImePanel.svelte'
  import Convert from './ui/Convert.svelte'
  import { reveal } from './lib/reveal.js'

  const page = $derived(t().eumdokPage)
  const mb = (n) => `${(n / 1048576).toFixed(1)} MB`
  const files = downloads.eumdok ?? {}
  const platforms = [
    { key: 'windows', name: 'Windows', req: 'Windows 10 · 11', builds: builds(files.windows, files.windowsArm64) },
    { key: 'linux', name: 'Linux', req: 'fcitx5 5.1+', builds: builds(files.linux, files.linuxArm64) },
    { key: 'android', name: 'Android', req: 'Android 8.0+, arm64 · arm32 · x86_64', builds: [files.android] },
  ].filter((p) => files[p.key])
  const langs = [
    ['ko', '한', 'ko'],
    ['kore', '韓', 'ko'],
    ['en', 'EN', 'en'],
  ]

  const rows = $derived(Array.from({ length: 6 }, (_, i) => page.wall.slice(i * 6, i * 6 + 6)))

  let theme = $state(document.documentElement.dataset.theme ?? 'auto')
  function cycleTheme() {
    theme = theme === 'auto' ? 'light' : theme === 'light' ? 'dark' : 'auto'
    if (theme === 'auto') delete document.documentElement.dataset.theme
    else document.documentElement.dataset.theme = theme
    try {
      localStorage.setItem('seokdok:theme', theme)
    } catch {}
  }

  const ime = new Ime(
    new Composer(),
    async (text) => (await engine.segments(input(text))).map((seg) => ({ ...seg, candidates: output(seg.candidates) })),
    async (text) => output(await engine.convert(input(text))),
  )
  let storage = null
  try {
    storage = localStorage
  } catch {}
  const picks = new Picks(Learned, storage)
  addEventListener('pagehide', () => picks.flush())
  addEventListener('storage', (e) => {
    if (e.key === STORAGE_KEY || e.key === null) picks.reload()
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
        <button aria-pressed={i18n.lang === code} onclick={() => setLang(code)} {lang}>{label}</button>
      {/each}
    </div>
    <button class="theme" onclick={cycleTheme} title={plain(t().theme[theme])} aria-label={plain(t().theme[theme])}>
      <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="8" /><path d="M12 4a8 8 0 0 1 0 16z" class="fill" /></svg>
    </button>
  </div>
</header>

<main id="top">
  <section class="band">
    <div class="hero">
      <div class="text">
        <p class="eyebrow">{@html r(page.eyebrow)}</p>
        <h1><span class="big">音讀</span></h1>
        <p class="claim">{@html r(page.claim)}</p>
        <p class="lede">{@html r(page.lede)}</p>
        <div class="cta">
          <a class="btn" data-variant="primary" data-size="lg" href="#install">{@html r(page.download)}</a>
          <a class="btn" data-size="lg" href="#ime">{@html r(page.tryIt)}</a>
        </div>
      </div>
      <div class="wall" aria-hidden="true">
        {#each rows as row, i}
          <div class="row" style:--i={i}>
            {#each [...row, ...row] as word, c}
              <span class="word" style:--size="{1 + ((i * 7 + c * 3) % 5) * 0.12}em">{@html r(word)}</span>
            {/each}
          </div>
        {/each}
      </div>
    </div>
  </section>

  <section class="section">
    <div class="compare" use:reveal>
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
    <div class="points">
      {#each page.points as [head, body], i}
        <article class="card point" use:reveal={i}>
          <h3>{@html r(head)}</h3>
          <p>{@html r(body)}</p>
        </article>
      {/each}
    </div>
  </section>

  <ImePanel {ime} {picks} eumdok strings={page.ime} />

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
            <div class="builds">
              {#each p.builds as b (b.file)}
                <div class="build">
                  <a class="btn dl" data-variant="primary" href="/downloads/{b.file}" download>
                    <span>{@html r(page.download)}{b.arch ? ` · ${b.arch}` : ''}</span>
                    <small>{mb(b.size)}</small>
                  </a>
                  <p class="file">{b.file}</p>
                </div>
              {/each}
            </div>
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

<section class="section more">
  <a class="card link" href="https://wiki.韓國語.漢字.net/" target="_blank" rel="noopener" use:reveal>
    <b>國漢大百科</b>
    <span>{@html r(page.wiki)}</span>
  </a>
</section>

<footer class="section foot">
  <p class="elsewhere">{@html r(page.moreHanja)} <a href="https://seokdok.mkpo.li/">釋讀</a></p>
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
  .band {
    overflow: hidden;
    background:
      radial-gradient(60% 60% at 78% 35%, var(--c-accent-soft), transparent 70%),
      linear-gradient(180deg, var(--c-canvas), var(--c-canvas-2));
  }
  .hero {
    display: grid;
    grid-template-columns: minmax(0, 1.05fr) minmax(0, 1fr);
    align-items: center;
    gap: var(--s-6);
    width: min(var(--w-page), 100%);
    min-height: calc(100svh - 56px);
    margin-inline: auto;
    padding: var(--s-7) var(--s-4);
  }
  .text > * {
    animation: rise 800ms var(--ease) backwards;
  }
  .text > :nth-child(2) {
    animation-delay: 90ms;
  }
  .text > :nth-child(3) {
    animation-delay: 200ms;
  }
  .text > :nth-child(4) {
    animation-delay: 300ms;
  }
  .text > :nth-child(5) {
    animation-delay: 380ms;
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(24px);
    }
  }
  h1 {
    margin: 0;
    line-height: 1;
  }
  .big {
    display: block;
    margin-left: -0.04em;
    color: var(--c-accent);
    font: 900 clamp(120px, 19vw, 230px) / 1.02 var(--f-serif);
    letter-spacing: -0.02em;
  }
  .claim {
    margin: var(--s-4) 0 var(--s-3);
    text-wrap: balance;
    font: 800 clamp(22px, 2.8vw, 30px) / 1.5 var(--f-serif);
  }
  .lede {
    max-width: 46ch;
    margin: 0 0 var(--s-5);
    color: var(--c-soft);
    font-size: 16.5px;
    line-height: 2;
  }
  .cta {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-3);
  }
  .wall {
    display: flex;
    flex-direction: column;
    gap: 0.5em;
    font: 500 clamp(20px, 2.4vw, 30px) / 1.2 var(--f-serif);
    transform: perspective(900px) rotateY(-24deg) rotateX(4deg) skewY(-3deg);
    transform-origin: left center;
    mask-image: linear-gradient(90deg, transparent, #000 14%, #000 70%, transparent);
  }
  .row {
    display: flex;
    gap: 1em;
    width: max-content;
    animation: drift calc(48s + var(--i) * 6s) linear infinite;
  }
  .row:nth-child(even) {
    animation-direction: reverse;
  }
  .word {
    color: var(--c-ink);
    font-size: var(--size);
    white-space: nowrap;
    opacity: 0.86;
  }
  .word:nth-child(5n + 2) {
    color: var(--c-accent);
    opacity: 1;
  }
  @keyframes drift {
    to {
      transform: translateX(-50%);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .row {
      animation: none;
    }
  }
  @media (max-width: 860px) {
    .hero {
      grid-template-columns: 1fr;
      min-height: auto;
      padding-top: var(--s-6);
    }
    .wall {
      order: -1;
      height: 210px;
      overflow: hidden;
      transform: perspective(700px) rotateY(-14deg) skewY(-3deg);
      font-size: 22px;
    }
  }
  .points {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(min(100%, 260px), 1fr));
    gap: var(--s-4);
    margin-top: var(--s-5);
  }
  .point {
    padding: var(--s-5);
  }
  .point h3 {
    margin: 0 0 var(--s-2);
    font: 700 21px/1.5 var(--f-serif);
  }
  .point p {
    margin: 0;
    color: var(--c-soft);
    line-height: 1.9;
  }
  .link {
    display: grid;
    gap: var(--s-1);
    padding: var(--s-5);
    color: inherit;
    text-decoration: none;
  }
  .link b {
    color: var(--c-accent);
    font: 800 26px/1.3 var(--f-serif);
  }
  .link span {
    color: var(--c-soft);
  }
  .elsewhere {
    font-size: 16px;
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
  .build .dl {
    align-self: flex-start;
  }
  .builds {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-3) var(--s-4);
  }
  .build {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
    min-width: 0;
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
