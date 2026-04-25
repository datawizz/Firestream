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
  build: {
    lib: {
      entry: {
        index: path.resolve(__dirname, 'src/index.ts'),
        'types/index': path.resolve(__dirname, 'src/types/index.ts'),
        'services/index': path.resolve(__dirname, 'src/services/index.ts'),
        'hooks/index': path.resolve(__dirname, 'src/hooks/index.ts'),
        'stores/index': path.resolve(__dirname, 'src/stores/index.ts'),
        'context/index': path.resolve(__dirname, 'src/context/index.ts'),
        'adapters/index': path.resolve(__dirname, 'src/adapters/index.ts'),
        'adapters/web/index': path.resolve(__dirname, 'src/adapters/web/index.ts'),
        'adapters/tauri/index': path.resolve(__dirname, 'src/adapters/tauri/index.ts'),
        'adapters/dev/index': path.resolve(__dirname, 'src/adapters/dev/index.ts'),
        'server/index': path.resolve(__dirname, 'src/server/index.ts'),
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
        '@supabase/supabase-js',
        '@supabase/ssr',
        'stripe',
        'resend',
      ],
      output: {
        preserveModules: true,
        preserveModulesRoot: './src',
        exports: 'named',
      },
    },
    sourcemap: true,
    minify: false,
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
});
