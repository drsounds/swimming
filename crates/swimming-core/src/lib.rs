//! swimming-core: the virtual shadow tree and native layout engine.
//!
//! The React reconciler drives this through a small command API (create / append /
//! insert / remove / update). There is no DOM: CSS is resolved with lightningcss
//! and laid out with Taffy directly from the shadow tree.
pub mod html;
pub mod layout;
pub mod style;

use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::HashMap;

pub use layout::{Box as LayoutBox, Computed, LaidOut};

/// A node of React's virtual tree.
#[derive(Debug, Clone, Deserialize)]
pub struct VNode {
    pub id: u64,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub text: Option<String>,
    /// Inline CSS text, e.g. "width: 10px; color: red".
    #[serde(default)]
    pub style: String,
    #[serde(default)]
    pub props: Map<String, Value>,
    #[serde(default)]
    pub children: Vec<VNode>,
}

struct Node {
    kind: String,
    text: Option<String>,
    style: String,
    props: Map<String, Value>,
    children: Vec<u64>,
    parent: Option<u64>,
}

/// Id of the implicit root container.
pub const ROOT: u64 = 0;

pub struct SwimmingEngine {
    nodes: HashMap<u64, Node>,
    next_id: u64,
    pub width: f32,
    pub height: f32,
    laid_out: Option<LaidOut>,
}

impl Default for SwimmingEngine {
    fn default() -> Self { Self::new() }
}

impl SwimmingEngine {
    pub fn new() -> Self {
        let mut nodes = HashMap::new();
        nodes.insert(ROOT, Node { kind: "root".into(), text: None, style: String::new(), props: Map::new(), children: vec![], parent: None });
        SwimmingEngine { nodes, next_id: 1, width: 800.0, height: 600.0, laid_out: None }
    }

    fn alloc(&mut self, node: Node) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.insert(id, node);
        id
    }

    pub fn create_node(&mut self, tag: &str, style_css: &str, props_json: &str) -> u64 {
        let props = serde_json::from_str(props_json).unwrap_or_default();
        self.alloc(Node { kind: tag.into(), text: None, style: style_css.into(), props, children: vec![], parent: None })
    }

    pub fn create_text(&mut self, text: &str) -> u64 {
        self.alloc(Node { kind: "#text".into(), text: Some(text.into()), style: String::new(), props: Map::new(), children: vec![], parent: None })
    }

    pub fn set_text(&mut self, id: u64, text: &str) {
        if let Some(n) = self.nodes.get_mut(&id) { n.text = Some(text.into()); }
    }

    pub fn update_node(&mut self, id: u64, style_css: &str, props_json: &str) {
        if let Some(n) = self.nodes.get_mut(&id) {
            n.style = style_css.into();
            n.props = serde_json::from_str(props_json).unwrap_or_default();
        }
    }

    pub fn append_child(&mut self, parent: u64, child: u64) { self.insert_before(parent, child, None); }

    /// Inserts `child` into `parent` before `before` (or at the end), detaching it from any old parent.
    pub fn insert_before(&mut self, parent: u64, child: u64, before: Option<u64>) {
        if !self.nodes.contains_key(&parent) || !self.nodes.contains_key(&child) || parent == child { return; }
        self.detach(child);
        let p = self.nodes.get_mut(&parent).unwrap();
        let idx = before.and_then(|b| p.children.iter().position(|&c| c == b)).unwrap_or(p.children.len());
        p.children.insert(idx, child);
        self.nodes.get_mut(&child).unwrap().parent = Some(parent);
    }

    fn detach(&mut self, child: u64) {
        if let Some(old) = self.nodes.get(&child).and_then(|n| n.parent) {
            if let Some(o) = self.nodes.get_mut(&old) { o.children.retain(|&c| c != child); }
        }
        if let Some(n) = self.nodes.get_mut(&child) { n.parent = None; }
    }

    /// Removes `child` from `parent` and frees the whole subtree.
    pub fn remove_child(&mut self, parent: u64, child: u64) {
        if self.nodes.get(&child).and_then(|n| n.parent) != Some(parent) { return; }
        self.detach(child);
        let mut stack = vec![child];
        while let Some(id) = stack.pop() {
            if let Some(n) = self.nodes.remove(&id) { stack.extend(n.children); }
        }
    }

    pub fn node_count(&self) -> usize { self.nodes.len() }

    pub fn to_vnode(&self, id: u64) -> Option<VNode> {
        let n = self.nodes.get(&id)?;
        Some(VNode {
            id, kind: n.kind.clone(), text: n.text.clone(), style: n.style.clone(), props: n.props.clone(),
            children: n.children.iter().filter_map(|&c| self.to_vnode(c)).collect(),
        })
    }

    pub fn compute_layout(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
        let root = self.to_vnode(ROOT).unwrap();
        self.laid_out = Some(layout::layout(&root, width, height));
    }

    pub fn layout(&self) -> Option<&LaidOut> { self.laid_out.as_ref() }

    /// Deepest element under the point (never the root container).
    pub fn hit_test(&self, x: f32, y: f32) -> Option<u64> {
        self.laid_out.as_ref()?.hit_test(x, y).filter(|&id| id != ROOT)
    }

    /// Whether the element has an `on<Event>` handler flagged by the JS side.
    pub fn has_handler(&self, id: u64, prop: &str) -> bool {
        self.nodes.get(&id).map(|n| n.props.get(prop).map(|v| v == &Value::Bool(true)).unwrap_or(false)).unwrap_or(false)
    }

    pub fn parent(&self, id: u64) -> Option<u64> { self.nodes.get(&id).and_then(|n| n.parent) }
}
