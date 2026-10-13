//! swimming-native: browser-engine-free renderer. Reads React virtual-tree
//! frames as JSON lines on stdin, resolves CSS (lightningcss), lays out (taffy),
//! paints (tiny-skia) and reports events/layout as JSON lines on stdout.
use std::io::{BufRead, Write};
use swimming_native::{Engine, Message};

fn main() {
    let mut out_png: Option<String> = None;
    let (mut w, mut h) = (800u32, 600u32);
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--out" => out_png = args.next(),
            "--width" => w = args.next().and_then(|v| v.parse().ok()).unwrap_or(w),
            "--height" => h = args.next().and_then(|v| v.parse().ok()).unwrap_or(h),
            _ => {}
        }
    }
    let mut engine = Engine::new(w, h);
    let stdout = std::io::stdout();
    for line in std::io::stdin().lock().lines() {
        let Ok(line) = line else { break };
        let Ok(msg) = serde_json::from_str::<Message>(&line) else { continue };
        let reply = engine.handle(msg);
        if let (Some(path), true) = (&out_png, reply.frame) {
            if let Some(pm) = engine.render() {
                let _ = pm.save_png(path);
            }
        }
        let mut o = stdout.lock();
        let _ = writeln!(o, "{}", serde_json::to_string(&reply.json).unwrap());
        let _ = o.flush();
    }
}
