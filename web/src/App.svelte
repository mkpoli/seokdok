<script>
  import { Composer, Learned } from '../pkg/seokdok.js'
  import { engine } from './lib/engine.svelte.js'
  import { Ime } from './lib/ime.svelte.js'
  import { Picks, STORAGE_KEY } from './lib/learned.js'
  import TopBar from './ui/TopBar.svelte'
  import Hero from './ui/Hero.svelte'
  import ImePanel from './ui/ImePanel.svelte'
  import Reader from './ui/Reader.svelte'
  import Convert from './ui/Convert.svelte'
  import Rules from './ui/Rules.svelte'
  import Words from './ui/Words.svelte'
  import Join from './ui/Join.svelte'
  import Install from './ui/Install.svelte'
  import Eumdok from './ui/Eumdok.svelte'
  import OldHangul from './ui/OldHangul.svelte'
  import Footer from './ui/Footer.svelte'
  import Samples from './ui/Samples.svelte'
  import Custom from './ui/Custom.svelte'
  import { input, output } from './lib/custom.svelte.js'

  // Custom rules wrap the engine: input rules before, output rules after.
  const ime = new Ime(
    new Composer(),
    async (text) => (await engine.segments(input(text))).map((s) => ({ ...s, candidates: output(s.candidates) })),
    async (text) => output(await engine.convert(input(text))),
  )

  let storage = null
  try {
    storage = localStorage
  } catch {}
  const picks = new Picks(Learned, storage)
  addEventListener('pagehide', () => picks.flush())
  addEventListener('storage', (e) => {
    if (e.key === STORAGE_KEY || e.key === null) picks.reload()
  })

  /** Put a word into the input method and convert it (from the word grid). */
  async function tryWord(word) {
    if (ime.segs) ime.commit()
    ime.setCursor([...ime.doc].length)
    if (ime.doc && !/\s$/.test(ime.doc)) ime.setDoc(ime.doc + ' ')
    ime.composer.clear()
    ime.insertText(word)
    document.getElementById('ime')?.scrollIntoView({ behavior: 'smooth', block: 'start' })
    await ime.convert()
    document.querySelector('.sink')?.focus({ preventScroll: true })
  }
</script>

<div class="progress" aria-hidden="true"></div>
<TopBar />
<main>
  <Hero />
  <ImePanel {ime} {picks} />
  <OldHangul />
  <Custom />
  <Samples onopen={tryWord} />
  <Rules />
  <Words onpick={tryWord} />
  <Convert />
  <Reader />
  <Install />
  <Eumdok />
  <Join />
</main>
<Footer />
