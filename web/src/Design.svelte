<script>
  // The design-system reference: every token and control, in both schemes.
  let scheme = $state('auto')
  $effect(() => {
    if (scheme === 'auto') delete document.documentElement.dataset.theme
    else document.documentElement.dataset.theme = scheme
  })
  const colors = [
    'canvas', 'canvas-2', 'surface', 'sunken', 'hover', 'ink', 'soft', 'muted', 'border', 'border-strong',
    'accent', 'accent-hover', 'accent-soft', 'on-accent', 'ruby', 'preedit', 'telegram',
  ]
  const space = [1, 2, 3, 4, 5, 6, 7, 8, 9]
  let a = $state(true)
  let b = $state(false)
  let pick = $state('surface')
  let on = $state(true)
  let seg = $state(1)
</script>

<main class="section">
  <p class="eyebrow">seokdok design system</p>
  <h1 class="h2">釋讀 · Design</h1>
  <div class="seg" role="group" aria-label="scheme">
    {#each ['auto', 'light', 'dark'] as s}
      <button aria-pressed={scheme === s} onclick={() => (scheme = s)}>{s}</button>
    {/each}
  </div>

  <h2>Colour</h2>
  <div class="swatches">
    {#each colors as c}
      <figure><span style:background="var(--c-{c})"></span><figcaption>--c-{c}</figcaption></figure>
    {/each}
  </div>

  <h2>Type</h2>
  <p class="display">釋讀</p>
  <p class="club">풀어읽기모임</p>
  <p class="serif"><ruby>音讀<rt>음독</rt></ruby>에<ruby>疲<rt>지치</rt></ruby>ㄴ <ruby>當身<rt>당신</rt></ruby>을<ruby>爲<rt>위</rt></ruby>한、 — --f-serif</p>
  <p>Body text in --f-sans. 本文은 Noto Sans KR로 짭니다.</p>

  <h2>Space</h2>
  <div class="space">
    {#each space as n}<span style:width="var(--s-{n})" title="--s-{n}"></span>{/each}
  </div>

  <h2>Controls</h2>
  <div class="row">
    <button class="btn" data-variant="primary">Primary</button>
    <button class="btn">Default</button>
    <button class="btn" data-size="lg">Large</button>
    <button class="btn" disabled>Disabled</button>
  </div>
  <div class="row">
    <label class="ui-field"><input class="ui-check" type="checkbox" bind:checked={a} />Checkbox</label>
    <label class="ui-field"><input class="ui-check" type="checkbox" bind:checked={b} />Unchecked</label>
    <label class="ui-field"><input class="ui-check" type="checkbox" disabled />Disabled</label>
  </div>
  <div class="row">
    <label class="ui-field"><input class="ui-radio" type="radio" bind:group={pick} value="surface" />表音式</label>
    <label class="ui-field"><input class="ui-radio" type="radio" bind:group={pick} value="morpheme" />形態式</label>
  </div>
  <div class="row">
    <label class="ui-field"><input class="ui-switch" type="checkbox" role="switch" bind:checked={on} />Switch</label>
    <div class="seg" role="group" aria-label="segmented">
      {#each ['한', '韓', 'EN'] as l, i}<button aria-pressed={seg === i} onclick={() => (seg = i)}>{l}</button>{/each}
    </div>
    <input class="ui-input" placeholder="Search" aria-label="search" />
  </div>

  <h2>Scrollbar</h2>
  <div class="scroll card">{'訓讀 '.repeat(400)}</div>
</main>

<style>
  h2 {
    margin: var(--s-7) 0 var(--s-3);
    font: 700 18px/1.4 var(--f-sans);
  }
  .swatches {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
    gap: var(--s-3);
  }
  figure {
    margin: 0;
  }
  figure span {
    display: block;
    height: 56px;
    border: 1px solid var(--c-border);
    border-radius: var(--r-2);
  }
  figcaption {
    margin-top: 4px;
    color: var(--c-muted);
    font: 12px var(--f-mono);
  }
  .display {
    margin: 0;
    color: var(--c-accent);
    font: 900 96px/1 var(--f-serif);
  }
  .club {
    margin: var(--s-2) 0;
    color: var(--c-accent);
    font: 700 28px/1.2 var(--f-round);
    letter-spacing: 0.3em;
  }
  .serif {
    font: 700 22px/2 var(--f-serif);
  }
  .space {
    display: flex;
    align-items: end;
    gap: var(--s-3);
  }
  .space span {
    height: 24px;
    background: var(--c-accent);
    border-radius: 3px;
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--s-4);
    margin-bottom: var(--s-4);
  }
  .scroll {
    height: 140px;
    overflow: auto;
    padding: var(--s-4);
    font-family: var(--f-serif);
  }
</style>
