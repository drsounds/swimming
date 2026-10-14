// Browser polyfills for the embedded JS engine (no DOM): window, location, history,
// timers, queueMicrotask, performance. Lets React and routers like React Router load unchanged.
type Listener = (ev: unknown) => void;

class MemoryHistory {
  private stack: string[] = ["/"];
  private index = 0;
  private states: unknown[] = [null];
  private listeners = new Set<() => void>();

  get location() {
    const url = new URL(this.stack[this.index], "http://swimming.local");
    return {
      pathname: url.pathname, search: url.search, hash: url.hash, href: url.href,
      origin: url.origin, host: url.host, hostname: url.hostname, protocol: url.protocol,
    };
  }
  get state() { return this.states[this.index]; }
  get length() { return this.stack.length; }

  pushState(state: unknown, _unused: string, url?: string | null) {
    if (url == null) return;
    this.stack = this.stack.slice(0, this.index + 1).concat(url);
    this.states = this.states.slice(0, this.index + 1).concat([state]);
    this.index++;
    this.notify();
  }
  replaceState(state: unknown, _unused: string, url?: string | null) {
    if (url != null) this.stack[this.index] = url;
    this.states[this.index] = state;
    this.notify();
  }
  go(delta: number) {
    const i = this.index + delta;
    if (i < 0 || i >= this.stack.length) return;
    this.index = i;
    this.notify(true);
  }
  listen(fn: () => void) {
    this.listeners.add(fn);
    return () => { this.listeners.delete(fn); };
  }
  private notify(pop = false) {
    this.listeners.forEach((fn) => fn());
    if (pop) g.dispatchEvent?.({ type: "popstate", state: this.state });
  }
}

export const memoryHistory = new MemoryHistory();

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const g = globalThis as any;

// --- timers: run by the native host via __SWIMMING_TICK__ (QuickJS has no event loop) ---
const timers: { id: number; at: number; fn: () => void; every?: number }[] = [];
let timerId = 1;
let now = 0; // virtual clock advanced by the host
if (!g.setTimeout) {
  g.setTimeout = (fn: (...a: unknown[]) => void, ms = 0, ...args: unknown[]) => {
    const id = timerId++;
    timers.push({ id, at: now + Math.max(0, ms), fn: () => fn(...args) });
    return id;
  };
  g.clearTimeout = (id: number) => { const i = timers.findIndex((t) => t.id === id); if (i >= 0) timers.splice(i, 1); };
  g.setInterval = (fn: () => void, ms = 0) => {
    const id = timerId++;
    timers.push({ id, at: now + Math.max(1, ms), fn, every: Math.max(1, ms) });
    return id;
  };
  g.clearInterval = g.clearTimeout;
}
if (!g.queueMicrotask) g.queueMicrotask = (fn: () => void) => { Promise.resolve().then(fn); };
if (!g.performance) g.performance = { now: () => now };
if (!g.console) g.console = { log() {}, info() {}, warn() {}, error() {}, debug() {} };

/** Runs the next due timer; returns true if more work is pending. Called repeatedly by the host. */
g.__SWIMMING_TICK__ = (advanceMs = 0): boolean => {
  now += advanceMs;
  timers.sort((a, b) => a.at - b.at);
  const t = timers.find((x) => x.at <= now) ?? (advanceMs === 0 ? timers.find((x) => !x.every) : undefined);
  if (t) {
    if (t.at > now) now = t.at;
    if (t.every) t.at = now + t.every; else timers.splice(timers.indexOf(t), 1);
    t.fn();
  }
  return timers.some((x) => !x.every);
};

// --- window / document / location / history ---
if (typeof g.window === "undefined") {
  const listeners: Record<string, Set<Listener>> = {};
  g.window = g;
  g.self = g;
  g.addEventListener = (type: string, fn: Listener) => { (listeners[type] ??= new Set()).add(fn); };
  g.removeEventListener = (type: string, fn: Listener) => { listeners[type]?.delete(fn); };
  g.dispatchEvent = (ev: { type: string }) => { listeners[ev.type]?.forEach((fn) => fn(ev)); return true; };
  Object.defineProperty(g, "location", { get: () => memoryHistory.location, configurable: true });
  g.history = {
    pushState: (s: unknown, u: string, url?: string | null) => memoryHistory.pushState(s, u, url),
    replaceState: (s: unknown, u: string, url?: string | null) => memoryHistory.replaceState(s, u, url),
    back: () => memoryHistory.go(-1),
    forward: () => memoryHistory.go(1),
    go: (d: number) => memoryHistory.go(d),
    get state() { return memoryHistory.state; },
    get length() { return memoryHistory.length; },
  };
  g.navigator = g.navigator ?? { userAgent: "Swimming", platform: "swimming" };
  g.document = g.document ?? {
    // Inert stand-ins so DOM-touching utility libraries load; there is no DOM tree.
    createElement: (tag: string) => ({ tagName: tag.toUpperCase(), style: {}, setAttribute() {}, appendChild() {}, addEventListener() {} }),
    addEventListener: g.addEventListener,
    removeEventListener: g.removeEventListener,
    documentElement: { style: {} },
    body: { style: {}, appendChild() {} },
    head: { appendChild() {} },
    get location() { return memoryHistory.location; },
  };
}
