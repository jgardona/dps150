mod dps150;

use dps150::{DPS150, DPSUpdate, commands};
use eframe::egui::{self, Color32, Pos2, Sense, Stroke, Vec2};
use std::{
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
}

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
        thread::spawn(move || serial_thread_main(port_name, evt_tx, cmd_rx, ctx));

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
                secondary_icon: SecondaryIcon::Lock,
            });
            ui.add_space(6.0);
            ui.add(ReadoutRow {
                icon: "A",
                accent: Color32::from_rgb(0, 200, 150),
                primary: format!("{:05.3} A", self.state.output_current.unwrap_or(0.0)),
                secondary_label: "Iset",
                secondary_value: format!("{:05.3} A", self.state.cset.unwrap_or(0.0)),
                secondary_icon: SecondaryIcon::Fan,
            });
            ui.add_space(6.0);
            ui.add(ReadoutRow {
                icon: "W",
                accent: Color32::from_rgb(60, 140, 255),
                primary: format!("{:05.2} W", self.state.output_power.unwrap_or(0.0)),
                secondary_label: "Temp",
                secondary_value: format!("{:.0}\u{b0}C", self.state.temperature.unwrap_or(0.0)),
                secondary_icon: SecondaryIcon::Thermometer,
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

        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::Panel::bottom("info_panel")
                .resizable(false)
                .min_size(60.0)
                .show_inside(ui, |ui| self.bottom_info_panel(ui));

            ui.columns(2, |cols| {
                self.left_readouts(&mut cols[0]);
                self.right_buttons(&mut cols[1]);
            });
        });

        ui.ctx().request_repaint_after(Duration::from_millis(100));
    }
}

enum SecondaryIcon {
    Lock,
    Fan,
    Thermometer,
}

struct ReadoutRow<'a> {
    icon: &'a str,
    accent: Color32,
    primary: String,
    secondary_label: &'a str,
    secondary_value: String,
    secondary_icon: SecondaryIcon,
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
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "{} {}",
                                self.secondary_label, self.secondary_value
                            ))
                            .size(13.0)
                            .weak(),
                        );
                        draw_secondary_icon(ui, self.secondary_icon);
                    });
                });
            })
            .response
    }
}

fn draw_secondary_icon(ui: &mut egui::Ui, icon: SecondaryIcon) {
    match icon {
        SecondaryIcon::Lock => draw_lock_icon(ui),
        SecondaryIcon::Fan => draw_fan_icon(ui),
        SecondaryIcon::Thermometer => draw_thermometer_icon(ui),
    }
}

fn draw_lock_icon(ui: &mut egui::Ui) {
    let size = Vec2::splat(14.0);
    let (rect, _resp) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter();
    let stroke = Stroke::new(1.3f32, Color32::LIGHT_GRAY);
    let shackle_center = Pos2::new(rect.center().x, rect.top() + 4.0);
    painter.circle_stroke(shackle_center, 3.5, stroke);
    let body = egui::Rect::from_min_size(
        Pos2::new(rect.left() + 1.0, rect.center().y),
        Vec2::new(rect.width() - 2.0, rect.height() / 2.0 - 1.0),
    );
    painter.rect_filled(body, 2.0, Color32::LIGHT_GRAY);
}

fn draw_fan_icon(ui: &mut egui::Ui) {
    let size = Vec2::splat(14.0);
    let (rect, _resp) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter();
    let stroke = Stroke::new(1.3f32, Color32::LIGHT_GRAY);
    let center = rect.center();
    let radius = rect.width() / 2.0 - 1.0;
    for i in 0..3 {
        let angle = std::f32::consts::FRAC_PI_2 + (i as f32) * std::f32::consts::TAU / 3.0;
        let tip = Pos2::new(
            center.x + radius * angle.cos(),
            center.y - radius * angle.sin(),
        );
        painter.line_segment([center, tip], stroke);
    }
    painter.circle_filled(center, 1.5, Color32::LIGHT_GRAY);
}

fn draw_thermometer_icon(ui: &mut egui::Ui) {
    let size = Vec2::splat(14.0);
    let (rect, _resp) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter();
    let stem = egui::Rect::from_min_size(
        Pos2::new(rect.center().x - 1.2, rect.top() + 1.0),
        Vec2::new(2.4, rect.height() - 5.0),
    );
    painter.rect_filled(stem, 1.0, Color32::LIGHT_GRAY);
    painter.circle_filled(
        Pos2::new(rect.center().x, rect.bottom() - 2.5),
        3.0,
        Color32::from_rgb(220, 60, 60),
    );
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

fn format_hhmmss(d: Duration) -> String {
    let s = d.as_secs();
    format!("{:02}:{:02}:{:02}", s / 3600, (s % 3600) / 60, s % 60)
}

fn serial_thread_main(
    port_name: String,
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
    let poll_interval = Duration::from_millis(100);

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

const WIDTH: f32 = 560.0;
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
