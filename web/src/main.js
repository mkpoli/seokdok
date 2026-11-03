import init from '../pkg/seokdok.js'
import { mount } from 'svelte'
import './app.css'
import App from './App.svelte'

// The 두벌식 composer runs on the page; the engine loads in a worker.
await init()
mount(App, { target: document.getElementById('app') })
