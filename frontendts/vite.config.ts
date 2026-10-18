import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
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
