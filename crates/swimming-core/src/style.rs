//! CSS parsing via lightningcss into a flat, inherited-aware computed style.
use lightningcss::printer::PrinterOptions;
use lightningcss::properties::Property;
use lightningcss::stylesheet::{ParserOptions, StyleAttribute};
use lightningcss::values::color::CssColor;
use std::collections::HashMap;

pub type Decls = HashMap<String, String>;

pub fn parse_decls(css: &str) -> Decls {
    let mut out = Decls::new();
    if let Ok(attr) = StyleAttribute::parse(css, ParserOptions::default()) {
        let all = attr.declarations.declarations.iter().chain(attr.declarations.important_declarations.iter());
        for p in all {
            if let Ok(v) = p.value_to_css_string(PrinterOptions::default()) {
                out.insert(p.property_id().name().to_string(), v);
            }
        }
    }
    out
}

pub fn parse_color(s: &str) -> Option<[u8; 4]> {
    let css = format!("color:{s}");
    let attr = StyleAttribute::parse(&css, ParserOptions::default()).ok()?;
    match attr.declarations.declarations.first()? {
        Property::Color(c) => match c.to_rgb().ok()? {
            CssColor::RGBA(c) => Some([c.red, c.green, c.blue, c.alpha]),
            _ => None,
        },
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Len { Auto, Px(f32), Pct(f32) }

pub fn parse_len(s: &str) -> Option<Len> {
    let s = s.trim();
    if s == "auto" { return Some(Len::Auto); }
    if let Some(p) = s.strip_suffix('%') { return p.trim().parse().ok().map(|v: f32| Len::Pct(v / 100.0)); }
    if let Some(p) = s.strip_suffix("px") { return p.trim().parse().ok().map(Len::Px); }
    if let Some(p) = s.strip_suffix("em") { return p.trim().parse().ok().map(|v: f32| Len::Px(v * 16.0)); }
    if let Some(p) = s.strip_suffix("rem") { return p.trim().parse().ok().map(|v: f32| Len::Px(v * 16.0)); }
    if s == "0" { return Some(Len::Px(0.0)); }
    s.parse().ok().map(Len::Px)
}

/// Expands 1-4 value box shorthands (margin/padding/border-width) into top,right,bottom,left.
pub fn expand_box(v: &str) -> Option<[Len; 4]> {
    let p: Vec<Len> = v.split_whitespace().filter_map(parse_len).collect();
    Some(match p.len() {
        1 => [p[0], p[0], p[0], p[0]],
        2 => [p[0], p[1], p[0], p[1]],
        3 => [p[0], p[1], p[2], p[1]],
        4 => [p[0], p[1], p[2], p[3]],
        _ => return None,
    })
}

/// Merge: later maps win. Shorthands are expanded into longhands first.
pub fn merge(dst: &mut Decls, src: Decls) {
    for (k, v) in src {
        if matches!(k.as_str(), "margin" | "padding" | "border-width") {
            if let Some(b) = expand_box(&v) {
                let base = k.clone();
                let suffix = if base == "border-width" { "border-{}-width" } else { "" };
                for (i, side) in ["top", "right", "bottom", "left"].iter().enumerate() {
                    let key = if suffix.is_empty() { format!("{base}-{side}") } else { format!("border-{side}-width") };
                    let val = match b[i] { Len::Px(x) => format!("{x}px"), Len::Pct(x) => format!("{}%", x * 100.0), Len::Auto => "auto".into() };
                    dst.insert(key, val);
                }
                continue;
            }
        }
        dst.insert(k, v);
    }
}
