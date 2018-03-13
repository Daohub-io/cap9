import Vue from 'vue'
import App from './App'
import router from './router'

import Contracts from './contracts'
import BootstrapVue from 'bootstrap-vue'

Vue.use(BootstrapVue)
Vue.use(Contracts)

Vue.config.productionTip = false

/* eslint-disable no-new */
new Vue({
  el: '#app',
  router,
  render: h => h(App)
})
