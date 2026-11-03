// `use:reveal` marks an element to rise into view when it first scrolls in.
// Without IntersectionObserver, or with reduced motion, it is shown at once.
let observer

function shared() {
  if (observer || typeof IntersectionObserver === 'undefined') return observer
  observer = new IntersectionObserver(
    (entries) => {
      for (const e of entries) {
        if (!e.isIntersecting) continue
        e.target.setAttribute('data-shown', '')
        e.target.dispatchEvent(new CustomEvent('revealed'))
        observer.unobserve(e.target)
      }
    },
    { rootMargin: '0px 0px -12% 0px', threshold: 0.08 },
  )
  return observer
}

/** @param {HTMLElement} node @param {number} [i] stagger index */
export function reveal(node, i = 0) {
  node.setAttribute('data-reveal', '')
  node.style.setProperty('--i', String(i))
  const o = shared()
  if (!o || matchMedia('(prefers-reduced-motion: reduce)').matches) {
    node.setAttribute('data-shown', '')
    queueMicrotask(() => node.dispatchEvent(new CustomEvent('revealed')))
    return {}
  }
  o.observe(node)
  return { destroy: () => o.unobserve(node) }
}
