import { defineConfig } from 'vite'

// https://vitejs.dev/config/
export default defineConfig({
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: [
        '**/.cache/**',
        '**/.flatpak-builder/**',
        '**/flatpak-build/**',
        '**/flatpak-repo/**',
        '**/flatpak-source/**',
        '**/target/**',
      ],
    },
  },
})
