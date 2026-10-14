//! swimming-bridge: embeds QuickJS, exposes the native `__SWIMMING_NATIVE__` command API
//! to the React reconciler and routes native events back into JS.
use rquickjs::{Context, Exception, Function, Object, Runtime};
use std::sync::{Arc, Mutex};
use swimming_core::SwimmingEngine;

pub mod gui;
pub mod loader;

pub struct JsBridge {
    // Field order matters: the context must drop before the runtime.
    context: Context,
    _runtime: Runtime,
    engine: Arc<Mutex<SwimmingEngine>>,
}

impl JsBridge {
    pub fn new(width: f32, height: f32) -> rquickjs::Result<Self> {
        let runtime = Runtime::new()?;
        let context = Context::full(&runtime)?;
        let engine = Arc::new(Mutex::new(SwimmingEngine::new()));
        engine.lock().unwrap().compute_layout(width, height);
        let bridge = JsBridge { context, _runtime: runtime, engine };
        bridge.register_globals()?;
        Ok(bridge)
    }

    pub fn engine(&self) -> Arc<Mutex<SwimmingEngine>> { self.engine.clone() }

    pub fn resize(&self, width: f32, height: f32) {
        self.engine.lock().unwrap().compute_layout(width, height);
    }

    fn register_globals(&self) -> rquickjs::Result<()> {
        let engine = self.engine.clone();
        self.context.with(|ctx| {
            let native = Object::new(ctx.clone())?;
            let e = engine.clone();
            native.set("createNode", Function::new(ctx.clone(), move |tag: String, style: String, props: String| -> f64 {
                e.lock().unwrap().create_node(&tag, &style, &props) as f64
            })?)?;
            let e = engine.clone();
            native.set("createText", Function::new(ctx.clone(), move |text: String| -> f64 {
                e.lock().unwrap().create_text(&text) as f64
            })?)?;
            let e = engine.clone();
            native.set("setText", Function::new(ctx.clone(), move |id: f64, text: String| {
                e.lock().unwrap().set_text(id as u64, &text);
            })?)?;
            let e = engine.clone();
            native.set("appendChild", Function::new(ctx.clone(), move |parent: f64, child: f64| {
                e.lock().unwrap().append_child(parent as u64, child as u64);
            })?)?;
            let e = engine.clone();
            native.set("insertBefore", Function::new(ctx.clone(), move |parent: f64, child: f64, before: f64| {
                e.lock().unwrap().insert_before(parent as u64, child as u64, (before >= 0.0).then_some(before as u64));
            })?)?;
            let e = engine.clone();
            native.set("removeChild", Function::new(ctx.clone(), move |parent: f64, child: f64| {
                e.lock().unwrap().remove_child(parent as u64, child as u64);
            })?)?;
            let e = engine.clone();
            native.set("updateNode", Function::new(ctx.clone(), move |id: f64, style: String, props: String| {
                e.lock().unwrap().update_node(id as u64, &style, &props);
            })?)?;
            let e = engine.clone();
            native.set("commit", Function::new(ctx.clone(), move || {
                let mut g = e.lock().unwrap();
                let (w, h) = (g.width, g.height);
                g.compute_layout(w, h);
            })?)?;
            ctx.globals().set("__SWIMMING_NATIVE__", native)?;
            Ok(())
        })
    }

    /// Runs a bundled script (e.g. the output of vite-plugin-swimming).
    pub fn eval(&self, source: &str) -> rquickjs::Result<()> {
        self.context.with(|ctx| ctx.eval::<(), _>(source))?;
        self.run_until_idle()
    }

    pub fn eval_script(&self, source: &str) -> Result<(), String> {
        self.context.with(|ctx| {
            ctx.eval::<(), _>(source).map_err(|error| {
                ctx.catch()
                    .into_object()
                    .and_then(Exception::from_object)
                    .map(|exception| exception.to_string())
                    .unwrap_or_else(|| error.to_string())
            })
        })?;
        self.run_until_idle_detailed()
    }

    fn run_until_idle_detailed(&self) -> Result<(), String> {
        for _ in 0..10_000 {
            self.drain_jobs();
            let pending = self.context.with(|ctx| {
                match ctx.globals().get::<_, Function>("__SWIMMING_TICK__") {
                    Ok(function) => function.call::<_, bool>((0,)).map_err(|error| {
                        ctx.catch()
                            .into_object()
                            .and_then(Exception::from_object)
                            .map(|exception| exception.to_string())
                            .unwrap_or_else(|| error.to_string())
                    }),
                    Err(_) => Ok(false),
                }
            })?;
            if !pending {
                break;
            }
        }
        self.drain_jobs();
        Ok(())
    }

    /// QuickJS has no event loop: drain promise jobs and run polyfilled timers (`__SWIMMING_TICK__`) until idle.
    pub fn run_until_idle(&self) -> rquickjs::Result<()> {
        for _ in 0..10_000 {
            self.drain_jobs();
            let pending = self.context.with(|ctx| -> rquickjs::Result<bool> {
                match ctx.globals().get::<_, Function>("__SWIMMING_TICK__") {
                    Ok(f) => f.call::<_, bool>((0,)),
                    Err(_) => Ok(false),
                }
            })?;
            if !pending { break; }
        }
        self.drain_jobs();
        Ok(())
    }

    pub fn tick(&self, elapsed_ms: u32) -> rquickjs::Result<()> {
        self.context.with(|ctx| -> rquickjs::Result<()> {
            if let Ok(f) = ctx.globals().get::<_, Function>("__SWIMMING_TICK__") {
                let _: bool = f.call((elapsed_ms,))?;
            }
            Ok(())
        })?;
        self.run_until_idle()
    }

    fn drain_jobs(&self) {
        while self._runtime.execute_pending_job().unwrap_or(false) {}
    }

    /// Hit-tests natively, then calls `globalThis.__SWIMMING_DISPATCH__(id, event)` so JS handlers run.
    pub fn pointer(&self, event: &str, x: f32, y: f32) -> rquickjs::Result<Option<u64>> {
        let target = self.engine.lock().unwrap().hit_test(x, y);
        if let Some(id) = target {
            self.context.with(|ctx| -> rquickjs::Result<()> {
                if let Ok(f) = ctx.globals().get::<_, Function>("__SWIMMING_DISPATCH__") {
                    f.call::<_, ()>((id as f64, event.to_string()))?;
                }
                Ok(())
            })?;
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
