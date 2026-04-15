use eframe::egui;

const WIDTH: f32 = 500.0;
const HEIGHT: f32 = 400.0;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([WIDTH, HEIGHT]),
        ..Default::default()
    };

    eframe::run_native(
        "FNIRSI Power Suppy Controller",
        options,
        Box::new(|_| Ok(Box::<ControllerModel>::default())),
    )
}

#[derive(Default)]
struct ControllerModel;

impl eframe::App for ControllerModel {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        todo!()
    }
}
