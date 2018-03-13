import Vue from 'vue'
import Router from 'vue-router'

import CreateOrg from '@/components/CreateOrg'
import ListOrg from '@/components/ListOrg'

Vue.use(Router)

export default new Router({
  routes: [
    {
      path: '/createOrg',
      name: 'CreateOrg',
      component: CreateOrg
    },
    {
      path: '/listOrg',
      name: 'ListOrg',
      component: ListOrg
    }
  ]
})
