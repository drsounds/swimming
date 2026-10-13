# Swimming Architecture Specification

**Swimming** is a lightweight, native cross-platform GUI framework for Rust with a TypeScript/TSX frontend. Inspired by React Native and Tauri's process model, Swimming delivers native desktop performance without relying on any web browser engines or WebViews.

It supports building via **Vite** and provides built-in browser polyfills (`window`, `document`, `location`, `history`) so existing React web applications and standard routing solutions (such as React Router) can be ported with minimal to no code changes.

---

## 1. System Overview

```
┌─────────────────────────────────────────────────────────────┐
│ 1. TypeScript / TSX Application Layer (Vite-bundled)        │
│    Declarative UI supporting HTML tags (<div/>, <span/>, etc)│
│    Includes React Router & Web APIs (history, location)    │
├─────────────────────────────────────────────────────────────┤
│ 2. Web API Polyfill Layer (packages/swimming-web-compat)    │
│    Provides window, document, location, and history mocks   │
├─────────────────────────────────────────────────────────────┤
│ 3. Custom React Reconciler (packages/swimming-react)        │
│    Translates React virtual nodes into native FFI commands  │
├─────────────────────────────────────────────────────────────┤
│ 4. Embedded JS Engine (crates/swimming-bridge)              │
│    Executes TSX bundle via QuickJS (rquickjs) or Boa        │
├─────────────────────────────────────────────────────────────┤
│ 5. Native Layout Engine (crates/swimming-core)              │
│    Computes pixel coordinates using Taffy (Flexbox & Grid)  │
├─────────────────────────────────────────────────────────────┤
│ 6. Native GPU Render Loop (crates/swimming-render)          │
│    Renders UI primitives directly via Vello / WGPU + Swash  │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Repository Layout

```text
swimming/
├── crates/
│   ├── swimming-core/       # Virtual Shadow Tree & Taffy layout engine
│   ├── swimming-bridge/     # FFI bindings & JS engine integration (rquickjs)
│   └── swimming-render/     # Native GPU renderer (Vello/WGPU + Swash text)
├── packages/
│   ├── swimming-react/      # Custom React Reconciler (<div/>, <span/>, View, Text)
│   ├── swimming-web-compat/ # Polyfills for window, document, location, history
│   └── vite-plugin-swimming/ # Vite plugin for bundling TSX into Swimming JS runtime
└── examples/
    └── hello-swimming/      # TSX example application running natively with React Router
```

---

## 3. Web Compatibility & Routing Architecture

To allow seamless porting of web applications without a browser DOM:

1. **Vite Integration (`vite-plugin-swimming`):**
   - Bundles the application's TSX code into a single lightweight JavaScript file compatible with embedded engines (QuickJS/Boa).
   - Injects necessary polyfills at the bundle entry point.

2. **Browser Polyfills (`swimming-web-compat`):**
   - **`window` & `document`:** Mocked globals allowing DOM-dependent utility libraries to load without throwing errors.
   - **`location` & `history`:** Implements an in-memory browser history engine (`pushState`, `replaceState`, `popstate` events) enabling frameworks like **React Router** or **TanStack Router** to handle client-side routing natively.

---

## 4. Component Mapping (HTML to Native Flexbox)

Standard HTML elements are normalized into Flexbox layout rules evaluated by Taffy:

| HTML Element | Mapped Node Type | Default Taffy Layout Rules |
| :--- | :--- | :--- |
| `<div>`, `<section>`, `<article>`, `<main>` | Container (`View`) | `display: flex`, `flexDirection: column` |
| `<span>`, `<p>`, `<h1>`–`<h6>` | Text Node (`Text`) | Text shaping via Swash/Vello, `display: flex` |
| `<button>`, `<a>` | Interactive Node | `display: flex`, centered alignment + event/navigation handlers |
| `<img>` | Image Node (`Image`) | Asynchronous texture loading to GPU memory |

---

## 5. Complete Code Reference

### A. Web Environment & History Polyfill (`packages/swimming-web-compat/src/index.ts`)

```typescript
// Memory-based implementation of location and history for Routing compatibility
class MemoryHistory {
  private stack: string[] = ['/'];
  private index: number = 0;
  private listeners: Array<() => void> = [];

  get location() {
    const url = new URL(this.stack[this.index], 'http://swimming.local');
    return {
      pathname: url.pathname,
      search: url.search,
      hash: url.hash,
      href: url.href,
    };
  }

  pushState(state: any, unused: string, url?: string | null) {
    if (url) {
      this.stack = this.stack.slice(0, this.index + 1);
      this.stack.push(url);
      this.index++;
      this.notify();
    }
  }

  replaceState(state: any, unused: string, url?: string | null) {
    if (url) {
      this.stack[this.index] = url;
      this.notify();
    }
  }

  listen(fn: () => void) {
    this.listeners.push(fn);
    return () => {
      this.listeners = this.listeners.filter((l) => l !== fn);
    };
  }

  private notify() {
    this.listeners.forEach((fn) => fn());
  }
}

export const memoryHistory = new MemoryHistory();

// Polyfill globals for React web code and React Router
if (typeof globalThis.window === 'undefined') {
  (globalThis as any).window = globalThis;
  (globalThis as any).location = memoryHistory.location;
  (globalThis as any).history = {
    pushState: (s: any, u: string, url?: string) => memoryHistory.pushState(s, u, url),
    replaceState: (s: any, u: string, url?: string) => memoryHistory.replaceState(s, u, url),
    back: () => {},
    forward: () => {},
  };
}
```

---

### B. Custom React Reconciler (`packages/swimming-react/src/reconciler.ts`)

```typescript
import Reconciler from 'react-reconciler';

declare const __SWIMMING_NATIVE__: {
  createNode(tag: string, styleJson: string): number;
  appendChild(parentId: number, childId: number): void;
  removeChild(parentId: number, childId: number): void;
  updateNodeStyle(nodeId: number, styleJson: string): void;
};

const hostConfig: Reconciler.HostConfig<any, any, any, any, any, any, any, any, any, any, any, any> = {
  createInstance(type, props) {
    const style = normalizeHtmlStyles(type, props.style);
    const nodeId = __SWIMMING_NATIVE__.createNode(type, JSON.stringify(style));
    return { type, nodeId };
  },

  appendChild(parent, child) {
    __SWIMMING_NATIVE__.appendChild(parent.nodeId, child.nodeId);
  },

  appendInitialChild(parent, child) {
    __SWIMMING_NATIVE__.appendChild(parent.nodeId, child.nodeId);
  },

  removeChild(parent, child) {
    __SWIMMING_NATIVE__.removeChild(parent.nodeId, child.nodeId);
  },

  commitUpdate(instance, updatePayload, type, oldProps, newProps) {
    if (newProps.style !== oldProps.style) {
      const style = normalizeHtmlStyles(type, newProps.style);
      __SWIMMING_NATIVE__.updateNodeStyle(instance.nodeId, JSON.stringify(style));
    }
  },

  createTextInstance(text) {
    const nodeId = __SWIMMING_NATIVE__.createNode('span', JSON.stringify({}));
    return { type: 'span', nodeId, text };
  },

  supportsMutation: true,
  isPrimaryRenderer: true,
  getRootHostContext: () => ({}),
  getChildHostContext: () => ({}),
  prepareForCommit: () => null,
  resetAfterCommit: () => {},
  shouldSetTextContent: () => false,
  clearContainer: () => {},
};

function normalizeHtmlStyles(tag: string, style: Record<string, any> = {}) {
  const htmlDefaults: Record<string, any> = {
    div: { display: 'flex', flexDirection: 'column' },
    span: { display: 'flex', flexDirection: 'row' },
    button: { display: 'flex', alignItems: 'center', justifyContent: 'center' },
  };

  return { ...(htmlDefaults[tag] || {}), ...style };
}

export const SwimmingRenderer = Reconciler(hostConfig);
```

---

### C. Core Engine & Layout Manager (`crates/swimming-core/src/lib.rs`)

```rust
use taffy::prelude::*;
use std::collections::HashMap;

pub struct SwimmingEngine {
    taffy: TaffyTree<()>,
    nodes: HashMap<u64, NodeId>,
    next_id: u64,
}

impl SwimmingEngine {
    pub fn new() -> Self {
        Self {
            taffy: TaffyTree::new(),
            nodes: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn create_node(&mut self, _tag: &str, style_json: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let style = parse_taffy_style(style_json);
        let taffy_node = self.taffy.new_leaf(style).unwrap();

        self.nodes.insert(id, taffy_node);
        id
    }

    pub fn append_child(&mut self, parent_id: u64, child_id: u64) {
        if let (Some(&parent), Some(&child)) = (self.nodes.get(&parent_id), self.nodes.get(&child_id)) {
            let _ = self.taffy.add_child(parent, child);
        }
    }

    pub fn remove_child(&mut self, parent_id: u64, child_id: u64) {
        if let (Some(&parent), Some(&child)) = (self.nodes.get(&parent_id), self.nodes.get(&child_id)) {
            let _ = self.taffy.remove_child(parent, child);
        }
    }

    pub fn update_node_style(&mut self, node_id: u64, style_json: &str) {
        if let Some(&node) = self.nodes.get(&node_id) {
            let new_style = parse_taffy_style(style_json);
            let _ = self.taffy.set_style(node, new_style);
        }
    }

    pub fn compute_layout(&mut self, root_id: u64, width: f32, height: f32) {
        if let Some(&root) = self.nodes.get(&root_id) {
            self.taffy
                .compute_layout(
                    root,
                    Size {
                        width: AvailableSpace::Definite(width),
                        height: AvailableSpace::Definite(height),
                    },
                )
                .unwrap();
        }
    }

    pub fn get_layout(&self, node_id: u64) -> Option<&Layout> {
        self.nodes.get(&node_id).and_then(|&node| self.taffy.layout(node).ok())
    }
}

fn parse_taffy_style(_json: &str) -> Style {
    Style::default()
}
```

---

### D. Embedded JS Bridge (`crates/swimming-bridge/src/lib.rs`)

```rust
use rquickjs::{Context, Runtime, Function, Result};
use std::sync::{Arc, Mutex};
use swimming_core::SwimmingEngine;

pub struct JsBridge {
    engine: Arc<Mutex<SwimmingEngine>>,
}

impl JsBridge {
    pub fn new(engine: Arc<Mutex<SwimmingEngine>>) -> Self {
        Self { engine }
    }

    pub fn register_globals(&self, context: &Context) -> Result<()> {
        let engine = self.engine.clone();

        context.with(|ctx| {
            let globals = ctx.globals();
            let bridge_obj = rquickjs::Object::new(ctx)?;

            let engine_clone = engine.clone();
            bridge_obj.set(
                "createNode",
                Function::new(ctx.clone(), move |tag: String, style_json: String| -> u64 {
                    engine_clone.lock().unwrap().create_node(&tag, &style_json)
                })?,
            )?;

            let engine_clone = engine.clone();
            bridge_obj.set(
                "appendChild",
                Function::new(ctx.clone(), move |parent_id: u64, child_id: u64| {
                    engine_clone.lock().unwrap().append_child(parent_id, child_id);
                })?,
            )?;

            globals.set("__SWIMMING_NATIVE__", bridge_obj)?;
            Ok(())
        })
    }
}
```

---

### E. Example Web Application with Routing (`examples/hello-swimming/App.tsx`)

```tsx
import React from 'react';
import { MemoryRouter, Routes, Route, Link } from 'react-router-dom';

function Home() {
  return (
    <div style={{ padding: 24, backgroundColor: '#0f172a' }}>
      <h1 style={{ color: '#38bdf8' }}>Swimming + Vite + Routing</h1>
      <p style={{ color: '#94a3b8' }}>
        Existing web app code running natively on Rust without WebViews.
      </p>
      <Link to="/about" style={{ color: '#0284c7', marginTop: 12 }}>
        Go to About Page
      </Link>
    </div>
  );
}

function About() {
  return (
    <div style={{ padding: 24, backgroundColor: '#0f172a' }}>
      <h1 style={{ color: '#38bdf8' }}>About Swimming</h1>
      <Link to="/" style={{ color: '#0284c7', marginTop: 12 }}>
        Back to Home
      </Link>
    </div>
  );
}

export function App() {
  return (
    <MemoryRouter>
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/about" element={<About />} />
      </Routes>
    </MemoryRouter>
  );
}
```