---
sidebar_position: 1
---

# Swimming

Swimming is a cross-platform native GUI framework with a TypeScript/TSX frontend. It combines a
custom React reconciler and browser-compatibility shims with a Rust layout and rendering pipeline.
Unlike webview-based frameworks, Swimming does not embed a browser engine or DOM.

## How it works

```text
TSX app → Vite bundle → web compatibility layer → React reconciler
  → QuickJS bridge → Rust layout engine → native renderer
```

Use the guide to [build and run the example](./getting-started), or read about the
[architecture](./architecture).
