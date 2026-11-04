import init from '../pkg/seokdok.js'
import { mount } from 'svelte'
import './fonts.js'
import './app.css'
import App from './App.svelte'
import { engineReady } from './lib/engine.svelte.js'

// The 두벌식 composer runs on the page; the engine loads in a worker.
await init()
mount(App, { target: document.getElementById('app') })

// See .skip-offscreen in design/base.css. Wait for what sets the sections'
// heights: the web fonts, and the engine that fills the word list. Not for long.
const settled = Promise.all([document.fonts.ready, engineReady]).then(() => new Promise((done) => setTimeout(done, 400)))
Promise.race([settled, new Promise((done) => setTimeout(done, 8000))]).then(() => document.documentElement.classList.add('skip-offscreen'))
