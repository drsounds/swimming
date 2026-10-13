import { test } from "node:test";
import assert from "node:assert/strict";
import React, { useState } from "react";
import { createRoot, JsonBackend, type VNode } from "../src/index.js";

const flush = () => new Promise((r) => setTimeout(r, 20));
const find = (n: VNode, t: string): VNode | undefined =>
  n.type === t ? n : n.children.map((c) => find(c, t)).find(Boolean);

test("emits a plain virtual tree with CSS serialized for the Rust engine", async () => {
  const b = new JsonBackend();
  createRoot(b).render(
    <div style={{ flexDirection: "row", padding: 10, opacity: 0.5 }}>
      <table style={"width: 50px" as never}><caption>hi</caption></table>
    </div>
  );
  await flush();
  assert.equal(b.last!.type, "div");
  assert.equal(b.last!.style, "flex-direction:row;padding:10px;opacity:0.5");
  assert.equal(b.last!.children[0].style, "width: 50px");
  assert.equal(find(b.last!, "caption")!.children[0].text, "hi");
});

test("dispatches native events and re-renders", async () => {
  const b = new JsonBackend();
  function App() {
    const [n, set] = useState(0);
    return <button onClick={() => set(n + 1)}>{`count ${n}`}</button>;
  }
  createRoot(b).render(<App />);
  await flush();
  assert.equal(find(b.last!, "#text")!.text, "count 0");
  assert.equal(b.last!.props.onClick, true);
  b.dispatch!(b.last!.id, "click");
  await flush();
  assert.equal(find(b.last!, "#text")!.text, "count 1");
});
