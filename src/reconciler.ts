import Reconciler from "react-reconciler";
import { DefaultEventPriority } from "react-reconciler/constants.js";
import Yoga from "yoga-layout";
import { applyStyle, normalizeStyle, parseCss } from "./css.js";
import type { HostNode, SceneNode } from "./types.js";

let nextId = 1;

export interface Container {
  root: HostNode;
  onCommit: (root: HostNode) => void;
  nodes: Map<number, HostNode>;
}

function styleOf(props: Record<string, unknown>) {
  const st = props.style;
  return typeof st === "string" ? parseCss(st) : normalizeStyle(st);
}

function create(type: string, props: Record<string, unknown>, container?: Container): HostNode {
  const node: HostNode = {
    id: nextId++, type, props, style: styleOf(props), children: [], parent: null,
    yoga: Yoga.Node.create(), layout: { x: 0, y: 0, width: 0, height: 0 },
  };
  applyStyle(node.yoga, node.style);
  container?.nodes.set(node.id, node);
  return node;
}

function textNode(text: string): HostNode {
  const n = create("#text", {});
  n.text = text;
  return n;
}

function insert(parent: HostNode, child: HostNode, before?: HostNode) {
  remove(parent, child);
  const idx = before ? parent.children.indexOf(before) : parent.children.length;
  parent.children.splice(idx < 0 ? parent.children.length : idx, 0, child);
  child.parent = parent;
  parent.yoga.insertChild(child.yoga, idx < 0 ? parent.children.length - 1 : idx);
}

function remove(parent: HostNode, child: HostNode) {
  const i = parent.children.indexOf(child);
  if (i < 0) return;
  parent.children.splice(i, 1);
  parent.yoga.removeChild(child.yoga);
  child.parent = null;
}

function dispose(n: HostNode) {
  n.children.forEach(dispose);
  n.yoga.free();
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
    createInstance: (type: string, props: Record<string, unknown>) => create(type, props),
    createTextInstance: (text: string) => textNode(text),
    appendInitialChild: insert,
    appendChild: insert,
    appendChildToContainer: (c: Container, child: HostNode) => insert(c.root, child),
    insertBefore: insert,
    insertInContainerBefore: (c: Container, child: HostNode, before: HostNode) => insert(c.root, child, before),
    removeChild: (p: HostNode, c: HostNode) => { remove(p, c); dispose(c); },
    removeChildFromContainer: (c: Container, child: HostNode) => { remove(c.root, child); dispose(child); },
    commitUpdate: (node: HostNode, _t: string, _o: unknown, props: Record<string, unknown>) => {
      node.props = props;
      node.style = styleOf(props);
      applyStyle(node.yoga, node.style);
    },
    commitTextUpdate: (node: HostNode, _o: string, text: string) => { node.text = text; },
    finalizeInitialChildren: () => false,
    prepareUpdate: () => true,
    shouldSetTextContent: () => false,
    getRootHostContext: () => ({}),
    getChildHostContext: (ctx: unknown) => ctx,
    getPublicInstance: (i: HostNode) => i,
    prepareForCommit: () => null,
    resetAfterCommit: (c: Container) => c.onCommit(c.root),
    clearContainer: (c: Container) => { c.root.children.slice().forEach((ch) => remove(c.root, ch)); },
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

/** Computes layout, then converts the host tree into a serializable scene. */
export function toScene(node: HostNode): SceneNode {
  const l = node.yoga.getComputedLayout();
  node.layout = { x: l.left, y: l.top, width: l.width, height: l.height };
  const props: Record<string, unknown> = {};
  for (const [k, v] of Object.entries(node.props)) {
    if (k === "children" || k === "style") continue;
    props[k] = typeof v === "function" ? true : v; // functions are kept JS-side; flag only
  }
  return {
    id: node.id, type: node.type, text: node.text, style: node.style, props,
    layout: node.layout, children: node.children.map(toScene),
  };
}

export { create as createHostNode };
