import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import dts from 'vite-plugin-dts';
import path from 'path';

export default defineConfig({
  plugins: [
    react(),
    dts({
      insertTypesEntry: true,
      include: ['src/**/*.ts', 'src/**/*.tsx'],
      exclude: ['**/*.test.*', '**/*.spec.*'],
    }),
  ],
  css: {
    postcss: './postcss.config.js',
  },
  build: {
    lib: {
      entry: {
        index: path.resolve(__dirname, 'src/index.ts'),
        'components/index': path.resolve(__dirname, 'src/components/index.ts'),
        'stores/index': path.resolve(__dirname, 'src/stores/index.ts'),
        'hooks/index': path.resolve(__dirname, 'src/hooks/index.ts'),
        'services/index': path.resolve(__dirname, 'src/services/index.ts'),
        'utils/index': path.resolve(__dirname, 'src/utils/index.ts'),
      },
      formats: ['es', 'cjs'],
      fileName: (format, entryName) => {
        const ext = format === 'es' ? 'mjs' : 'js';
        return `${entryName}.${ext}`;
      },
    },
    rollupOptions: {
      external: [
        'react',
        'react-dom',
        'react/jsx-runtime',
        '@tanstack/react-query',
        'zustand',
        '@tauri-apps/api',
      ],
      output: {
        preserveModules: true,
        preserveModulesRoot: './src',
        exports: 'named',
      },
    },
    sourcemap: true,
    minify: false,
    cssCodeSplit: false,
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
});
