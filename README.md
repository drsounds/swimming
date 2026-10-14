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

## Run a native window

Run a built Swimming bundle in an egui window (the default backend):

```sh
cargo run -p swimming-bridge -- examples/hello-swimming/dist/bundle.js
```

To use the Fastframe shell integration, build with its optional feature and select it:

```sh
cargo run -p swimming-bridge --features fastframe -- examples/hello-swimming/dist/bundle.js --backend fastframe
```

The CLI also accepts a local HTML file or an HTTP URL. HTML body elements are added to the native layout tree, and inline or classic external JavaScript script tags run in document order. Vite's dev server is supported by the Swimming plugin's native bundle endpoint:

```sh
npm run dev -w hello-swimming -- --host 127.0.0.1
cargo run -p swimming-bridge -- http://127.0.0.1:5173/?swimming-native
```

`--out frame.png` keeps the existing one-frame, headless PNG mode. Windows tagged releases build `swimming.exe` with both GUI backends and attach it to the GitHub release.

## Deviations from the spec / not yet done
- Rendering uses a CPU painter (tiny-skia) instead of Vello/WGPU + Swash; **text glyphs are not drawn** (text is laid out with approximate metrics).
- Layout is Taffy flexbox with HTML UA defaults (block/inline/hidden); Grid and `<style>` sheets/selectors are not wired up.
- The JS engine is QuickJS only (Boa not supported). Native loading skips browser ES modules in favor of the Vite plugin's generated IIFE bundle. `<img>` loading and form-control behavior are not implemented.
