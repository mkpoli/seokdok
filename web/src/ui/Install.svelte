<script>
  import { reveal } from '../lib/reveal.js'
  import { t, r } from '../lib/i18n.svelte.js'
  import downloads from '../lib/downloads.json'
  import { builds } from '../lib/builds.js'

  const mb = (n) => `${(n / 1048576).toFixed(1)} MB`
  const platforms = [
    {
      key: 'windows',
      name: 'Windows',
      req: 'Windows 10 · 11',
      builds: builds(downloads.windows, downloads.windowsArm64),
      system: null,
      remove: '設定 › 앱 › 釋讀 Seokdok',
      where: '%APPDATA%\\seokdok\\custom.json',
    },
    {
      key: 'linux',
      name: 'Linux',
      req: 'fcitx5 5.1+',
      builds: builds(downloads.linux, downloads.linuxArm64),
      run: `tar xzf seokdok-${downloads.version}-$(uname -m)-linux-fcitx5.tar.gz\ncd seokdok-${downloads.version}-$(uname -m)-linux-fcitx5 && ./install.sh\nfcitx5 -rd`,
      system: 'sudo ./install.sh --system',
      remove: './install.sh --uninstall',
      where: '~/.config/seokdok/custom.json',
    },
    {
      key: 'android',
      name: 'Android',
      req: 'Android 8.0+, arm64 · arm32 · x86_64',
      file: downloads.android.file,
      size: downloads.android.size,
      system: null,
      remove: '設定 › 앱 › 釋讀',
      where: '釋讀 앱 › 내 낱말 › 가져오기',
    },
    {
      key: 'macos',
      name: 'macOS',
      req: 'macOS 13+, Apple silicon · Intel',
      file: downloads.macos.file,
      size: downloads.macos.size,
      run: `mkdir -p ~/Library/Input\\ Methods && cd ~/Library/Input\\ Methods\nditto -xk ~/Downloads/${downloads.macos.file} .\nxattr -dr com.apple.quarantine Seokdok.app\nSeokdok.app/Contents/MacOS/Seokdok --register`,
      system: null,
      remove: 'rm -rf ~/Library/Input\\ Methods/Seokdok.app',
      where: '~/Library/Application Support/seokdok/custom.json',
    },
    {
      key: 'ios',
      name: 'iOS',
      req: 'iOS 16+',
      file: downloads.ios.file,
      size: downloads.ios.size,
      system: null,
      remove: '釋讀 앱을 길게 눌러 삭제',
      where: '釋讀 앱',
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
        <div class="builds">
          {#each p.builds ?? [p] as b (b.file)}
            <div class="build">
              <a class="btn dl" data-variant="primary" data-size="lg" href="/downloads/{b.file}" download>
                <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 4v11m0 0-4.5-4.5M12 15l4.5-4.5M5 19h14" /></svg>
                <span>{@html r(t().install.download)}{b.arch ? ` · ${b.arch}` : ''}</span>
                <small>{mb(b.size)}</small>
              </a>
              <p class="file">{b.file}</p>
            </div>
          {/each}
        </div>
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
  <div class="after" use:reveal={5}>
    <p class="keys">{@html r(t().install.keys)}</p>
    <p>{@html r(t().install.custom)} {@html r(t().install.note)}</p>
  </div>
</section>

<style>
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
    margin: 0;
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
