<script>
  // Text with the reading of every 釋讀 word above it.
  import { engine, status } from '../lib/engine.svelte.js'

  let { text } = $props()
  const cache = new Map()
  let parts = $state([])

  const WORD = /([\p{Script=Han}]+[가-힣ㄱ-ㆎ]*)/u

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

{#each parts as p}{#if p.word && p.reading && p.reading !== p.text}<ruby>{p.text}<rt>{p.reading}</rt></ruby>{:else}{p.text}{/if}{/each}
