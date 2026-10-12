//! Painel do gráfico: legenda com seletor de cor, histórico com três escalas e controles.

use super::{
    theme::{CHART_AXIS_COL_W, COLOR_PICKER_WIDTH, DEFAULT_SERIES_COLORS},
    widgets::card,
};
use crate::logics::{
    app::{AppState, UPDATE_MS_RANGE},
    format::format_window,
    history::{HISTORY_LEN, History, Sample},
    scale::{AxisScale, CHART_Y_DIVS, CHART_Y_SUBDIVS},
};
use eframe::egui::{self, Color32, Pos2, Sense, Stroke, Vec2};
use std::time::Duration;

/// Série do gráfico: leitura do valor na amostra, cor, unidade e passo mínimo da escala.
type SeriesSpec = (fn(&Sample) -> f32, Color32, &'static str, f32);

pub(crate) fn chart_panel(ui: &mut egui::Ui, app: &mut AppState, colors: &mut [Color32; 3]) {
    card(ui, |ui| {
        let last = app.history().last();
        ui.horizontal(|ui| {
            let legend = [
                last.map(|s| format!("{:.2} V", s.voltage)),
                last.map(|s| format!("{:.3} A", s.current)),
                last.map(|s| format!("{:.2} W", s.power)),
            ];
            for (i, value) in legend.into_iter().enumerate() {
                let resp = ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(format!(
                                "\u{25a0} {}",
                                value.unwrap_or_else(|| "--".to_owned())
                            ))
                            .color(colors[i])
                            .size(12.0),
                        )
                        .frame(false),
                    )
                    .on_hover_text("Clique para mudar a cor");
                egui::Popup::from_toggle_button_response(&resp)
                    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                    .show(|ui| {
                        ui.spacing_mut().slider_width = COLOR_PICKER_WIDTH;
                        egui::color_picker::color_picker_color32(
                            ui,
                            &mut colors[i],
                            egui::color_picker::Alpha::Opaque,
                        );
                        if ui.button("Padrão").clicked() {
                            colors[i] = DEFAULT_SERIES_COLORS[i];
                        }
                    });
                ui.add_space(6.0);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let window = Duration::from_millis(app.update_ms() * HISTORY_LEN as u64);
                ui.label(
                    egui::RichText::new(format!("janela {}", format_window(window)))
                        .size(12.0)
                        .weak(),
                );
            });
        });

        let chart_h = (ui.available_height() - 30.0).max(60.0);
        draw_history_chart(ui, app.history(), *colors, chart_h);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Update").size(12.0));
            let mut update_ms = app.update_ms();
            let resp = ui.add(
                egui::Slider::new(&mut update_ms, UPDATE_MS_RANGE)
                    .step_by(50.0)
                    .suffix(" ms"),
            );
            if resp.changed() {
                app.set_update_ms(update_ms);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Limpar").clicked() {
                    app.clear_history();
                }
            });
        });
    });
}

/// Desenha as três séries do histórico, cada uma com sua própria escala
/// (à esquerda, na cor da série), já que V, A e W têm faixas bem diferentes.
fn draw_history_chart(ui: &mut egui::Ui, history: &History, colors: [Color32; 3], height: f32) {
    let size = Vec2::new(ui.available_width(), height);
    let (rect, _resp) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 4.0, Color32::from_gray(12));

    let series: [SeriesSpec; 3] = [
        (|s| s.voltage, colors[0], "V", 0.1),
        (|s| s.current, colors[1], "A", 0.01),
        (|s| s.power, colors[2], "W", 0.1),
    ];

    let axis_w = CHART_AXIS_COL_W * series.len() as f32;
    let plot = egui::Rect::from_min_max(
        Pos2::new(rect.left() + axis_w + 4.0, rect.top() + 20.0),
        Pos2::new(rect.right() - 6.0, rect.bottom() - 8.0),
    );

    let ticks = CHART_Y_DIVS * CHART_Y_SUBDIVS;
    let tick_y = |i: usize| plot.bottom() - plot.height() * (i as f32) / ticks as f32;
    let grid = Stroke::new(1.0f32, Color32::from_gray(40));
    let minor_grid = Stroke::new(1.0f32, Color32::from_gray(24));
    for i in 0..=ticks {
        let y = tick_y(i);
        let stroke = if i % CHART_Y_SUBDIVS == 0 {
            grid
        } else {
            minor_grid
        };
        painter.line_segment(
            [Pos2::new(plot.left(), y), Pos2::new(plot.right(), y)],
            stroke,
        );
    }
    // Em gráficos baixos, os rótulos intermediários se sobreporiam.
    let show_minor_labels = plot.height() / ticks as f32 >= 11.0;
    for i in 0..=6 {
        let x = plot.left() + plot.width() * (i as f32) / 6.0;
        painter.line_segment(
            [Pos2::new(x, plot.top()), Pos2::new(x, plot.bottom())],
            grid,
        );
    }

    let label_font = egui::FontId::proportional(10.0);
    let minor_label_font = egui::FontId::proportional(9.0);
    let step = plot.width() / (HISTORY_LEN - 1) as f32;
    // Alinha o ponto mais recente à direita; o gráfico "anda" para a esquerda.
    let x0 = plot.right() - step * history.len().saturating_sub(1) as f32;

    for (col, (value, color, unit, min_step)) in series.into_iter().enumerate() {
        let peak = history.iter().map(value).fold(0.0f32, f32::max);
        let scale = AxisScale::for_peak(peak, min_step);

        let label_x = rect.left() + CHART_AXIS_COL_W * (col as f32 + 1.0);
        painter.text(
            Pos2::new(label_x, rect.top() + 3.0),
            egui::Align2::RIGHT_TOP,
            unit,
            label_font.clone(),
            color,
        );
        for i in 0..=ticks {
            let major = i % CHART_Y_SUBDIVS == 0;
            if !major && !show_minor_labels {
                continue;
            }
            let (font, text_color) = if major {
                (label_font.clone(), color)
            } else {
                (minor_label_font.clone(), color.gamma_multiply(0.55))
            };
            painter.text(
                Pos2::new(label_x, tick_y(i)),
                egui::Align2::RIGHT_CENTER,
                format!("{:.*}", scale.decimals, scale.max * i as f32 / ticks as f32),
                font,
                text_color,
            );
        }

        if history.len() < 2 {
            continue;
        }
        let points: Vec<Pos2> = history
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let y = plot.bottom() - (value(s) / scale.max).clamp(0.0, 1.0) * plot.height();
                Pos2::new(x0 + step * i as f32, y)
            })
            .collect();
        painter.add(egui::Shape::line(points, Stroke::new(1.5f32, color)));
    }
}
