pub mod html;
mod style;
mod layout;
mod paint;

use serde::Deserialize;
use serde_json::{json, Value};

pub use layout::LaidOut;

/// A node of React's virtual tree as emitted by the JS reconciler.
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
    pub props: serde_json::Map<String, Value>,
    #[serde(default)]
    pub children: Vec<VNode>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Message {
    /// A new frame (whole virtual tree).
    Tree { root: VNode },
    /// A pointer event at viewport coordinates; hit-tested natively.
    Pointer { event: String, x: f32, y: f32 },
    Resize { width: u32, height: u32 },
}

pub struct Reply {
    pub json: Value,
    pub frame: bool,
}

pub struct Engine {
    pub width: u32,
    pub height: u32,
    pub laid_out: Option<LaidOut>,
}

impl Engine {
    pub fn new(width: u32, height: u32) -> Self {
        Engine { width, height, laid_out: None }
    }

    pub fn relayout(&mut self, root: &VNode) {
        self.laid_out = Some(layout::layout(root, self.width as f32, self.height as f32));
    }

    pub fn handle(&mut self, msg: Message) -> Reply {
        match msg {
            Message::Tree { root } => {
                self.relayout(&root);
                let l = self.laid_out.as_ref().unwrap();
                Reply { json: json!({"type": "layout", "boxes": l.summary()}), frame: true }
            }
            Message::Resize { width, height } => {
                self.width = width;
                self.height = height;
                Reply { json: json!({"type": "resized"}), frame: false }
            }
            Message::Pointer { event, x, y } => {
                let target = self.laid_out.as_ref().and_then(|l| l.hit_test(x, y));
                Reply { json: json!({"type": "event", "event": event, "target": target}), frame: false }
            }
        }
    }

    pub fn render(&self) -> Option<tiny_skia::Pixmap> {
        paint::paint(self.laid_out.as_ref()?, self.width, self.height)
    }
}
