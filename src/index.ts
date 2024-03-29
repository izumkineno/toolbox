import './index.css';
import { createApp } from 'vue'
import App from './App.vue'
import 'element-plus/dist/index.css'
import {createPinia} from "pinia";

const pinia = createPinia()

const app = createApp(App)
app.use(pinia)
app.mount('#root')