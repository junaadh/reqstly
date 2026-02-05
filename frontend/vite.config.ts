import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

// https://vite.dev/config/
export default defineConfig({
  base: '/',
  server: {
    host: '0.0.0.0',
    hmr: {
      host: 'localhost',
      port: 5173
    },
    watch: {
      usePolling: true,
      interval: 500,
      ignored: [
        '**/node_modules/**',
        '**/.git/**',
        '**/dist/**',
        '**/.turbo/**',
        '**/.next/**', 
      ],
    },
  },
  optimizeDeps: {
    include: ['react', 'react-dom', 'react-router-dom', '@tanstack/react-query', 'date-fns'],
    esbuildOptions: {
      minify: false
    }
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
})
