<script>
  import { t, r, rk, plain } from '../lib/i18n.svelte.js'

  const SLOGAN = '{音讀|음독}에{疲|지치}ㄴ {當身|당신}을{爲|위}한、'

  // The wall of 訓讀 verbs, after the third 釋讀會 poster: each Hanja with its
  // own reading as ruby, then the 末音添記. All are dictionary entries attested
  // in the group's chat.
  const WALL = [
    ['爲', '다', '하'], ['在', '다', '있'], ['成', '다', '되'], ['無', '다', '없'], ['不', 'ㅎ다', '안'], ['非', '다', '아니'],
    ['見', '다', '보'], ['同', '다', '같'], ['授', '다', '주'], ['去', '다', '가'], ['言', '하다', '말'], ['來', '다', '오'],
    ['知', '다', '알'], ['然', 'ㅎ다', '그러'], ['大', '다', '크'], ['多', '다', '많'], ['好', '다', '좋'], ['出', '다', '나'],
    ['思', '하다', '생각'], ['不知', '다', '모르'], ['作', '다', '만들'], ['食', '다', '먹'], ['何', '떻다', '어'], ['書', '다', '쓰'],
    ['讀', '다', '읽'], ['異', '다', '다르'], ['新', '롭다', '새'], ['歡', '갑다', '반'], ['樂', '겁다', '즐'], ['難', '다', '어렵'],
    ['易', '다', '쉽'], ['適', '다', '맞'], ['殘', '다', '남'], ['用', '이다', '쓰'], ['似', '하다', '비슷'], ['入', '다', '들'],
  ]
  const rows = Array.from({ length: 6 }, (_, r) => WALL.slice(r * 6, r * 6 + 6))
</script>

<section class="hero-band" id="top">
<div class="hero">
  <div class="text">
    <p class="slogan" aria-label={plain(SLOGAN)}>{@html rk(SLOGAN)}</p>
    <p class="club">{@html r(t().hero.kicker)}</p>
    <h1 class="title" aria-label="釋讀 seokdok">
      <span class="big">釋讀</span>
    </h1>
    <p class="claim">
      <span>{@html r(t().hero.title1)}</span>
      <span class="claim-han">{@html r(t().hero.title2)}</span>
    </p>
    <p class="lede">{@html r(t().hero.lede)}</p>
    <div class="cta">
      <a class="btn" data-variant="primary" data-size="lg" href="#ime">{@html r(t().hero.try)}</a>
      <a class="btn tg" data-size="lg" href="https://t.me/pimsdh" rel="noopener" target="_blank">
        <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="12" fill="var(--c-telegram)" /><path fill="#fff" d="M5.4 11.8 16.6 7.5c.5-.2 1 .1.8.9l-1.9 9c-.1.6-.5.8-1 .5l-2.8-2.1-1.4 1.3c-.2.2-.3.3-.6.3l.2-2.9 5.3-4.8c.2-.2 0-.3-.3-.1L8.4 13.7l-2.8-.9c-.6-.2-.6-.6.1-.9z" /></svg>
        {@html r(t().hero.join)}
      </a>
    </div>
  </div>
  <div class="wall" aria-hidden="true">
    {#each rows as row, r}
      <div class="row" style:--d="{r % 2 ? -1 : 1}" style:--i={r}>
        {#each [...row, ...row] as [han, okuri, reading], c}
          <span class="tile" style:--size="{1 + ((r * 7 + c * 3) % 5) * 0.12}em"><ruby>{han}<rt>{reading}</rt></ruby>{okuri}</span>
        {/each}
      </div>
    {/each}
  </div>
</div>
</section>

<style>
  .hero-band {
    background:
      radial-gradient(60% 60% at 78% 35%, var(--c-canvas-2), transparent 70%),
      linear-gradient(180deg, var(--c-canvas), var(--c-canvas-2));
    overflow: hidden;
  }
  .hero {
    position: relative;
    display: grid;
    grid-template-columns: minmax(0, 1.05fr) minmax(0, 1fr);
    align-items: center;
    gap: var(--s-6);
    width: min(var(--w-page), 100%);
    min-height: calc(100svh - 56px);
    margin-inline: auto;
    padding: var(--s-7) var(--s-4);
  }
  .slogan {
    margin: 0;
    font: 700 clamp(20px, 2.6vw, 28px) / 2 var(--f-serif);
    letter-spacing: 0.02em;
  }
  .slogan rt {
    font-size: 0.45em;
  }
  .club {
    margin: var(--s-2) 0 0;
    color: var(--c-accent);
    font: 700 clamp(18px, 2.4vw, 28px) / 1.5 var(--f-round);
    letter-spacing: 0.08em;
  }
  .title {
    margin: 0;
    line-height: 1;
  }
  .big {
    display: block;
    color: var(--c-accent);
    font: 900 clamp(120px, 19vw, 230px) / 1.02 var(--f-serif);
    letter-spacing: -0.02em;
    margin-left: -0.04em;
  }
  .claim {
    display: flex;
    flex-wrap: wrap;
    gap: 0 0.4em;
    margin: var(--s-4) 0 var(--s-3);
    font: 800 clamp(22px, 2.8vw, 30px) / 1.35 var(--f-serif);
  }
  .claim-han {
    color: var(--c-accent);
  }
  .lede {
    max-width: 46ch;
    margin: 0 0 var(--s-5);
    color: var(--c-soft);
    font-size: 16.5px;
    line-height: 2;
  }
  .cta {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-3);
  }
  .tg svg {
    width: 22px;
    height: 22px;
  }

  .wall {
    display: flex;
    flex-direction: column;
    gap: 0.35em;
    font: 500 clamp(20px, 2.4vw, 30px) / 1 var(--f-serif);
    transform: perspective(900px) rotateY(-24deg) rotateX(4deg) skewY(-3deg);
    transform-origin: left center;
    mask-image: linear-gradient(90deg, transparent, #000 14%, #000 70%, transparent);
  }
  .row {
    display: flex;
    gap: 0.9em;
    width: max-content;
    animation: drift calc(48s + var(--i) * 6s) linear infinite;
    animation-direction: normal;
  }
  .row:nth-child(even) {
    animation-direction: reverse;
  }
  .tile {
    display: inline-block;
    font-size: var(--size);
    color: var(--c-ink);
    white-space: nowrap;
    opacity: 0.86;
  }
  .tile rt {
    font: 500 0.36em/1.3 var(--f-sans);
    color: var(--c-muted);
  }
  .tile:nth-child(5n + 2) {
    color: var(--c-accent);
    opacity: 1;
  }
  @keyframes drift {
    from {
      transform: translateX(0);
    }
    to {
      transform: translateX(-50%);
    }
  }

  /* As the page scrolls past the hero, the title sinks and the wall tilts
     away, where scroll timelines exist and motion is welcome. */
  @supports (animation-timeline: view()) {
    @media (prefers-reduced-motion: no-preference) {
      .big {
        animation: sink linear both;
        animation-timeline: view();
        animation-range: exit 0% exit 100%;
      }
      .wall {
        animation: tilt linear both;
        animation-timeline: view();
        animation-range: exit 0% exit 100%;
      }
    }
  }
  @keyframes sink {
    to {
      transform: translateY(18%) scale(0.94);
      opacity: 0.2;
    }
  }
  @keyframes tilt {
    to {
      transform: perspective(900px) rotateY(-40deg) rotateX(10deg) skewY(-3deg) translateX(-8%);
      opacity: 0;
    }
  }
  /* The hero text arrives in sequence on load. */
  .text > * {
    animation: rise 800ms var(--ease) backwards;
  }
  .text > :nth-child(2) {
    animation-delay: 90ms;
  }
  .text > :nth-child(3) {
    animation-delay: 180ms;
  }
  .text > :nth-child(4) {
    animation-delay: 300ms;
  }
  .text > :nth-child(5) {
    animation-delay: 380ms;
  }
  .text > :nth-child(6) {
    animation-delay: 460ms;
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(24px);
    }
  }

  @media (max-width: 860px) {
    .hero {
      grid-template-columns: 1fr;
      min-height: auto;
      padding-top: var(--s-6);
    }
    .wall {
      order: -1;
      height: 220px;
      overflow: hidden;
      transform: perspective(700px) rotateY(-14deg) skewY(-3deg);
      font-size: 22px;
    }
  }
</style>
