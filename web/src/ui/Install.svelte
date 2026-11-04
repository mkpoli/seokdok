<script>
  import { reveal } from '../lib/reveal.js'
  import { t, r } from '../lib/i18n.svelte.js'
  import downloads from '../lib/downloads.json'

  const mb = (n) => `${(n / 1048576).toFixed(1)} MB`
  const linuxDir = downloads.linux.file.replace(/\.tar\.gz$/, '')
  const platforms = [
    {
      key: 'windows',
      name: 'Windows',
      req: 'Windows 10 · 11, x64',
      file: downloads.windows.file,
      size: downloads.windows.size,
      system: null,
      remove: '設定 › 앱 › 釋讀 seokdok',
      where: '%APPDATA%\\seokdok\\custom.json',
    },
    {
      key: 'linux',
      name: 'Linux',
      req: 'fcitx5 5.1+, x86_64',
      file: downloads.linux.file,
      size: downloads.linux.size,
      run: `tar xzf ${downloads.linux.file}\ncd ${linuxDir} && ./install.sh\nfcitx5 -rd`,
      system: 'sudo ./install.sh --system',
      remove: './install.sh --uninstall',
      where: '~/.config/seokdok/custom.json',
    },
  ]
</script>

<section class="section" id="install">
  <p class="eyebrow" use:reveal>{@html r(t().install.eyebrow)}</p>
  <h2 class="h2" use:reveal={1}>{@html r(t().install.title)}</h2>
  <p class="lede" use:reveal={2}>{@html r(t().install.lede)}</p>
  <div class="cards">
    {#each platforms as p, i (p.key)}
      <article class="card plat" use:reveal={3 + i}>
        <header>
          <h3>{p.name}</h3>
          <span class="req">{p.req}</span>
        </header>
        <a class="btn dl" data-variant="primary" data-size="lg" href="/downloads/{p.file}" download>
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 4v11m0 0-4.5-4.5M12 15l4.5-4.5M5 19h14" /></svg>
          <span>{@html r(t().install.download)}</span>
          <small>{mb(p.size)}</small>
        </a>
        <p class="file">{p.file}</p>
        <ol>
          {#each t().install[p.key] as step, n}
            <li>
              {@html r(step)}
              {#if n === 0 && p.run}<pre><code>{p.run}</code></pre>{/if}
            </li>
          {/each}
        </ol>
        <dl>
          {#if p.system}
            <dt>{@html r(t().install.system)}</dt>
            <dd><code>{p.system}</code></dd>
          {/if}
          <dt>{@html r(t().install.remove)}</dt>
          <dd><code>{p.remove}</code></dd>
          <dt>custom.json</dt>
          <dd><code>{p.where}</code></dd>
        </dl>
      </article>
    {/each}
  </div>
  <h3 class="more" use:reveal={5}>{@html r(t().install.more)}</h3>
  <p class="more-note" use:reveal={5}>{@html r(t().install.moreNote)}</p>
  <div class="cards small">
    {#each ['ios', 'macos', 'android'] as key, i (key)}
      <article class="card plat" use:reveal={6 + i}>
        <header>
          <h3>{{ ios: 'iOS', macos: 'macOS', android: 'Android' }[key]}</h3>
        </header>
        <ol>
          {#each t().install[key] as step}
            <li>{@html r(step)}</li>
          {/each}
        </ol>
        <a class="btn" href="https://github.com/mkpoli/seokdok/tree/main/{key}" target="_blank" rel="noopener">{@html r(t().install.source)}</a>
      </article>
    {/each}
  </div>
  <div class="after" use:reveal={5}>
    <p class="keys">{@html r(t().install.keys)}</p>
    <p>{@html r(t().install.custom)} {@html r(t().install.note)}</p>
  </div>
</section>

<style>
  .more {
    margin: var(--s-7) 0 var(--s-1);
    font: 700 22px/1.3 var(--f-serif);
  }
  .more-note {
    margin: 0 0 var(--s-4);
    color: var(--c-soft);
  }
  .cards.small {
    grid-template-columns: repeat(auto-fit, minmax(min(100%, 290px), 1fr));
  }
  .cards.small .btn {
    justify-self: start;
    align-self: start;
    margin-top: auto;
  }
  .cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(min(100%, 420px), 1fr));
    gap: var(--s-5);
  }
  .plat {
    display: flex;
    flex-direction: column;
    gap: var(--s-3);
    min-width: 0;
    padding: var(--s-5);
  }
  header {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--s-2);
  }
  h3 {
    margin: 0;
    font: 800 26px/1.2 var(--f-serif);
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
  .dl svg {
    width: 18px;
    height: 18px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2.2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .dl small {
    opacity: 0.8;
    font-weight: 500;
  }
  .file {
    margin: calc(-1 * var(--s-2)) 0 0;
    color: var(--c-muted);
    font: 12px/1.4 var(--f-mono);
    overflow-wrap: anywhere;
  }
  ol {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: var(--s-2);
    margin: 0;
    padding-left: 1.4em;
    line-height: 1.65;
    overflow-wrap: anywhere;
  }
  pre {
    margin: var(--s-2) 0 0;
    padding: var(--s-2) var(--s-3);
    border-radius: var(--r-1);
    background: var(--c-sunken);
    font: 13px/1.6 var(--f-mono);
    overflow-wrap: normal;
    overflow-x: auto;
  }
  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: var(--s-1) var(--s-3);
    margin: auto 0 0;
    padding-top: var(--s-3);
    border-top: 1px solid var(--c-border);
    font-size: 14px;
  }
  dt {
    color: var(--c-muted);
  }
  dd {
    margin: 0;
    min-width: 0;
  }
  dd code {
    font: 12.5px/1.6 var(--f-mono);
    overflow-wrap: anywhere;
  }
  .after {
    margin-top: var(--s-5);
    color: var(--c-soft);
    font-size: 15px;
  }
  .after p {
    max-width: 75ch;
    margin: 0 0 var(--s-2);
  }
  .keys {
    color: var(--c-ink);
    font-weight: 600;
  }
  @media (max-width: 520px) {
    dl {
      grid-template-columns: 1fr;
    }
  }
</style>
