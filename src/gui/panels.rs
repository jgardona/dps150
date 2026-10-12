//! Barra superior, coluna de controle e barra inferior.

use super::{
    theme::{ACTION_COLOR, ERROR_COLOR, OK_COLOR, WARN_COLOR},
    widgets::{ReadoutRow, card, info_pair, status_dot},
};
use crate::logics::{
    app::{AppState, ConnectionState},
    format::format_hhmmss,
};
use eframe::egui::{self, Color32};
use std::time::Instant;

pub(crate) fn top_bar(ui: &mut egui::Ui, app: &mut AppState) {
    let is_idle = app.is_idle();

    ui.horizontal(|ui| {
        let (status_color, status_text) = match app.connection() {
            ConnectionState::Disconnected => (Color32::from_gray(110), "Desconectado"),
            ConnectionState::Connecting => (WARN_COLOR, "Conectando..."),
            ConnectionState::Connected => (OK_COLOR, "Conectado"),
            ConnectionState::Error(_) => (ERROR_COLOR, "Erro"),
        };
        status_dot(ui, status_color).on_hover_text(status_text);

        ui.add_enabled_ui(is_idle, |ui| {
            let selected_text = if app.selected_port().is_empty() {
                "-- porta --".to_owned()
            } else {
                app.selected_port().to_owned()
            };
            egui::ComboBox::from_id_salt("serial_port_combo")
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    for port in app.available_ports().to_vec() {
                        ui.selectable_value(app.selected_port_mut(), port.clone(), port);
                    }
                });
        });

        let label = match app.connection() {
            ConnectionState::Disconnected | ConnectionState::Error(_) => "Conectar",
            ConnectionState::Connecting => "Cancelar",
            ConnectionState::Connected => "Desconectar",
        };
        if ui.button(label).clicked() {
            if is_idle {
                let ctx = ui.ctx().clone();
                app.connect(move || ctx.request_repaint());
            } else {
                app.disconnect();
            }
        }

        if let ConnectionState::Error(msg) = app.connection() {
            ui.colored_label(ERROR_COLOR, msg);
        }

        if !app.is_connected() {
            return;
        }
        let device = app.device();
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            info_pair(
                ui,
                "Temp",
                &format!("{:.0}\u{b0}C", device.temperature.unwrap_or(0.0)),
            );
            ui.separator();
            info_pair(
                ui,
                "Entrada",
                &format!("{:.2} V", device.input_voltage.unwrap_or(0.0)),
            );
            ui.separator();
            if let Some(fw) = &device.firmware_version {
                info_pair(ui, "fw", fw);
            }
            if let Some(model) = &device.model_name {
                ui.label(egui::RichText::new(model).strong());
            }
        });
    });
}

pub(crate) fn control_column(ui: &mut egui::Ui, app: &mut AppState, colors: &[Color32; 3]) {
    readouts(ui, app, colors);
    ui.add_space(8.0);
    setpoint_card(ui, app, colors);
    ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
        output_button(ui, app);
    });
}

fn readouts(ui: &mut egui::Ui, app: &AppState, colors: &[Color32; 3]) {
    let device = app.device();
    let mode = device.cc_cv.as_deref();
    ui.add(ReadoutRow {
        unit: "V",
        accent: colors[0],
        value: format!("{:05.2}", device.output_voltage.unwrap_or(0.0)),
        badge: (mode == Some("CV")).then_some("CV"),
    });
    ui.add_space(4.0);
    ui.add(ReadoutRow {
        unit: "A",
        accent: colors[1],
        value: format!("{:05.3}", device.output_current.unwrap_or(0.0)),
        badge: (mode == Some("CC")).then_some("CC"),
    });
    ui.add_space(4.0);
    ui.add(ReadoutRow {
        unit: "W",
        accent: colors[2],
        value: format!("{:05.2}", device.output_power.unwrap_or(0.0)),
        badge: None,
    });
}

fn setpoint_card(ui: &mut egui::Ui, app: &mut AppState, colors: &[Color32; 3]) {
    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Setpoint").strong());
            if app.setpoint().is_dirty() {
                ui.colored_label(WARN_COLOR, "\u{2022} não aplicado");
            }
        });

        let mut enter_pressed = false;
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("V").color(colors[0]).strong());
            let v_resp = ui.add(
                egui::DragValue::new(&mut app.setpoint_mut().vset_input)
                    .speed(0.05)
                    .range(0.0..=30.0)
                    .fixed_decimals(2)
                    .suffix(" V"),
            );
            ui.add_space(10.0);
            ui.label(egui::RichText::new("A").color(colors[1]).strong());
            let c_resp = ui.add(
                egui::DragValue::new(&mut app.setpoint_mut().cset_input)
                    .speed(0.01)
                    .range(0.0..=5.0)
                    .fixed_decimals(3)
                    .suffix(" A"),
            );

            if v_resp.changed() || c_resp.changed() {
                app.mark_setpoint_edited();
            }
            app.set_editing(
                v_resp.dragged() || v_resp.has_focus(),
                c_resp.dragged() || c_resp.has_focus(),
            );
            enter_pressed = (v_resp.lost_focus() || c_resp.lost_focus())
                && ui.input(|i| i.key_pressed(egui::Key::Enter));
        });

        ui.horizontal(|ui| {
            let dirty = app.setpoint().is_dirty();
            let apply = egui::Button::new(
                egui::RichText::new("Aplicar")
                    .color(Color32::BLACK)
                    .strong(),
            )
            .fill(if dirty {
                ACTION_COLOR
            } else {
                Color32::from_gray(90)
            });
            let clicked = ui
                .add_enabled(app.is_connected(), apply)
                .on_hover_text("Envia Vset/Iset ao aparelho (ou Enter)")
                .clicked();
            if clicked || (enter_pressed && app.is_connected()) {
                app.apply_setpoints();
            }
            if app.setpoint().is_dirty() && ui.button("Desfazer").clicked() {
                app.discard_setpoint_edits();
            }
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Memória");
            for n in 1..=6u8 {
                let hover = match app.group_values(n) {
                    Some((v, c)) => format!("M{n}: {v:.2} V / {c:.3} A — clique para carregar"),
                    None => format!("M{n}: sem dados"),
                };
                if ui
                    .selectable_label(app.selected_profile() == n, format!("M{n}"))
                    .on_hover_text(hover)
                    .clicked()
                {
                    app.load_profile(n);
                }
            }
            let target = app.selected_profile();
            if ui
                .add_enabled(app.is_connected(), egui::Button::new("Salvar"))
                .on_hover_text(format!("Salva os valores dos campos em M{target}"))
                .clicked()
            {
                app.save_profile(target);
            }
        });
    });
}

fn output_button(ui: &mut egui::Ui, app: &mut AppState) {
    let (label, fill, text_color, hover) = if app.output_on() {
        (
            "SAÍDA LIGADA",
            OK_COLOR,
            Color32::BLACK,
            "Clique para desligar a saída",
        )
    } else {
        (
            "SAÍDA DESLIGADA",
            Color32::from_gray(55),
            Color32::WHITE,
            "Clique para ligar a saída",
        )
    };
    let button = egui::Button::new(
        egui::RichText::new(label)
            .color(text_color)
            .strong()
            .size(20.0),
    )
    .fill(fill);
    let w = ui.available_width();
    let resp = ui
        .add_enabled_ui(app.is_connected(), |ui| ui.add_sized([w, 56.0], button))
        .inner;
    if resp.on_hover_text(hover).clicked() {
        app.toggle_output();
    }
}

pub(crate) fn bottom_bar(ui: &mut egui::Ui, app: &AppState, now: Instant) {
    let device = app.device();
    ui.horizontal(|ui| {
        info_pair(
            ui,
            "Capacidade",
            &format!("{:.3} Ah", device.output_capacity.unwrap_or(0.0)),
        );
        ui.separator();
        info_pair(
            ui,
            "Energia",
            &format!("{:.2} Wh", device.output_energy.unwrap_or(0.0)),
        );
        ui.separator();
        info_pair(ui, "Tempo", &format_hhmmss(app.current_elapsed(now)));
    });
}
