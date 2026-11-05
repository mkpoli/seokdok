import { mount } from 'svelte'
import './fonts.js'
import './app.css'
import EumdokPage from './EumdokPage.svelte'

mount(EumdokPage, { target: document.getElementById('app') })
