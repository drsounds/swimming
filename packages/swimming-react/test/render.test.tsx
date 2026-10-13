import { test } from "node:test";
import assert from "node:assert/strict";
import React, { useState } from "react";
import { createRoot, dispatchEvent, type SwimmingNative } from "../src/index.js";

interface N { id: number; tag: string; style: string; props: string; text?: string; kids: number[] }

function fakeNative() {
  const nodes = new Map<number, N>([[0, { id: 0, tag: "root", style: "", props: "{}", kids: [] }]]);
  let next = 1, commits = 0;
  const detach = (c: number) => nodes.forEach((n) => { n.kids = n.kids.filter((k) => k !== c); });
  const native: SwimmingNative = {
    createNode: (tag, style, props) => { nodes.set(next, { id: next, tag, style, props, kids: [] }); return next++; },
    createText: (text) => { nodes.set(next, { id: next, tag: "#text", style: "", props: "{}", text, kids: [] }); return next++; },
    setText: (id, t) => { nodes.get(id)!.text = t; },
    appendChild: (p, c) => { detach(c); nodes.get(p)!.kids.push(c); },
    insertBefore: (p, c, b) => { detach(c); const k = nodes.get(p)!.kids; k.splice(b < 0 ? k.length : k.indexOf(b), 0, c); },
    removeChild: (p, c) => { detach(c); nodes.delete(c); },
    updateNode: (id, style, props) => { Object.assign(nodes.get(id)!, { style, props }); },
    commit: () => { commits++; },
  };
  const text = (id: number): string => { const n = nodes.get(id)!; return n.text ?? n.kids.map(text).join(""); };
  return { native, nodes, text, commits: () => commits };
}

test("drives native commands with CSS serialized for the Rust engine", () => {
  const f = fakeNative();
  createRoot(f.native).render(
    <div style={{ flexDirection: "row", padding: 10, opacity: 0.5 }}>
      <table style={"width: 50px" as never}><caption>hi</caption></table>
    </div>
  );
  const div = f.nodes.get(f.nodes.get(0)!.kids[0])!;
  assert.equal(div.tag, "div");
  assert.equal(div.style, "flex-direction:row;padding:10px;opacity:0.5");
  assert.equal(f.nodes.get(div.kids[0])!.style, "width: 50px");
  assert.equal(f.text(0), "hi");
  assert.ok(f.commits() > 0);
});

test("dispatches native events, bubbles, and re-renders", () => {
  const f = fakeNative();
  function App() {
    const [n, set] = useState(0);
    return <div onClick={() => set((c) => c + 10)}><button onClick={() => set((c) => c + 1)}>{`count ${n}`}</button></div>;
  }
  createRoot(f.native).render(<App />);
  const div = f.nodes.get(0)!.kids[0];
  const button = f.nodes.get(div)!.kids[0];
  assert.equal(f.nodes.get(button)!.props, '{"onClick":true}');
  assert.equal(f.text(0), "count 0");
  dispatchEvent(button, "click");
  assert.equal(f.text(0), "count 11");
});
