use dps150::gui::GuiApp;
use eframe::egui;

const WIDTH: f32 = 1000.0;
const HEIGHT: f32 = 540.0;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([WIDTH, HEIGHT]),
        ..Default::default()
    };

    eframe::run_native(
        "DPS150 Controller",
        options,
        Box::new(|_cc| Ok(Box::new(GuiApp::new()))),
    )
}
