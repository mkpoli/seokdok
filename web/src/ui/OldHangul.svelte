<script>
  import { reveal } from '../lib/reveal.js'
  import { t, r } from '../lib/i18n.svelte.js'

  // Keys as on Microsoft Old Hangul; every example is checked by the
  // composer's tests (crates/seokdok-core/src/dubeolsik.rs).
  const letters = [
    ['K', 'ㆍ', 'araea'],
    ['A', 'ㅿ', 'pansios'],
    ['D', 'ㆁ', 'yesieung'],
    ['G', 'ㆆ', 'yeorinhieuh'],
    ['Z', 'ᄼᅠ', 'chitueumS'],
    ['X', 'ᄾᅠ', 'ceongchieumS'],
    ['C', 'ᅎᅠ', 'chitueumJ'],
    ['V', 'ᅐᅠ', 'ceongchieumJ'],
    ['B', 'ᅔᅠ', 'chitueumCh'],
    ['N', 'ᅕᅠ', 'ceongchieumCh'],
  ]
  const examples = [
    [['G', '⇧K'], 'ᄒᆞ'],
    [['G', '⇧K', 'S'], 'ᄒᆞᆫ'],
    [['G', '⇧K', 'D', 'U'], 'ᄒᆞ여'],
    [['R', '⇧K', 'L'], 'ᄀᆡ'],
    [['R', '⇧K', '⇧K'], 'ᄀᆢ'],
    [['⇧A', 'K'], 'ᅀᅡ'],
  ]
  const clusters = [
    [['T', 'R', 'N', 'A'], 'ᄭᅮᆷ'],
    [['Q', 'D', 'K'], 'ᄫᅡ'],
    [['Q', 'T', 'R', 'K'], 'ᄢᅡ'],
    [['G', 'G'], 'ㆅ'],
  ]
</script>

<section class="section" id="old">
  <p class="eyebrow" use:reveal>{@html r(t().old.eyebrow)}</p>
  <h2 class="h2" use:reveal={1}>{@html r(t().old.title)}</h2>
  <p class="lede" use:reveal={2}>{@html r(t().old.lede)}</p>

  <div class="grid">
    <div class="card block" use:reveal={3}>
      <h3>{@html r(t().old.shift)}</h3>
      <ul class="keys">
        {#each letters as [k, j, name]}
          <li><span class="combo"><kbd>⇧</kbd>+<kbd>{k}</kbd></span><span class="j" lang="ko">{j}</span><span class="n">{@html r(t().old.names[name])}</span></li>
        {/each}
      </ul>
    </div>
    <div class="card block" use:reveal={4}>
      <h3>{@html r(t().old.examples)}</h3>
      <ul class="ex">
        {#each examples as [keys, out]}
          <li><span class="seq">{#each keys as k}<kbd>{k}</kbd>{/each}</span><span class="arrow">→</span><span class="j" lang="ko">{out}</span></li>
        {/each}
      </ul>
      <h3>{@html r(t().old.clusters)}</h3>
      <p class="note">{@html r(t().old.clustersNote)}</p>
      <ul class="ex">
        {#each clusters as [keys, out]}
          <li><span class="seq">{#each keys as k}<kbd>{k}</kbd>{/each}</span><span class="arrow">→</span><span class="j" lang="ko">{out}</span></li>
        {/each}
      </ul>
    </div>
  </div>
  <p class="after" use:reveal={5}>{@html r(t().old.conversion)} {@html r(t().old.fonts)}</p>
</section>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(min(100%, 380px), 1fr));
    gap: var(--s-5);
  }
  .block {
    padding: var(--s-5);
  }
  h3 {
    margin: 0 0 var(--s-3);
    font: 800 20px/1.4 var(--f-serif);
  }
  h3:not(:first-child) {
    margin-top: var(--s-5);
  }
  ul {
    display: grid;
    gap: var(--s-2);
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .keys li {
    display: grid;
    grid-template-columns: max-content 2.2em 1fr;
    align-items: center;
    gap: var(--s-1) var(--s-3);
  }
  .combo {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    white-space: nowrap;
  }
  .j {
    font: 700 26px/1.2 var(--f-serif);
    color: var(--c-accent);
  }
  .n {
    color: var(--c-soft);
    font-size: 14px;
  }
  .ex li {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--s-2);
  }
  .seq {
    display: inline-flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .arrow {
    color: var(--c-muted);
  }
  .note {
    margin: 0 0 var(--s-3);
    color: var(--c-soft);
    font-size: 14px;
  }
  .after {
    max-width: 75ch;
    margin: var(--s-5) 0 0;
    color: var(--c-soft);
    font-size: 15px;
  }
</style>
