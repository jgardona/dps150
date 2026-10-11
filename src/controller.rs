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

    /// Campos de edição alterados e ainda não enviados ao aparelho.
    setpoint_dirty: bool,
    /// Grupo de memória (1..6) selecionado; alvo do "Salvar".
    selected_profile: u8,

    /// Intervalo (ms) entre leituras do dispositivo e amostras do gráfico.
    update_ms: u64,
    history: VecDeque<Sample>,
    last_sample: Option<Instant>,

    /// Cores de V, A e W (nesta ordem), escolhidas pela legenda do gráfico.
    series_colors: [Color32; 3],
}

/// Uma amostra de tensão/corrente/potência para o gráfico.
#[derive(Clone, Copy)]
struct Sample {
    voltage: f32,
    current: f32,
    power: f32,
}

const HISTORY_LEN: usize = 300;
/// Fração da largura ocupada pela coluna de controle (o resto é do gráfico).
const CONTROL_SHARE: f32 = 0.36;
const CONTROL_MIN_WIDTH: f32 = 330.0;
const CONTROL_MAX_WIDTH: f32 = 440.0;
const UPDATE_MS_RANGE: std::ops::RangeInclusive<u64> = 100..=1000;
/// Largura do seletor de cor da legenda (mesmo valor do color_edit_button do egui).
const COLOR_PICKER_WIDTH: f32 = 200.0;

// Cores padrão de V, A e W, usadas nas leituras, nos campos e no gráfico.
const DEFAULT_SERIES_COLORS: [Color32; 3] = [
    Color32::from_rgb(230, 60, 60),
    Color32::from_rgb(60, 200, 90),
    Color32::from_rgb(60, 140, 255),
];

const OK_COLOR: Color32 = Color32::from_rgb(60, 200, 90);
const WARN_COLOR: Color32 = Color32::from_rgb(230, 180, 40);
const ERROR_COLOR: Color32 = Color32::from_rgb(255, 90, 90);
const ACTION_COLOR: Color32 = Color32::from_rgb(60, 140, 255);

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
            setpoint_dirty: false,
            selected_profile: 1,
            update_ms: 100,
            history: VecDeque::with_capacity(HISTORY_LEN),
            last_sample: None,
            series_colors: DEFAULT_SERIES_COLORS,
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
        if self.setpoint_dirty {
            return;
        }
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

    fn apply_setpoints(&mut self) {
        self.send_cmd(AppCommand::SetVoltage(self.vset_input));
        self.send_cmd(AppCommand::SetCurrent(self.cset_input));
        self.setpoint_dirty = false;
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

    /// Carrega nos campos de edição os valores do grupo de memória `n` (1..6),
    /// sem enviá-los ao aparelho — o envio fica a cargo de "Aplicar".
    fn load_profile(&mut self, n: u8) {
        if let Some((v, c)) = self.group_values(n) {
            self.vset_input = v;
            self.cset_input = c;
            self.setpoint_dirty = true;
        }
        self.selected_profile = n;
    }

    /// Salva o Vset/Iset atualmente editado no grupo de memória `n` (1..6).
    fn save_profile(&mut self, n: u8) {
        self.send_cmd(AppCommand::SaveProfile(n, self.vset_input, self.cset_input));
        self.selected_profile = n;
    }

    fn is_connected(&self) -> bool {
        matches!(self.connection, ConnectionState::Connected)
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        let is_idle = matches!(
            self.connection,
            ConnectionState::Disconnected | ConnectionState::Error(_)
        );
        if is_idle {
            self.refresh_available_ports();
        }

        ui.horizontal(|ui| {
            let (status_color, status_text) = match &self.connection {
                ConnectionState::Disconnected => (Color32::from_gray(110), "Desconectado"),
                ConnectionState::Connecting => (WARN_COLOR, "Conectando..."),
                ConnectionState::Connected => (OK_COLOR, "Conectado"),
                ConnectionState::Error(_) => (ERROR_COLOR, "Erro"),
            };
            status_dot(ui, status_color).on_hover_text(status_text);

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
                            ui.selectable_value(&mut self.selected_port, port.clone(), port);
                        }
                    });
            });

            let label = match self.connection {
                ConnectionState::Disconnected | ConnectionState::Error(_) => "Conectar",
                ConnectionState::Connecting => "Cancelar",
                ConnectionState::Connected => "Desconectar",
            };
            if ui.button(label).clicked() {
                if is_idle {
                    self.connect(ui.ctx());
                } else {
                    self.disconnect();
                }
            }

            if let ConnectionState::Error(msg) = &self.connection {
                ui.colored_label(ERROR_COLOR, msg);
            }

            if !self.is_connected() {
                return;
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                info_pair(
                    ui,
                    "Temp",
                    &format!("{:.0}\u{b0}C", self.state.temperature.unwrap_or(0.0)),
                );
                ui.separator();
                info_pair(
                    ui,
                    "Entrada",
                    &format!("{:.2} V", self.state.input_voltage.unwrap_or(0.0)),
                );
                ui.separator();
                if let Some(fw) = &self.state.firmware_version {
                    info_pair(ui, "fw", fw);
                }
                if let Some(model) = &self.state.model_name {
                    ui.label(egui::RichText::new(model).strong());
                }
            });
        });
    }

    fn control_column(&mut self, ui: &mut egui::Ui) {
        self.readouts(ui);
        ui.add_space(8.0);
        self.setpoint_card(ui);
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            self.output_button(ui);
        });
    }

    fn readouts(&self, ui: &mut egui::Ui) {
        let mode = self.state.cc_cv.as_deref();
        ui.add(ReadoutRow {
            unit: "V",
            accent: self.series_colors[0],
            value: format!("{:05.2}", self.state.output_voltage.unwrap_or(0.0)),
            badge: (mode == Some("CV")).then_some("CV"),
        });
        ui.add_space(4.0);
        ui.add(ReadoutRow {
            unit: "A",
            accent: self.series_colors[1],
            value: format!("{:05.3}", self.state.output_current.unwrap_or(0.0)),
            badge: (mode == Some("CC")).then_some("CC"),
        });
        ui.add_space(4.0);
        ui.add(ReadoutRow {
            unit: "W",
            accent: self.series_colors[2],
            value: format!("{:05.2}", self.state.output_power.unwrap_or(0.0)),
            badge: None,
        });
    }

    fn setpoint_card(&mut self, ui: &mut egui::Ui) {
        card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Setpoint").strong());
                if self.setpoint_dirty {
                    ui.colored_label(WARN_COLOR, "\u{2022} não aplicado");
                }
            });

            let mut enter_pressed = false;
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("V")
                        .color(self.series_colors[0])
                        .strong(),
                );
                let v_resp = ui.add(
                    egui::DragValue::new(&mut self.vset_input)
                        .speed(0.05)
                        .range(0.0..=30.0)
                        .fixed_decimals(2)
                        .suffix(" V"),
                );
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("A")
                        .color(self.series_colors[1])
                        .strong(),
                );
                let c_resp = ui.add(
                    egui::DragValue::new(&mut self.cset_input)
                        .speed(0.01)
                        .range(0.0..=5.0)
                        .fixed_decimals(3)
                        .suffix(" A"),
                );

                if v_resp.changed() || c_resp.changed() {
                    self.setpoint_dirty = true;
                }
                self.editing_vset = v_resp.dragged() || v_resp.has_focus();
                self.editing_cset = c_resp.dragged() || c_resp.has_focus();
                enter_pressed = (v_resp.lost_focus() || c_resp.lost_focus())
                    && ui.input(|i| i.key_pressed(egui::Key::Enter));
            });

            ui.horizontal(|ui| {
                let apply = egui::Button::new(
                    egui::RichText::new("Aplicar")
                        .color(Color32::BLACK)
                        .strong(),
                )
                .fill(if self.setpoint_dirty {
                    ACTION_COLOR
                } else {
                    Color32::from_gray(90)
                });
                let clicked = ui
                    .add_enabled(self.is_connected(), apply)
                    .on_hover_text("Envia Vset/Iset ao aparelho (ou Enter)")
                    .clicked();
                if clicked || (enter_pressed && self.is_connected()) {
                    self.apply_setpoints();
                }
                if self.setpoint_dirty && ui.button("Desfazer").clicked() {
                    self.setpoint_dirty = false;
                    self.sync_edit_buffers_if_idle();
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Memória");
                for n in 1..=6u8 {
                    let hover = match self.group_values(n) {
                        Some((v, c)) => format!("M{n}: {v:.2} V / {c:.3} A — clique para carregar"),
                        None => format!("M{n}: sem dados"),
                    };
                    if ui
                        .selectable_label(self.selected_profile == n, format!("M{n}"))
                        .on_hover_text(hover)
                        .clicked()
                    {
                        self.load_profile(n);
                    }
                }
                let target = self.selected_profile;
                if ui
                    .add_enabled(self.is_connected(), egui::Button::new("Salvar"))
                    .on_hover_text(format!("Salva os valores dos campos em M{target}"))
                    .clicked()
                {
                    self.save_profile(target);
                }
            });
        });
    }

    fn output_button(&mut self, ui: &mut egui::Ui) {
        // output_opened == Some(true) significa saída DESLIGADA (circuito aberto).
        let output_on = self.state.output_opened == Some(false);
        let (label, fill, text_color, hover) = if output_on {
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
        let connected = self.is_connected();
        let resp = ui
            .add_enabled_ui(connected, |ui| ui.add_sized([w, 56.0], button))
            .inner;
        if resp.on_hover_text(hover).clicked() {
            self.send_cmd(AppCommand::EnableOutput(!output_on));
        }
    }

    fn chart_panel(&mut self, ui: &mut egui::Ui) {
        card(ui, |ui| {
            let last = self.history.back().copied();
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
                                .color(self.series_colors[i])
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
                                &mut self.series_colors[i],
                                egui::color_picker::Alpha::Opaque,
                            );
                            if ui.button("Padrão").clicked() {
                                self.series_colors[i] = DEFAULT_SERIES_COLORS[i];
                            }
                        });
                    ui.add_space(6.0);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let window = Duration::from_millis(self.update_ms * HISTORY_LEN as u64);
                    ui.label(
                        egui::RichText::new(format!("janela {}", format_window(window)))
                            .size(12.0)
                            .weak(),
                    );
                });
            });

            let chart_h = (ui.available_height() - 30.0).max(60.0);
            draw_history_chart(ui, &self.history, self.series_colors, chart_h);

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
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Limpar").clicked() {
                        self.history.clear();
                        self.last_sample = None;
                    }
                });
            });
        });
    }

    fn bottom_bar(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            info_pair(
                ui,
                "Capacidade",
                &format!("{:.3} Ah", self.state.output_capacity.unwrap_or(0.0)),
            );
            ui.separator();
            info_pair(
                ui,
                "Energia",
                &format!("{:.2} Wh", self.state.output_energy.unwrap_or(0.0)),
            );
            ui.separator();
            info_pair(ui, "Tempo", &format_hhmmss(self.current_elapsed()));
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
            egui::Panel::top("top_bar")
                .resizable(false)
                .show_inside(ui, |ui| self.top_bar(ui));
            egui::Panel::bottom("info_bar")
                .resizable(false)
                .show_inside(ui, |ui| self.bottom_bar(ui));

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

            self.control_column(&mut column_ui(ui, control_rect));
            self.chart_panel(&mut column_ui(ui, chart_rect));
            ui.advance_cursor_after_rect(full);
        });

        ui.ctx()
            .request_repaint_after(self.update_interval().min(Duration::from_millis(100)));
    }
}

/// Leitura grande de uma grandeza (V, A ou W) com selo opcional de modo (CV/CC).
struct ReadoutRow<'a> {
    unit: &'a str,
    accent: Color32,
    value: String,
    badge: Option<&'a str>,
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
fn card(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
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
fn info_pair(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.label(egui::RichText::new(label).weak());
    ui.label(egui::RichText::new(value).strong());
}

fn status_dot(ui: &mut egui::Ui, color: Color32) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(12.0), Sense::hover());
    ui.painter().circle_filled(rect.center(), 5.0, color);
    resp
}

/// Formata a duração da janela do gráfico ("30 s", "2.5 min").
fn format_window(d: Duration) -> String {
    let s = d.as_secs_f32();
    if s < 60.0 {
        format!("{s:.0} s")
    } else {
        format!("{:.1} min", s / 60.0).replace(".0 min", " min")
    }
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
fn draw_history_chart(
    ui: &mut egui::Ui,
    history: &VecDeque<Sample>,
    colors: [Color32; 3],
    height: f32,
) {
    let size = Vec2::new(ui.available_width(), height);
    let (rect, _resp) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 4.0, Color32::from_gray(12));

    let series: [(fn(&Sample) -> f32, Color32, &str, f32); 3] = [
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
const HEIGHT: f32 = 540.0;

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
