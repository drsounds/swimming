---
sidebar_position: 2
---

# Getting started

## Prerequisites

- Node.js and npm
- Rust and Cargo

## Build the example

From the repository root, install the JavaScript dependencies and build the example app:

```sh
npm install
npm run build:example
```

Run the generated bundle in a native window:

```sh
cargo run -p swimming-bridge -- examples/hello-swimming/dist/bundle.js
```

To render one frame to a PNG instead, use the headless mode:

```sh
cargo run -p swimming-bridge -- examples/hello-swimming/dist/bundle.js \
  --width 500 --height 300 --out frame.png
```

Run the existing test suites with:

```sh
npm test
cargo test
```

For more details about capabilities and current limitations, see the
[project README](https://github.com/drsounds/swimming#readme).
