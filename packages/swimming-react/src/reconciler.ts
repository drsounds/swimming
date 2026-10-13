import Reconciler from "react-reconciler";
import { DefaultEventPriority } from "react-reconciler/constants.js";

/** Native command API injected by swimming-bridge (crates/swimming-bridge). */
export interface SwimmingNative {
  createNode(tag: string, styleCss: string, propsJson: string): number;
  createText(text: string): number;
  setText(id: number, text: string): void;
  appendChild(parent: number, child: number): void;
  insertBefore(parent: number, child: number, before: number): void;
  removeChild(parent: number, child: number): void;
  updateNode(id: number, styleCss: string, propsJson: string): void;
  commit(): void;
}

export interface HostInstance {
  id: number;
  type: string;
  props: Record<string, unknown>;
  parent: HostInstance | null;
}
export interface Container extends HostInstance { native: SwimmingNative }

const kebab = (s: string) => s.replace(/[A-Z]/g, (c) => "-" + c.toLowerCase());
const UNITLESS = new Set(["flex", "flex-grow", "flex-shrink", "opacity", "z-index", "font-weight", "line-height", "order"]);

/** Serializes a style prop (object or CSS string) to inline CSS text for the Rust CSS engine. */
export function styleToCss(style: unknown): string {
  if (typeof style === "string") return style;
  if (!style || typeof style !== "object") return "";
  return Object.entries(style as Record<string, unknown>)
    .filter(([, v]) => v !== undefined && v !== null && v !== false)
    .map(([k, v]) => `${kebab(k)}:${typeof v === "number" && v !== 0 && !UNITLESS.has(kebab(k)) ? v + "px" : v}`)
    .join(";");
}

/** Props sent to Rust: everything except children/style; functions become `true` (handlers stay in JS). */
export function serializeProps(props: Record<string, unknown>): string {
  const out: Record<string, unknown> = {};
  for (const [k, v] of Object.entries(props)) {
    if (k === "children" || k === "style" || v === undefined) continue;
    if (typeof v === "function") out[k] = true;
    else if (v === null || ["string", "number", "boolean"].includes(typeof v)) out[k] = v;
  }
  return JSON.stringify(out);
}

const EVENT_PROPS: Record<string, string> = {
  click: "onClick", dblclick: "onDoubleClick", input: "onInput", change: "onChange", keydown: "onKeyDown",
  keyup: "onKeyUp", mouseenter: "onMouseEnter", mouseleave: "onMouseLeave", mousedown: "onMouseDown", mouseup: "onMouseUp",
};

const instances = new Map<number, HostInstance>();

/** Delivers a native event to JS handlers, bubbling like a DOM event. Installed as __SWIMMING_DISPATCH__. */
export function dispatchEvent(id: number, event: string, payload?: unknown) {
  let n = instances.get(id) ?? null;
  let stopped = false;
  const evt = { type: event, target: id, payload, stopPropagation: () => { stopped = true; }, preventDefault: () => {} };
  const prop = EVENT_PROPS[event] ?? "on" + event[0].toUpperCase() + event.slice(1);
  while (n && !stopped) {
    const h = n.props[prop];
    if (typeof h === "function") h(evt);
    n = n.parent;
  }
}

export function createReconciler(native: SwimmingNative) {
  const track = (i: HostInstance) => { instances.set(i.id, i); return i; };
  const insert = (parent: HostInstance, child: HostInstance, before?: HostInstance) => {
    child.parent = parent;
    if (before) native.insertBefore(parent.id, child.id, before.id);
    else native.appendChild(parent.id, child.id);
  };
  const remove = (parent: HostInstance, child: HostInstance) => {
    const drop = (i: HostInstance) => { instances.delete(i.id); };
    drop(child);
    child.parent = null;
    native.removeChild(parent.id, child.id);
  };

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const config: any = {
    supportsMutation: true, supportsPersistence: false, supportsHydration: false, isPrimaryRenderer: true,
    noTimeout: -1, scheduleTimeout: setTimeout, cancelTimeout: clearTimeout,
    createInstance: (type: string, props: Record<string, unknown>) =>
      track({ id: native.createNode(type, styleToCss(props.style), serializeProps(props)), type, props, parent: null }),
    createTextInstance: (text: string) => track({ id: native.createText(text), type: "#text", props: {}, parent: null }),
    appendInitialChild: insert, appendChild: insert, insertBefore: insert,
    appendChildToContainer: insert,
    insertInContainerBefore: insert,
    removeChild: remove, removeChildFromContainer: remove,
    commitUpdate: (i: HostInstance, _t: string, _o: unknown, props: Record<string, unknown>) => {
      i.props = props;
      native.updateNode(i.id, styleToCss(props.style), serializeProps(props));
    },
    commitTextUpdate: (i: HostInstance, _o: string, text: string) => native.setText(i.id, text),
    finalizeInitialChildren: () => false, prepareUpdate: () => true, shouldSetTextContent: () => false,
    getRootHostContext: () => ({}), getChildHostContext: (c: unknown) => c, getPublicInstance: (i: HostInstance) => i,
    prepareForCommit: () => null, resetAfterCommit: () => native.commit(),
    clearContainer: () => {}, preparePortalMount: () => {},
    getCurrentEventPriority: () => DefaultEventPriority,
    getInstanceFromNode: () => null, beforeActiveInstanceBlur: () => {}, afterActiveInstanceBlur: () => {},
    prepareScopeUpdate: () => {}, getInstanceFromScope: () => null, detachDeletedInstance: () => {},
  };
  return Reconciler(config);
}
