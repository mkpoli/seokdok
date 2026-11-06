import { mount } from 'svelte'
import './fonts.js'
import './app.css'
import DictPage from './DictPage.svelte'

mount(DictPage, { target: document.getElementById('app') })
