<script>
  // /dict: search and page through every row of the committed dictionary files.
  // The rows live in a worker (lib/dict.worker.js); the page holds one page of them.
  import { i18n, r, plain } from './lib/i18n.svelte.js'
  import text from './lib/dict-text.js'
  import { FILES, LICENCE_URLS, README_URL, fileUrl, notePreview, posKey } from './lib/dict-browse.js'
  import TopBar from './ui/TopBar.svelte'
  import Footer from './ui/Footer.svelte'

  const SIZE = 50
  const fmt = (n) => n.toLocaleString('en-US')
  const stems = FILES.map((f) => f.stem)

  const params = new URLSearchParams(location.search)
  let q = $state(params.get('q') ?? '')
  let file = $state(stems.includes(params.get('src')) ? params.get('src') : '')
  let page = $state(Math.max(1, Number(params.get('p')) || 1))

  let ready = $state(false)
  let failed = $state('')
  let progress = $state(0)
  let fileRows = $state({})
  let size = $state(0)
  let result = $state(null)

  const d = $derived(i18n.lang === 'en' ? text.en : text.ko)
  const counts = $derived(result ? Object.fromEntries(stems.map((s, i) => [s, result.counts[i] ?? 0])) : fileRows)
  const shown = $derived(result ? result.counts.reduce((a, b) => a + b, 0) : size)
  const chosen = $derived(FILES.find((f) => f.stem === file))

  const worker = new Worker(new URL('./lib/dict.worker.js', import.meta.url), { type: 'module' })
  let lastId = 0

  // The script itself failed to load, as when this page outlives a deploy.
  worker.onerror = (e) => {
    failed = e.message || 'worker'
  }
  worker.onmessage = ({ data }) => {
    if (data.progress) {
      const { loaded, total } = data.progress
      progress = Math.min(1, loaded / total)
    } else if (data.ready) {
      fileRows = Object.fromEntries(data.stems.map((s, i) => [s, data.fileRows[i]]))
      size = data.size
      ready = true
      run()
    } else if (data.failed) {
      failed = data.failed
    } else if (data.id === lastId) {
      const first = !result
      result = data
      page = data.page
      syncUrl()
      // The section's height depends on rows that were not there on load.
      if (first && location.hash) requestAnimationFrame(() => document.querySelector(location.hash)?.scrollIntoView())
    }
  }

  function run() {
    worker.postMessage({ id: ++lastId, q, file, page, size: SIZE })
  }

  function syncUrl() {
    const p = new URLSearchParams()
    if (q.trim()) p.set('q', q.trim())
    if (file) p.set('src', file)
    if (page > 1) p.set('p', page)
    const s = p.toString()
    history.replaceState(null, '', (s ? `/dict?${s}` : '/dict') + location.hash)
  }

  let timer
  function onSearch() {
    clearTimeout(timer)
    timer = setTimeout(() => {
      page = 1
      run()
    }, 120)
  }

  function choose(stem) {
    file = stem
    page = 1
    run()
  }

  // A filter picked lower down the page shows its rows from the top of the results.
  let top
  let status = $state()
  // Keyboard focus follows: the control pressed may be off screen or now disabled.
  function showResults() {
    top?.scrollIntoView()
    status?.focus({ preventScroll: true })
  }
  function chooseAndShow(stem) {
    choose(stem)
    showResults()
  }

  function go(p) {
    page = p
    run()
    showResults()
  }

  const countLabel = (n) => r((n === 1 ? d.row : d.rows).replace('%n', fmt(n)))
  const name = (stem) => d.files[stem].name
  const licenceLabel = (l) => l.split(':')
</script>

{#snippet pager()}
  <nav class="pager" aria-label={plain(d.pager.label)}>
    <button class="btn" disabled={result.page <= 1} onclick={() => go(1)} aria-label={plain(d.pager.first)} title={plain(d.pager.first)}>«</button>
    <button class="btn" disabled={result.page <= 1} onclick={() => go(result.page - 1)} aria-label={plain(d.pager.prev)} title={plain(d.pager.prev)}>‹</button>
    <span class="where">{@html r(d.pager.page)} {fmt(result.page)} / {fmt(result.pages)}</span>
    <button class="btn" disabled={result.page >= result.pages} onclick={() => go(result.page + 1)} aria-label={plain(d.pager.next)} title={plain(d.pager.next)}>›</button>
    <button class="btn" disabled={result.page >= result.pages} onclick={() => go(result.pages)} aria-label={plain(d.pager.last)} title={plain(d.pager.last)}>»</button>
  </nav>
{/snippet}

{#snippet licences(entry)}
  {#each entry.licences as l, i}
    {@const [id, scope] = licenceLabel(l)}
    {#if i > 0}{' · '}{/if}
    <a href={LICENCE_URLS[id]} rel="noopener">{#if id === 'permission'}{@html r(d.permission)}{:else}{id}{/if}</a>{#if scope}{' '}({@html r(d.scopes[scope])}){/if}
  {/each}
{/snippet}

<div class="progress" aria-hidden="true"></div>
<TopBar home="/" />
<main class="section dict">
  <h1 class="h2">{@html r(d.title)}</h1>
  <p class="lede">{@html r(d.lede)}</p>

  <div class="controls" bind:this={top}>
    <label class="search">
      <span class="label">{@html r(d.search)}</span>
      <input class="ui-input" type="search" lang="ko" autocomplete="off" spellcheck="false" placeholder={plain(d.placeholder)} bind:value={q} oninput={onSearch} />
    </label>

    <label class="pick">
      <span class="label">{@html r(d.filter)}</span>
      <select class="ui-input" value={file} onchange={(e) => choose(e.currentTarget.value)}>
        <option value="">{plain(d.all)} ({fmt(shown)})</option>
        {#each FILES as f (f.stem)}
          <option value={f.stem}>{plain(name(f.stem))} · {f.stem} ({fmt(counts[f.stem] ?? 0)})</option>
        {/each}
      </select>
    </label>

    <div class="chips" role="group" aria-label={plain(d.filter)}>
      <button class="chip" aria-pressed={!file} onclick={() => choose('')}>{@html r(d.all)} <small>{fmt(shown)}</small></button>
      {#each FILES as f (f.stem)}
        <button class="chip" aria-pressed={file === f.stem} onclick={() => choose(f.stem)}>
          {@html r(name(f.stem))}
          <code>{f.stem}</code>
          <small>{fmt(counts[f.stem] ?? 0)}</small>
        </button>
      {/each}
    </div>
  </div>

  {#if chosen}
    <aside class="about">
      <p>{@html r(d.files[file].about)}</p>
      <p class="links">
        <a href={fileUrl(file)} rel="noopener"><code>{file}.tsv</code></a> · {@html r(d.licence)}: {@render licences(chosen)}
      </p>
    </aside>
  {/if}

  {#if failed}
    <p class="message" role="alert">{@html r(d.failed)} <a href="https://github.com/mkpoli/seokdok/tree/main/dict" rel="noopener">github.com/mkpoli/seokdok/tree/main/dict</a></p>
  {:else if !ready || !result}
    <div class="message" role="status">
      <p>{@html r(d.loading)}</p>
      <div class="bar"><span style:width="{progress * 100}%"></span></div>
    </div>
  {:else}
    <p class="status" role="status" tabindex="-1" bind:this={status}>{@html countLabel(result.total)}</p>
    {#if result.rows.length === 0}
      <p class="message">{@html r(d.empty)}</p>
    {:else}
      <div class="card sheet">
        <table>
          <thead>
            <tr>
              <th scope="col">{@html r(d.columns.reading)}</th>
              <th scope="col">{@html r(d.columns.spelling)}</th>
              <th scope="col">{@html r(d.columns.pos)}</th>
              <th scope="col" class="num">{@html r(d.columns.count)}</th>
              <th scope="col">{@html r(d.columns.source)}</th>
              <th scope="col">{@html r(d.columns.note)}</th>
            </tr>
          </thead>
          <tbody>
            {#each result.rows as row, i (`${result.id}:${i}`)}
              {@const preview = notePreview(row.note)}
              {@const pos = posKey(row.pos)}
              <tr>
                <td class="reading" lang="ko" translate="no">{row.reading}</td>
                <td class="spelling" lang="ko" translate="no">{row.spelling}</td>
                <td class="pos">{#if pos}{@html r(d.pos[pos])}{:else}{row.pos}{/if}</td>
                <td class="num">{#if row.count}<span class="cell-label">{@html r(d.columns.count)}</span> {row.count}{/if}</td>
                <td class="src">
                  <button class="badge" title={plain(d.files[row.file].about)} onclick={() => chooseAndShow(row.file)}>{@html r(name(row.file))}</button>
                </td>
                <td class="note">
                  {#if row.tag}<span class="tag">{#if d.tags[row.tag]}{@html r(d.tags[row.tag])}{:else}{row.tag}{/if}</span>{/if}
                  {#if preview}
                    <details lang="ko" translate="no">
                      <summary><span class="preview">{preview}…</span></summary>
                      {row.note}
                    </details>
                  {:else if row.note}
                    <span lang="ko" translate="no">{row.note}</span>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
      {@render pager()}
    {/if}
  {/if}

  <section id="sources" class="sources">
    <h2>{@html r(d.sourcesTitle)}</h2>
    <dl>
      {#each FILES as f (f.stem)}
        <div class="source">
          <dt>
            {@html r(name(f.stem))}
            <a href={fileUrl(f.stem)} rel="noopener"><code>{f.stem}.tsv</code></a>
            {#if fileRows[f.stem]}<small>{@html countLabel(fileRows[f.stem])}</small>{/if}
          </dt>
          <dd>
            <p>{@html r(d.files[f.stem].about)}</p>
            <p class="links">
              {@html r(d.licence)}: {@render licences(f)} ·
              <button class="link" onclick={() => chooseAndShow(f.stem)}>{@html r(d.show)}</button>
            </p>
          </dd>
        </div>
      {/each}
    </dl>
    <p class="notice">{@html r(d.notice)} <a href="/dict/NOTICE.txt">NOTICE.txt</a> · <a href={README_URL} rel="noopener">dict/README.md</a></p>
  </section>
</main>
<Footer />

<style>
  .dict {
    min-height: 70vh;
    padding-block: var(--s-5) var(--s-7);
  }
  /* The results are what the page is for: the heading stays out of their way. */
  .dict .h2 {
    margin-bottom: var(--s-2);
    font-size: clamp(26px, 3.4vw, 34px);
  }
  .lede {
    max-width: none;
    margin-bottom: var(--s-4);
    font-size: 15px;
  }
  .controls {
    display: grid;
    gap: var(--s-4);
    margin-bottom: var(--s-4);
    scroll-margin-top: 72px;
  }
  .label {
    display: block;
    margin-bottom: var(--s-1);
    color: var(--c-soft);
    font: 600 14px/2 var(--f-sans);
  }
  .search input,
  .pick select {
    width: 100%;
  }
  .search input {
    height: 48px;
    font-size: 18px;
  }
  /* Seventeen chips are too many for a phone; the list takes their place there. */
  .pick {
    display: none;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
  }
  .chip {
    display: inline-flex;
    align-items: baseline;
    gap: 6px;
    padding: 4px var(--s-3) 3px;
    border: 1px solid var(--c-border);
    border-radius: 999px;
    background: var(--c-surface);
    color: var(--c-soft);
    font: 600 13px/2 var(--f-sans);
    cursor: pointer;
  }
  .chip:hover {
    border-color: var(--c-accent);
  }
  .chip[aria-pressed='true'] {
    border-color: var(--c-accent);
    background: var(--c-accent);
    color: var(--c-on-accent);
  }
  .chip[aria-pressed='true'] :global(rt),
  .chip[aria-pressed='true'] small,
  .chip[aria-pressed='true'] code {
    color: inherit;
  }
  .chip code,
  .links code,
  dt code {
    font: 12px var(--f-mono);
  }
  .chip code,
  .chip small {
    color: var(--c-muted);
    font-weight: 400;
  }
  .chip small {
    font-size: 12px;
    font-variant-numeric: tabular-nums;
  }
  .about {
    margin: 0 0 var(--s-4);
    padding: var(--s-3) var(--s-4);
    border-left: 3px solid var(--c-accent);
    border-radius: 0 var(--r-2) var(--r-2) 0;
    background: var(--c-accent-soft);
    color: var(--c-soft);
    font-size: 15px;
    line-height: 2;
  }
  .about p {
    margin: 0;
  }
  .links {
    font-size: 14px;
  }
  .status {
    outline: 0;
    margin: 0 0 var(--s-2);
    color: var(--c-muted);
    font: 600 14px/2 var(--f-sans);
    font-variant-numeric: tabular-nums;
  }
  .message {
    margin: var(--s-5) 0;
    color: var(--c-soft);
  }
  .message p {
    margin: 0 0 var(--s-2);
  }
  .bar {
    height: 6px;
    max-width: 320px;
    border-radius: 999px;
    background: var(--c-sunken);
    overflow: hidden;
  }
  .bar span {
    display: block;
    height: 100%;
    background: var(--c-accent);
  }
  /* A very long reading can outgrow the table; it scrolls within the sheet. */
  .sheet {
    overflow-x: auto;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    table-layout: auto;
  }
  th {
    padding: var(--s-1) var(--s-3);
    border-bottom: 1px solid var(--c-border-strong);
    background: var(--c-sunken);
    color: var(--c-soft);
    font: 600 13px/2 var(--f-sans);
    text-align: left;
    white-space: nowrap;
  }
  td {
    padding: var(--s-2) var(--s-3);
    border-bottom: 1px solid var(--c-border);
    vertical-align: baseline;
    font-size: 14px;
    line-height: 1.7;
  }
  tbody tr:last-child td {
    border-bottom: 0;
  }
  tbody tr:hover {
    background: var(--c-hover);
  }
  .reading {
    font-weight: 600;
    white-space: nowrap;
  }
  .spelling {
    color: var(--c-accent);
    font: 700 18px/1.5 var(--f-serif);
    overflow-wrap: anywhere;
  }
  .pos {
    color: var(--c-soft);
    white-space: nowrap;
  }
  .num {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
  .cell-label {
    display: none;
  }
  .src {
    white-space: nowrap;
  }
  .badge {
    padding: 2px 8px 1px;
    border: 1px solid var(--c-border);
    border-radius: var(--r-1);
    background: var(--c-sunken);
    color: var(--c-soft);
    font: 600 12px/1.9 var(--f-sans);
    cursor: pointer;
  }
  .badge:hover {
    border-color: var(--c-accent);
    color: var(--c-accent);
  }
  .note {
    color: var(--c-soft);
    font-size: 13px;
    overflow-wrap: anywhere;
  }
  .tag {
    display: block;
    color: var(--c-muted);
  }
  summary {
    cursor: pointer;
  }
  details[open] .preview {
    display: none;
  }
  .pager {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--s-2);
    margin-top: var(--s-4);
  }
  .pager .btn {
    justify-content: center;
    min-width: 40px;
    padding: 0 var(--s-2);
    font-size: 18px;
  }
  .pager .btn:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .where {
    min-width: 11ch;
    color: var(--c-soft);
    font-size: 14px;
    text-align: center;
    font-variant-numeric: tabular-nums;
  }
  .sources {
    margin-top: var(--s-8);
  }
  h2 {
    margin: 0 0 var(--s-4);
    font: 800 clamp(22px, 3vw, 28px) / 1.4 var(--f-serif);
  }
  dl {
    margin: 0;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(min(420px, 100%), 1fr));
    gap: var(--s-5) var(--s-6);
  }
  dt {
    font: 700 16px/2 var(--f-sans);
  }
  dt a {
    margin-inline: 4px;
  }
  dt small {
    color: var(--c-muted);
    font: 400 12px var(--f-sans);
  }
  dd {
    margin: 0;
    color: var(--c-soft);
    font-size: 14px;
    line-height: 2;
  }
  dd p {
    margin: 0;
    overflow-wrap: anywhere;
  }
  .link {
    padding: 0;
    border: 0;
    background: none;
    color: var(--c-accent);
    font: inherit;
    text-decoration: underline;
    text-underline-offset: 3px;
    cursor: pointer;
  }
  .notice {
    margin: var(--s-6) 0 0;
    color: var(--c-soft);
    font-size: 14px;
    line-height: 2;
  }

  @media (max-width: 760px) {
    .chips {
      display: none;
    }
    .pick {
      display: block;
    }
    .pick select {
      font-size: 16px;
    }
    /* Each row becomes a small card: the word, what kind it is, then its note. */
    thead {
      position: absolute;
      width: 1px;
      height: 1px;
      overflow: hidden;
      clip: rect(0 0 0 0);
    }
    table,
    tbody {
      display: block;
    }
    tbody tr {
      display: grid;
      grid-template-columns: auto 1fr auto;
      align-items: baseline;
      gap: 2px var(--s-2);
      padding: var(--s-3) var(--s-4);
      border-bottom: 1px solid var(--c-border);
    }
    tbody tr:last-child {
      border-bottom: 0;
    }
    td,
    tbody tr:last-child td {
      padding: 0;
      border: 0;
    }
    td:empty {
      display: none;
    }
    .reading {
      grid-column: 1;
      font-size: 16px;
      white-space: normal;
      overflow-wrap: anywhere;
    }
    .spelling {
      grid-column: 2;
    }
    .src {
      grid-column: 3;
      grid-row: 1;
    }
    .pos,
    .num {
      font-size: 13px;
      text-align: left;
      color: var(--c-muted);
    }
    .pos {
      grid-column: 1;
    }
    .num {
      grid-column: 2;
    }
    .cell-label {
      display: inline;
    }
    .note {
      grid-column: 1 / -1;
    }
    .note:empty {
      display: none;
    }
  }
</style>
