// vite.config.ts
import { defineConfig } from "file:///C:/Users/moqiq/PycharmProjects/NClientT/node_modules/vite/dist/node/index.js";
import vue from "file:///C:/Users/moqiq/PycharmProjects/NClientT/node_modules/@vitejs/plugin-vue/dist/index.mjs";
import { fileURLToPath, URL } from "node:url";
var __vite_injected_original_import_meta_url = "file:///C:/Users/moqiq/PycharmProjects/NClientT/vite.config.ts";
var host = process.env.TAURI_DEV_HOST;
var vite_config_default = defineConfig(async () => ({
  plugins: [vue()],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", __vite_injected_original_import_meta_url))
    }
  },
  // Vite will tailor the build to the right target depending on the platform.
  clearScreen: false,
  server: {
    // Tauri requires a fixed port; fall back if it is busy.
    port: 1420,
    strictPort: true,
    host: host || "127.0.0.1",
    hmr: host ? { protocol: "ws", host, port: 1421 } : void 0,
    watch: {
      // Tell Vite to ignore the Rust backend during HMR.
      ignored: ["**/src-tauri/**"]
    }
  },
  build: {
    // Tauri uses Chromium on Windows / WebKit on macOS / WebKitGTK on Linux.
    target: "es2021",
    minify: "esbuild",
    sourcemap: false
  }
}));
export {
  vite_config_default as default
};
//# sourceMappingURL=data:application/json;base64,ewogICJ2ZXJzaW9uIjogMywKICAic291cmNlcyI6IFsidml0ZS5jb25maWcudHMiXSwKICAic291cmNlc0NvbnRlbnQiOiBbImNvbnN0IF9fdml0ZV9pbmplY3RlZF9vcmlnaW5hbF9kaXJuYW1lID0gXCJDOlxcXFxVc2Vyc1xcXFxtb3FpcVxcXFxQeWNoYXJtUHJvamVjdHNcXFxcTkNsaWVudFRcIjtjb25zdCBfX3ZpdGVfaW5qZWN0ZWRfb3JpZ2luYWxfZmlsZW5hbWUgPSBcIkM6XFxcXFVzZXJzXFxcXG1vcWlxXFxcXFB5Y2hhcm1Qcm9qZWN0c1xcXFxOQ2xpZW50VFxcXFx2aXRlLmNvbmZpZy50c1wiO2NvbnN0IF9fdml0ZV9pbmplY3RlZF9vcmlnaW5hbF9pbXBvcnRfbWV0YV91cmwgPSBcImZpbGU6Ly8vQzovVXNlcnMvbW9xaXEvUHljaGFybVByb2plY3RzL05DbGllbnRUL3ZpdGUuY29uZmlnLnRzXCI7aW1wb3J0IHsgZGVmaW5lQ29uZmlnIH0gZnJvbSBcInZpdGVcIjtcclxuaW1wb3J0IHZ1ZSBmcm9tIFwiQHZpdGVqcy9wbHVnaW4tdnVlXCI7XHJcbmltcG9ydCB7IGZpbGVVUkxUb1BhdGgsIFVSTCB9IGZyb20gXCJub2RlOnVybFwiO1xyXG5cclxuLy8gVGF1cmkgZXhwZWN0cyBhIGZpeGVkIHBvcnQgYW5kIGEgY2xlYXIgc2V0IG9mIGhvc3QgcnVsZXMuXHJcbmNvbnN0IGhvc3QgPSBwcm9jZXNzLmVudi5UQVVSSV9ERVZfSE9TVDtcclxuXHJcbmV4cG9ydCBkZWZhdWx0IGRlZmluZUNvbmZpZyhhc3luYyAoKSA9PiAoe1xyXG4gIHBsdWdpbnM6IFt2dWUoKV0sXHJcbiAgcmVzb2x2ZToge1xyXG4gICAgYWxpYXM6IHtcclxuICAgICAgXCJAXCI6IGZpbGVVUkxUb1BhdGgobmV3IFVSTChcIi4vc3JjXCIsIGltcG9ydC5tZXRhLnVybCkpLFxyXG4gICAgfSxcclxuICB9LFxyXG4gIC8vIFZpdGUgd2lsbCB0YWlsb3IgdGhlIGJ1aWxkIHRvIHRoZSByaWdodCB0YXJnZXQgZGVwZW5kaW5nIG9uIHRoZSBwbGF0Zm9ybS5cclxuICBjbGVhclNjcmVlbjogZmFsc2UsXHJcbiAgc2VydmVyOiB7XHJcbiAgICAvLyBUYXVyaSByZXF1aXJlcyBhIGZpeGVkIHBvcnQ7IGZhbGwgYmFjayBpZiBpdCBpcyBidXN5LlxyXG4gICAgcG9ydDogMTQyMCxcclxuICAgIHN0cmljdFBvcnQ6IHRydWUsXHJcbiAgICBob3N0OiBob3N0IHx8IFwiMTI3LjAuMC4xXCIsXHJcbiAgICBobXI6IGhvc3RcclxuICAgICAgPyB7IHByb3RvY29sOiBcIndzXCIsIGhvc3QsIHBvcnQ6IDE0MjEgfVxyXG4gICAgICA6IHVuZGVmaW5lZCxcclxuICAgIHdhdGNoOiB7XHJcbiAgICAgIC8vIFRlbGwgVml0ZSB0byBpZ25vcmUgdGhlIFJ1c3QgYmFja2VuZCBkdXJpbmcgSE1SLlxyXG4gICAgICBpZ25vcmVkOiBbXCIqKi9zcmMtdGF1cmkvKipcIl0sXHJcbiAgICB9LFxyXG4gIH0sXHJcbiAgYnVpbGQ6IHtcclxuICAgIC8vIFRhdXJpIHVzZXMgQ2hyb21pdW0gb24gV2luZG93cyAvIFdlYktpdCBvbiBtYWNPUyAvIFdlYktpdEdUSyBvbiBMaW51eC5cclxuICAgIHRhcmdldDogXCJlczIwMjFcIixcclxuICAgIG1pbmlmeTogXCJlc2J1aWxkXCIsXHJcbiAgICBzb3VyY2VtYXA6IGZhbHNlLFxyXG4gIH0sXHJcbn0pKTtcclxuIl0sCiAgIm1hcHBpbmdzIjogIjtBQUFpVCxTQUFTLG9CQUFvQjtBQUM5VSxPQUFPLFNBQVM7QUFDaEIsU0FBUyxlQUFlLFdBQVc7QUFGNEosSUFBTSwyQ0FBMkM7QUFLaFAsSUFBTSxPQUFPLFFBQVEsSUFBSTtBQUV6QixJQUFPLHNCQUFRLGFBQWEsYUFBYTtBQUFBLEVBQ3ZDLFNBQVMsQ0FBQyxJQUFJLENBQUM7QUFBQSxFQUNmLFNBQVM7QUFBQSxJQUNQLE9BQU87QUFBQSxNQUNMLEtBQUssY0FBYyxJQUFJLElBQUksU0FBUyx3Q0FBZSxDQUFDO0FBQUEsSUFDdEQ7QUFBQSxFQUNGO0FBQUE7QUFBQSxFQUVBLGFBQWE7QUFBQSxFQUNiLFFBQVE7QUFBQTtBQUFBLElBRU4sTUFBTTtBQUFBLElBQ04sWUFBWTtBQUFBLElBQ1osTUFBTSxRQUFRO0FBQUEsSUFDZCxLQUFLLE9BQ0QsRUFBRSxVQUFVLE1BQU0sTUFBTSxNQUFNLEtBQUssSUFDbkM7QUFBQSxJQUNKLE9BQU87QUFBQTtBQUFBLE1BRUwsU0FBUyxDQUFDLGlCQUFpQjtBQUFBLElBQzdCO0FBQUEsRUFDRjtBQUFBLEVBQ0EsT0FBTztBQUFBO0FBQUEsSUFFTCxRQUFRO0FBQUEsSUFDUixRQUFRO0FBQUEsSUFDUixXQUFXO0FBQUEsRUFDYjtBQUNGLEVBQUU7IiwKICAibmFtZXMiOiBbXQp9Cg==
