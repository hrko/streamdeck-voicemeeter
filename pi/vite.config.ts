import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  base: "./",
  build: { target: "chrome110", outDir: "dist" },
  test: { environment: "jsdom", include: ["src/**/*.test.{ts,tsx}"] },
});
