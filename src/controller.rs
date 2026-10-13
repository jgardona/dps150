use dps150::gui::{GuiApp, install_fonts};
use eframe::egui;

const WIDTH: f32 = 1000.0;
const HEIGHT: f32 = 700.0;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([WIDTH, HEIGHT]),
        ..Default::default()
    };

    eframe::run_native(
        "DPS150 Controller",
        options,
        Box::new(|cc| {
            // A fonte precisa estar instalada antes do primeiro quadro.
            install_fonts(&cc.egui_ctx);
            Ok(Box::new(GuiApp::new()))
        }),
    )
}
