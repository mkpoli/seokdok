<script>
  // A 두벌식 keyboard for touch screens and for readers learning the layout.
  // Each key shows the jamo it types and, in its corner, the one on the other
  // layer (Shift), drawn larger and darker when it is an old letter (ㆍ ㅿ ㆁ ㆆ ᄼ …).
  // Every key looks the same otherwise. With `map` it is
  // a picture of the layout only, without an input method behind it.
  import { keyToJamo } from '../../pkg/seokdok.js'
  import { t, r, plain } from '../lib/i18n.svelte.js'

  let { ime = null, focus = () => {}, map = false } = $props()
  let shift = $state(false)

  const rows = ['qwertyuiop', 'asdfghjkl', 'zxcvbnm']
  // An initial-only jamo (ᄼ) needs a vowel filler to show on its own.
  const shown = (j) => (/[\u1100-\u115f]/.test(j) ? j + '\u1160' : j)
  const isOld = (j) => /[ㆍㅿㆁㆆ\u1100-\u115f]/.test(j)
  const hangul = $derived(map || ime?.mode === 'hangul')
  const layer = (k, up) => keyToJamo(up ? k.toUpperCase() : k)
  const label = (k) => (hangul ? shown(layer(k, shift)) : '') || (shift ? k.toUpperCase() : k)
  /** The other layer's jamo, when it differs. */
  const hint = (k) => {
    if (!hangul) return ''
    const other = layer(k, !shift)
    return other && other !== layer(k, shift) ? other : ''
  }

  function press(key) {
    ime.key({ key, shift })
    if (shift && key.length === 1) shift = false
    focus()
  }
</script>

<div class="kb card" class:map role="group" aria-label={plain(t().ime.keyboard)}>
  {#each rows as row, r}
    <div class="row" style:padding-left="{r * 18}px">
      {#if r === 2}
        {#if map}
          <span class="key wide">⇧</span>
        {:else}
          <button class="key wide" aria-pressed={shift} onclick={() => (shift = !shift)}>⇧</button>
        {/if}
      {/if}
      {#each row as k}
        {@const h = hint(k)}
        {#if map}
          <span class="key" title={h ? `Shift+${k.toUpperCase()} → ${h}` : undefined}>
            {#if h}<span class="h" class:old={isOld(h)} lang="ko">{shown(h)}</span>{/if}<span class="j" lang="ko">{label(k)}</span><span class="l">{k}</span>
          </span>
        {:else}
          <button
            class="key"
            class:hit={ime.lastKey.toLowerCase() === k}
            onclick={() => press(shift ? k.toUpperCase() : k)}
            aria-label={label(k)}
          >
            {#if h}<span class="h" class:old={isOld(h)} lang="ko">{shown(h)}</span>{/if}<span class="j" lang="ko">{label(k)}</span><span class="l">{k}</span>
          </button>
        {/if}
      {/each}
      {#if r === 2 && !map}
        <button class="key wide" onclick={() => press('Backspace')} aria-label="Backspace">⌫</button>
      {/if}
    </div>
  {/each}
  {#if !map}
  <div class="row">
    <button class="key wide" onclick={() => press('Escape')}>Esc</button>
    <button class="key wide mode" aria-label="韓 / A" onclick={() => { ime.toggleMode(); focus() }}>{ime.mode === 'hangul' ? '韓' : 'A'}</button>
    <button class="key space" onclick={() => press(' ')}>{#if ime?.hangulOnly}␣{:else}{@html r(t().ime.convert)}{/if}</button>
    <button class="key wide" onclick={() => ime.key({ key: ' ', shift: true }) || focus()}>␣</button>
    <button class="key wide accent" onclick={() => press('Enter')}>{@html r(t().ime.commit)}</button>
  </div>
  {/if}
</div>

<style>
  .kb {
    display: grid;
    gap: 6px;
    margin-top: var(--s-3);
    padding: var(--s-3);
    user-select: none;
  }
  .row {
    display: flex;
    justify-content: center;
    gap: 6px;
  }
  .key {
    position: relative;
    display: grid;
    place-items: center;
    flex: 0 1 52px;
    min-width: 0;
    height: 52px;
    padding: 0;
    border: 1px solid var(--c-border);
    border-radius: var(--r-1);
    background: var(--c-surface);
    color: var(--c-ink);
    box-shadow: 0 2px 0 var(--c-border);
    cursor: pointer;
    transition:
      transform 80ms,
      background var(--t-fast);
  }
  .key:active,
  .key.hit {
    transform: translateY(2px);
    box-shadow: none;
    background: var(--c-accent-soft);
  }
  .key[aria-pressed='true'] {
    background: var(--c-accent-soft);
    color: var(--c-accent);
  }
  .j {
    font: 600 20px/1 var(--f-sans);
  }
  /* The other layer's jamo, in the corner. */
  .h {
    position: absolute;
    top: 4px;
    right: 5px;
    font: 600 12px/1 var(--f-sans);
    color: var(--c-muted);
  }
  .h.old {
    top: 3px;
    right: 4px;
    font: 800 16px/1 var(--f-sans);
    color: var(--c-ink);
  }
  .map .key {
    cursor: default;
  }
  .l {
    font: 600 10px/1 var(--f-mono);
    color: var(--c-muted);
  }
  .wide {
    flex-basis: 76px;
    font: 700 14px/1 var(--f-sans);
  }
  .space {
    flex: 1 1 280px;
    font: 700 14px/1 var(--f-sans);
  }
  .mode {
    font: 800 17px/1 var(--f-serif);
    color: var(--c-accent);
  }
  .accent {
    background: var(--c-accent);
    border-color: var(--c-accent);
    color: var(--c-on-accent);
  }
  @media (max-width: 520px) {
    .key {
      height: 46px;
    }
    .j {
      font-size: 15px;
    }
    .l {
      display: none;
    }
    .h {
      top: 2px;
      right: 2px;
      font-size: 9px;
    }
    .h.old {
      top: 2px;
      right: 2px;
      font-size: 12px;
    }
    .row {
      gap: 4px;
      padding-left: 0 !important;
    }
  }
</style>
