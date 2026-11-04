// Types into the input panel the way each kind of keyboard does, in real
// browsers, and reads the result off the page. Run by hand against `vite preview`:
//
//   bun add --no-save playwright && bunx playwright install chromium firefox webkit
//   bunx vite preview --port 4173 --strictPort
//   bun scripts/check-keyboards.mjs http://localhost:4173/ [chromium-mobile chromium firefox webkit]
//
// Chromium drives its real text input through CDP (Input.insertText,
// Input.imeSetComposition). Firefox and WebKit have no such API here, so there a
// composition is dispatched as real DOM events in the order that engine uses,
// and plain text goes through Playwright's insertText. Deletes are the browser's
// own delete command, which fires input as a keyboard's delete does.
import { chromium, firefox, webkit, devices } from 'playwright'

const url = process.argv[2] ?? 'http://localhost:4173/'
const names = process.argv.slice(3)

const BROWSERS = {
  'chromium-mobile': { type: chromium, context: { ...devices['Pixel 7'] }, cdp: true, mobile: true },
  chromium: { type: chromium, context: {}, cdp: true },
  firefox: { type: firefox, context: {}, order: 'firefox' },
  webkit: { type: webkit, context: {}, order: 'webkit' },
}

const HAN = '\u1112\u119E\u11AB' // ᄒᆞᆫ

/** Runs in the page: a composition as this engine dispatches it. */
function domComposition(el, [steps, order]) {
  const region = el.value.length
  const fire = (Type, name, init) => el.dispatchEvent(new Type(name, { bubbles: true, ...init }))
  fire(CompositionEvent, 'compositionstart', { data: '' })
  for (const data of steps) {
    fire(CompositionEvent, 'compositionupdate', { data })
    fire(InputEvent, 'beforeinput', { inputType: 'insertCompositionText', data, isComposing: true })
    el.value = el.value.slice(0, region) + data
    fire(InputEvent, 'input', { inputType: 'insertCompositionText', data, isComposing: true })
  }
  const data = steps.at(-1)
  if (order === 'webkit') {
    el.value = el.value.slice(0, region)
    fire(InputEvent, 'input', { inputType: 'deleteCompositionText', data: null, isComposing: true })
    el.value += data
    fire(InputEvent, 'input', { inputType: 'insertFromComposition', data, isComposing: true })
  }
  fire(CompositionEvent, 'compositionend', { data })
  if (order === 'firefox') fire(InputEvent, 'input', { inputType: 'insertCompositionText', data })
}

function driver(page, spec, cdp) {
  const keyless = async () => {
    // A phone keyboard: keydown says nothing, then the text arrives.
    if (cdp) await cdp.send('Input.dispatchKeyEvent', { type: 'rawKeyDown', key: 'Unidentified', code: '', windowsVirtualKeyCode: 229 })
    if (cdp) await cdp.send('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Unidentified', code: '', windowsVirtualKeyCode: 229 })
  }
  const sink = () => page.locator('#ime .sink')
  return {
    keys: async (s) => {
      for (const k of s) await page.keyboard.press(k === ' ' ? 'Space' : k)
    },
    /** Text with no usable keydown, one character per event. */
    send: async (s) => {
      for (const c of s) {
        await keyless()
        await page.keyboard.insertText(c)
      }
    },
    /** Text sent whole in one event. */
    whole: async (s) => {
      await keyless()
      await page.keyboard.insertText(s)
    },
    del: async (n = 1) => {
      for (let i = 0; i < n; i++) await page.evaluate(() => document.execCommand('delete'))
    },
    /** One composition through to its commit. */
    compose: async (steps) => {
      if (!cdp) return sink().evaluate(domComposition, [steps, spec.order])
      for (const text of steps) {
        await keyless()
        await cdp.send('Input.imeSetComposition', { text, selectionStart: text.length, selectionEnd: text.length })
      }
      await cdp.send('Input.insertText', { text: steps.at(-1) })
    },
    paste: (text) =>
      sink().evaluate((el, text) => {
        const clipboardData = new DataTransfer()
        clipboardData.setData('text', text)
        el.dispatchEvent(new ClipboardEvent('paste', { bubbles: true, cancelable: true, clipboardData }))
      }, text),
  }
}

const read = (page) =>
  page.evaluate(() => {
    const q = (s) => [...document.querySelectorAll('#ime ' + s)].map((e) => e.textContent)
    return { pre: q('.pre').join(''), doc: q('.doc').join(''), segs: q('.segment'), win: q('.win').length > 0, value: q('.sink').length ? document.querySelector('#ime .sink').value : '' }
  })

/** Each check: type, then what the page must show. */
const CHECKS = {
  'Latin, real keys: dkssud, Space, Space, Enter': async (d, see) => {
    await d.keys('dkssud')
    await see({ pre: '안녕' })
    await d.keys(' ')
    await see({ segs: ['安寧'] })
    await d.keys(' ')
    await see({ win: true })
    await d.keys('1')
    await see({ doc: '安寧' })
  },
  'Latin, input only: letters one at a time, Space from input': async (d, see) => {
    await d.send('dkssud')
    await see({ pre: '안녕', value: '\u200B' })
    await d.send(' ')
    await see({ segs: ['安寧'] })
    await d.send(' ')
    await see({ win: true })
    await d.send('\n')
    await see({ doc: '안녕', segs: [] })
  },
  'Latin word composition (Gboard): d, dk, dks … then Space': async (d, see) => {
    await d.compose(['d', 'dk', 'dks', 'dkss', 'dkssu', 'dkssud'])
    await see({ pre: '안녕' })
    await d.send(' ')
    await see({ segs: ['安寧'] })
  },
  'Latin swipe: a word at once passes through': async (d, see) => {
    await d.compose(['hello'])
    await see({ doc: 'hello', pre: '' })
  },
  'iOS jamo one at a time: ㅇㅏㄴㄴㅕㅇ, Space, Enter': async (d, see) => {
    await d.send('ㅇㅏㄴㄴㅕㅇ')
    await see({ pre: '안녕' })
    await d.send(' ')
    await see({ segs: ['安寧'] })
    await d.send('\n')
    await see({ doc: '安寧', segs: [] })
  },
  'iOS delete-then-reinsert': async (d, see) => {
    await d.send('ㅇ')
    await d.del()
    await d.send('아')
    await d.del()
    await d.send('안')
    await d.send('ㄴ')
    await d.del()
    await d.send('녀')
    await d.del()
    await d.send('녕')
    await see({ pre: '안녕', value: '\u200B안녕' })
  },
  'iOS backspace: letters, then the document': async (d, see) => {
    await d.send('rk')
    await d.send('\n')
    await see({ doc: '가' })
    await d.send('ㅎㅏㄴ')
    await d.del()
    await see({ pre: '하' })
    await d.del(2)
    await see({ pre: '' })
    await d.del()
    await see({ doc: '', value: '\u200B' })
  },
  'Own composition by syllable (Windows, macOS IME)': async (d, see) => {
    await d.compose(['ㅇ', '아', '안'])
    await see({ pre: '안' })
    await d.compose(['ㄴ', '녀', '녕'])
    await see({ pre: '안녕' })
    await d.send(' ')
    await see({ segs: ['安寧'] })
  },
  'Own composition moving the final: 안 + ㅕ': async (d, see) => {
    await d.compose(['ㅇ', '아', '안', '아'])
    await d.compose(['녀'])
    await see({ pre: '아녀' })
  },
  'Own composition in one region, syllables replaced (Gboard, Samsung)': async (d, see) => {
    await d.compose(['ㅇ', '아', '안', '안ㄴ', '안녀', '안녕'])
    await see({ pre: '안녕' })
    await d.send(' ')
    await see({ segs: ['安寧'] })
  },
  'Old letters one at a time: ㅎㆍㄴ ㅿㅏ': async (d, see) => {
    await d.send('ㅎㆍㄴ')
    await see({ pre: HAN })
    await d.send('\n')
    await see({ doc: HAN })
    await d.send('ㅿㅏ')
    await see({ pre: '\u1140\u1161' })
  },
  'Conjoining jamo one at a time (첫가끝)': async (d, see) => {
    await d.send(HAN)
    await see({ pre: HAN, one: true })
    await d.send('\u1112\u1161\u11AB')
    await see({ pre: HAN + '한' })
  },
  'Conjoining jamo pasted, Jamo Extended included': async (d, see) => {
    await d.paste('\uA960\u1161\u11A8')
    await see({ pre: '\uA960\u1161\u11A8', one: true })
    await d.send(' ')
    await see({ segs: ['\uA960\u1161\u11A8'] })
  },
  'Dictation: finished words wait to convert': async (d, see) => {
    await d.whole('하늘을 봤다')
    await see({ pre: '하늘을 봤다', doc: '' })
    await d.send(' ')
    await see({ doc: '', pre: '' })
  },
  'Emoji and other text pass through': async (d, see) => {
    await d.whole('😀')
    await see({ doc: '😀' })
    await d.whole('hello')
    await see({ doc: '😀hello', pre: '' })
  },
  'A mode: Latin stays, Korean still composes': async (d, see, page) => {
    await page.locator('#ime .toolbar .seg button').nth(1).click()
    await page.locator('#ime .sink').focus()
    await d.send('hi')
    await see({ doc: 'hi' })
    await d.compose(['ㅎ', '하', '한'])
    await see({ pre: '한', doc: 'hi' })
    await d.send(' ')
    await see({ segs: ['韓'], doc: 'hi' })
  },
}

async function run(name) {
  const spec = BROWSERS[name]
  const browser = await spec.type.launch()
  const context = await browser.newContext({ ...spec.context, locale: 'ko-KR' })
  const page = await context.newPage()
  const cdp = spec.cdp ? await context.newCDPSession(page) : null
  const results = []
  await page.goto(url)
  await page.locator('#ime .sink').scrollIntoViewIfNeeded()
  await page.waitForFunction(() => !document.querySelector('#ime .loading'), null, { timeout: 60000 })

  const attrs = await page.locator('#ime .sink').evaluate((el) => Object.fromEntries(['inputmode', 'enterkeyhint', 'autocapitalize', 'autocomplete', 'autocorrect', 'spellcheck'].map((a) => [a, el.getAttribute(a)])))

  for (const [title, check] of Object.entries(CHECKS)) {
    if (title.startsWith('Latin, real keys') && spec.mobile) continue // A phone has no such keys.
    await page.reload()
    await page.waitForFunction(() => !document.querySelector('#ime .loading'), null, { timeout: 60000 })
    await page.locator('#ime .sink').focus()
    const d = driver(page, spec, cdp)
    const see = async (want) => {
      const ok = (s) => Object.entries(want).every(([k, v]) => (k === 'one' ? [...new Intl.Segmenter('ko', { granularity: 'grapheme' }).segment(s.pre)].length === 1 : JSON.stringify(s[k]) === JSON.stringify(v)))
      let s
      for (let i = 0; i < 100; i++) {
        s = await read(page)
        if (ok(s)) return
        await page.waitForTimeout(20)
      }
      throw new Error(`want ${JSON.stringify(want)}, page shows ${JSON.stringify(s)}`)
    }
    try {
      await check(d, see, page)
      results.push([title, 'ok'])
    } catch (e) {
      results.push([title, 'FAIL ' + e.message.split('\n')[0]])
    }
  }

  // The engine's own order, for the record.
  await page.reload()
  await page.locator('#ime .sink').focus()
  await page.evaluate(() => {
    const el = document.querySelector('#ime .sink')
    window.trace = []
    for (const t of ['keydown', 'beforeinput', 'compositionstart', 'compositionupdate', 'compositionend', 'input']) {
      el.addEventListener(t, (e) => window.trace.push([t, e.inputType ?? e.key ?? '', e.data ?? '', e.isComposing ? 'composing' : ''].filter(Boolean).join(' ')), true)
    }
  })
  const d = driver(page, spec, cdp)
  await d.compose(['ㅇ', '아'])
  await d.send('ㄴ')
  await d.del()
  const trace = await page.evaluate(() => window.trace)
  await browser.close()
  return { name, version: browser.version(), results, attrs, trace }
}

let failed = 0
for (const name of names.length ? names : Object.keys(BROWSERS)) {
  const { version, results, attrs, trace } = await run(name)
  console.log(`\n${name} (${version})`)
  console.log('  textarea', JSON.stringify(attrs))
  for (const [title, result] of results) {
    if (result !== 'ok') failed++
    console.log(`  ${result === 'ok' ? 'ok  ' : 'FAIL'} ${title}${result === 'ok' ? '' : '\n       ' + result}`)
  }
  console.log('  events:', trace.join(' | '))
}
process.exit(failed ? 1 : 0)
