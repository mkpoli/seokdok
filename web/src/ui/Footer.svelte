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
    <li><a href="https://kaikki.org/dictionary/Korean/" rel="noopener">English Wiktionary</a> via kaikki.org (wiktextract) — CC BY-SA 4.0</li>
    <li><a href="https://ko.wikisource.org/" rel="noopener">Korean Wikisource</a> transcriptions of 訓蒙字會 (1527), 光州千字文 (1575), 石峯千字文 (1583), 類合, 倭語類解, 譯語類解, 諺解, and books of 1882-1945 such as 勞動夜學讀本 (1908) — CC BY-SA 4.0; the books are in the public domain</li>
    <li><a href="https://www.edrdg.org/jmdict/j_jmdict.html" rel="noopener">JMdict</a> — CC BY-SA 4.0, EDRDG</li>
    <li><a href="https://github.com/BYVoid/OpenCC" rel="noopener">OpenCC</a> — Apache-2.0</li>
    <li><a href="https://github.com/notofonts/noto-cjk" rel="noopener">Noto CJK</a> fonts — SIL OFL 1.1</li>
    <li>
      <a href="https://fontsource.org/" rel="noopener">Fontsource</a> builds of Noto Sans KR and Noto Serif KR (Google) and
      <a href="https://github.com/yangheeryu/Gowun-Batang" rel="noopener">Gowun Batang</a> (Gowun Batang Project Authors) — SIL OFL 1.1
    </li>
    <li>{@html r(t().footer.suggested)}</li>
    <li>釋讀會 · 漢字混用 Telegram — 2022–2026</li>
    <li><a href="/dict/NOTICE.txt">NOTICE.txt</a></li>
  </ul>
  <p class="meta">{@html r(t().footer.licence)} · {@html r(t().footer.made)} <a href="https://mkpo.li/">mkpoli</a> · <a href="https://github.com/mkpoli/seokdok" rel="noopener">GitHub</a></p>
  <p class="meta legal">
    <a href="/privacy">{@html r(t().footer.privacy)}</a> · <a href="/support">{@html r(t().footer.support)}</a>
  </p>
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
  .legal {
    margin-top: var(--s-2);
  }
</style>
