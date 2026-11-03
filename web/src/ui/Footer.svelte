<script>
  import { t, r, plain } from '../lib/i18n.svelte.js'
</script>

<footer class="section foot">
  <h2>{@html r(t().footer.sources)}</h2>
  <ul>
    <li>
      <a href="https://docs.google.com/spreadsheets/d/1J7bPuQ4buOM17hvLR6UHLfcrERfYhaVVj1H-3szh7J0" rel="noopener">標準國語釋讀辭典</a>
      — 釋讀會
    </li>
    <li><a href="https://github.com/libhangul/libhangul" rel="noopener">libhangul</a> hanja.txt — BSD-3-Clause, © Choe Hwanjin</li>
    <li><a href="https://github.com/Lee0701/FusionIME" rel="noopener">FusionIME (十八字)</a> — GPL-3.0, Lee0701</li>
    <li>釋讀會 · 漢字混用 Telegram — 2022–2026</li>
    <li><a href="/dict/NOTICE.txt">NOTICE.txt</a></li>
  </ul>
  <p class="meta">{@html r(t().footer.licence)} · {@html r(t().footer.made)} <a href="https://mkpo.li/">mkpoli</a> · <a href="https://github.com/mkpoli/seokdok" rel="noopener">GitHub</a></p>
</footer>

<style>
  .foot {
    padding-block: var(--s-6);
    color: var(--c-muted);
    font-size: 14px;
  }
  h2 {
    margin: 0 0 var(--s-2);
    color: var(--c-soft);
    font: 700 14px/1.4 var(--f-sans);
  }
  ul {
    margin: 0 0 var(--s-4);
    padding-left: 1.2em;
  }
  .meta {
    margin: 0;
  }
</style>
