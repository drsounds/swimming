//! Virtual tree -> computed styles -> taffy layout. No DOM is built.
use crate::html::{default_display, ua_style, DefaultDisplay};
use crate::style::*;
use crate::VNode;
use serde_json::{json, Value};
use taffy::prelude::*;

#[derive(Debug, Clone)]
pub struct Computed {
    pub color: [u8; 4],
    pub font_size: f32,
    pub bold: bool,
    pub background: Option<[u8; 4]>,
    pub border_color: [u8; 4],
    pub border_width: f32,
    pub radius: f32,
    pub opacity: f32,
}

#[derive(Debug, Clone)]
pub struct Box {
    pub id: Option<u64>,
    pub tag: String,
    pub text: Option<String>,
    pub computed: Computed,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub children: Vec<Box>,
}

pub struct LaidOut { pub root: Box }

impl LaidOut {
    pub fn summary(&self) -> Value {
        fn go(b: &Box) -> Value {
            json!({"id": b.id, "type": b.tag, "x": b.x, "y": b.y, "width": b.w, "height": b.h,
                   "children": b.children.iter().map(go).collect::<Vec<_>>()})
        }
        go(&self.root)
    }

    /// Deepest real element containing the point; ids of text/anonymous boxes resolve to None.
    pub fn hit_test(&self, x: f32, y: f32) -> Option<u64> {
        fn go(b: &Box, x: f32, y: f32, ox: f32, oy: f32) -> Option<u64> {
            let (bx, by) = (ox + b.x, oy + b.y);
            if x < bx || y < by || x > bx + b.w || y > by + b.h { return None; }
            for c in b.children.iter().rev() {
                if let Some(id) = go(c, x, y, bx, by) { return Some(id); }
            }
            b.id
        }
        go(&self.root, x, y, 0.0, 0.0)
    }
}

struct Built {
    node: NodeId,
    tmpl: Tmpl,
}
struct Tmpl {
    id: Option<u64>,
    tag: String,
    text: Option<String>,
    computed: Computed,
    children: Vec<Built>,
}

fn dim(l: Len) -> Dimension {
    match l { Len::Auto => Dimension::auto(), Len::Px(v) => Dimension::length(v), Len::Pct(v) => Dimension::percent(v) }
}
fn lpa(l: Len) -> LengthPercentageAuto {
    match l { Len::Auto => LengthPercentageAuto::auto(), Len::Px(v) => LengthPercentageAuto::length(v), Len::Pct(v) => LengthPercentageAuto::percent(v) }
}
fn lp(l: Len) -> LengthPercentage {
    match l { Len::Px(v) => LengthPercentage::length(v), Len::Pct(v) => LengthPercentage::percent(v), Len::Auto => LengthPercentage::length(0.0) }
}

fn get_len(d: &Decls, k: &str) -> Option<Len> { d.get(k).and_then(|v| parse_len(v)) }

fn text_size(text: &str, fs: f32) -> (f32, f32) {
    // No font stack yet: approximate advance of 0.55em per char.
    let lines: Vec<&str> = text.split('\n').collect();
    let w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as f32 * fs * 0.55;
    (w, lines.len() as f32 * fs * 1.2)
}

fn is_inline(v: &VNode, d: &Decls) -> bool {
    if v.kind == "#text" { return true; }
    match d.get("display").map(String::as_str) {
        Some("inline") | Some("inline-block") | Some("inline-flex") => true,
        Some(_) => false,
        None => default_display(&v.kind) == DefaultDisplay::Inline,
    }
}

fn computed_decls(v: &VNode) -> Decls {
    let mut d = parse_decls(ua_style(&v.kind));
    merge(&mut d, parse_decls(&v.style));
    d
}

fn build(tree: &mut TaffyTree<()>, v: &VNode, parent: &Computed) -> Option<Built> {
    if v.kind == "#text" {
        let text = v.text.clone().unwrap_or_default();
        let (w, h) = text_size(&text, parent.font_size);
        let node = tree.new_leaf(Style { size: Size { width: Dimension::length(w), height: Dimension::length(h) }, flex_shrink: 0.0, ..Default::default() }).ok()?;
        return Some(Built { node, tmpl: Tmpl { id: None, tag: "#text".into(), text: Some(text), computed: parent.clone(), children: vec![] } });
    }
    let d = computed_decls(v);
    let hidden = d.get("display").map(|s| s == "none").unwrap_or(default_display(&v.kind) == DefaultDisplay::None);
    if hidden { return None; }

    let color = d.get("color").and_then(|c| parse_color(c)).unwrap_or(parent.color);
    let font_size = get_len(&d, "font-size").and_then(|l| if let Len::Px(p) = l { Some(p) } else { None }).unwrap_or(parent.font_size);
    let bold = d.get("font-weight").map(|w| w == "bold" || w.parse::<u32>().map(|n| n >= 600).unwrap_or(false)).unwrap_or(parent.bold);
    let computed = Computed {
        color, font_size, bold,
        background: d.get("background-color").and_then(|c| parse_color(c)),
        border_color: d.get("border-color").or(d.get("border-top-color")).and_then(|c| parse_color(c)).unwrap_or(color),
        border_width: get_len(&d, "border-top-width").and_then(|l| if let Len::Px(p) = l { Some(p) } else { None }).unwrap_or(0.0),
        radius: get_len(&d, "border-radius").and_then(|l| if let Len::Px(p) = l { Some(p) } else { None }).unwrap_or(0.0),
        opacity: d.get("opacity").and_then(|o| o.parse().ok()).unwrap_or(1.0),
    };

    // Children: group consecutive inline-level runs into anonymous wrapping rows.
    let mut kids: Vec<Built> = vec![];
    let mut run: Vec<Built> = vec![];
    let flush = |tree: &mut TaffyTree<()>, run: &mut Vec<Built>, kids: &mut Vec<Built>, c: &Computed| {
        if run.is_empty() { return; }
        let ids: Vec<NodeId> = run.iter().map(|b| b.node).collect();
        let node = tree.new_with_children(Style { display: Display::Flex, flex_wrap: FlexWrap::Wrap, align_items: AlignItems::FLEX_START, flex_shrink: 0.0, ..Default::default() }, &ids).unwrap();
        kids.push(Built { node, tmpl: Tmpl { id: None, tag: "#anon".into(), text: None, computed: c.clone(), children: std::mem::take(run) } });
    };
    let flex_parent = matches!(d.get("display").map(String::as_str), Some("flex") | Some("inline-flex"));
    for c in &v.children {
        let cd = if c.kind == "#text" { Decls::new() } else { computed_decls(c) };
        let inline = !flex_parent && is_inline(c, &cd);
        if let Some(b) = build(tree, c, &computed) {
            if inline { run.push(b) } else { flush(tree, &mut run, &mut kids, &computed); kids.push(b) }
        }
    }
    flush(tree, &mut run, &mut kids, &computed);

    let m = |side: &str| get_len(&d, &format!("margin-{side}")).unwrap_or(Len::Px(0.0));
    let p = |side: &str| get_len(&d, &format!("padding-{side}")).unwrap_or(Len::Px(0.0));
    let b = |side: &str| get_len(&d, &format!("border-{side}-width")).unwrap_or(Len::Px(computed.border_width));
    let disp = d.get("display").map(String::as_str).unwrap_or("block");
    let dir = match d.get("flex-direction").map(String::as_str) {
        Some("row") => FlexDirection::Row, Some("row-reverse") => FlexDirection::RowReverse,
        Some("column-reverse") => FlexDirection::ColumnReverse, _ => FlexDirection::Column,
    };
    let jc = d.get("justify-content").and_then(|s| match s.as_str() {
        "flex-start" | "start" => Some(JustifyContent::FLEX_START), "flex-end" | "end" => Some(JustifyContent::FLEX_END),
        "center" => Some(JustifyContent::CENTER), "space-between" => Some(JustifyContent::SPACE_BETWEEN),
        "space-around" => Some(JustifyContent::SPACE_AROUND), "space-evenly" => Some(JustifyContent::SPACE_EVENLY), _ => None });
    let align = |s: &str| match s {
        "flex-start" | "start" => Some(AlignItems::FLEX_START), "flex-end" | "end" => Some(AlignItems::FLEX_END),
        "center" => Some(AlignItems::CENTER), "stretch" => Some(AlignItems::STRETCH), "baseline" => Some(AlignItems::BASELINE), _ => None };
    let gap = get_len(&d, "gap").map(lp).unwrap_or(LengthPercentage::length(0.0));
    let mut grow = d.get("flex-grow").and_then(|s| s.parse().ok());
    let mut shrink = d.get("flex-shrink").and_then(|s| s.parse().ok());
    let mut basis = get_len(&d, "flex-basis");
    if let Some(f) = d.get("flex") {
        let parts: Vec<&str> = f.split_whitespace().collect();
        if let Some(g) = parts.first().and_then(|s| s.parse::<f32>().ok()) { grow = Some(g); if parts.len() == 1 { shrink = Some(1.0); basis = Some(Len::Pct(0.0)); } }
        if let Some(s) = parts.get(1).and_then(|s| s.parse::<f32>().ok()) { shrink = Some(s); }
        if let Some(bs) = parts.get(2).and_then(|s| parse_len(s)) { basis = Some(bs); }
    }
    let style = Style {
        display: if matches!(disp, "flex" | "inline-flex") { Display::Flex } else { Display::Flex },
        flex_direction: dir,
        flex_wrap: if d.get("flex-wrap").map(|s| s == "wrap").unwrap_or(false) { FlexWrap::Wrap } else { FlexWrap::NoWrap },
        justify_content: jc.unwrap_or(JustifyContent::NORMAL),
        align_items: d.get("align-items").and_then(|s| align(s)).unwrap_or(AlignItems::STRETCH),
        align_self: d.get("align-self").and_then(|s| align(s)),
        flex_grow: grow.unwrap_or(0.0),
        flex_shrink: shrink.unwrap_or(1.0),
        flex_basis: basis.map(dim).unwrap_or(Dimension::auto()),
        position: if d.get("position").map(|s| s == "absolute").unwrap_or(false) { Position::Absolute } else { Position::Relative },
        inset: Rect { top: lpa(get_len(&d, "top").unwrap_or(Len::Auto)), right: lpa(get_len(&d, "right").unwrap_or(Len::Auto)),
                      bottom: lpa(get_len(&d, "bottom").unwrap_or(Len::Auto)), left: lpa(get_len(&d, "left").unwrap_or(Len::Auto)) },
        size: Size { width: dim(get_len(&d, "width").unwrap_or(Len::Auto)), height: dim(get_len(&d, "height").unwrap_or(Len::Auto)) },
        min_size: Size { width: lpa(get_len(&d, "min-width").unwrap_or(Len::Auto)), height: lpa(get_len(&d, "min-height").unwrap_or(Len::Auto)) },
        max_size: Size { width: lpa(get_len(&d, "max-width").unwrap_or(Len::Auto)), height: lpa(get_len(&d, "max-height").unwrap_or(Len::Auto)) },
        margin: Rect { top: lpa(m("top")), right: lpa(m("right")), bottom: lpa(m("bottom")), left: lpa(m("left")) },
        padding: Rect { top: lp(p("top")), right: lp(p("right")), bottom: lp(p("bottom")), left: lp(p("left")) },
        border: Rect { top: lp(b("top")), right: lp(b("right")), bottom: lp(b("bottom")), left: lp(b("left")) },
        gap: Size { width: gap, height: gap },
        ..Default::default()
    };
    let ids: Vec<NodeId> = kids.iter().map(|k| k.node).collect();
    let node = tree.new_with_children(style, &ids).ok()?;
    Some(Built { node, tmpl: Tmpl { id: Some(v.id), tag: v.kind.clone(), text: None, computed, children: kids } })
}

fn collect(tree: &TaffyTree<()>, b: Built) -> Box {
    let l = tree.layout(b.node).unwrap();
    Box {
        id: b.tmpl.id, tag: b.tmpl.tag, text: b.tmpl.text, computed: b.tmpl.computed,
        x: l.location.x, y: l.location.y, w: l.size.width, h: l.size.height,
        children: b.tmpl.children.into_iter().map(|c| collect(tree, c)).collect(),
    }
}

pub fn layout(root: &VNode, width: f32, height: f32) -> LaidOut {
    let mut tree: TaffyTree<()> = TaffyTree::new();
    let base = Computed { color: [0, 0, 0, 255], font_size: 16.0, bold: false, background: None, border_color: [0, 0, 0, 255], border_width: 0.0, radius: 0.0, opacity: 1.0 };
    let mut wrapper = root.clone();
    wrapper.style = format!("width:{width}px;height:{height}px;{}", root.style);
    let built = build(&mut tree, &wrapper, &base).expect("root must be visible");
    tree.compute_layout(built.node, Size { width: AvailableSpace::Definite(width), height: AvailableSpace::Definite(height) }).unwrap();
    LaidOut { root: collect(&tree, built) }
}
