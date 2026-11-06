<script>
  import { i18n, setLang, t, r, plain } from '../lib/i18n.svelte.js'
  import { options, saveOptions, showRubyPosition } from '../lib/options.svelte.js'
  import SiteSelect from './SiteSelect.svelte'
  import SponsorDialog from './SponsorDialog.svelte'

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

  let sponsorOpen = $state(false)
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
  <SiteSelect current="seokdok" {home}>
    <span class="mark"><img class="mark-logo" src="/favicon.svg" alt="" width="28" height="28" /><span class="mark-han">釋讀</span><span class="mark-latn">Seokdok</span></span>
  </SiteSelect>
  <nav aria-label="sections">
    <a href="{home}#ime">{@html r(t().nav.ime)}</a>
    <a href="{home}#custom">{@html r(t().nav.custom)}</a>
    <a href="{home}#rules">{@html r(t().nav.rules)}</a>
    <a href="{home}#words">{@html r(t().nav.words)}</a>
    <a href="/dict">{@html r(t().nav.dict)}</a>
    <a href="{home}#convert">{@html r(t().nav.convert)}</a>
    <a href="{home}#old">{@html r(t().nav.old)}</a>
    <a href="{home}#install">{@html r(t().nav.install)}</a>
    <a href="{home}#eumdok">{@html r(t().nav.eumdok)}</a>
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
    <button type="button" class="sponsor" title={t().sponsor.label} aria-label={t().sponsor.label} aria-haspopup="dialog" aria-expanded={sponsorOpen} onclick={() => { sponsorOpen = true }}>
      <span class="sponsor-face" aria-hidden="true">
        <img src="/sponsor-face.webp" alt="" width="24" height="24" decoding="async" />
        <svg class="sponsor-heart" viewBox="0 0 24 24"><path d="M12 21s-8.5-5.1-8.5-11.3A4.7 4.7 0 0 1 12 6.9a4.7 4.7 0 0 1 8.5 2.8C20.5 15.9 12 21 12 21Z" /></svg>
      </span>
    </button>
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
<SponsorDialog bind:open={sponsorOpen} />

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
  }
  .mark-logo {
    align-self: center;
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
  /* The links scroll inside the space the logo and the tools leave. */
  nav {
    flex: 1 1 0;
    min-width: 0;
    overflow-x: auto;
    scrollbar-width: none;
    display: flex;
    /* Links with and without ruby share one baseline (國漢文混用 drops some). */
    align-items: baseline;
    gap: var(--s-3);
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
  /* A soft pink round button with the author's face and a small heart that beats. */
  .sponsor {
    --sp-from: light-dark(#ffe6ef, #3d1a2b);
    --sp-to: light-dark(#fff0e0, #3a2318);
    --sp-ink: light-dark(#c2185b, #ffb3cd);
    --sp-heart: light-dark(#ff4f88, #ff7aa6);
    --sp-line: light-dark(#ffd0de, #5a2a3f);
    display: grid;
    place-items: center;
    width: 36px;
    height: 36px;
    padding: 0;
    border: 1px solid var(--sp-line);
    border-radius: 999px;
    background: linear-gradient(135deg, var(--sp-from), var(--sp-to));
    cursor: pointer;
    transition: transform var(--t-fast) var(--ease), box-shadow var(--t-fast) var(--ease);
  }
  .sponsor:hover,
  .sponsor:focus-visible {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgb(255 79 136 / 0.28);
  }
  .sponsor:active {
    transform: translateY(0) scale(0.97);
  }
  .sponsor:focus-visible {
    outline: 2px solid var(--c-accent);
    outline-offset: 2px;
  }
  .sponsor-face {
    position: relative;
    flex: none;
    width: 24px;
    height: 24px;
  }
  .sponsor-face img {
    display: block;
    width: 100%;
    height: 100%;
    border-radius: 50%;
    object-fit: cover;
    box-shadow: 0 0 0 1.5px light-dark(#fff, #3d1a2b), 0 0 0 2.5px var(--sp-line);
  }
  .sponsor-heart {
    position: absolute;
    right: -4px;
    bottom: -3px;
    width: 12px;
    height: 12px;
    fill: var(--sp-heart);
    stroke: light-dark(#fff, #3d1a2b);
    stroke-width: 2.5;
    paint-order: stroke;
    overflow: visible;
    transform-origin: 50% 60%;
    animation: sponsor-beat 2.8s ease-in-out infinite;
  }
  @keyframes sponsor-beat {
    0%, 70%, 100% { transform: scale(1); }
    78% { transform: scale(1.22); }
    86% { transform: scale(0.96); }
    93% { transform: scale(1.12); }
  }
  @media (prefers-reduced-motion: reduce) {
    .sponsor,
    .sponsor-heart {
      animation: none;
      transition: none;
    }
  }
  @media (max-width: 640px) {
    nav {
      display: none;
    }
    .mark-logo,
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
    .sponsor {
      width: 30px;
      height: 30px;
    }
    .theme {
      width: 30px;
      height: 30px;
    }
  }
  /* Too narrow for both labels; the switch still reads as 釋讀 on or off. */
  @media (max-width: 440px) {
    .side:first-child {
      display: none;
    }
  }
</style>
