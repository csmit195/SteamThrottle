import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";

const host = Reflect.get(globalThis, "process")?.env?.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [sveltekit()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: { ignored: ["../src/**", "../target/**"] },
  },
}));
