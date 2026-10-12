//! Widgets e helpers de layout reaproveitados pelos painéis.

use eframe::egui::{self, Color32, Sense, Vec2};

/// Leitura grande de uma grandeza (V, A ou W) com selo opcional de modo (CV/CC).
pub(crate) struct ReadoutRow<'a> {
    pub unit: &'a str,
    pub accent: Color32,
    pub value: String,
    pub badge: Option<&'a str>,
}

impl<'a> egui::Widget for ReadoutRow<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        egui::Frame::new()
            .fill(Color32::from_gray(20))
            .corner_radius(6.0)
            .inner_margin(egui::Margin::symmetric(10, 4))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(self.value)
                            .color(self.accent)
                            .strong()
                            .size(34.0),
                    );
                    ui.label(egui::RichText::new(self.unit).color(self.accent).size(20.0));
                    if let Some(badge) = self.badge {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            egui::Frame::new()
                                .fill(self.accent)
                                .corner_radius(4.0)
                                .inner_margin(egui::Margin::symmetric(8, 3))
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new(badge)
                                            .color(Color32::BLACK)
                                            .strong()
                                            .size(14.0),
                                    );
                                });
                        });
                    }
                });
            })
            .response
    }
}

/// Moldura padrão dos blocos da interface.
pub(crate) fn card(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(Color32::from_gray(25))
        .corner_radius(6.0)
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add_contents(ui);
        });
}

/// Rótulo discreto seguido do valor, ex.: "Energia 0.00 Wh".
pub(crate) fn info_pair(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.label(egui::RichText::new(label).weak());
    ui.label(egui::RichText::new(value).strong());
}

pub(crate) fn status_dot(ui: &mut egui::Ui, color: Color32) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(12.0), Sense::hover());
    ui.painter().circle_filled(rect.center(), 5.0, color);
    resp
}

/// Cria uma sub-UI vertical restrita a `rect`.
pub(crate) fn column_ui(ui: &mut egui::Ui, rect: egui::Rect) -> egui::Ui {
    ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    )
}
