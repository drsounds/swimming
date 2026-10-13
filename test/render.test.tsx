import { test } from "node:test";
import assert from "node:assert/strict";
import React, { useState } from "react";
import { createRoot, JsonBackend, type SceneNode } from "../src/index.js";

const flush = () => new Promise((r) => setTimeout(r, 20));
const find = (n: SceneNode, t: string): SceneNode | undefined =>
  n.type === t ? n : n.children.map((c) => find(c, t)).find(Boolean);

test("renders html elements with yoga layout", async () => {
  const b = new JsonBackend(() => {}, { width: 200, height: 100 });
  createRoot(b).render(
    <div style={{ flexDirection: "row", padding: 10 }}>
      <span style={{ width: 50, height: 20 }}>hi</span>
      <span style={{ width: 30, height: 20, marginLeft: 5 }}>yo</span>
    </div>
  );
  await flush();
  const div = b.last!.children[0];
  assert.equal(div.type, "div");
  assert.deepEqual(div.children[0].layout, { x: 10, y: 10, width: 50, height: 20 });
  assert.equal(div.children[1].layout.x, 65);
});

test("dispatches DOM events and re-renders", async () => {
  const b = new JsonBackend(() => {});
  function App() {
    const [n, set] = useState(0);
    return <button onClick={() => set(n + 1)}>{`count ${n}`}</button>;
  }
  createRoot(b).render(<App />);
  await flush();
  const btn = find(b.last!, "button")!;
  assert.equal(find(btn, "#text")!.text, "count 0");
  b.dispatch!(btn.id, "click");
  await flush();
  assert.equal(find(find(b.last!, "button")!, "#text")!.text, "count 1");
});
