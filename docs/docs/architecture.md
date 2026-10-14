---
sidebar_position: 3
---

# Architecture

Swimming separates the TypeScript application layer from its native Rust runtime:

1. **Vite plugin** bundles the TSX application into a single JavaScript IIFE suitable for the
   embedded runtime.
2. **Web compatibility layer** provides in-memory `window`, `document`, `location`, and `history`
   APIs for application and routing compatibility.
3. **React reconciler** turns React elements and events into commands for the native runtime.
4. **Rust bridge** embeds QuickJS and routes commands and events between JavaScript and Rust.
5. **Core** builds the shadow tree, parses styles, computes Flexbox layout, and hit-tests input.
6. **Renderer** paints the resulting native scene.

## Repository components

| Path | Responsibility |
| --- | --- |
| `crates/swimming-core` | Shadow tree, CSS parsing, layout, and hit testing |
| `crates/swimming-bridge` | QuickJS integration, native window, and CLI |
| `crates/swimming-render` | Software rendering |
| `packages/swimming-react` | Custom React reconciler |
| `packages/swimming-web-compat` | Browser API compatibility shims |
| `packages/vite-plugin-swimming` | Vite integration and native bundle generation |
| `examples/hello-swimming` | Example React application |

Swimming is under active development. See the
[technical specification](https://github.com/drsounds/swimming/blob/main/Swimming%20Technical%20Specification.md)
and [README](https://github.com/drsounds/swimming#readme) for details and known limitations.
