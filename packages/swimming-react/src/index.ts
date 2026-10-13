import type { ReactNode } from "react";
import { createReconciler, dispatchEvent, type Container, type SwimmingNative } from "./reconciler.js";

export { dispatchEvent, styleToCss, serializeProps, type SwimmingNative } from "./reconciler.js";

export interface Root {
  render(element: ReactNode): void;
  unmount(): void;
}

/**
 * Mounts a React tree into the native Swimming engine (root node id 0).
 * `native` defaults to the `__SWIMMING_NATIVE__` global injected by swimming-bridge.
 */
export function createRoot(native: SwimmingNative = (globalThis as any).__SWIMMING_NATIVE__): Root {
  const reconciler = createReconciler(native);
  const container: Container = { id: 0, type: "root", props: {}, parent: null, native };
  (globalThis as any).__SWIMMING_DISPATCH__ = dispatchEvent;
  const fiber = reconciler.createContainer(container, 0, null, false, null, "swimming", () => {}, null);
  return {
    render: (el) => reconciler.updateContainer(el, fiber, null, () => {}),
    unmount: () => reconciler.updateContainer(null, fiber, null, () => {}),
  };
}
