<script>
  // The logo as a select: the two input methods, and the wider set of writing-system projects.
  import { i18n, t, r, plain } from '../lib/i18n.svelte.js'

  // `home` is where the current site's own entry leads.
  let { current, home = '', children } = $props()

  const SITES = [
    { id: 'seokdok', href: '/', logo: '/favicon.svg', han: '釋讀', latn: 'Seokdok', lang: 'ko' },
    { id: 'eumdok', href: '/eumdok', logo: '/favicon-eumdok.svg', han: '音讀', latn: 'Eumdok', lang: 'ko' },
  ]
  // The trigger and the menu carry the current site's name as well as the label.
  const label = $derived.by(() => {
    const own = SITES.find((s) => s.id === current)
    return `${own.han} ${own.latn}: ${plain(t().sites.label)}`
  })
  const more = $derived(`https://scripts.mkpo.li/${i18n.lang === 'en' ? '' : 'ko/'}`)

  let open = $state(false)
  let root = $state()
  let trigger = $state()
  const items = () => [...root.querySelectorAll('[role="menuitem"]')]

  function show() {
    open = true
    // After the menu has rendered, as a select opens on its current option.
    requestAnimationFrame(() => root?.querySelector('[aria-current]')?.focus())
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

  // The menu's keys stay in it, so that page shortcuts do not act behind it.
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
</script>

<div class="select" bind:this={root}>
  <button
    bind:this={trigger}
    class="trigger"
    type="button"
    aria-haspopup="menu"
    aria-expanded={open}
    aria-label={label}
    onclick={() => (open ? hide() : show())}
    onkeydown={onTriggerKey}
  >
    {@render children()}
    <svg class="chevron" class:up={open} viewBox="0 0 10 10" aria-hidden="true"><path d="M1.5 3.5 5 7l3.5-3.5" /></svg>
  </button>

  {#if open}
    <div class="menu" role="menu" tabindex="-1" aria-label={label} onkeydown={onMenuKey}>
      {#each SITES as site (site.id)}
        <a
          class="item"
          role="menuitem"
          tabindex="-1"
          href={site.id === current ? `${home}#top` : site.href}
          aria-current={site.id === current ? 'true' : undefined}
          onclick={() => hide(site.id === current)}
        >
          <img src={site.logo} alt="" width="32" height="32" />
          <span class="name"><span class="han" lang={site.lang}>{site.han}</span><span class="latn">{site.latn}</span></span>
          {#if site.id === current}
            <svg class="tick" viewBox="0 0 16 16" aria-hidden="true"><path d="M3 8.5 6.5 12 13 4.5" /></svg>
          {/if}
        </a>
      {/each}
      <a class="item more" role="menuitem" tabindex="-1" href={more} onclick={() => hide()}>
        <span class="more-label">{@html r(t().sites.more)}</span>
        <span class="out" aria-hidden="true">↗</span>
      </a>
    </div>
  {/if}
</div>

<style>
  .select {
    position: relative;
    flex-shrink: 0;
    margin-right: auto;
  }
  .trigger {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: 4px;
    margin: -4px;
    border: 0;
    border-radius: var(--r-1);
    background: none;
    color: var(--c-accent);
    cursor: pointer;
  }
  .trigger:hover {
    background: var(--c-hover);
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
    transition: transform 120ms ease;
  }
  .chevron.up {
    transform: rotate(180deg);
  }
  .menu {
    position: absolute;
    top: calc(100% + var(--s-3));
    left: -4px;
    z-index: 30;
    width: 232px;
    padding: var(--s-1);
    background: var(--c-surface);
    border: 1px solid var(--c-border);
    border-radius: var(--r-3);
    box-shadow: var(--shadow-2);
    outline: none;
  }
  .item {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    padding: var(--s-2);
    border-radius: var(--r-2);
    color: var(--c-ink);
    text-decoration: none;
    outline: none;
  }
  .item:hover,
  .item:focus-visible {
    background: var(--c-hover);
  }
  .item:focus-visible {
    outline: 2px solid var(--c-accent);
    outline-offset: -2px;
  }
  .item[aria-current] {
    background: var(--c-accent-soft);
  }
  .item img {
    flex: none;
    border-radius: 18%;
  }
  .name {
    display: flex;
    flex: 1;
    align-items: baseline;
    gap: var(--s-2);
    min-width: 0;
  }
  .han {
    font: 900 20px/1 var(--f-serif);
  }
  .latn {
    font: 700 12px/1 var(--f-sans);
    letter-spacing: 0.08em;
    color: var(--c-muted);
  }
  .tick {
    flex: none;
    width: 16px;
    height: 16px;
    fill: none;
    stroke: var(--c-accent);
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .more {
    justify-content: space-between;
    margin-top: var(--s-1);
    padding-block: var(--s-3);
    border-top: 1px solid var(--c-border);
    border-radius: 0 0 var(--r-2) var(--r-2);
    color: var(--c-soft);
    font: 600 13px/1.4 var(--f-sans);
  }
  .more-label :global(rt) {
    font-size: 0.5em;
  }
  .out {
    color: var(--c-muted);
  }
  @media (prefers-reduced-motion: reduce) {
    .chevron {
      transition: none;
    }
  }
</style>
