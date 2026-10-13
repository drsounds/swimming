export type Style = Record<string, string | number | undefined>;

export interface HostNode {
  id: number;
  type: string;
  props: Record<string, unknown>;
  text?: string;
  children: HostNode[];
  parent: HostNode | null;
}

/**
 * Serializable React virtual-tree node sent to the Rust engine. There is no DOM:
 * CSS resolution, layout and painting all happen natively from this tree.
 */
export interface VNode {
  id: number;
  type: string;
  text?: string;
  /** Inline CSS text. */
  style: string;
  props: Record<string, unknown>;
  children: VNode[];
}

export interface Backend {
  /** Called after every commit with the new virtual tree. */
  present(root: VNode): void;
  /** Backend registers a dispatcher to deliver native events to DOM-style handlers. */
  setEventDispatcher?(dispatch: (nodeId: number, event: string, payload?: unknown) => void): void;
}
