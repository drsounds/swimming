//! swimming-bridge: embeds a JavaScript engine (V8 or QuickJS, chosen at compile time), exposes
//! the native `__SWIMMING_NATIVE__` command API to the React reconciler and routes native events back into JS.
use engine::{JsValue, NativeFunction};
use std::sync::{Arc, Mutex};
use swimming_core::SwimmingEngine;

pub mod engine;
pub mod gui;
pub mod loader;

pub use engine::{DefaultEngine, JsEngine, JsError};

/// The bridge over the engine selected by Cargo features; see [`engine`].
pub type JsBridge = Bridge<DefaultEngine>;

pub struct Bridge<E: JsEngine> {
    js: E,
    engine: Arc<Mutex<SwimmingEngine>>,
}

impl<E: JsEngine> Bridge<E> {
    pub fn new(width: f32, height: f32) -> Result<Self, JsError> {
        let engine = Arc::new(Mutex::new(SwimmingEngine::new()));
        engine.lock().unwrap().compute_layout(width, height);
        let bridge = Bridge { js: E::new()?, engine };
        bridge.register_globals()?;
        Ok(bridge)
    }

    pub fn engine(&self) -> Arc<Mutex<SwimmingEngine>> { self.engine.clone() }

    /// Name of the underlying JavaScript engine.
    pub fn js_engine_name(&self) -> &'static str { E::NAME }

    pub fn resize(&self, width: f32, height: f32) {
        self.engine.lock().unwrap().compute_layout(width, height);
    }

    fn register_globals(&self) -> Result<(), JsError> {
        let native = |f: Box<dyn Fn(&mut SwimmingEngine, &[JsValue]) -> JsValue>| -> NativeFunction {
            let e = self.engine.clone();
            Box::new(move |args| f(&mut e.lock().unwrap(), args))
        };
        let arg = |args: &[JsValue], i: usize| args.get(i).cloned().unwrap_or_default();
        self.js.register_object("__SWIMMING_NATIVE__", vec![
            ("createNode", native(Box::new(move |g, a| {
                JsValue::Number(g.create_node(arg(a, 0).as_str(), arg(a, 1).as_str(), arg(a, 2).as_str()) as f64)
            }))),
            ("createText", native(Box::new(move |g, a| JsValue::Number(g.create_text(arg(a, 0).as_str()) as f64)))),
            ("setText", native(Box::new(move |g, a| {
                g.set_text(arg(a, 0).as_f64() as u64, arg(a, 1).as_str());
                JsValue::Undefined
            }))),
            ("appendChild", native(Box::new(move |g, a| {
                g.append_child(arg(a, 0).as_f64() as u64, arg(a, 1).as_f64() as u64);
                JsValue::Undefined
            }))),
            ("insertBefore", native(Box::new(move |g, a| {
                let before = arg(a, 2).as_f64();
                g.insert_before(arg(a, 0).as_f64() as u64, arg(a, 1).as_f64() as u64, (before >= 0.0).then_some(before as u64));
                JsValue::Undefined
            }))),
            ("removeChild", native(Box::new(move |g, a| {
                g.remove_child(arg(a, 0).as_f64() as u64, arg(a, 1).as_f64() as u64);
                JsValue::Undefined
            }))),
            ("updateNode", native(Box::new(move |g, a| {
                g.update_node(arg(a, 0).as_f64() as u64, arg(a, 1).as_str(), arg(a, 2).as_str());
                JsValue::Undefined
            }))),
            ("commit", native(Box::new(|g, _| {
                let (w, h) = (g.width, g.height);
                g.compute_layout(w, h);
                JsValue::Undefined
            }))),
        ])
    }

    /// Runs a bundled script (e.g. the output of vite-plugin-swimming).
    pub fn eval(&self, source: &str) -> Result<(), JsError> {
        self.js.eval(source)?;
        self.run_until_idle()
    }

    pub fn eval_script(&self, source: &str) -> Result<(), String> {
        self.eval(source).map_err(|error| error.to_string())
    }

    /// The engines have no event loop: drain promise jobs and run polyfilled timers (`__SWIMMING_TICK__`) until idle.
    pub fn run_until_idle(&self) -> Result<(), JsError> {
        for _ in 0..10_000 {
            self.js.run_jobs();
            let pending = self.js.call_global("__SWIMMING_TICK__", &[JsValue::Number(0.0)])?;
            if !pending.is_some_and(|v| v.is_true()) { break; }
        }
        self.js.run_jobs();
        Ok(())
    }

    pub fn tick(&self, elapsed_ms: u32) -> Result<(), JsError> {
        self.js.call_global("__SWIMMING_TICK__", &[JsValue::Number(elapsed_ms as f64)])?;
        self.run_until_idle()
    }

    /// Hit-tests natively, then calls `globalThis.__SWIMMING_DISPATCH__(id, event)` so JS handlers run.
    pub fn pointer(&self, event: &str, x: f32, y: f32) -> Result<Option<u64>, JsError> {
        let target = self.engine.lock().unwrap().hit_test(x, y);
        if let Some(id) = target {
            self.js.call_global("__SWIMMING_DISPATCH__", &[JsValue::Number(id as f64), JsValue::String(event.to_owned())])?;
            self.run_until_idle()?;
        }
        Ok(target)
    }

    pub fn render(&self) -> Option<tiny_skia_pixmap::Pixmap> {
        let g = self.engine.lock().unwrap();
        swimming_render::paint(g.layout()?, g.width as u32, g.height as u32)
    }
}

pub use swimming_render::tiny_skia as tiny_skia_pixmap;
