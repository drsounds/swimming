import type { Plugin } from "vite";

/**
 * Bundles a TSX app into one script for the embedded QuickJS runtime:
 * a single IIFE file, ES2020, with the swimming-web-compat polyfills injected first.
 */
export default function swimming(options: { entry?: string; fileName?: string } = {}): Plugin {
  const entry = options.entry ?? "src/main.tsx";
  const fileName = options.fileName ?? "bundle.js";
  return {
    name: "vite-plugin-swimming",
    enforce: "pre",
    config: () => ({
      define: { "process.env.NODE_ENV": JSON.stringify(process.env.NODE_ENV ?? "production") },
      esbuild: { jsx: "automatic" as const },
      build: {
        target: "es2020",
        minify: false,
        cssCodeSplit: false,
        lib: { entry, formats: ["iife" as const], name: "SwimmingApp", fileName: () => fileName },
        rollupOptions: { output: { inlineDynamicImports: true } },
      },
    }),
    transform(code, id) {
      if (id.split("?")[0].endsWith(entry.replace(/^\.?\//, "")) || id.endsWith("/" + entry)) {
        return { code: `import "swimming-web-compat";\n${code}`, map: null };
      }
      return null;
    },
  };
}
