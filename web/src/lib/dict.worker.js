// Loads public/dict/browse.bin and answers the /dict page's queries:
// { id, q, file, page, size } → { id, rows, total, page, pages, counts }.
// Progress while loading is posted as { progress: { loaded, total } }, then
// { ready: true, stems, fileRows, size }, or { failed } if it could not load.
import { DictIndex, expandRows, pageOf } from './dict-browse.js'

async function fetchText(url) {
  const res = await fetch(url)
  if (!res.ok) throw new Error(`${url}: ${res.status}`)
  // The size of browse.bin near enough, where the server sends no Content-Length.
  const total = Number(res.headers.get('content-length')) || 3_700_000
  const reader = res.body.getReader()
  const chunks = []
  let loaded = 0
  for (;;) {
    const { done, value } = await reader.read()
    if (done) break
    chunks.push(value)
    loaded += value.length
    postMessage({ progress: { loaded, total } })
  }
  const blob = new Blob(chunks)
  // A server may already have undone the gzip (Content-Encoding); check the magic bytes.
  const head = new Uint8Array(await blob.slice(0, 2).arrayBuffer())
  if (head[0] !== 0x1f || head[1] !== 0x8b) return await blob.text()
  return await new Response(blob.stream().pipeThrough(new DecompressionStream('gzip'))).text()
}

let index = null
let queued = null
// Paging and filtering reuse the last search.
let last = { q: null, hits: null }

function answer({ id, q, file, page, size }) {
  if (last.q !== q) last = { q, hits: index.search(q) }
  postMessage({ id, ...pageOf(index, last.hits, { file, page, size }) })
}

// Only the newest query matters, so one waiting slot is enough.
onmessage = ({ data }) => {
  if (index) answer(data)
  else queued = data
}

fetchText('/dict/browse.bin')
  .then((text) => {
    index = new DictIndex(expandRows(text))
    postMessage({ ready: true, stems: index.stems, fileRows: index.fileRows, size: index.size })
    if (queued) answer(queued)
  })
  .catch((e) => postMessage({ failed: String(e) }))
