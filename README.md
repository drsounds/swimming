# swimming
A "react-native-rust" for desktop apps: a Tauri-style shell with **no browser engine**. Write UI in React `.tsx` with ordinary HTML elements, DOM-style events and CSS; a Rust engine renders it natively.

```
React (TSX) -> custom reconciler -> virtual tree (JSON lines) -> Rust engine
                                                                 ├─ CSS:    lightningcss
                                                                 ├─ layout: taffy (flexbox)
                                                                 └─ paint:  tiny-skia (pluggable window backends: Druid, imgui, ...)
```

There is no DOM and no DOM emulation layer: the reconciler emits React's virtual tree and Rust resolves CSS and lays it out directly. Events are hit-tested natively and sent back to JS handlers, bubbling like DOM events.

- `src/` – TypeScript: React reconciler, `createRoot(backend)`, `NativeBackend` (spawns the Rust engine), `JsonBackend` (in-process/testing).
- `native/` – Rust crate `swimming-native`: UA defaults for all HTML elements (hidden elements such as `head`/`script`, block vs. inline), inline CSS, flexbox layout, anonymous inline runs, hit testing, software painting (backgrounds, borders, radius) to PNG with `--out`.

```tsx
import { createRoot, NativeBackend } from "swimming";
createRoot(new NativeBackend("native/target/release/swimming-native", ["--out", "frame.png"]))
  .render(<div style={{ padding: 10 }}><button onClick={() => console.log("hi")}>Click</button></div>);
```

Current limitations: no text glyph rendering yet (text is laid out with approximate metrics), no `<style>` sheets/selectors yet (inline styles + UA defaults), no window backend yet (headless PNG), no form-control behaviour, images, or non-px/%/em units beyond basics.

Scripts: `npm run build`, `npm test`; `cd native && cargo test`.
