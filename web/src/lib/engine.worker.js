// Loads the WebAssembly engine and the dictionaries, then answers requests:
// { id, op, args } → { id, result } | { id, error }. Progress while loading is
// posted as { progress: { loaded, total } }, and { ready: true } once built.
import init, { Engine } from '../../pkg/seokdok.js'

let engine = null
const queue = []

async function fetchText(url, onBytes) {
  const res = await fetch(url)
  if (!res.ok) throw new Error(`${url}: ${res.status}`)
  const reader = res.body.getReader()
  const chunks = []
  for (;;) {
    const { done, value } = await reader.read()
    if (done) break
    chunks.push(value)
    onBytes(value.length)
  }
  const blob = new Blob(chunks)
  // A server may already have undone the gzip (Content-Encoding); check the magic bytes.
  const head = new Uint8Array(await blob.slice(0, 2).arrayBuffer())
  if (head[0] !== 0x1f || head[1] !== 0x8b) return await blob.text()
  const stream = blob.stream().pipeThrough(new DecompressionStream('gzip'))
  return await new Response(stream).text()
}

async function load() {
  // Sizes for the progress bar; refined from Content-Length when present.
  let loaded = 0
  const total = 3_100_000
  const tick = (n) => {
    loaded += n
    postMessage({ progress: { loaded, total } })
  }
  const [, native, sino] = await Promise.all([
    init(),
    fetchText('/dict/native.bin', tick),
    fetchText('/dict/sino.bin', tick),
  ])
  engine = new Engine([['native.tsv', native]], [['sino.tsv', sino]])
  postMessage({ ready: true })
  for (const m of queue.splice(0)) handle(m)
}

const ops = {
  options: (o) => engine.setOptions(JSON.stringify(o)),
  convert: (text) => JSON.parse(engine.convert(text)),
  segments: (text) => JSON.parse(engine.segments(text)),
  read: (text) => engine.read(text),
  entries: (limit) => JSON.parse(engine.entries(limit)),
  // Convert under other options without changing the current ones.
  convertWith: ([text, opts, restore]) => {
    engine.setOptions(JSON.stringify(opts))
    const r = JSON.parse(engine.convert(text))
    engine.setOptions(JSON.stringify(restore))
    return r
  },
}

function handle({ data }) {
  const { id, op, args } = data
  try {
    postMessage({ id, result: ops[op](args) })
  } catch (e) {
    postMessage({ id, error: String(e) })
  }
}

onmessage = (m) => (engine ? handle(m) : queue.push(m))
load().catch((e) => postMessage({ failed: String(e) }))
