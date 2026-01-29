import { defineConfig } from "vite";

// https://vitejs.dev/config/
export default defineConfig({
  clearScreen: false,
  base: "./",
  server: {
    proxy: {
      "/api": "http://localhost:8482",
    },
  },
  define: {
    __APP_VERSION__: JSON.stringify(process.env.APP_VERSION || "0.0.0-dev"),
  },
});
