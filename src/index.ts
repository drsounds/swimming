import type { ReactNode } from "react";
import { createReconciler, createHostNode, toVNode, type Container } from "./reconciler.js";
import type { Backend, HostNode } from "./types.js";

export * from "./types.js";
export { styleToCss } from "./reconciler.js";
export { JsonBackend } from "./backends/json.js";
export { NativeBackend } from "./backends/native.js";

const reconciler = createReconciler();

export interface Root {
  render(element: ReactNode): void;
  unmount(): void;
}

const EVENT_PROPS: Record<string, string> = {
  click: "onClick", input: "onInput", change: "onChange", keydown: "onKeyDown",
  keyup: "onKeyUp", mouseenter: "onMouseEnter", mouseleave: "onMouseLeave",
};

/** Mounts a React tree and presents its virtual tree to the given native backend. */
export function createRoot(backend: Backend): Root {
  const rootNode: HostNode = createHostNode("root", {});
  const container: Container = {
    root: rootNode,
    // The Rust engine renders a single root element; wrap multiple children in one.
    onCommit: (root) => backend.present(toVNode(root.children.length === 1 ? root.children[0] : root)),
  };
  const find = (n: HostNode, id: number): HostNode | undefined =>
    n.id === id ? n : n.children.map((c) => find(c, id)).find(Boolean);

  backend.setEventDispatcher?.((nodeId, event, payload) => {
    // Bubble from target up through ancestors like DOM events.
    let n = find(rootNode, nodeId) ?? null;
    let stopped = false;
    const evt = { type: event, target: nodeId, payload, stopPropagation: () => { stopped = true; } };
    while (n && !stopped) {
      const h = n.props[EVENT_PROPS[event] ?? "on" + event[0].toUpperCase() + event.slice(1)];
      if (typeof h === "function") h(evt);
      n = n.parent;
    }
  });

  const fiber = reconciler.createContainer(container, 0, null, false, null, "swimming", () => {}, null);
  return {
    render: (el) => reconciler.updateContainer(el, fiber, null, () => {}),
    unmount: () => reconciler.updateContainer(null, fiber, null, () => {}),
  };
}
