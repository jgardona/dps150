use eframe::egui::{self, DragValue, Label, RichText};

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
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default_margins().show_inside(ui, |ui| {
            egui::Frame::new().inner_margin(100.0).show(ui, |ui| {
                egui::Grid::new("PowerGrid")
                    .num_columns(2)
                    .spacing([40.0, 10.0])
                    .show(ui, |ui| {
                        // Line one.
                        ui.add(VIPLabel::new("00.00V", egui::Color32::WHITE));
                        ui.add(
                            DragValue::new(&mut 3)
                                .speed(0.01)
                                .range(0.0..=36.0)
                                .suffix(" VSet"),
                        );
                        ui.end_row();

                        // Line 2.
                        ui.add(VIPLabel::new("00.000A", egui::Color32::WHITE));
                        ui.add(
                            DragValue::new(&mut 3)
                                .speed(0.01)
                                .range(0.0..=36.0)
                                .suffix(" CSet"),
                        );
                        ui.end_row();

                        // Line 3.
                        ui.add(VIPLabel::new("00.00W", egui::Color32::WHITE));
                        ui.label(
                            RichText::new("TEMP: 28 C")
                                .size(14.0)
                                .color(egui::Color32::WHITE)
                                .strong(),
                        );
                        ui.end_row();

                        // Line 4.
                        ui.add(Label::new("Capacity: "));
                        ui.add(Label::new("0000.000 Ah"));
                        ui.end_row();

                        // Line 5.
                        ui.add(Label::new("Energy"));
                        ui.add(Label::new("0000.000 Wh"));
                        ui.end_row();

                        // Line 6.
                        ui.add(Label::new("Time:"));
                        ui.add(Label::new("00:03:37"));
                        ui.end_row();
                    });
            });
        });
    }
}

struct VIPLabel<'a> {
    text: &'a str,
    color: egui::Color32,
}

impl<'a> VIPLabel<'a> {
    fn new(text: &'a str, color: egui::Color32) -> Self {
        Self { text, color }
    }
}

impl<'a> egui::Widget for VIPLabel<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.label(
            egui::RichText::new(self.text)
                .color(self.color)
                .size(30.0)
                .strong(), // Equivalente a negrito.
        )
    }
}
