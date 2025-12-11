import { fileURLToPath, URL } from 'node:url'

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import vueJsx from '@vitejs/plugin-vue-jsx'
import vueDevTools from 'vite-plugin-vue-devtools'
import tailwindcss from '@tailwindcss/vite'

// https://vite.dev/config/
export default defineConfig(({ mode }) => {
  const API_HOST = process.env.API_HOST ?? "http://localhost:3000";
  console.log(`Running in ${mode} with API_HOST: ${API_HOST}`);
  return {
    plugins: [
      vue(),
      tailwindcss(),
      vueJsx(),
      vueDevTools(),
    ],
    resolve: {
      alias: {
        '@': fileURLToPath(new URL('./src', import.meta.url))
      },
    },

    ...(mode === "development" && {
      server: {
        proxy: {
          "/api": {
            target: API_HOST,
            changeOrigin: true,
          },
        },
      },
    }),
  }

})
