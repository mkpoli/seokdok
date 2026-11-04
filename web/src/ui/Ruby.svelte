<script>
  // Text with the reading of every 釋讀 word above it.
  import { engine, status } from '../lib/engine.svelte.js'
  import { rubify, WORD } from '../lib/rubify.js'

  let { text } = $props()
  const cache = new Map()
  let parts = $state([])

  $effect(() => {
    const src = text
    const ready = status.ready
    const pieces = src.split(WORD)
    parts = pieces.map((p, i) => ({ text: p, word: i % 2 === 1, reading: cache.get(p) }))
    if (!ready) return
    const todo = [...new Set(pieces.filter((p, i) => i % 2 === 1 && !cache.has(p)))]
    if (!todo.length) return
    Promise.all(todo.map(async (w) => cache.set(w, await engine.read(w)))).then(() => {
      if (text === src) parts = pieces.map((p, i) => ({ text: p, word: i % 2 === 1, reading: cache.get(p) }))
    })
  })
</script>

{#each parts as p}{@const rb = p.word ? rubify(p.text, p.reading) : null}{#if rb}<ruby>{rb.base}<rt>{rb.rt}</rt></ruby>{rb.tail}{:else}{p.text}{/if}{/each}
