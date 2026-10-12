//! Widgets e helpers de layout reaproveitados pelos painéis.

use crate::logics::memory_setup::SetupField;
use eframe::egui::{self, Color32, Sense, Vec2};

use super::theme::ERROR_COLOR;

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

/// Campo do quadro de setup: rótulo pequeno em cima e valor editável embaixo.
///
/// `accent` dá a cor do rótulo (V, A e W); `invalid` pinta rótulo e valor de vermelho. Devolve o
/// novo valor quando o usuário o altera. Sem valor (`None`), mostra "—" desabilitado.
pub(crate) fn setup_field(
    ui: &mut egui::Ui,
    field: SetupField,
    value: Option<f32>,
    accent: Option<Color32>,
    enabled: bool,
    invalid: bool,
) -> Option<f32> {
    let star = if field.is_global() { "*" } else { "" };
    let label = egui::RichText::new(format!("{}{star}", field.label())).small();
    let label = match (invalid, accent) {
        (true, _) => label.color(ERROR_COLOR),
        (false, Some(color)) => label.color(color).strong(),
        (false, None) => label.weak(),
    };
    let mut edited = None;
    ui.vertical(|ui| {
        ui.label(label);
        let Some(mut v) = value else {
            ui.add_enabled(false, egui::Label::new("\u{2014}"));
            return;
        };
        if invalid {
            ui.visuals_mut().override_text_color = Some(ERROR_COLOR);
        }
        let drag = egui::DragValue::new(&mut v)
            .range(0.0..=field.max())
            .clamp_existing_to_range(false)
            .speed(field.max() * 0.002)
            .fixed_decimals(field.decimals())
            .suffix(format!(" {}", field.unit()));
        if ui.add_enabled(enabled, drag).changed() {
            edited = Some(v);
        }
    });
    edited
}
