import type { Backend, VNode } from "../types.js";

/** In-process backend: keeps the last virtual tree and emits it as a JSON line. */
export class JsonBackend implements Backend {
  last?: VNode;
  dispatch?: (nodeId: number, event: string, payload?: unknown) => void;
  constructor(private out: (line: string) => void = () => {}) {}
  present(root: VNode) { this.last = root; this.out(JSON.stringify({ type: "tree", root })); }
  setEventDispatcher(d: (nodeId: number, event: string, payload?: unknown) => void) { this.dispatch = d; }
}
