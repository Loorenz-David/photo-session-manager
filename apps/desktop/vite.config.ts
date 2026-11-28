import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: { alias: { '@': new URL('./src', import.meta.url).pathname } },
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  // LV2 (master_plan_live §4.5): the desktop app and the phone app build into one dist/.
  build: { rollupOptions: { input: { main: new URL('./index.html', import.meta.url).pathname, phone: new URL('./phone.html', import.meta.url).pathname } } },
  test: { environment: 'jsdom', setupFiles: ['./src/test/setup.ts'] }
});

