import { mount } from 'svelte'
import './app.css'
import DocPage from './DocPage.svelte'

mount(DocPage, { target: document.getElementById('app'), props: { page: 'support' } })
