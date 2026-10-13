import { defineConfig } from "vite";
import swimming from "vite-plugin-swimming";

export default defineConfig({ plugins: [swimming({ entry: "src/main.tsx" })] });
