//! Software painter (tiny-skia). Draws backgrounds, borders and rounded corners;
//! glyph rasterization is a TODO (text boxes are laid out but not yet drawn).
use crate::layout::{Box, LaidOut};
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Stroke, Transform};

fn rrect(x: f32, y: f32, w: f32, h: f32, r: f32) -> Option<tiny_skia::Path> {
    let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
    let mut pb = PathBuilder::new();
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.quad_to(x + w, y, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.quad_to(x + w, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.quad_to(x, y + h, x, y + h - r);
    pb.line_to(x, y + r);
    pb.quad_to(x, y, x + r, y);
    pb.close();
    pb.finish()
}

fn paint_box(pm: &mut Pixmap, b: &Box, ox: f32, oy: f32) {
    let (x, y) = (ox + b.x, oy + b.y);
    let c = &b.computed;
    if b.id.is_some() && b.w > 0.0 && b.h > 0.0 {
        if let Some(path) = rrect(x, y, b.w, b.h, c.radius) {
            if let Some(bg) = c.background {
                let mut p = Paint::default();
                p.set_color(Color::from_rgba8(bg[0], bg[1], bg[2], (bg[3] as f32 * c.opacity) as u8));
                p.anti_alias = true;
                pm.fill_path(&path, &p, FillRule::Winding, Transform::identity(), None);
            }
            if c.border_width > 0.0 {
                let mut p = Paint::default();
                let bc = c.border_color;
                p.set_color(Color::from_rgba8(bc[0], bc[1], bc[2], bc[3]));
                let s = Stroke { width: c.border_width, ..Default::default() };
                pm.stroke_path(&path, &p, &s, Transform::identity(), None);
            }
        }
    }
    for ch in &b.children { paint_box(pm, ch, x, y); }
}

pub fn paint(l: &LaidOut, width: u32, height: u32) -> Option<Pixmap> {
    let mut pm = Pixmap::new(width.max(1), height.max(1))?;
    pm.fill(Color::WHITE);
    paint_box(&mut pm, &l.root, 0.0, 0.0);
    Some(pm)
}
