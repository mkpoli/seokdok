<script>
  import { i18n, setLang, t, r, plain } from '../lib/i18n.svelte.js'
  import { options, saveOptions, showRubyPosition } from '../lib/options.svelte.js'

  // On a page other than the front page, `home` is '/' and the section links lead back to it.
  let { home = '' } = $props()

  // The switch moves at once; the page re-renders on the next frame.
  let pending = $state(null)
  const sino = $derived(pending ?? options.sinoOnly)
  function setMode(v) {
    pending = v
    requestAnimationFrame(() =>
      setTimeout(() => {
        options.sinoOnly = v
        pending = null
      }),
    )
  }

  let theme = $state(document.documentElement.dataset.theme ?? 'auto')

  function cycleTheme() {
    theme = theme === 'auto' ? 'light' : theme === 'light' ? 'dark' : 'auto'
    if (theme === 'auto') delete document.documentElement.dataset.theme
    else document.documentElement.dataset.theme = theme
    try {
      localStorage.setItem('seokdok:theme', theme)
    } catch {}
  }

  function toggleRuby() {
    options.rubyPosition = options.rubyPosition === 'right' ? 'above' : 'right'
    showRubyPosition()
    saveOptions()
  }

  // 한: Korean in 釋讀 with Hangul ruby. 韓: the same in Hanja only.
  const langs = [
    ['ko', '한', 'ko'],
    ['kore', '韓', 'ko'],
    ['en', 'EN', 'en'],
  ]
</script>

<header class="bar">
  <a class="mark" href="{home}#top" aria-label="seokdok">
    <span class="mark-han">釋讀</span><span class="mark-latn">seokdok</span>
  </a>
  <nav aria-label="sections">
    <a href="{home}#ime">{@html r(t().nav.ime)}</a>
    <a href="{home}#custom">{@html r(t().nav.custom)}</a>
    <a href="{home}#rules">{@html r(t().nav.rules)}</a>
    <a href="{home}#words">{@html r(t().nav.words)}</a>
    <a href="{home}#convert">{@html r(t().nav.convert)}</a>
    <a href="{home}#old">{@html r(t().nav.old)}</a>
    <a href="{home}#install">{@html r(t().nav.install)}</a>
    <a href="{home}#join">{@html r(t().nav.join)}</a>
  </nav>
  <div class="tools">
    <div class="mode" title={plain(t().options.sinoTip)}>
      <button class="side" aria-pressed={sino} onclick={() => setMode(true)}>{@html r(t().options.sinoSwitch)}</button>
      <input class="ui-switch" type="checkbox" role="switch" checked={!sino} onchange={(e) => setMode(!e.currentTarget.checked)} aria-label={plain(t().options.seokdokSide)} />
      <button class="side" aria-pressed={!sino} onclick={() => setMode(false)}>{@html r(t().options.seokdokSide)}</button>
    </div>
    <div class="seg" role="group" aria-label="language">
      {#each langs as [code, label, lang]}
        <button aria-pressed={i18n.lang === code} onclick={() => setLang(code)} {lang}>{label}</button>
      {/each}
    </div>
    <button class="theme ruby-side" class:right={options.rubyPosition === 'right'} onclick={toggleRuby} title={plain(options.rubyPosition === 'right' ? t().options.rubyRight : t().options.rubyAbove)} aria-label={plain(options.rubyPosition === 'right' ? t().options.rubyRight : t().options.rubyAbove)}>
      <span aria-hidden="true"><b>字</b><small>자</small></span>
    </button>
    <button class="theme" onclick={cycleTheme} title={plain(t().theme[theme])} aria-label={plain(t().theme[theme])}>
      {#if theme === 'light'}
        <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="4.5" /><path d="M12 2v2.5M12 19.5V22M2 12h2.5M19.5 12H22M4.9 4.9l1.8 1.8M17.3 17.3l1.8 1.8M4.9 19.1l1.8-1.8M17.3 6.7l1.8-1.8" /></svg>
      {:else if theme === 'dark'}
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M20 14.5A8 8 0 0 1 9.5 4a8 8 0 1 0 10.5 10.5z" /></svg>
      {:else}
        <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="8" /><path d="M12 4a8 8 0 0 1 0 16z" class="fill" /></svg>
      {/if}
    </button>
  </div>
</header>

<style>
  .bar {
    position: sticky;
    top: env(safe-area-inset-top, 0px);
    z-index: 20;
    display: flex;
    align-items: center;
    gap: var(--s-5);
    height: 56px;
    padding: 0 max(var(--s-4), calc((100vw - var(--w-page)) / 2));
    background: color-mix(in srgb, var(--c-canvas) 82%, transparent);
    backdrop-filter: blur(12px);
    border-bottom: 1px solid var(--c-border);
  }
  .mark {
    display: flex;
    align-items: baseline;
    gap: var(--s-2);
    color: var(--c-accent);
    text-decoration: none;
  }
  .mark-han {
    font: 900 22px/1 var(--f-serif);
  }
  .mark-latn {
    font: 700 13px/1 var(--f-sans);
    letter-spacing: 0.08em;
    color: var(--c-muted);
  }
  /* The links scroll inside the space the logo and the tools leave. */
  nav {
    flex: 1 1 0;
    min-width: 0;
    overflow-x: auto;
    scrollbar-width: none;
    display: flex;
    /* Links with and without ruby share one baseline (國漢文混用 drops some). */
    align-items: baseline;
    gap: var(--s-4);
    margin-right: auto;
  }
  nav a {
    flex-shrink: 0;
    white-space: nowrap;
    color: var(--c-soft);
    font-weight: 600;
    font-size: 14px;
    text-decoration: none;
  }
  nav a:hover {
    color: var(--c-accent);
  }
  .mark,
  .tools {
    flex-shrink: 0;
  }
  .tools {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }
  .mode {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    margin-right: var(--s-2);
  }
  .side {
    padding: 4px 0 0;
    border: 0;
    background: none;
    color: var(--c-muted);
    font: 600 14px/2.1 var(--f-sans);
    white-space: nowrap;
    cursor: pointer;
  }
  .side[aria-pressed='true'] {
    color: var(--c-accent);
  }
  .side :global(rt) {
    font-size: 0.5em;
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
    stroke-width: 1.8;
    stroke-linecap: round;
  }
  .theme .fill,
  .theme svg path:only-child {
    fill: currentColor;
    stroke: none;
  }
  .ruby-side span {
    display: inline-flex;
    flex-direction: column-reverse;
    align-items: center;
    line-height: 1;
  }
  .ruby-side.right span {
    flex-direction: row;
    align-items: baseline;
  }
  .ruby-side b {
    font: 700 15px/1 var(--f-serif);
  }
  .ruby-side small {
    font: 600 8px/1 var(--f-sans);
  }
  @media (max-width: 640px) {
    nav {
      display: none;
    }
    .mark {
      margin-right: auto;
    }
    .mark-latn {
      display: none;
    }
    .bar {
      gap: var(--s-2);
    }
    .mark-han {
      white-space: nowrap;
    }
    .mode {
      gap: 6px;
      margin-right: 0;
    }
    .side {
      padding: 0;
      font-size: 13px;
    }
    .side :global(rt) {
      display: none;
    }
    .tools {
      gap: 4px;
    }
    .tools :global(.seg button) {
      padding: 0 6px;
    }
    .theme {
      width: 30px;
      height: 30px;
    }
  }
  @media (max-width: 380px) {
    .mark {
      display: none;
    }
    .tools {
      margin-left: auto;
    }
  }
</style>
