use crate::JsBridge;
use eframe::egui;
use std::time::Instant;

pub trait GuiBackend {
    fn run(self, bridge: JsBridge, title: &str) -> Result<(), String>;
}

pub struct EguiBackend;

impl GuiBackend for EguiBackend {
    fn run(self, bridge: JsBridge, title: &str) -> Result<(), String> {
        eframe::run_native(
            title,
            native_options(),
            Box::new(move |_| Ok(Box::new(SwimmingApp::new(bridge)))),
        )
        .map_err(|error| error.to_string())
    }
}

fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    }
}

struct SwimmingApp {
    bridge: JsBridge,
    texture: Option<egui::TextureHandle>,
    last_tick: Instant,
}

impl SwimmingApp {
    fn new(bridge: JsBridge) -> Self {
        Self {
            bridge,
            texture: None,
            last_tick: Instant::now(),
        }
    }

    fn show(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let size = ui.available_size().max(egui::Vec2::splat(1.0));
        self.bridge.resize(size.x, size.y);
        let elapsed = self.last_tick.elapsed().as_millis().min(u32::MAX as u128) as u32;
        self.last_tick = Instant::now();
        let _ = self.bridge.tick(elapsed);

        if let Some(pixmap) = self.bridge.render() {
            let image = egui::ColorImage::from_rgba_premultiplied(
                [pixmap.width() as usize, pixmap.height() as usize],
                pixmap.data(),
            );
            if let Some(texture) = &mut self.texture {
                texture.set(image, egui::TextureOptions::LINEAR);
            } else {
                self.texture =
                    Some(ctx.load_texture("swimming", image, egui::TextureOptions::LINEAR));
            }
            if let Some(texture) = &self.texture {
                let response =
                    ui.add_sized(size, egui::Image::new(texture).sense(egui::Sense::click()));
                if response.clicked_by(egui::PointerButton::Primary) {
                    if let Some(point) = response.interact_pointer_pos() {
                        let _ = self.bridge.pointer(
                            "click",
                            point.x - response.rect.min.x,
                            point.y - response.rect.min.y,
                        );
                    }
                }
            }
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(16));
    }
}

impl eframe::App for SwimmingApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.show(&ctx, ui);
    }
}

#[cfg(feature = "fastframe")]
pub struct FastframeBackend;

#[cfg(feature = "fastframe")]
impl GuiBackend for FastframeBackend {
    fn run(self, bridge: JsBridge, title: &str) -> Result<(), String> {
        use fastframe_shell::{Closed, Headless, Resident, Shell, Waker};

        struct ResidentApp(SwimmingApp);
        impl Resident for ResidentApp {
            fn closed(&self) -> Closed {
                Closed::Quit
            }
            fn window_gone(&mut self) {}
            fn headless_frame(&mut self, _ctx: &egui::Context) -> Headless {
                Headless::Quit
            }
            fn shutdown(&mut self) {}
        }

        struct WindowApp(fastframe_shell::Held<ResidentApp>);
        impl eframe::App for WindowApp {
            fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
                let ctx = ui.ctx().clone();
                self.0.0.show(&ctx, ui);
            }
        }

        let waker = Waker::default();
        Shell::new(ResidentApp(SwimmingApp::new(bridge)), &waker)
            .run(|lease| {
                eframe::run_native(
                    title,
                    native_options(),
                    Box::new(move |cc| Ok(Box::new(WindowApp(lease.take(&cc.egui_ctx))))),
                )
            })
            .map_err(|error| error.to_string())
    }
}
