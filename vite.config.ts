import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { fileURLToPath, URL } from "node:url";

// Tauri expects a fixed port and a clear set of host rules.
const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [vue()],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
  // Vite will tailor the build to the right target depending on the platform.
  clearScreen: false,
  server: {
    // Tauri requires a fixed port; fall back if it is busy.
    port: 1420,
    strictPort: true,
    host: host || "127.0.0.1",
    hmr: host
      ? { protocol: "ws", host, port: 1421 }
      : undefined,
    watch: {
      // Tell Vite to ignore the Rust backend during HMR.
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    // Tauri uses Chromium on Windows / WebKit on macOS / WebKitGTK on Linux.
    target: "es2021",
    minify: "esbuild",
    sourcemap: false,
  },
}));
