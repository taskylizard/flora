import { createRouter, createWebHistory } from 'vue-router'
import AuthRedirect from '@/views/AuthRedirect.vue'
import Dashboard from '@/views/Dashboard.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'auth-redirect',
      component: AuthRedirect,
    },
    {
      path: '/dashboard',
      name: 'dashboard',
      component: Dashboard,
    },
  ],
})

export default router
