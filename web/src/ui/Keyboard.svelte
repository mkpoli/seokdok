<script>
  // A 두벌식 keyboard for touch screens and for readers learning the layout.
  import { keyToJamo } from '../../pkg/seokdok.js'
  import { t, r, plain } from '../lib/i18n.svelte.js'

  let { ime, focus } = $props()
  let shift = $state(false)

  const rows = ['qwertyuiop', 'asdfghjkl', 'zxcvbnm']
  const label = (k) => (ime.mode === 'hangul' ? keyToJamo(shift ? k.toUpperCase() : k) : '') || (shift ? k.toUpperCase() : k)

  function press(key) {
    ime.key({ key, shift })
    if (shift && key.length === 1) shift = false
    focus()
  }
</script>

<div class="kb card" role="group" aria-label={plain(t().ime.keyboard)}>
  {#each rows as row, r}
    <div class="row" style:padding-left="{r * 18}px">
      {#if r === 2}
        <button class="key wide" aria-pressed={shift} onclick={() => (shift = !shift)}>⇧</button>
      {/if}
      {#each row as k}
        <button
          class="key"
          class:hit={ime.lastKey.toLowerCase() === k}
          onclick={() => press(shift ? k.toUpperCase() : k)}
          aria-label={label(k)}
        >
          <span class="j">{label(k)}</span><span class="l">{k}</span>
        </button>
      {/each}
      {#if r === 2}
        <button class="key wide" onclick={() => press('Backspace')} aria-label="Backspace">⌫</button>
      {/if}
    </div>
  {/each}
  <div class="row">
    <button class="key wide" onclick={() => press('Escape')}>Esc</button>
    <button class="key wide mode" aria-label="韓 / A" onclick={() => { ime.toggleMode(); focus() }}>{ime.mode === 'hangul' ? '韓' : 'A'}</button>
    <button class="key space" onclick={() => press(' ')}>{@html r(t().ime.convert)}</button>
    <button class="key wide" onclick={() => ime.key({ key: ' ', shift: true }) || focus()}>␣</button>
    <button class="key wide accent" onclick={() => press('Enter')}>{@html r(t().ime.commit)}</button>
  </div>
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
      font-size: 17px;
    }
    .l {
      display: none;
    }
    .row {
      gap: 4px;
      padding-left: 0 !important;
    }
  }
</style>
