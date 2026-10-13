import Yoga, { Node, Edge, Gutter } from "yoga-layout";
import type { Style } from "./types.js";

const kebab = (s: string) => s.replace(/[A-Z]/g, (c) => "-" + c.toLowerCase());

/** Normalizes camelCase/kebab-case keys and expands shorthands. */
export function normalizeStyle(style: unknown): Style {
  const out: Style = {};
  if (!style || typeof style !== "object") return out;
  for (const [k, v] of Object.entries(style as Style)) {
    if (v !== undefined) out[kebab(k)] = v;
  }
  return out;
}

/** Parses a CSS text such as "width: 10px; flex-direction: row" to a Style. */
export function parseCss(text: string): Style {
  const out: Style = {};
  for (const decl of text.split(";")) {
    const i = decl.indexOf(":");
    if (i < 0) continue;
    const key = decl.slice(0, i).trim();
    const val = decl.slice(i + 1).trim();
    if (key) out[kebab(key)] = val;
  }
  return out;
}

type Dim = number | `${number}%` | "auto" | undefined;
function dim(v: string | number | undefined): Dim {
  if (v === undefined) return undefined;
  if (typeof v === "number") return v;
  const s = v.trim();
  if (s === "auto") return "auto";
  if (s.endsWith("%")) return s as Dim;
  const n = parseFloat(s);
  return Number.isNaN(n) ? undefined : n;
}

const num = (v: string | number | undefined) => {
  const d = dim(v);
  return typeof d === "number" ? d : undefined;
};

const FLEX_DIR = { row: Yoga.FLEX_DIRECTION_ROW, column: Yoga.FLEX_DIRECTION_COLUMN, "row-reverse": Yoga.FLEX_DIRECTION_ROW_REVERSE, "column-reverse": Yoga.FLEX_DIRECTION_COLUMN_REVERSE } as const;
const JUSTIFY = { "flex-start": Yoga.JUSTIFY_FLEX_START, center: Yoga.JUSTIFY_CENTER, "flex-end": Yoga.JUSTIFY_FLEX_END, "space-between": Yoga.JUSTIFY_SPACE_BETWEEN, "space-around": Yoga.JUSTIFY_SPACE_AROUND, "space-evenly": Yoga.JUSTIFY_SPACE_EVENLY } as const;
const ALIGN = { auto: Yoga.ALIGN_AUTO, "flex-start": Yoga.ALIGN_FLEX_START, center: Yoga.ALIGN_CENTER, "flex-end": Yoga.ALIGN_FLEX_END, stretch: Yoga.ALIGN_STRETCH, baseline: Yoga.ALIGN_BASELINE } as const;

const EDGES: [string, Edge][] = [["top", Yoga.EDGE_TOP], ["right", Yoga.EDGE_RIGHT], ["bottom", Yoga.EDGE_BOTTOM], ["left", Yoga.EDGE_LEFT]];

/** Applies a normalized CSS style to a Yoga node. */
export function applyStyle(node: Node, style: Style): void {
  const s = (k: string) => style[k];
  const w = dim(s("width")), h = dim(s("height"));
  node.setWidth(w); node.setHeight(h);
  node.setMinWidth(dim(s("min-width")) as number | `${number}%` | undefined);
  node.setMinHeight(dim(s("min-height")) as number | `${number}%` | undefined);
  node.setMaxWidth(dim(s("max-width")) as number | `${number}%` | undefined);
  node.setMaxHeight(dim(s("max-height")) as number | `${number}%` | undefined);

  const fd = String(s("flex-direction") ?? "");
  // CSS default is row for web, but html divs are block (column) – use column unless told otherwise.
  node.setFlexDirection(FLEX_DIR[fd as keyof typeof FLEX_DIR] ?? Yoga.FLEX_DIRECTION_COLUMN);
  const jc = JUSTIFY[String(s("justify-content")) as keyof typeof JUSTIFY];
  if (jc !== undefined) node.setJustifyContent(jc);
  const ai = ALIGN[String(s("align-items")) as keyof typeof ALIGN];
  if (ai !== undefined) node.setAlignItems(ai);
  const as = ALIGN[String(s("align-self")) as keyof typeof ALIGN];
  if (as !== undefined) node.setAlignSelf(as);
  node.setFlexWrap(s("flex-wrap") === "wrap" ? Yoga.WRAP_WRAP : Yoga.WRAP_NO_WRAP);
  node.setFlexGrow(num(s("flex-grow")) ?? num(s("flex")) ?? 0);
  node.setFlexShrink(num(s("flex-shrink")) ?? 1);
  const fb = dim(s("flex-basis"));
  if (fb !== undefined) node.setFlexBasis(fb);
  node.setPositionType(s("position") === "absolute" ? Yoga.POSITION_TYPE_ABSOLUTE : Yoga.POSITION_TYPE_RELATIVE);
  node.setDisplay(s("display") === "none" ? Yoga.DISPLAY_NONE : Yoga.DISPLAY_FLEX);

  for (const [name, edge] of EDGES) {
    const m = dim(s("margin-" + name) ?? s("margin"));
    node.setMargin(edge, m as number | `${number}%` | "auto" | undefined);
    node.setPadding(edge, num(s("padding-" + name) ?? s("padding")));
    node.setBorder(edge, num(s("border-" + name + "-width") ?? s("border-width")));
    node.setPosition(edge, dim(s(name)) as number | `${number}%` | undefined);
  }
  const gap = num(s("gap"));
  if (gap !== undefined) node.setGap(Gutter.All, gap);
}
