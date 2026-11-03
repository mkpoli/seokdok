<script>
  import { t, r, p } from '../lib/i18n.svelte.js'

  // The wall of 訓讀 verbs, after the third 釋讀會 poster: the written form with
  // its stem above. All are dictionary entries attested in the group's chat.
  const WALL = [
    ['爲다', '하'], ['在다', '있'], ['成다', '되'], ['無다', '없'], ['不ㅎ다', '않'], ['非다', '아니'],
    ['見다', '보'], ['同다', '같'], ['授다', '주'], ['去다', '가'], ['言하다', '말하'], ['來다', '오'],
    ['知다', '알'], ['然ㅎ다', '그렇'], ['大다', '크'], ['多다', '많'], ['好다', '좋'], ['出다', '나'],
    ['思하다', '생각하'], ['不知다', '모르'], ['作다', '만들'], ['食다', '먹'], ['何떻다', '어떻'], ['書다', '쓰'],
    ['讀다', '읽'], ['異다', '다르'], ['新롭다', '새롭'], ['歡갑다', '반갑'], ['樂겁다', '즐겁'], ['難다', '어렵'],
    ['易다', '쉽'], ['適다', '맞'], ['殘다', '남'], ['用이다', '쓰이'], ['似하다', '비슷하'], ['入다', '들'],
  ]
  const rows = Array.from({ length: 6 }, (_, r) => WALL.slice(r * 6, r * 6 + 6))
</script>

<section class="hero-band" id="top">
<div class="hero">
  <div class="text">
    <p class="slogan" aria-label="音讀에 疲ㄴ 當身을 爲한">
      <ruby>音讀<rt>음독</rt></ruby>에<ruby>疲<rt>지치</rt></ruby>ㄴ
      <ruby>當身<rt>당신</rt></ruby>을<ruby>爲<rt>위</rt></ruby>한、
    </p>
    <p class="club">풀어읽기모임</p>
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
        {#each [...row, ...row] as [w, reading], c}
          <span class="tile" style:--size="{1 + ((r * 7 + c * 3) % 5) * 0.12}em">
            <small>{reading}</small>{w}
          </span>
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
    font: 700 clamp(28px, 4.4vw, 48px) / 1.1 var(--f-round);
    letter-spacing: 0.32em;
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
    display: inline-flex;
    flex-direction: column;
    align-items: flex-start;
    font-size: var(--size);
    color: var(--c-ink);
    white-space: nowrap;
    opacity: 0.86;
  }
  .tile small {
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
    animation: rise 800ms var(--ease) both;
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
