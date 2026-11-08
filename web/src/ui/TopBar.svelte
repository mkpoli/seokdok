<script>
  import { i18n, setLang, t, r, plain } from '../lib/i18n.svelte.js'
  import { options, saveOptions, showRubyPosition } from '../lib/options.svelte.js'
  import SiteSelect from './SiteSelect.svelte'
  import SponsorDialog from './SponsorDialog.svelte'

  // On a page other than the front page, `home` is '/' and the section links lead back to it.
  let { home = '' } = $props()

  // Five sections stay in the bar; the rest fold into the menu. From 641px to
  // 999px the menu button is the only navigation; phones keep the existing row
  // without section links.
  const primary = ['ime', 'custom', 'dict', 'convert', 'install']
  const sections = [...primary, 'rules', 'words', 'old', 'eumdok', 'join']
  const href = (key) => (key === 'dict' ? '/dict' : `${home}#${key}`)

  let open = $state(false)
  let root = $state()
  let trigger = $state()
  // Inline links are display:none at wide sizes, so only the visible items count.
  const items = () => [...root.querySelectorAll('[role="menuitem"]')].filter((el) => el.getClientRects().length)

  function show() {
    open = true
    requestAnimationFrame(() => items()[0]?.focus())
  }
  function hide(refocus = false) {
    open = false
    if (refocus) trigger?.focus()
  }

  function onTriggerKey(event) {
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault()
      show()
    }
  }

  function onMenuKey(event) {
    event.stopPropagation()
    const list = items()
    const at = list.indexOf(document.activeElement)
    if (event.key === 'Escape') {
      event.preventDefault()
      hide(true)
    } else if (event.key === 'Tab') {
      hide(true)
    } else if (event.key === ' ') {
      event.preventDefault()
      list[at]?.click()
    } else {
      const to = { ArrowDown: at + 1, ArrowUp: at - 1, Home: 0, End: list.length - 1 }[event.key]
      if (to === undefined) return
      event.preventDefault()
      list[(to + list.length) % list.length].focus()
    }
  }

  $effect(() => {
    if (!open) return
    const away = (event) => {
      if (!event.composedPath().includes(root)) hide()
    }
    document.addEventListener('pointerdown', away, true)
    return () => document.removeEventListener('pointerdown', away, true)
  })

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
  <nav aria-label={plain(t().nav.sections)}>
    {#each primary as key}
      <a href={href(key)}>{@html r(t().nav[key])}</a>
    {/each}
    <div class="more" bind:this={root}>
      <button
        bind:this={trigger}
        type="button"
        class="more-btn"
        aria-haspopup="menu"
        aria-expanded={open}
        aria-label={plain(t().nav.more)}
        onclick={() => (open ? hide() : show())}
        onkeydown={onTriggerKey}
      >
        <span class="more-text">{@html r(t().nav.more)}</span>
        <svg class="chevron" class:up={open} viewBox="0 0 10 10" aria-hidden="true"><path d="M1.5 3.5 5 7l3.5-3.5" /></svg>
        <svg class="menu-icon" viewBox="0 0 16 16" aria-hidden="true"><path d="M2.5 4.5h11M2.5 8h11M2.5 11.5h11" /></svg>
      </button>
      {#if open}
        <div class="menu" role="menu" tabindex="-1" aria-label={plain(t().nav.more)} onkeydown={onMenuKey}>
          {#each sections as key}
            <a class="item" class:inline={primary.includes(key)} role="menuitem" tabindex="-1" href={href(key)} onclick={() => hide()}>
              {@html r(t().nav[key])}
            </a>
          {/each}
        </div>
      {/if}
    </div>
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
  /* The links left after the menu are short enough to fit at 1000px and up. */
  nav {
    flex: 1 1 0;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: var(--s-3);
  }
  nav > a {
    flex-shrink: 0;
    white-space: nowrap;
    color: var(--c-soft);
    font-weight: 600;
    font-size: 14px;
    line-height: 1.6;
    text-decoration: none;
  }
  nav > a:hover {
    color: var(--c-accent);
  }
  .more {
    position: relative;
    flex: none;
  }
  .more-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 36px;
    padding: 0 4px;
    border: 1px solid transparent;
    border-radius: 999px;
    background: none;
    color: var(--c-soft);
    font: 600 14px/1.6 var(--f-sans);
    white-space: nowrap;
    cursor: pointer;
  }
  .more-btn:hover,
  .more-btn[aria-expanded='true'] {
    color: var(--c-accent);
    background: var(--c-hover);
  }
  .more-btn:focus-visible {
    outline: 2px solid var(--c-accent);
    outline-offset: 2px;
  }
  .more-btn :global(rt) {
    font-size: 0.5em;
  }
  .chevron {
    width: 10px;
    height: 10px;
    flex: none;
    fill: none;
    stroke: var(--c-muted);
    stroke-width: 1.5;
    stroke-linecap: round;
    stroke-linejoin: round;
    transition: transform var(--t-fast) var(--ease);
  }
  .chevron.up {
    transform: rotate(180deg);
  }
  .menu-icon {
    display: none;
    width: 18px;
    height: 18px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.8;
    stroke-linecap: round;
  }
  .menu {
    position: absolute;
    top: calc(100% + var(--s-3));
    left: 0;
    z-index: 30;
    min-width: 176px;
    max-height: calc(100dvh - 80px);
    overflow-y: auto;
    padding: var(--s-1);
    background: var(--c-surface);
    border: 1px solid var(--c-border);
    border-radius: var(--r-3);
    box-shadow: var(--shadow-2);
  }
  .item {
    display: block;
    padding: var(--s-2) var(--s-3);
    border-radius: var(--r-2);
    color: var(--c-ink);
    font: 600 14px/1.5 var(--f-sans);
    text-decoration: none;
    white-space: nowrap;
  }
  .item:hover,
  .item:focus-visible {
    background: var(--c-hover);
    color: var(--c-accent);
  }
  .item:focus-visible {
    outline: 2px solid var(--c-accent);
    outline-offset: -2px;
  }
  .item :global(rt) {
    font-size: 0.5em;
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
  @media (max-width: 1099px) {
    .mark-latn {
      display: none;
    }
  }
  /* The bar keeps one round menu button; its menu lists every section. */
  @media (max-width: 999px) {
    nav > a {
      display: none;
    }
    .more {
      margin-left: auto;
    }
    .more-btn {
      width: 36px;
      height: 36px;
      justify-content: center;
      padding: 0;
      border-color: var(--c-border);
      background: var(--c-surface);
    }
    .more-text,
    .chevron {
      display: none;
    }
    .menu-icon {
      display: block;
    }
  }
  @media (min-width: 1000px) {
    .item.inline {
      display: none;
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
