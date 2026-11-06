import init from '../pkg/seokdok.js'
import { mount } from 'svelte'
import './fonts.js'
import './app.css'
import EumdokPage from './EumdokPage.svelte'

// The 두벌식 composer runs on the page; the engine loads in a worker.
await init()
mount(EumdokPage, { target: document.getElementById('app') })
