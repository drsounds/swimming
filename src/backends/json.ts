import type { Backend, SceneNode } from "../types.js";

/**
 * Reference backend: emits each scene as one JSON line. A Rust host
 * (Druid, imgui, ...) can consume this stream and send events back.
 */
export class JsonBackend implements Backend {
  last?: SceneNode;
  dispatch?: (nodeId: number, event: string, payload?: unknown) => void;
  constructor(private out: (line: string) => void = () => {}, private dims = { width: 800, height: 600 }) {}
  present(scene: SceneNode) { this.last = scene; this.out(JSON.stringify(scene)); }
  setEventDispatcher(d: (nodeId: number, event: string, payload?: unknown) => void) { this.dispatch = d; }
  size() { return this.dims; }
}
