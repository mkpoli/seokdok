// The page's handle on the engine worker.
const worker = new Worker(new URL('./engine.worker.js', import.meta.url), { type: 'module' })
const pending = new Map()
let next = 1

export const status = $state({ ready: false, loaded: 0, total: 1, failed: '' })

worker.onmessage = ({ data }) => {
  if (data.progress) {
    status.loaded = data.progress.loaded
    status.total = Math.max(data.progress.total, data.progress.loaded)
  } else if (data.ready) {
    status.ready = true
  } else if (data.failed) {
    status.failed = data.failed
  } else {
    const p = pending.get(data.id)
    pending.delete(data.id)
    if (!p) return
    data.error ? p.reject(new Error(data.error)) : p.resolve(data.result)
  }
}

function call(op, args) {
  const id = next++
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject })
    worker.postMessage({ id, op, args })
  })
}

export const engine = {
  options: (o) => call('options', o),
  convert: (text) => call('convert', text),
  segments: (text) => call('segments', text),
  read: (text) => call('read', text),
  entries: (limit) => call('entries', limit),
  convertWith: (text, opts, restore) => call('convertWith', [text, opts, restore]),
}
