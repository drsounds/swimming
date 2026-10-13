import type { ReactNode } from "react";
import { createReconciler, createHostNode, toScene, type Container } from "./reconciler.js";
import type { Backend, HostNode } from "./types.js";

export * from "./types.js";
export { parseCss, normalizeStyle } from "./css.js";
export { JsonBackend } from "./backends/json.js";

const reconciler = createReconciler();

export interface Root {
  render(element: ReactNode): void;
  unmount(): void;
}

const EVENT_PROPS: Record<string, string> = {
  click: "onClick", input: "onInput", change: "onChange", keydown: "onKeyDown",
  keyup: "onKeyUp", mouseenter: "onMouseEnter", mouseleave: "onMouseLeave",
};

/** Mounts a React tree and presents laid out scenes to the given native backend. */
export function createRoot(backend: Backend): Root {
  const rootNode: HostNode = createHostNode("root", {});
  const container: Container = {
    root: rootNode,
    nodes: new Map(),
    onCommit: (root) => {
      const size = backend.size?.() ?? { width: 800, height: 600 };
      root.yoga.setWidth(size.width);
      root.yoga.setHeight(size.height);
      root.yoga.calculateLayout(size.width, size.height);
      backend.present(toScene(root));
    },
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
