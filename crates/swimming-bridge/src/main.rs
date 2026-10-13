//! `swimming <bundle.js> [--out frame.png] [--width N] [--height N]`
//! Runs a Vite-built Swimming bundle and writes the first rendered frame as PNG.
use swimming_bridge::JsBridge;

fn main() {
    let mut args = std::env::args().skip(1);
    let (mut bundle, mut out, mut w, mut h) = (None, None, 800f32, 600f32);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--out" => out = args.next(),
            "--width" => w = args.next().and_then(|v| v.parse().ok()).unwrap_or(w),
            "--height" => h = args.next().and_then(|v| v.parse().ok()).unwrap_or(h),
            _ => bundle = Some(a),
        }
    }
    let Some(bundle) = bundle else { eprintln!("usage: swimming <bundle.js> [--out frame.png] [--width N] [--height N]"); std::process::exit(2) };
    let src = std::fs::read_to_string(&bundle).expect("cannot read bundle");
    let bridge = JsBridge::new(w, h).expect("js init");
    bridge.eval(&src).expect("bundle error");
    if let (Some(path), Some(pm)) = (out, bridge.render()) {
        pm.save_png(path).expect("png");
    }
}
