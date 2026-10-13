export type Style = Record<string, string | number | undefined>;

export interface HostNode {
  id: number;
  type: string;
  props: Record<string, unknown>;
  style: Style;
  text?: string;
  children: HostNode[];
  parent: HostNode | null;
  yoga: import("yoga-layout").Node;
  layout: Layout;
}

export interface Layout {
  x: number;
  y: number;
  width: number;
  height: number;
}

/** Serializable scene node handed to a native backend (e.g. a Rust Druid or imgui host). */
export interface SceneNode {
  id: number;
  type: string;
  text?: string;
  style: Style;
  props: Record<string, unknown>;
  layout: Layout;
  children: SceneNode[];
}

export interface Backend {
  /** Called after every commit with the laid out scene tree. */
  present(scene: SceneNode): void;
  /** Backend registers a dispatcher to deliver native events to DOM handlers. */
  setEventDispatcher?(dispatch: (nodeId: number, event: string, payload?: unknown) => void): void;
  size?(): { width: number; height: number };
}
