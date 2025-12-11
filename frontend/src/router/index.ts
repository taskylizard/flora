import { createRouter, createWebHistory } from 'vue-router'
import AuthRedirect from '@/views/AuthRedirect.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'auth-redirect',
      component: AuthRedirect,
    },
  ],
})

export default router
