import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

import { defineConfig } from 'vite';
import { viteSingleFile } from 'vite-plugin-singlefile';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

export default defineConfig({
  plugins: [viteSingleFile({ useRecommendedBuildConfig: false })],
  base: './',
  build: {
    assetsInlineLimit: () => true,
    chunkSizeWarningLimit: 100000000,
    cssCodeSplit: false,
    assetsDir: '',
    outDir: 'dist/widgets',
    emptyOutDir: false,
    rollupOptions: {
      input: {
        'services-list': resolve(__dirname, 'src/widgets/src/services-list.html'),
        'task-form': resolve(__dirname, 'src/widgets/src/task-form.html'),
        'user-dashboard': resolve(__dirname, 'src/widgets/src/user-dashboard.html'),
      },
      output: {
        entryFileNames: '[name].js',
        chunkFileNames: '[name].js',
        assetFileNames: '[name][extname]',
      },
    },
  },
});
