import { spawn, type ChildProcess } from "node:child_process";
import { createInterface } from "node:readline";
import type { Backend, VNode } from "../types.js";

/**
 * Spawns the Rust `swimming-native` engine and speaks JSON lines over stdio:
 * virtual trees go in, natively hit-tested events come back.
 */
export class NativeBackend implements Backend {
  private child: ChildProcess;
  private dispatch?: (nodeId: number, event: string, payload?: unknown) => void;

  constructor(binary: string, args: string[] = []) {
    this.child = spawn(binary, args, { stdio: ["pipe", "pipe", "inherit"] });
    createInterface({ input: this.child.stdout! }).on("line", (line) => {
      try {
        const m = JSON.parse(line);
        if (m.type === "event" && m.target != null) this.dispatch?.(m.target, m.event);
      } catch { /* ignore malformed lines */ }
    });
  }
  present(root: VNode) { this.send({ type: "tree", root }); }
  pointer(event: string, x: number, y: number) { this.send({ type: "pointer", event, x, y }); }
  resize(width: number, height: number) { this.send({ type: "resize", width, height }); }
  setEventDispatcher(d: (nodeId: number, event: string, payload?: unknown) => void) { this.dispatch = d; }
  close() { this.child.stdin!.end(); }
  private send(m: unknown) { this.child.stdin!.write(JSON.stringify(m) + "\n"); }
}
