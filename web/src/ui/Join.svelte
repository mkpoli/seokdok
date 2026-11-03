<script>
  import { t, r, plain } from '../lib/i18n.svelte.js'
  import { reveal } from '../lib/reveal.js'
</script>

<section class="join" id="join">
  <div class="inner">
    <div use:reveal>
      <p class="club">{@html r(t().join.title)}</p>
      <p class="big">釋讀會</p>
    </div>
    <div class="body" use:reveal={1}>
      <p class="lede">{@html r(t().join.lede)}</p>
      <blockquote lang="ko">
        {@html r(t().join.disclaimer)}
        <cite>{@html r(t().join.disclaimerSource)}</cite>
      </blockquote>
      <figure class="pres">
        <blockquote lang="ko">
          <p>釋讀會 - 풀어읽기모임<br />정상 운영 中입니다！</p>
          <p class="bio">풀어읽기모임(釋讀會)은 흥미 본위의 훈독 사용 동호회입니다！</p>
        </blockquote>
        <figcaption>
          — {@html r(t().join.president)}
          (<a href="https://x.com/chabek6/status/2040036944189743390" target="_blank" rel="noopener">@chabek6</a>, 2026-04-03)
        </figcaption>
      </figure>
      <a class="tg" href="https://t.me/pimsdh" target="_blank" rel="noopener">
        <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="12" fill="var(--c-telegram)" /><path fill="#fff" d="M5.4 11.8 16.6 7.5c.5-.2 1 .1.8.9l-1.9 9c-.1.6-.5.8-1 .5l-2.8-2.1-1.4 1.3c-.2.2-.3.3-.6.3l.2-2.9 5.3-4.8c.2-.2 0-.3-.3-.1L8.4 13.7l-2.8-.9c-.6-.2-.6-.6.1-.9z" /></svg>
        <span>https://t.me/pimsdh</span>
      </a>
    </div>
  </div>
</section>

<style>
  .join {
    margin-top: var(--s-7);
    background: var(--c-accent);
    color: var(--c-on-accent);
  }
  .inner {
    display: grid;
    grid-template-columns: auto 1fr;
    align-items: center;
    gap: var(--s-7);
    width: min(var(--w-page), 100%);
    margin-inline: auto;
    padding: var(--s-8) var(--s-4);
  }
  .club {
    margin: 0;
    font: 700 clamp(22px, 3vw, 34px) / 1.2 var(--f-round);
    letter-spacing: 0.3em;
  }
  .big {
    margin: 0;
    font: 900 clamp(84px, 12vw, 150px) / 1.05 var(--f-serif);
  }
  .lede {
    max-width: none;
    margin: 0 0 var(--s-4);
    color: inherit;
    font-size: 18px;
  }
  blockquote {
    margin: 0 0 var(--s-5);
    padding-left: var(--s-4);
    border-left: 3px solid currentColor;
    font: 700 17px/1.6 var(--f-sans);
  }
  cite {
    display: block;
    margin-top: var(--s-1);
    font: 400 13px/1.4 var(--f-sans);
    opacity: 0.8;
  }
  .pres {
    margin: 0 0 var(--s-5);
    padding: var(--s-4) var(--s-5);
    border-radius: var(--r-2);
    background: color-mix(in srgb, var(--c-on-accent) 12%, transparent);
  }
  .pres blockquote {
    margin: 0;
    padding: 0;
    border: 0;
  }
  .pres p {
    margin: 0;
    font: 700 clamp(19px, 2.2vw, 24px) / 1.5 var(--f-serif);
  }
  .pres .bio {
    margin-top: var(--s-2);
    font: 500 15px/1.6 var(--f-sans);
    opacity: 0.9;
  }
  .pres figcaption {
    margin-top: var(--s-2);
    font-size: 13px;
    opacity: 0.85;
  }
  .pres a {
    color: inherit;
  }
  .tg {
    display: inline-flex;
    align-items: center;
    gap: var(--s-3);
    padding: var(--s-2) var(--s-5) var(--s-2) var(--s-2);
    border-radius: 999px;
    background: var(--c-on-accent);
    color: var(--c-accent);
    font: 700 clamp(18px, 2.2vw, 24px) / 1 var(--f-serif);
    text-decoration: none;
    transition: transform var(--t-fast) var(--ease);
  }
  .tg:hover {
    transform: translateY(-2px);
  }
  .tg svg {
    width: 44px;
    height: 44px;
  }
  @media (max-width: 760px) {
    .inner {
      grid-template-columns: 1fr;
      gap: var(--s-5);
    }
  }
</style>
