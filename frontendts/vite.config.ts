import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  cacheDir: process.env.VITE_CACHE_DIR,
  plugins: [
    sveltekit(),
    tailwindcss(),
  ],
  server: {
    proxy: {
      "^/grpc\\..*": {
        target: "http://localhost:50051",
        changeOrigin: true,
      },
    },
  },
});
