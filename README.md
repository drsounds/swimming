# swimming
A native cross-platform GUI framework for Rust with a TypeScript/TSX frontend, in the spirit of React Native and Tauri's process model: **no web browser engine, no WebView, no DOM**. See [Swimming Technical Specification.md](Swimming%20Technical%20Specification.md).

```
TSX app (Vite bundle) -> swimming-web-compat (window/location/history/timers polyfills)
  -> swimming-react (React reconciler) -> __SWIMMING_NATIVE__ commands
  -> swimming-bridge (QuickJS via rquickjs) -> swimming-core (shadow tree, lightningcss, Taffy)
  -> swimming-render (painter)
```

| Path | Role |
| --- | --- |
| `crates/swimming-core` | Shadow tree command API, CSS (lightningcss), UA defaults for all HTML elements, Taffy layout, hit testing |
| `crates/swimming-bridge` | Embeds QuickJS, exposes `__SWIMMING_NATIVE__`, runs timers/jobs, routes pointer events to JS; `swimming` CLI |
| `crates/swimming-render` | Painter (tiny-skia software renderer: backgrounds, borders, radius) |
| `packages/swimming-react` | Custom React reconciler (`createRoot`), DOM-style event bubbling |
| `packages/swimming-web-compat` | In-memory `window`, `document`, `location`, `history` (React Router works), timers |
| `packages/vite-plugin-swimming` | Vite plugin bundling TSX into one QuickJS-friendly IIFE |
| `examples/hello-swimming` | React Router example |

```
npm install
npm run build:example
cargo run -p swimming-bridge -- examples/hello-swimming/dist/bundle.js --width 500 --height 300 --out frame.png
npm test && cargo test
```

## Deviations from the spec / not yet done
- Rendering uses a CPU painter (tiny-skia, PNG output) instead of Vello/WGPU + Swash; no window or GPU yet, and **text glyphs are not drawn** (text is laid out with approximate metrics).
- Layout is Taffy flexbox with HTML UA defaults (block/inline/hidden); Grid and `<style>` sheets/selectors are not wired up.
- The JS engine is QuickJS only (Boa not supported). `<img>` loading and form-control behavior are not implemented.
