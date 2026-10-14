use swimming_bridge::{
    JsBridge,
    gui::{EguiBackend, GuiBackend},
    loader::run_source,
};

fn main() {
    let mut args = std::env::args().skip(1);
    let (mut source, mut out, mut width, mut height, mut backend) =
        (None, None, 800.0f32, 600.0f32, "egui".to_owned());
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" => out = args.next(),
            "--width" => width = args.next().and_then(|v| v.parse().ok()).unwrap_or(width),
            "--height" => height = args.next().and_then(|v| v.parse().ok()).unwrap_or(height),
            "--backend" => backend = args.next().unwrap_or(backend),
            _ => source = Some(arg),
        }
    }
    let Some(source) = source else {
        eprintln!(
            "usage: swimming <bundle.js|index.html|URL> [--backend egui|fastframe] [--out frame.png] [--width N] [--height N]"
        );
        std::process::exit(2);
    };

    let bridge = JsBridge::new(width, height).unwrap_or_else(|error| {
        eprintln!("cannot initialize JavaScript runtime: {error}");
        std::process::exit(1);
    });
    if let Err(error) = run_source(&bridge, &source) {
        eprintln!("cannot load app: {error}");
        std::process::exit(1);
    }

    if let Some(path) = out {
        let Some(pixmap) = bridge.render() else {
            eprintln!("cannot render app");
            std::process::exit(1);
        };
        if let Err(error) = pixmap.save_png(path) {
            eprintln!("cannot write PNG: {error}");
            std::process::exit(1);
        }
        return;
    }

    let result = match backend.as_str() {
        "egui" => EguiBackend.run(bridge, "Swimming"),
        #[cfg(feature = "fastframe")]
        "fastframe" => swimming_bridge::gui::FastframeBackend.run(bridge, "Swimming"),
        #[cfg(not(feature = "fastframe"))]
        "fastframe" => {
            Err("fastframe backend is disabled; rebuild with --features fastframe".to_owned())
        }
        other => Err(format!(
            "unknown GUI backend '{other}' (expected egui or fastframe)"
        )),
    };
    if let Err(error) = result {
        eprintln!("cannot start GUI: {error}");
        std::process::exit(1);
    }
}
