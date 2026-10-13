import Reconciler from "react-reconciler";
import { DefaultEventPriority } from "react-reconciler/constants.js";
import type { HostNode, VNode } from "./types.js";

let nextId = 1;

export interface Container {
  root: HostNode;
  onCommit: (root: HostNode) => void;
}

function create(type: string, props: Record<string, unknown>): HostNode {
  return { id: nextId++, type, props, children: [], parent: null };
}

function insert(parent: HostNode, child: HostNode, before?: HostNode) {
  remove(parent, child);
  const idx = before ? parent.children.indexOf(before) : -1;
  parent.children.splice(idx < 0 ? parent.children.length : idx, 0, child);
  child.parent = parent;
}

function remove(parent: HostNode, child: HostNode) {
  const i = parent.children.indexOf(child);
  if (i >= 0) parent.children.splice(i, 1);
  child.parent = null;
}

export function createReconciler() {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const config: any = {
    supportsMutation: true,
    supportsPersistence: false,
    supportsHydration: false,
    isPrimaryRenderer: true,
    noTimeout: -1,
    scheduleTimeout: setTimeout,
    cancelTimeout: clearTimeout,
    createInstance: create,
    createTextInstance: (text: string) => Object.assign(create("#text", {}), { text }),
    appendInitialChild: insert,
    appendChild: insert,
    appendChildToContainer: (c: Container, child: HostNode) => insert(c.root, child),
    insertBefore: insert,
    insertInContainerBefore: (c: Container, child: HostNode, before: HostNode) => insert(c.root, child, before),
    removeChild: remove,
    removeChildFromContainer: (c: Container, child: HostNode) => remove(c.root, child),
    commitUpdate: (node: HostNode, _t: string, _o: unknown, props: Record<string, unknown>) => { node.props = props; },
    commitTextUpdate: (node: HostNode, _o: string, text: string) => { node.text = text; },
    finalizeInitialChildren: () => false,
    prepareUpdate: () => true,
    shouldSetTextContent: () => false,
    getRootHostContext: () => ({}),
    getChildHostContext: (ctx: unknown) => ctx,
    getPublicInstance: (i: HostNode) => i,
    prepareForCommit: () => null,
    resetAfterCommit: (c: Container) => c.onCommit(c.root),
    clearContainer: (c: Container) => { c.root.children.length = 0; },
    preparePortalMount: () => {},
    getCurrentEventPriority: () => DefaultEventPriority,
    getInstanceFromNode: () => null,
    beforeActiveInstanceBlur: () => {},
    afterActiveInstanceBlur: () => {},
    prepareScopeUpdate: () => {},
    getInstanceFromScope: () => null,
    detachDeletedInstance: () => {},
  };
  return Reconciler(config);
}

const kebab = (s: string) => s.replace(/[A-Z]/g, (c) => "-" + c.toLowerCase());

/** Serializes a style prop (object or CSS string) to inline CSS text. */
export function styleToCss(style: unknown): string {
  if (typeof style === "string") return style;
  if (!style || typeof style !== "object") return "";
  return Object.entries(style as Record<string, unknown>)
    .filter(([, v]) => v !== undefined && v !== null && v !== false)
    .map(([k, v]) => `${kebab(k)}:${typeof v === "number" && !UNITLESS.has(kebab(k)) && v !== 0 ? v + "px" : v}`)
    .join(";");
}
const UNITLESS = new Set(["flex", "flex-grow", "flex-shrink", "opacity", "z-index", "font-weight", "line-height", "order"]);

export function toVNode(node: HostNode): VNode {
  const props: Record<string, unknown> = {};
  for (const [k, v] of Object.entries(node.props)) {
    if (k === "children" || k === "style") continue;
    props[k] = typeof v === "function" ? true : v; // handlers stay in JS; flag only
  }
  return {
    id: node.id, type: node.type, text: node.text, style: styleToCss(node.props.style),
    props, children: node.children.map(toVNode),
  };
}

export { create as createHostNode };
