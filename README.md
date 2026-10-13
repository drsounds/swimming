# swimming
A TypeScript framework for porting React.JS web apps (Electron/Chromium) to native Rust UI.

- Write UI in `.tsx` with common HTML elements (`div`, `span`, `button`, ...), DOM-style events (`onClick`, ...) and CSS (`style` objects or strings).
- A custom React reconciler (`react-reconciler`) builds a host tree; layout is computed with [Yoga](https://yogalayout.dev).
- After every commit the laid-out scene (`SceneNode`) is passed to a pluggable `Backend` (e.g. a Rust Druid or imgui host). Backends deliver native events back through `setEventDispatcher`, which bubbles them like DOM events.
- Node.js packages can be used freely in the JS side.

```tsx
import { createRoot, JsonBackend } from "swimming";
const backend = new JsonBackend((line) => process.stdout.write(line + "\n"));
createRoot(backend).render(<div style={{ padding: 10 }}>Hello</div>);
```

`JsonBackend` is a reference backend streaming scenes as JSON lines for a Rust host to consume.

Scripts: `npm run build`, `npm test`.
