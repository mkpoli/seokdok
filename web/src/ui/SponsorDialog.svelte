<script>
  // The author's portrait, what a tip pays for, then three steps that end at the tip buttons: Ko-fi
  // and OFUSE. Each button carries its service's own mark and colour, unaltered. The contents and the roll's script load when the dialog first opens, so a
  // page view asks the supporters service for nothing.
  import { t, plain } from '../lib/i18n.svelte.js'
  /** @type {{ open: boolean }} */
  let { open = $bindable(false) } = $props()
  /** @type {HTMLDialogElement | null} */
  let dialog = $state(null)
  let rollLoaded = false
  let pressedOutside = false

  $effect(() => {
    if (!dialog) return
    if (open && !dialog.open) dialog.showModal()
    if (!open && dialog.open) dialog.close()
    if (open && !rollLoaded) {
      rollLoaded = true
      const script = document.createElement('script')
      script.type = 'module'
      script.src = 'https://supporters.mkpo.li/roll.js'
      document.head.append(script)
    }
  })

  /** Whether an event lands on the backdrop. A drag that leaves the box, or the box's own scrollbar, also targets the dialog, so the test is by position. @param {MouseEvent} event */
  function outside(event) {
    const box = /** @type {HTMLDialogElement} */ (dialog).getBoundingClientRect()
    return event.clientX < box.left || event.clientX > box.right || event.clientY < box.top || event.clientY > box.bottom
  }
</script>

{#snippet arrow()}
  <svg class="arrow" viewBox="0 0 20 20" width="18" height="18" aria-hidden="true"><path d="M4 10h12M11 5l5 5-5 5" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" /></svg>
{/snippet}

{#snippet kofi()}
  <a class="tip kofi" href="https://ko-fi.com/mkpoli" target="_blank" rel="noreferrer">
    <img class="mark" src="/brand/kofi-symbol.svg" alt="" width="34" height="27" />
    <span class="copy"><strong>Ko-fi</strong><small>{t().sponsor.kofi}</small></span>
    {@render arrow()}
  </a>
{/snippet}

{#snippet ofuse()}
  <a class="tip ofuse" href="https://ofuse.me/mkpoli" target="_blank" rel="noreferrer">
    <img class="mark" src="/brand/ofuse-icon.svg" alt="" width="26" height="28" />
    <span class="copy"><strong>OFUSE</strong><small>{t().sponsor.ofuse}</small></span>
    {@render arrow()}
  </a>
{/snippet}

<dialog class="sponsor-dialog" bind:this={dialog} aria-labelledby="sponsor-title"
  onclose={() => { open = false }}
  onpointerdown={(/** @type {PointerEvent} */ event) => { pressedOutside = outside(event) }}
  onclick={(/** @type {MouseEvent} */ event) => { if (pressedOutside && event.target === dialog && outside(event)) open = false; pressedOutside = false }}>
  {#if open}
    <header>
      <h2 id="sponsor-title">{t().sponsor.label}</h2>
      <button class="close" type="button" aria-label={t().sponsor.close} title={t().sponsor.close} onclick={() => { open = false }}>
        <svg viewBox="0 0 20 20" width="16" height="16" aria-hidden="true"><path d="M5 5l10 10M15 5L5 15" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" /></svg>
      </button>
    </header>
    <div class="sheet">
      <div class="hero" aria-hidden="true">
        <span class="portrait">
          <img src="/sponsor-portrait.webp" alt="" width="112" height="112" decoding="async" />
          <svg class="heart" viewBox="0 0 24 24">
            <path d="M12 21s-8.5-5.1-8.5-11.3A4.7 4.7 0 0 1 12 6.9a4.7 4.7 0 0 1 8.5 2.8C20.5 15.9 12 21 12 21Z" />
            <ellipse class="shine" cx="7.6" cy="9.6" rx="1.5" ry="2.1" transform="rotate(-30 7.6 9.6)" />
          </svg>
        </span>
        <span class="spark s1">✦</span><span class="spark s2">✦</span><span class="spark s3">✦</span>
      </div>
      <p class="lead">{plain(t().sponsor.lead)}</p>
      <p class="ask">{plain(t().sponsor.ask)}</p>
      <section class="how">
        <h3>{t().sponsor.howTo}</h3>
        <ol class="steps" role="list">
          {#each t().sponsor.steps as step}
            <li>{step}</li>
          {/each}
        </ol>
      </section>
      <div class="tips">
        {@render kofi()}
        {@render ofuse()}
      </div>
      <a class="company" href="mailto:mkpoli@mkpo.li?subject=Sponsorship%20on%20seokdok.mkpo.li">
        <svg viewBox="0 0 20 20" width="16" height="16" aria-hidden="true"><path d="M3 5h14v10H3zM3 6l7 5 7-5" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round" /></svg>
        <span class="copy"><strong>{t().sponsor.company}</strong><small>{t().sponsor.companyHint}</small></span>
      </a>
      <supporters-roll label={t().sponsor.supporters} anonymous={t().sponsor.anonymous}></supporters-roll>
      <p class="thanks">{plain(t().sponsor.thanks)}</p>
    </div>
  {/if}
</dialog>

<style>
  .sponsor-dialog{inline-size:min(480px,calc(100vw - 32px));max-block-size:min(86dvh,760px);padding:0;overflow:auto;overscroll-behavior:contain;border:1px solid var(--c-border);border-radius:var(--r-3);background:var(--c-surface);color:var(--c-ink);box-shadow:var(--shadow-2)}
  .sponsor-dialog::backdrop{background:rgb(10 15 32 / .55)}
  header{position:sticky;top:0;z-index:1;display:flex;align-items:center;justify-content:space-between;gap:var(--s-4);padding:14px 16px 10px 20px;background:var(--c-surface);border-bottom:1px solid var(--c-border)}
  h2{margin:0;font:700 17px/1.3 var(--f-sans)}
  .close{display:grid;place-items:center;inline-size:32px;block-size:32px;border:0;border-radius:999px;background:none;color:var(--c-soft);cursor:pointer}
  .close:hover{background:var(--c-hover);color:var(--c-ink)}
  .close:focus-visible{outline:2px solid var(--c-accent);outline-offset:2px}
  .sheet{--sp-ink:light-dark(#c2185b,#ffb3cd);--sp-heart:light-dark(#ff4f88,#ff7aa6);--sp-soft:light-dark(#fff1f5,#2f1822);
    --sp-line:light-dark(#ffd0de,#5a2a3f);display:flex;flex-direction:column;gap:var(--s-4);padding:16px 20px 20px;font-family:var(--f-sans)}
  .hero{position:relative;display:grid;place-items:center;block-size:136px;border-radius:var(--r-3);
    background:radial-gradient(closest-side,light-dark(#ffd9e5,#4a1e31),transparent)}
  .portrait{position:relative;inline-size:112px;block-size:112px}
  .portrait img{display:block;inline-size:100%;block-size:100%;border-radius:50%;object-fit:cover;
    box-shadow:0 0 0 4px var(--c-surface),0 0 0 6px var(--sp-line),0 10px 24px rgb(255 79 136 / .3)}
  .heart{position:absolute;right:-2px;bottom:2px;inline-size:34px;block-size:34px;fill:var(--sp-heart);stroke:var(--c-surface);stroke-width:2;paint-order:stroke;
    overflow:visible;transform-origin:50% 60%;animation:beat 2.8s ease-in-out infinite;filter:drop-shadow(0 3px 6px rgb(255 79 136 / .35))}
  .heart .shine{fill:#fff;opacity:.6}
  .spark{position:absolute;color:var(--sp-heart);opacity:.75;animation:twinkle 2.4s ease-in-out infinite}
  .s1{top:22px;left:calc(50% - 84px);font-size:14px}
  .s2{top:12px;left:calc(50% + 66px);font-size:10px;animation-delay:.8s}
  .s3{bottom:20px;left:calc(50% - 76px);font-size:11px;animation-delay:1.6s}
  @keyframes beat{0%,70%,100%{transform:scale(1)}78%{transform:scale(1.18)}86%{transform:scale(.96)}93%{transform:scale(1.1)}}
  @keyframes twinkle{0%,100%{opacity:.2;transform:scale(.7)}50%{opacity:.9;transform:scale(1)}}
  .lead{margin:0;font-size:15px;line-height:1.65;word-break:keep-all}
  .ask{margin:0;font-size:15px;font-weight:700;line-height:1.55;color:var(--sp-ink);word-break:keep-all}
  .how{padding:var(--s-3) var(--s-4);border:1px dashed var(--sp-line);border-radius:var(--r-3);background:var(--sp-soft)}
  h3{margin:0 0 var(--s-2);font-size:12px;font-weight:700;color:var(--sp-ink)}
  .steps{display:flex;flex-direction:column;gap:var(--s-1);margin:0;padding:0;list-style:none;counter-reset:step;word-break:keep-all}
  .steps li{display:flex;align-items:baseline;gap:var(--s-2);font-size:14px;line-height:1.45;counter-increment:step}
  .steps li::before{content:counter(step);display:grid;place-items:center;flex:none;inline-size:20px;block-size:20px;border-radius:50%;
    background:var(--sp-ink);color:var(--sp-soft);font-size:11px;font-weight:700;line-height:1;transform:translateY(-1px)}
  .tips{display:flex;flex-direction:column;gap:var(--s-2)}
  .tip{display:flex;align-items:center;gap:var(--s-3);padding:var(--s-3) var(--s-4) var(--s-3) var(--s-3);border-radius:999px;
    color:#fff;text-decoration:none;box-shadow:0 6px 16px var(--tip-glow);transition:transform var(--t-fast) var(--ease),box-shadow var(--t-fast) var(--ease)}
  .kofi{--tip-glow:rgb(32 32 32 / .28);--tip-glow-hover:rgb(32 32 32 / .38);background:#202020;border:1px solid light-dark(#202020,rgb(255 255 255 / .2))}
  .ofuse{--tip-glow:rgb(38 125 160 / .3);--tip-glow-hover:rgb(38 125 160 / .42);background:#267da0;border:1px solid #267da0}
  .tip:hover{transform:translateY(-1px);box-shadow:0 8px 22px var(--tip-glow-hover)}
  .tip:active{transform:scale(.98)}
  .tip:focus-visible,.company:focus-visible{outline:2px solid var(--c-accent);outline-offset:2px}
  .tip .mark{display:block;flex:none;inline-size:38px;block-size:30px;object-fit:contain}
  .tip .arrow{flex:none}
  .tip .copy strong{font-size:17px}
  .tip .copy small{color:#fff}
  .copy{display:flex;flex:1;flex-direction:column;gap:1px;min-inline-size:0}
  .copy strong{font-size:15px;font-weight:700}
  .copy small{font-size:12px;line-height:1.4}
  .company{display:flex;align-items:flex-start;gap:var(--s-2);padding:var(--s-2) var(--s-3);border-radius:var(--r-2);color:var(--c-muted);text-decoration:none}
  .company:hover{background:var(--c-hover);color:var(--c-ink)}
  .company svg{flex:none;margin-top:2px}
  .company small{word-break:keep-all}
  supporters-roll{--sp-text:var(--c-ink);--sp-muted:var(--c-muted)}
  .company .copy strong{font-size:14px;color:var(--c-ink)}
  .thanks{margin:0;font-size:14px;line-height:1.55;color:var(--c-muted);text-align:center;word-break:keep-all}
  @media (max-width:640px){
    .sponsor-dialog{inset:auto 0 0;margin:0;inline-size:100%;max-inline-size:none;max-block-size:85dvh;border-radius:var(--r-3) var(--r-3) 0 0;padding-bottom:env(safe-area-inset-bottom)}
  }
  @media (prefers-reduced-motion:reduce){.heart,.spark,.tip{animation:none;transition:none}}
</style>
