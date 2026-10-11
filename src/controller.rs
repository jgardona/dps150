mod dps150;

use dps150::{DPS150, DPSUpdate, commands};
use eframe::egui::{self, Color32, Pos2, Sense, Stroke, Vec2};
use std::{
    collections::VecDeque,
    io::{Read, Write},
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

/// Comando enviado da GUI para a thread serial.
enum AppCommand {
    SetVoltage(f32),
    SetCurrent(f32),
    EnableOutput(bool),
    SaveProfile(u8, f32, f32),
    SetPollInterval(Duration),
    Disconnect,
}

/// Evento enviado da thread serial para a GUI.
enum SerialEvent {
    Update(Box<DPSUpdate>),
    Connected,
    Error(String),
}

enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

struct AppModel {
    state: DPSUpdate,
    connection: ConnectionState,
    rx: Option<Receiver<SerialEvent>>,
    cmd_tx: Option<Sender<AppCommand>>,

    available_ports: Vec<String>,
    selected_port: String,

    vset_input: f32,
    cset_input: f32,
    editing_vset: bool,
    editing_cset: bool,

    run_start: Option<Instant>,
    frozen_elapsed: Duration,

    last_applied_profile: Option<u8>,

    /// Intervalo (ms) entre leituras do dispositivo e amostras do gráfico.
    update_ms: u64,
    history: VecDeque<Sample>,
    last_sample: Option<Instant>,
}

/// Uma amostra de tensão/corrente/potência para o gráfico.
#[derive(Clone, Copy)]
struct Sample {
    voltage: f32,
    current: f32,
    power: f32,
}

const HISTORY_LEN: usize = 300;
/// Largura da coluna de leituras (V/A/W, setpoints, conexão).
const LEFT_COLUMN_WIDTH: f32 = 300.0;
/// Fração do espaço à direita das leituras ocupada pelos botões (o resto é do gráfico).
const BUTTONS_SHARE: f32 = 0.4;
const UPDATE_MS_RANGE: std::ops::RangeInclusive<u64> = 100..=1000;
const CHART_VOLTAGE_COLOR: Color32 = Color32::from_rgb(230, 60, 60);
const CHART_CURRENT_COLOR: Color32 = Color32::from_rgb(60, 200, 90);
const CHART_POWER_COLOR: Color32 = Color32::from_rgb(60, 140, 255);

impl AppModel {
    fn new() -> Self {
        Self {
            state: DPSUpdate::default(),
            connection: ConnectionState::Disconnected,
            rx: None,
            cmd_tx: None,
            available_ports: Vec::new(),
            selected_port: String::new(),
            vset_input: 0.0,
            cset_input: 0.0,
            editing_vset: false,
            editing_cset: false,
            run_start: None,
            frozen_elapsed: Duration::ZERO,
            last_applied_profile: None,
            update_ms: 100,
            history: VecDeque::with_capacity(HISTORY_LEN),
            last_sample: None,
        }
    }

    fn send_cmd(&self, cmd: AppCommand) {
        if let Some(tx) = &self.cmd_tx {
            let _ = tx.send(cmd);
        }
    }

    fn refresh_available_ports(&mut self) {
        self.available_ports = serialport::available_ports()
            .map(|ports| ports.into_iter().map(|p| p.port_name).collect())
            .unwrap_or_default();
        if !self.available_ports.contains(&self.selected_port) {
            self.selected_port = self.available_ports.first().cloned().unwrap_or_default();
        }
    }

    fn reset_telemetry(&mut self) {
        self.state = DPSUpdate::default();
        self.run_start = None;
        self.frozen_elapsed = Duration::ZERO;
        self.history.clear();
        self.last_sample = None;
    }

    fn update_interval(&self) -> Duration {
        Duration::from_millis(self.update_ms)
    }

    /// Registra uma amostra no histórico do gráfico a cada `update_ms`.
    fn record_sample_if_due(&mut self) {
        if !matches!(self.connection, ConnectionState::Connected) {
            return;
        }
        if self
            .last_sample
            .is_some_and(|t| t.elapsed() < self.update_interval())
        {
            return;
        }
        if self.history.len() == HISTORY_LEN {
            self.history.pop_front();
        }
        self.history.push_back(Sample {
            voltage: self.state.output_voltage.unwrap_or(0.0),
            current: self.state.output_current.unwrap_or(0.0),
            power: self.state.output_power.unwrap_or(0.0),
        });
        self.last_sample = Some(Instant::now());
    }

    fn connect(&mut self, ctx: &egui::Context) {
        if self.selected_port.is_empty() {
            self.connection = ConnectionState::Error(
                "Nenhum dispositivo/porta serial disponível para conexão.".to_owned(),
            );
            return;
        }
        let (evt_tx, evt_rx) = mpsc::channel::<SerialEvent>();
        let (cmd_tx, cmd_rx) = mpsc::channel::<AppCommand>();
        let ctx = ctx.clone();
        let port_name = self.selected_port.clone();
        let poll_interval = self.update_interval();
        thread::spawn(move || serial_thread_main(port_name, poll_interval, evt_tx, cmd_rx, ctx));

        self.rx = Some(evt_rx);
        self.cmd_tx = Some(cmd_tx);
        self.connection = ConnectionState::Connecting;
        self.reset_telemetry();
    }

    fn disconnect(&mut self) {
        self.send_cmd(AppCommand::Disconnect);
        self.rx = None;
        self.cmd_tx = None;
        self.connection = ConnectionState::Disconnected;
        self.reset_telemetry();
    }

    fn merge_state(&mut self, new: Box<DPSUpdate>) {
        macro_rules! merge_opt {
            ($field:ident) => {
                if new.$field.is_some() {
                    self.state.$field = new.$field;
                }
            };
        }

        merge_opt!(input_voltage);
        merge_opt!(output_voltage);
        merge_opt!(output_current);
        merge_opt!(output_power);
        merge_opt!(temperature);
        merge_opt!(model_name);
        merge_opt!(protection_state);
        merge_opt!(vset);
        merge_opt!(cset);
        merge_opt!(g1_vset);
        merge_opt!(g1_cset);
        merge_opt!(g2_vset);
        merge_opt!(g2_cset);
        merge_opt!(g3_vset);
        merge_opt!(g3_cset);
        merge_opt!(g4_vset);
        merge_opt!(g4_cset);
        merge_opt!(g5_vset);
        merge_opt!(g5_cset);
        merge_opt!(g6_vset);
        merge_opt!(g6_cset);
        merge_opt!(ovp);
        merge_opt!(ocp);
        merge_opt!(opp);
        merge_opt!(otp);
        merge_opt!(lvp);
        merge_opt!(brightness);
        merge_opt!(volume);
        merge_opt!(metering);
        merge_opt!(output_capacity);
        merge_opt!(output_energy);
        merge_opt!(cc_cv);
        merge_opt!(upper_limit_voltage);
        merge_opt!(upper_limit_current);
        merge_opt!(firmware_version);
        merge_opt!(hardware_version);
        merge_opt!(output_opened);
    }

    fn sync_edit_buffers_if_idle(&mut self) {
        if !self.editing_vset
            && let Some(v) = self.state.vset
        {
            self.vset_input = v;
        }
        if !self.editing_cset
            && let Some(c) = self.state.cset
        {
            self.cset_input = c;
        }
    }

    fn update_elapsed_clock(&mut self) {
        // output_opened == Some(true) significa saída DESLIGADA (circuito aberto).
        let running = self.state.output_opened == Some(false);
        match (running, self.run_start) {
            (true, None) => {
                self.run_start = Some(Instant::now());
                self.frozen_elapsed = Duration::ZERO;
            }
            (false, Some(start)) => {
                self.frozen_elapsed = start.elapsed();
                self.run_start = None;
            }
            _ => {}
        }
    }

    fn current_elapsed(&self) -> Duration {
        match self.run_start {
            Some(start) => start.elapsed(),
            None => self.frozen_elapsed,
        }
    }

    fn apply_setpoints(&self) {
        self.send_cmd(AppCommand::SetVoltage(self.vset_input));
        self.send_cmd(AppCommand::SetCurrent(self.cset_input));
    }

    /// Valores V/I salvos no grupo de memória `n` (1..6), se ambos disponíveis.
    fn group_values(&self, n: u8) -> Option<(f32, f32)> {
        match n {
            1 => Some((self.state.g1_vset?, self.state.g1_cset?)),
            2 => Some((self.state.g2_vset?, self.state.g2_cset?)),
            3 => Some((self.state.g3_vset?, self.state.g3_cset?)),
            4 => Some((self.state.g4_vset?, self.state.g4_cset?)),
            5 => Some((self.state.g5_vset?, self.state.g5_cset?)),
            6 => Some((self.state.g6_vset?, self.state.g6_cset?)),
            _ => None,
        }
    }

    /// Aplica ao Vset/Iset ativo os valores salvos no grupo de memória `n` (1..6).
    fn apply_profile(&mut self, n: u8) {
        if let Some((v, c)) = self.group_values(n) {
            self.send_cmd(AppCommand::SetVoltage(v));
            self.send_cmd(AppCommand::SetCurrent(c));
            self.last_applied_profile = Some(n);
        }
    }

    /// Salva o Vset/Iset atualmente editado no grupo de memória `n` (1..6).
    fn save_profile(&mut self, n: u8) {
        self.send_cmd(AppCommand::SaveProfile(n, self.vset_input, self.cset_input));
        self.last_applied_profile = Some(n);
    }

    fn left_readouts(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.add(ReadoutRow {
                icon: "V",
                accent: Color32::from_rgb(230, 200, 30),
                primary: format!("{:05.2} V", self.state.output_voltage.unwrap_or(0.0)),
                secondary_label: "Vset",
                secondary_value: format!("{:05.2} V", self.state.vset.unwrap_or(0.0)),
            });
            ui.add_space(6.0);
            ui.add(ReadoutRow {
                icon: "A",
                accent: Color32::from_rgb(0, 200, 150),
                primary: format!("{:05.3} A", self.state.output_current.unwrap_or(0.0)),
                secondary_label: "Iset",
                secondary_value: format!("{:05.3} A", self.state.cset.unwrap_or(0.0)),
            });
            ui.add_space(6.0);
            ui.add(ReadoutRow {
                icon: "W",
                accent: Color32::from_rgb(60, 140, 255),
                primary: format!("{:05.2} W", self.state.output_power.unwrap_or(0.0)),
                secondary_label: "Temp",
                secondary_value: format!("{:.0}\u{b0}C", self.state.temperature.unwrap_or(0.0)),
            });

            ui.add_space(10.0);
            self.setpoint_editors(ui);

            ui.add_space(6.0);
            self.connection_controls(ui);
        });
    }

    fn connection_controls(&mut self, ui: &mut egui::Ui) {
        let is_idle = matches!(
            self.connection,
            ConnectionState::Disconnected | ConnectionState::Error(_)
        );
        if is_idle {
            self.refresh_available_ports();
        }

        egui::Frame::new()
            .fill(Color32::from_gray(30))
            .corner_radius(6.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_enabled_ui(is_idle, |ui| {
                        let selected_text = if self.selected_port.is_empty() {
                            "-- porta --".to_owned()
                        } else {
                            self.selected_port.clone()
                        };
                        egui::ComboBox::from_id_salt("serial_port_combo")
                            .selected_text(selected_text)
                            .show_ui(ui, |ui| {
                                for port in self.available_ports.clone() {
                                    ui.selectable_value(
                                        &mut self.selected_port,
                                        port.clone(),
                                        port,
                                    );
                                }
                            });
                    });

                    let (label, color) = match &self.connection {
                        ConnectionState::Disconnected | ConnectionState::Error(_) => {
                            ("Connect", Color32::from_rgb(60, 140, 255))
                        }
                        ConnectionState::Connecting => {
                            ("Connecting...", Color32::from_rgb(230, 180, 40))
                        }
                        ConnectionState::Connected => ("Connected", Color32::from_rgb(60, 200, 90)),
                    };
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(label).color(Color32::BLACK).strong(),
                            )
                            .fill(color),
                        )
                        .clicked()
                    {
                        if is_idle {
                            self.connect(ui.ctx());
                        } else {
                            self.disconnect();
                        }
                    }
                });

                if let ConnectionState::Error(msg) = &self.connection {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(format!("Aviso: {msg}"))
                            .color(Color32::from_rgb(255, 90, 90))
                            .strong(),
                    );
                }
            });
    }

    fn setpoint_editors(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(Color32::from_gray(30))
            .corner_radius(6.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Vset");
                    let v_resp = ui.add(
                        egui::DragValue::new(&mut self.vset_input)
                            .speed(0.05)
                            .range(0.0..=30.0)
                            .suffix(" V"),
                    );
                    if v_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.send_cmd(AppCommand::SetVoltage(self.vset_input));
                    }
                    self.editing_vset = v_resp.dragged() || v_resp.has_focus();

                    ui.add_space(12.0);

                    ui.label("Iset");
                    let c_resp = ui.add(
                        egui::DragValue::new(&mut self.cset_input)
                            .speed(0.01)
                            .range(0.0..=5.0)
                            .suffix(" A"),
                    );
                    if c_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.send_cmd(AppCommand::SetCurrent(self.cset_input));
                    }
                    self.editing_cset = c_resp.dragged() || c_resp.has_focus();
                });
            });
    }

    fn right_buttons(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            let w = ui.available_width();
            let h = (ui.available_height() / 4.0 - 6.0).max(24.0);

            if ui
                .add_sized(
                    [w, h],
                    egui::Button::new(egui::RichText::new("OK").color(Color32::BLACK).strong())
                        .fill(Color32::from_rgb(60, 200, 90)),
                )
                .clicked()
            {
                self.apply_setpoints();
            }

            ui.add_space(4.0);
            // Sempre mostra o perfil-alvo (último selecionado/aplicado/salvo),
            // não uma tentativa de "adivinhar" por comparação de valor — assim
            // editar Vset/Iset depois de selecionar Mn não faz o combo virar
            // "Custom" e perder de vista onde o Save vai gravar.
            let target_profile = self.last_applied_profile.unwrap_or(1);
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("memory_profile_combo")
                    .selected_text(format!("M{target_profile}"))
                    .width(w * 0.55)
                    .show_ui(ui, |ui| {
                        for n in 1..=6u8 {
                            if ui
                                .selectable_label(target_profile == n, format!("M{n}"))
                                .clicked()
                            {
                                self.apply_profile(n);
                            }
                        }
                    });
                if ui
                    .add_sized(
                        [ui.available_width(), 20.0],
                        egui::Button::new(
                            egui::RichText::new("Save")
                                .color(Color32::LIGHT_GRAY)
                                .strong()
                                .size(11.0),
                        ),
                    )
                    .on_hover_text(format!("Salvar Vset/Iset atual em M{target_profile}"))
                    .clicked()
                {
                    self.save_profile(target_profile);
                }
            });

            ui.add_space(4.0);
            let mode = self.state.cc_cv.as_deref().unwrap_or("--");
            let mode_color = if mode == "CV" {
                Color32::from_rgb(230, 180, 40)
            } else {
                Color32::from_rgb(60, 200, 90)
            };
            ui.add_sized(
                [w, h],
                egui::Button::new(egui::RichText::new(mode).color(Color32::BLACK).strong())
                    .fill(mode_color),
            );

            ui.add_space(4.0);
            // output_opened == Some(true) significa saída DESLIGADA (circuito aberto).
            let output_on = self.state.output_opened == Some(false);
            let (label, color) = if output_on {
                ("Running", Color32::from_rgb(60, 200, 90))
            } else {
                ("Stopped", Color32::from_rgb(220, 50, 50))
            };
            if ui
                .add_sized(
                    [w, h * 1.2],
                    egui::Button::new(
                        egui::RichText::new(label)
                            .color(Color32::WHITE)
                            .strong()
                            .size(20.0),
                    )
                    .fill(color),
                )
                .clicked()
            {
                self.send_cmd(AppCommand::EnableOutput(!output_on));
            }
        });
    }

    fn right_chart(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(Color32::from_gray(20))
            .corner_radius(6.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                let last = self.history.back().copied();
                ui.horizontal(|ui| {
                    let legend = [
                        (
                            "V",
                            CHART_VOLTAGE_COLOR,
                            last.map(|s| format!("{:.2} V", s.voltage)),
                        ),
                        (
                            "I",
                            CHART_CURRENT_COLOR,
                            last.map(|s| format!("{:.3} A", s.current)),
                        ),
                        (
                            "P",
                            CHART_POWER_COLOR,
                            last.map(|s| format!("{:.2} W", s.power)),
                        ),
                    ];
                    for (name, color, value) in legend {
                        ui.label(
                            egui::RichText::new(format!(
                                "\u{25a0} {name} {}",
                                value.unwrap_or_else(|| "--".to_owned())
                            ))
                            .color(color)
                            .size(12.0),
                        );
                        ui.add_space(6.0);
                    }
                });

                let chart_h = (ui.available_height() - 30.0).max(60.0);
                draw_history_chart(ui, &self.history, chart_h);

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Update").size(12.0));
                    let resp = ui.add(
                        egui::Slider::new(&mut self.update_ms, UPDATE_MS_RANGE)
                            .step_by(50.0)
                            .suffix(" ms"),
                    );
                    if resp.changed() {
                        self.send_cmd(AppCommand::SetPollInterval(self.update_interval()));
                    }
                });
            });
    }

    fn bottom_info_panel(&self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(Color32::from_gray(25))
            .corner_radius(6.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    draw_bar_chart_icon(ui);
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "Capacity   {:.3} Ah",
                                self.state.output_capacity.unwrap_or(0.0)
                            ))
                            .size(13.0),
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "Energy   {:.2} Wh",
                                self.state.output_energy.unwrap_or(0.0)
                            ))
                            .size(13.0),
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "Time   {}",
                                format_hhmmss(self.current_elapsed())
                            ))
                            .size(13.0),
                        );
                    });
                });
            });
    }
}

impl eframe::App for AppModel {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let mut events = Vec::new();
        if let Some(rx) = &self.rx {
            while let Ok(evt) = rx.try_recv() {
                events.push(evt);
            }
        }
        for evt in events {
            match evt {
                SerialEvent::Update(update) => {
                    self.merge_state(update);
                    self.sync_edit_buffers_if_idle();
                }
                SerialEvent::Connected => self.connection = ConnectionState::Connected,
                SerialEvent::Error(msg) => {
                    self.connection = ConnectionState::Error(msg);
                    self.rx = None;
                    self.cmd_tx = None;
                    self.reset_telemetry();
                }
            }
        }
        self.update_elapsed_clock();
        self.record_sample_if_due();

        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::Panel::bottom("info_panel")
                .resizable(false)
                .min_size(60.0)
                .show_inside(ui, |ui| self.bottom_info_panel(ui));

            // Coluna das leituras com largura fixa; o resto é dividido entre
            // botões e gráfico na proporção BUTTONS_SHARE.
            let full = ui.available_rect_before_wrap();
            let gap = ui.spacing().item_spacing.x;
            let left_w = LEFT_COLUMN_WIDTH.min(full.width() / 3.0);
            let rest = full.width() - left_w - 2.0 * gap;
            let buttons_w = rest * BUTTONS_SHARE;
            let chart_w = rest - buttons_w;

            let column = |x: f32, w: f32| {
                egui::Rect::from_min_size(Pos2::new(x, full.top()), Vec2::new(w, full.height()))
            };
            let left_rect = column(full.left(), left_w);
            let buttons_rect = column(left_rect.right() + gap, buttons_w);
            let chart_rect = column(buttons_rect.right() + gap, chart_w);

            self.left_readouts(&mut column_ui(ui, left_rect));
            self.right_buttons(&mut column_ui(ui, buttons_rect));
            self.right_chart(&mut column_ui(ui, chart_rect));
            ui.advance_cursor_after_rect(full);
        });

        ui.ctx()
            .request_repaint_after(self.update_interval().min(Duration::from_millis(100)));
    }
}

struct ReadoutRow<'a> {
    icon: &'a str,
    accent: Color32,
    primary: String,
    secondary_label: &'a str,
    secondary_value: String,
}

impl<'a> egui::Widget for ReadoutRow<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        egui::Frame::new()
            .fill(Color32::from_gray(20))
            .corner_radius(6.0)
            .inner_margin(6.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    egui::Frame::new()
                        .fill(self.accent)
                        .corner_radius(4.0)
                        .inner_margin(6.0)
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(self.icon)
                                    .color(Color32::BLACK)
                                    .strong()
                                    .size(18.0),
                            );
                        });

                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(self.primary)
                            .color(self.accent)
                            .strong()
                            .size(36.0),
                    );

                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(format!(
                            "{} {}",
                            self.secondary_label, self.secondary_value
                        ))
                        .size(13.0)
                        .weak(),
                    );
                });
            })
            .response
    }
}

fn draw_bar_chart_icon(ui: &mut egui::Ui) {
    let size = Vec2::new(16.0, 16.0);
    let (rect, _resp) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter();
    let heights = [0.4, 0.7, 1.0];
    let bar_w = rect.width() / 4.0;
    for (i, h) in heights.iter().enumerate() {
        let bar_h = rect.height() * h;
        let x = rect.left() + (i as f32) * bar_w * 1.3;
        let bar =
            egui::Rect::from_min_size(Pos2::new(x, rect.bottom() - bar_h), Vec2::new(bar_w, bar_h));
        painter.rect_filled(bar, 1.0, Color32::LIGHT_GRAY);
    }
    ui.add_space(6.0);
}

/// Cria uma sub-UI vertical restrita a `rect`.
fn column_ui(ui: &mut egui::Ui, rect: egui::Rect) -> egui::Ui {
    ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    )
}

/// Número de divisões principais do gráfico (marcas em cada escala).
const CHART_Y_DIVS: usize = 4;
/// Subdivisões de cada divisão principal (3 ticks intermediários).
const CHART_Y_SUBDIVS: usize = 4;
/// Largura reservada para cada uma das três escalas à esquerda.
const CHART_AXIS_COL_W: f32 = 40.0;

/// Escala de uma série: topo "redondo" (1, 2 ou 5 × 10^n por divisão)
/// e casas decimais suficientes para os rótulos dos ticks intermediários.
struct AxisScale {
    max: f32,
    decimals: usize,
}

/// Menor número de casas decimais que representa `step` exatamente (até 4).
fn decimals_for(step: f32) -> usize {
    (0..=4)
        .find(|&d| {
            let x = step * 10f32.powi(d as i32);
            (x - x.round()).abs() < 1e-3 * x.max(1.0)
        })
        .unwrap_or(4)
}

impl AxisScale {
    fn for_peak(peak: f32, min_step: f32) -> Self {
        let raw_step = (peak / CHART_Y_DIVS as f32).max(min_step);
        let magnitude = 10f32.powf(raw_step.log10().floor());
        let step = [1.0, 2.0, 5.0, 10.0]
            .into_iter()
            .map(|m| m * magnitude)
            .find(|&s| s >= raw_step * 0.999)
            .unwrap_or(10.0 * magnitude);
        let decimals = decimals_for(step / CHART_Y_SUBDIVS as f32);
        Self {
            max: step * CHART_Y_DIVS as f32,
            decimals,
        }
    }
}

/// Desenha as três séries do histórico, cada uma com sua própria escala
/// (à esquerda, na cor da série), já que V, A e W têm faixas bem diferentes.
fn draw_history_chart(ui: &mut egui::Ui, history: &VecDeque<Sample>, height: f32) {
    let size = Vec2::new(ui.available_width(), height);
    let (rect, _resp) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 4.0, Color32::from_gray(12));

    let series: [(fn(&Sample) -> f32, Color32, &str, f32); 3] = [
        (|s| s.voltage, CHART_VOLTAGE_COLOR, "V", 0.1),
        (|s| s.current, CHART_CURRENT_COLOR, "A", 0.01),
        (|s| s.power, CHART_POWER_COLOR, "W", 0.1),
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

fn format_hhmmss(d: Duration) -> String {
    let s = d.as_secs();
    format!("{:02}:{:02}:{:02}", s / 3600, (s % 3600) / 60, s % 60)
}

fn serial_thread_main(
    port_name: String,
    mut poll_interval: Duration,
    tx: Sender<SerialEvent>,
    cmd_rx: Receiver<AppCommand>,
    ctx: egui::Context,
) {
    let baud_rate = 115200;

    let mut port = match serialport::new(port_name.as_str(), baud_rate)
        .timeout(Duration::from_millis(50))
        .open()
    {
        Ok(p) => p,
        Err(e) => {
            let _ = tx.send(SerialEvent::Error(format!(
                "Falha ao abrir {port_name}: {e}"
            )));
            ctx.request_repaint();
            return;
        }
    };

    let _ = port.write_data_terminal_ready(true);
    let _ = port.write_request_to_send(true);

    thread::sleep(Duration::from_secs(2));

    let mut power_supply = DPS150::new();
    for cmd in power_supply.init_command() {
        let _ = port.write_all(&cmd);
        thread::sleep(Duration::from_millis(100));
    }

    let _ = tx.send(SerialEvent::Connected);
    ctx.request_repaint();

    let mut serial_buf = vec![0u8; 2048];
    let mut last_poll = Instant::now();

    loop {
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                AppCommand::SetVoltage(v) => {
                    let _ = port.write_all(&power_supply.set_float_value(commands::VOLTAGE_SET, v));
                }
                AppCommand::SetCurrent(c) => {
                    let _ = port.write_all(&power_supply.set_float_value(commands::CURRENT_SET, c));
                }
                AppCommand::EnableOutput(en) => {
                    let _ = port.write_all(&power_supply.enable_output(en));
                }
                AppCommand::SaveProfile(n, v, c) => {
                    if let Some(vid) = DPS150::group_voltage_id(n) {
                        let _ = port.write_all(&power_supply.set_float_value(vid, v));
                    }
                    if let Some(cid) = DPS150::group_current_id(n) {
                        let _ = port.write_all(&power_supply.set_float_value(cid, c));
                    }
                }
                AppCommand::SetPollInterval(interval) => poll_interval = interval,
                AppCommand::Disconnect => {
                    let _ = port.write_all(&power_supply.lock_session());
                    return;
                }
            }
        }

        if let Ok(n) = port.read(&mut serial_buf)
            && n > 0
        {
            let updates = power_supply.push_serial_data(&serial_buf[..n]);
            if !updates.is_empty() {
                for update in updates {
                    let _ = tx.send(SerialEvent::Update(Box::new(update)));
                }
                ctx.request_repaint();
            }
        }

        if last_poll.elapsed() >= poll_interval {
            let _ = port.write_all(&power_supply.get_all());
            last_poll = Instant::now();
        }
    }
}

const WIDTH: f32 = 1000.0;
const HEIGHT: f32 = 480.0;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([WIDTH, HEIGHT]),
        ..Default::default()
    };

    eframe::run_native(
        "DPS150 Controller",
        options,
        Box::new(|_cc| Ok(Box::new(AppModel::new()))),
    )
}
