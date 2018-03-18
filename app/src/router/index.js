import Vue from 'vue'
import Router from 'vue-router'

import CreateOrg from '@/components/CreateOrg'
import ListOrg from '@/components/ListOrg'
import ViewOrg from '@/components/ViewOrg'

import OrgStorage from '@/components/org/Storage.vue'
import OrgProcedures from '@/components/org/Procedures.vue'


Vue.use(Router)

export default new Router({
  routes: [
    {
      path: '/org/create',
      component: CreateOrg
    },
    {
      path: '/org/list',
      component: ListOrg
    },
    {
      path: '/org/:id',
      component: ViewOrg,
      children : [
        {
          name: 'k_storage',
          path: 'storage',
          component: OrgStorage
        },
        {
          name: 'k_procedures',
          path: 'procedures',
          component: OrgProcedures
        }
      ]
    }
  ]
})
