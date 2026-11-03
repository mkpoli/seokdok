<script>
  import { Composer } from '../pkg/seokdok.js'
  import { engine } from './lib/engine.svelte.js'
  import { Ime } from './lib/ime.svelte.js'
  import TopBar from './ui/TopBar.svelte'
  import Hero from './ui/Hero.svelte'
  import ImePanel from './ui/ImePanel.svelte'
  import Reader from './ui/Reader.svelte'
  import Rules from './ui/Rules.svelte'
  import Words from './ui/Words.svelte'
  import Join from './ui/Join.svelte'
  import Footer from './ui/Footer.svelte'
  import Samples from './ui/Samples.svelte'

  const ime = new Ime(
    new Composer(),
    (text) => engine.segments(text),
    (text) => engine.convert(text),
  )

  /** Put a word into the input method and convert it (from the word grid). */
  async function tryWord(word) {
    if (ime.segs) ime.commit()
    if (ime.doc && !ime.doc.endsWith(' ') && !ime.doc.endsWith('\n')) ime.doc += ' '
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
  <ImePanel {ime} />
  <Samples onopen={tryWord} />
  <Rules />
  <Words onpick={tryWord} />
  <Reader />
  <Join />
</main>
<Footer />
