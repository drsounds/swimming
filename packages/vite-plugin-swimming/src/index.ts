import type { Plugin } from "vite";
import { build, type ViteDevServer } from "vite";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

/**
 * Bundles a TSX app into one script for the embedded QuickJS runtime:
 * a single IIFE file, ES2020, with the swimming-web-compat polyfills injected first.
 */
export default function swimming(options: { entry?: string; fileName?: string } = {}): Plugin {
  const entry = options.entry ?? "src/main.tsx";
  const fileName = options.fileName ?? "bundle.js";
  const devBundle = async (server: ViteDevServer) => {
    const outDir = await mkdtemp(join(tmpdir(), "swimming-vite-"));
    try {
      await build({
        configFile: server.config.configFile,
        root: server.config.root,
        mode: server.config.mode,
        build: { outDir, emptyOutDir: true },
      });
      return await readFile(join(outDir, fileName.endsWith(".js") ? fileName : `${fileName}.js`), "utf8");
    } finally {
      await rm(outDir, { recursive: true, force: true });
    }
  };
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
    configureServer(server) {
      let cachedBundle: Promise<string> | undefined;
      server.watcher.on("change", () => {
        cachedBundle = undefined;
      });
      server.middlewares.use((req, res, next) => {
        if (req.url?.split("?")[0] !== "/@swimming/bundle.js") return next();
        if (req.method !== "GET") {
          res.statusCode = 405;
          res.end("Method not allowed");
          return;
        }
        cachedBundle ??= devBundle(server);
        void cachedBundle.then((code) => {
          res.setHeader("Content-Type", "application/javascript");
          res.setHeader("Cache-Control", "no-store");
          res.end(code);
        }).catch((error: unknown) => {
          cachedBundle = undefined;
          res.statusCode = 500;
          res.end(error instanceof Error ? error.message : String(error));
        });
      });
    },
    transformIndexHtml: {
      order: "post",
      handler(html, context) {
        if (!new URL(context.originalUrl ?? "/", "http://swimming.local").searchParams.has("swimming-native")) return html;
        return {
          html,
          tags: [{ tag: "script", attrs: { src: "/@swimming/bundle.js" }, injectTo: "body" }],
        };
      },
    },
  };
}
