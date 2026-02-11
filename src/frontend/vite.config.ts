import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { TanStackRouterVite } from '@tanstack/router-plugin/vite'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    TanStackRouterVite(),
    react()
  ],
  envDir: '../../',
  server: {
    host: true,
    watch: {
        usePolling: true
    }
  },
  preview: {
    host: true,
    port: 4173
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks: (id) => {
          if (id.includes('node_modules')) {
            if (id.includes('@mysten')) return 'mysten';
            if (id.includes('@tanstack')) return 'tanstack';
            // Keep react in vendor or split carefully. Let's fallback to vendor for others to avoid cycles.
            return 'vendor';
          }
        }
      }
    }
  }
})
