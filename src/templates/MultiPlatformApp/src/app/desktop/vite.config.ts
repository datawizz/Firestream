import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

const host = process.env.TAURI_DEV_HOST;

// Forward PUBLIC_* env vars from process.env (loaded by direnv) into import.meta.env.
// Vite only auto-loads .env files — it won't read .env.public by name,
// so we inject them via define.
const publicEnvDefines = Object.fromEntries(
  Object.entries(process.env)
    .filter(([key]) => key.startsWith('PUBLIC_'))
    .map(([key, val]) => [`import.meta.env.${key}`, JSON.stringify(val)])
);

export default defineConfig({
  plugins: [react()],
  envPrefix: 'PUBLIC_',
  define: publicEnvDefines,

  resolve: {
    preserveSymlinks: true,
    dedupe: ['react', 'react-dom', '@tanstack/react-query'],
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },

  optimizeDeps: {
    include: ["@multi-platform-app/shared", "@multi-platform-app/shared/components"],
  },

  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? {
      protocol: "ws",
      host,
      port: 1421,
    } : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
    fs: {
      allow: ['../../..']
    }
  },
});
