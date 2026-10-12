//! Interface gráfica do controlador (egui/eframe).
//!
//! Este módulo só apresenta o estado de [`crate::logics`] e repassa as ações do usuário.
//! Ele não contém regras de negócio nem acessa a porta serial.

mod chart;
mod panels;
mod theme;
mod widgets;

use crate::logics::app::AppState;
use eframe::egui::{self, Color32, Pos2, Vec2};
use std::time::{Duration, Instant};
use theme::{CONTROL_MAX_WIDTH, CONTROL_MIN_WIDTH, CONTROL_SHARE, DEFAULT_SERIES_COLORS};
use widgets::column_ui;

pub struct GuiApp {
    app: AppState,
    /// Cores de V, A e W (nesta ordem), escolhidas pela legenda do gráfico.
    series_colors: [Color32; 3],
}

impl Default for GuiApp {
    fn default() -> Self {
        Self::new()
    }
}

impl GuiApp {
    pub fn new() -> Self {
        Self {
            app: AppState::new(),
            series_colors: DEFAULT_SERIES_COLORS,
        }
    }
}

impl eframe::App for GuiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        self.app.refresh_ports_if_due(now);
        self.app.poll_events();
        self.app.tick(now);

        let app = &mut self.app;
        let colors = &mut self.series_colors;
        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::Panel::top("top_bar")
                .resizable(false)
                .show_inside(ui, |ui| panels::top_bar(ui, app));
            egui::Panel::bottom("info_bar")
                .resizable(false)
                .show_inside(ui, |ui| panels::bottom_bar(ui, app, now));

            // Coluna de controle com largura proporcional (limitada), gráfico com o resto.
            let full = ui.available_rect_before_wrap();
            let gap = ui.spacing().item_spacing.x * 2.0;
            let control_w = (full.width() * CONTROL_SHARE)
                .clamp(CONTROL_MIN_WIDTH, CONTROL_MAX_WIDTH)
                .min(full.width());
            let control_rect =
                egui::Rect::from_min_size(full.min, Vec2::new(control_w, full.height()));
            let chart_rect = egui::Rect::from_min_max(
                Pos2::new(control_rect.right() + gap, full.top()),
                full.max,
            );

            panels::control_column(&mut column_ui(ui, control_rect), app, colors);
            chart::chart_panel(&mut column_ui(ui, chart_rect), app, colors);
            ui.advance_cursor_after_rect(full);
        });

        ui.ctx()
            .request_repaint_after(self.app.update_interval().min(Duration::from_millis(100)));
    }
}
