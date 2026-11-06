<script>
  import { reveal } from '../lib/reveal.js'
  import { t, r } from '../lib/i18n.svelte.js'
  import downloads from '../lib/downloads.json'

  const mb = (n) => `${(n / 1048576).toFixed(1)} MB`
  const files = downloads.eumdok ?? {}
  const platforms = [
    { key: 'windows', name: 'Windows', req: 'Windows 10 · 11, x64' },
    { key: 'linux', name: 'Linux', req: 'fcitx5 5.1+, x86_64' },
    { key: 'android', name: 'Android', req: 'Android 8.0+, arm64 · arm32 · x86_64' },
    { key: 'macos', name: 'macOS', req: 'macOS 13+, Apple silicon · Intel', run: () => `mkdir -p ~/Library/Input\\ Methods && cd ~/Library/Input\\ Methods\nditto -xk ~/Downloads/${files.macos.file} .\nxattr -dr com.apple.quarantine Eumdok.app\nEumdok.app/Contents/MacOS/Eumdok --register` },
    { key: 'ios', name: 'iOS', req: 'iOS 16+' },
  ].filter((p) => files[p.key])
</script>

<section class="section" id="eumdok">
  <p class="eyebrow" use:reveal>{@html r(t().eumdok.eyebrow)}</p>
  <h2 class="h2" use:reveal={1}>
    <img src="/favicon-eumdok.svg" alt="" width="40" height="40" />
    {@html r(t().eumdok.title)}
  </h2>
  <p class="lede" use:reveal={2}>{@html r(t().eumdok.lede)}</p>
  <p use:reveal={2}><a class="btn" href="https://eumdok.mkpo.li/">{@html r(t().eumdok.page)}</a></p>

  <div class="compare" use:reveal={3}>
    <figure>
      <figcaption>{@html r(t().eumdok.typed)}</figcaption>
      <p>선생님께서 하늘을 보셨다</p>
    </figure>
    <figure>
      <figcaption>{@html r(t().eumdok.seokdok)}</figcaption>
      <p>先生主께서 天을 見셨다</p>
    </figure>
    <figure class="own">
      <figcaption>{@html r(t().eumdok.eumdok)}</figcaption>
      <p>先生님께서 하늘을 보셨다</p>
    </figure>
  </div>

  <h3 class="more" use:reveal={3}>{@html r(t().eumdok.switchTitle)}</h3>
  <p class="note" use:reveal={3}>{@html r(t().eumdok.switchBody)}</p>
  <table use:reveal={3}>
    <thead>
      <tr>
        <th></th>
        <th>{@html r(t().eumdok.seokdok)}</th>
        <th>{@html r(t().eumdok.eumdok)}</th>
      </tr>
    </thead>
    <tbody>
      {#each t().eumdok.rows as row}
        <tr>
          {#each row as cell}<td>{@html r(cell)}</td>{/each}
        </tr>
      {/each}
    </tbody>
  </table>

  <h3 class="more" use:reveal={4}>{@html r(t().eumdok.downloadTitle)}</h3>
  {#if platforms.length}
    <p class="note" use:reveal={4}>{@html r(t().eumdok.side)}</p>
    <div class="cards">
      {#each platforms as p, i (p.key)}
        <article class="card plat" use:reveal={4 + i}>
          <header>
            <h4>{p.name}</h4>
            <span class="req">{p.req}</span>
          </header>
          <a class="btn dl" data-variant="primary" href="/downloads/{files[p.key].file}" download>
            <span>{@html r(t().install.download)}</span>
            <small>{mb(files[p.key].size)}</small>
          </a>
          <p class="file">{files[p.key].file}</p>
          <p>{@html r(t().eumdok[p.key])}</p>
          {#if p.run}<pre><code>{p.run()}</code></pre>{/if}
        </article>
      {/each}
    </div>
  {:else}
    <p class="note" use:reveal={4}>{@html r(t().eumdok.soon)}</p>
  {/if}
  {#if files.ios}<p class="note apple" use:reveal={5}>{@html r(t().eumdok.apple)}</p>{/if}
</section>

<style>
  h2 img {
    display: inline-block;
    width: 1em;
    height: 1em;
    margin-right: 0.15em;
    border-radius: 18%;
    vertical-align: -0.12em;
  }
  .compare {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(min(100%, 260px), 1fr));
    gap: var(--s-3);
    margin: var(--s-4) 0 0;
  }
  figure {
    margin: 0;
    padding: var(--s-4);
    border: 1px solid var(--c-border);
    border-radius: var(--r-2, 12px);
  }
  figure.own {
    border-color: #7c2d23;
    box-shadow: inset 0 0 0 1px #7c2d23;
  }
  figcaption {
    margin-bottom: var(--s-2);
    color: var(--c-muted);
    font-size: 14px;
  }
  figure p {
    margin: 0;
    font: 600 20px/1.5 var(--f-serif);
  }
  .more {
    margin: var(--s-7) 0 var(--s-1);
    font: 700 22px/1.3 var(--f-serif);
  }
  .note {
    max-width: 75ch;
    margin: 0 0 var(--s-3);
    color: var(--c-soft);
  }
  .apple {
    margin-top: var(--s-4);
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 15px;
  }
  th,
  td {
    padding: var(--s-2) var(--s-3) var(--s-2) 0;
    border-bottom: 1px solid var(--c-border);
    text-align: left;
    vertical-align: top;
  }
  td:first-child {
    color: var(--c-muted);
  }
  .cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(min(100%, 290px), 1fr));
    gap: var(--s-5);
  }
  .plat {
    display: flex;
    flex-direction: column;
    gap: var(--s-3);
    min-width: 0;
    padding: var(--s-5);
  }
  .plat p {
    margin: 0;
  }
  header {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--s-2);
  }
  h4 {
    margin: 0;
    font: 800 24px/1.2 var(--f-serif);
  }
  .req {
    color: var(--c-muted);
    font: 500 13px/1.4 var(--f-mono);
  }
  .dl {
    align-self: flex-start;
    gap: var(--s-2);
    text-decoration: none;
  }
  .dl small {
    opacity: 0.8;
    font-weight: 500;
  }
  pre {
    margin: 0;
    padding: var(--s-2) var(--s-3);
    border-radius: var(--r-1);
    background: var(--c-sunken);
    font: 13px/1.6 var(--f-mono);
    overflow-x: auto;
  }
  .file {
    color: var(--c-muted);
    font: 12px/1.4 var(--f-mono);
    overflow-wrap: anywhere;
  }
</style>
