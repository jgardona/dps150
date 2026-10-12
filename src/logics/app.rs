//! Estado da aplicação e regras de negócio do controlador.

use super::{
    history::{History, Sample},
    protocol::DPSUpdate,
    session::{self, AppCommand, SerialEvent},
};
use std::{
    ops::RangeInclusive,
    sync::mpsc::{Receiver, Sender},
    time::{Duration, Instant},
};

/// Faixa aceita para o intervalo de leitura (ms).
pub const UPDATE_MS_RANGE: RangeInclusive<u64> = 100..=1000;
/// Intervalo mínimo entre atualizações da lista de portas enquanto ocioso.
pub const PORT_REFRESH_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// Valores de tensão/corrente em edição na interface.
#[derive(Debug, Default)]
pub struct SetpointEditor {
    pub vset_input: f32,
    pub cset_input: f32,
    /// Campos alterados e ainda não enviados ao aparelho.
    dirty: bool,
    editing_v: bool,
    editing_c: bool,
}

impl SetpointEditor {
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}

/// Tempo com a saída ligada: conta enquanto ligada e congela ao desligar.
#[derive(Debug, Default)]
struct OutputClock {
    run_start: Option<Instant>,
    frozen: Duration,
}

impl OutputClock {
    fn tick(&mut self, running: bool, now: Instant) {
        match (running, self.run_start) {
            (true, None) => {
                self.run_start = Some(now);
                self.frozen = Duration::ZERO;
            }
            (false, Some(start)) => {
                self.frozen = now.saturating_duration_since(start);
                self.run_start = None;
            }
            _ => {}
        }
    }

    fn elapsed(&self, now: Instant) -> Duration {
        match self.run_start {
            Some(start) => now.saturating_duration_since(start),
            None => self.frozen,
        }
    }
}

pub struct AppState {
    state: DPSUpdate,
    connection: ConnectionState,
    commands: Option<Sender<AppCommand>>,
    events: Option<Receiver<SerialEvent>>,

    available_ports: Vec<String>,
    selected_port: String,
    last_port_refresh: Option<Instant>,

    setpoint: SetpointEditor,
    /// Grupo de memória (1..6) selecionado; alvo do "Salvar".
    selected_profile: u8,

    clock: OutputClock,

    /// Intervalo (ms) entre leituras do dispositivo e amostras do gráfico.
    update_ms: u64,
    history: History,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            state: DPSUpdate::default(),
            connection: ConnectionState::Disconnected,
            commands: None,
            events: None,
            available_ports: Vec::new(),
            selected_port: String::new(),
            last_port_refresh: None,
            setpoint: SetpointEditor::default(),
            selected_profile: 1,
            clock: OutputClock::default(),
            update_ms: 100,
            history: History::new(),
        }
    }

    // --- Leitura do estado -------------------------------------------------

    pub fn device(&self) -> &DPSUpdate {
        &self.state
    }

    pub fn connection(&self) -> &ConnectionState {
        &self.connection
    }

    pub fn is_connected(&self) -> bool {
        matches!(self.connection, ConnectionState::Connected)
    }

    /// Sem sessão: a escolha de porta fica liberada e a lista é atualizada.
    pub fn is_idle(&self) -> bool {
        matches!(
            self.connection,
            ConnectionState::Disconnected | ConnectionState::Error(_)
        )
    }

    /// output_opened == Some(true) significa saída DESLIGADA (circuito aberto).
    pub fn output_on(&self) -> bool {
        self.state.output_opened == Some(false)
    }

    pub fn available_ports(&self) -> &[String] {
        &self.available_ports
    }

    pub fn selected_port(&self) -> &str {
        &self.selected_port
    }

    pub fn selected_port_mut(&mut self) -> &mut String {
        &mut self.selected_port
    }

    pub fn setpoint(&self) -> &SetpointEditor {
        &self.setpoint
    }

    pub fn setpoint_mut(&mut self) -> &mut SetpointEditor {
        &mut self.setpoint
    }

    pub fn selected_profile(&self) -> u8 {
        self.selected_profile
    }

    /// Valores V/I salvos no grupo de memória `n` (1..6), se ambos disponíveis.
    pub fn group_values(&self, n: u8) -> Option<(f32, f32)> {
        let s = &self.state;
        match n {
            1 => Some((s.g1_vset?, s.g1_cset?)),
            2 => Some((s.g2_vset?, s.g2_cset?)),
            3 => Some((s.g3_vset?, s.g3_cset?)),
            4 => Some((s.g4_vset?, s.g4_cset?)),
            5 => Some((s.g5_vset?, s.g5_cset?)),
            6 => Some((s.g6_vset?, s.g6_cset?)),
            _ => None,
        }
    }

    pub fn history(&self) -> &History {
        &self.history
    }

    pub fn update_ms(&self) -> u64 {
        self.update_ms
    }

    pub fn update_interval(&self) -> Duration {
        Duration::from_millis(self.update_ms)
    }

    pub fn current_elapsed(&self, now: Instant) -> Duration {
        self.clock.elapsed(now)
    }

    // --- Ciclo por quadro --------------------------------------------------

    /// Atualiza a lista de portas, no máximo a cada [`PORT_REFRESH_INTERVAL`] e só sem sessão.
    pub fn refresh_ports_if_due(&mut self, now: Instant) {
        if !self.is_idle() {
            return;
        }
        if self
            .last_port_refresh
            .is_some_and(|t| now.saturating_duration_since(t) < PORT_REFRESH_INTERVAL)
        {
            return;
        }
        self.set_available_ports(session::available_ports());
        self.last_port_refresh = Some(now);
    }

    /// Processa os eventos pendentes da sessão serial.
    pub fn poll_events(&mut self) {
        let Some(rx) = &self.events else {
            return;
        };
        let events: Vec<SerialEvent> = rx.try_iter().collect();
        for evt in events {
            match evt {
                SerialEvent::Update(update) => {
                    self.merge(*update);
                    self.sync_edit_buffers();
                }
                SerialEvent::Connected => self.connection = ConnectionState::Connected,
                SerialEvent::Error(msg) => {
                    self.connection = ConnectionState::Error(msg);
                    self.end_session();
                }
            }
        }
    }

    /// Avança o relógio de saída e, conectado, grava uma amostra no histórico a cada `update_ms`.
    pub fn tick(&mut self, now: Instant) {
        self.clock.tick(self.output_on(), now);
        if !self.is_connected() {
            return;
        }
        let sample = Sample {
            voltage: self.state.output_voltage.unwrap_or(0.0),
            current: self.state.output_current.unwrap_or(0.0),
            power: self.state.output_power.unwrap_or(0.0),
        };
        self.history
            .record_if_due(now, self.update_interval(), sample);
    }

    // --- Ações do usuário --------------------------------------------------

    pub fn connect(&mut self, notify: impl Fn() + Send + 'static) {
        if self.selected_port.is_empty() {
            self.connection = ConnectionState::Error(
                "Nenhum dispositivo/porta serial disponível para conexão.".to_owned(),
            );
            return;
        }
        let (commands, events) =
            session::spawn_session(self.selected_port.clone(), self.update_interval(), notify);
        self.commands = Some(commands);
        self.events = Some(events);
        self.connection = ConnectionState::Connecting;
        self.reset_telemetry();
    }

    /// Encerra a sessão (ou cancela a conexão em andamento).
    pub fn disconnect(&mut self) {
        self.send(AppCommand::Disconnect);
        self.connection = ConnectionState::Disconnected;
        self.end_session();
    }

    pub fn mark_setpoint_edited(&mut self) {
        self.setpoint.dirty = true;
    }

    /// Informa se os campos de tensão/corrente estão em foco ou sendo arrastados.
    pub fn set_editing(&mut self, voltage: bool, current: bool) {
        self.setpoint.editing_v = voltage;
        self.setpoint.editing_c = current;
    }

    pub fn apply_setpoints(&mut self) {
        self.send(AppCommand::SetVoltage(self.setpoint.vset_input));
        self.send(AppCommand::SetCurrent(self.setpoint.cset_input));
        self.setpoint.dirty = false;
    }

    /// Descarta a edição e volta aos valores atuais do aparelho.
    pub fn discard_setpoint_edits(&mut self) {
        self.setpoint.dirty = false;
        self.sync_edit_buffers();
    }

    /// Carrega nos campos de edição os valores do grupo de memória `n` (1..6),
    /// sem enviá-los ao aparelho — o envio fica a cargo de "Aplicar".
    pub fn load_profile(&mut self, n: u8) {
        if let Some((v, c)) = self.group_values(n) {
            self.setpoint.vset_input = v;
            self.setpoint.cset_input = c;
            self.setpoint.dirty = true;
        }
        self.selected_profile = n;
    }

    /// Salva o Vset/Iset atualmente editado no grupo de memória `n` (1..6).
    pub fn save_profile(&mut self, n: u8) {
        self.send(AppCommand::SaveProfile(
            n,
            self.setpoint.vset_input,
            self.setpoint.cset_input,
        ));
        self.selected_profile = n;
    }

    pub fn toggle_output(&mut self) {
        self.send(AppCommand::EnableOutput(!self.output_on()));
    }

    pub fn set_update_ms(&mut self, ms: u64) {
        self.update_ms = ms.clamp(*UPDATE_MS_RANGE.start(), *UPDATE_MS_RANGE.end());
        self.send(AppCommand::SetPollInterval(self.update_interval()));
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    // --- Internos -----------------------------------------------------------

    /// Sem sessão ativa, o comando é descartado.
    fn send(&self, cmd: AppCommand) {
        if let Some(tx) = &self.commands {
            let _ = tx.send(cmd);
        }
    }

    fn end_session(&mut self) {
        self.commands = None;
        self.events = None;
        self.last_port_refresh = None;
        self.reset_telemetry();
    }

    fn reset_telemetry(&mut self) {
        self.state = DPSUpdate::default();
        self.clock = OutputClock::default();
        self.history.clear();
    }

    /// Se a porta selecionada sumiu, passa a ser a primeira disponível (ou nenhuma).
    fn set_available_ports(&mut self, ports: Vec<String>) {
        self.available_ports = ports;
        if !self.available_ports.contains(&self.selected_port) {
            self.selected_port = self.available_ports.first().cloned().unwrap_or_default();
        }
    }

    /// Campos presentes em `new` substituem os guardados; ausentes mantêm o último valor.
    fn merge(&mut self, new: DPSUpdate) {
        macro_rules! merge_opt {
            ($($field:ident),* $(,)?) => {
                $(
                    if new.$field.is_some() {
                        self.state.$field = new.$field;
                    }
                )*
            };
        }

        merge_opt!(
            input_voltage,
            output_voltage,
            output_current,
            output_power,
            temperature,
            model_name,
            protection_state,
            vset,
            cset,
            g1_vset,
            g1_cset,
            g2_vset,
            g2_cset,
            g3_vset,
            g3_cset,
            g4_vset,
            g4_cset,
            g5_vset,
            g5_cset,
            g6_vset,
            g6_cset,
            ovp,
            ocp,
            opp,
            otp,
            lvp,
            brightness,
            volume,
            metering,
            output_capacity,
            output_energy,
            cc_cv,
            upper_limit_voltage,
            upper_limit_current,
            firmware_version,
            hardware_version,
            output_opened,
        );
    }

    /// Sem edição pendente, os campos acompanham o Vset/Iset do aparelho.
    fn sync_edit_buffers(&mut self) {
        if self.setpoint.dirty {
            return;
        }
        if !self.setpoint.editing_v
            && let Some(v) = self.state.vset
        {
            self.setpoint.vset_input = v;
        }
        if !self.setpoint.editing_c
            && let Some(c) = self.state.cset
        {
            self.setpoint.cset_input = c;
        }
    }

    /// Estado conectado ligado a canais de teste, sem thread nem porta serial.
    #[cfg(test)]
    fn with_test_session() -> (Self, Receiver<AppCommand>, Sender<SerialEvent>) {
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();
        let (evt_tx, evt_rx) = std::sync::mpsc::channel();
        let mut app = Self::new();
        app.commands = Some(cmd_tx);
        app.events = Some(evt_rx);
        app.connection = ConnectionState::Connected;
        (app, cmd_rx, evt_tx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn update(f: impl FnOnce(&mut DPSUpdate)) -> SerialEvent {
        let mut u = DPSUpdate::default();
        f(&mut u);
        SerialEvent::Update(Box::new(u))
    }

    #[test]
    fn edicao_pendente_e_preservada_e_desfazer_sincroniza() {
        let (mut app, _cmds, events) = AppState::with_test_session();
        app.setpoint_mut().vset_input = 5.0;
        app.mark_setpoint_edited();

        events.send(update(|u| u.vset = Some(3.3))).unwrap();
        app.poll_events();
        assert_eq!(app.setpoint().vset_input, 5.0);
        assert!(app.setpoint().is_dirty());

        app.discard_setpoint_edits();
        assert_eq!(app.setpoint().vset_input, 3.3);
        assert!(!app.setpoint().is_dirty());
    }

    #[test]
    fn carregar_memoria_nao_envia_comandos() {
        let (mut app, cmds, events) = AppState::with_test_session();
        events
            .send(update(|u| {
                u.g2_vset = Some(12.0);
                u.g2_cset = Some(1.5);
            }))
            .unwrap();
        app.poll_events();

        app.load_profile(2);
        assert_eq!(app.setpoint().vset_input, 12.0);
        assert_eq!(app.setpoint().cset_input, 1.5);
        assert!(app.setpoint().is_dirty());
        assert_eq!(app.selected_profile(), 2);
        assert!(cmds.try_recv().is_err());
    }

    #[test]
    fn aplicar_e_salvar_enviam_os_valores_dos_campos() {
        let (mut app, cmds, _events) = AppState::with_test_session();
        app.setpoint_mut().vset_input = 5.0;
        app.setpoint_mut().cset_input = 0.2;
        app.mark_setpoint_edited();

        app.apply_setpoints();
        assert_eq!(cmds.try_recv(), Ok(AppCommand::SetVoltage(5.0)));
        assert_eq!(cmds.try_recv(), Ok(AppCommand::SetCurrent(0.2)));
        assert!(!app.setpoint().is_dirty());

        app.save_profile(3);
        assert_eq!(cmds.try_recv(), Ok(AppCommand::SaveProfile(3, 5.0, 0.2)));
        assert_eq!(app.selected_profile(), 3);
    }

    #[test]
    fn merge_parcial_mantem_campos_ausentes() {
        let (mut app, _cmds, events) = AppState::with_test_session();
        events.send(update(|u| u.temperature = Some(34.0))).unwrap();
        events
            .send(update(|u| u.output_voltage = Some(5.0)))
            .unwrap();
        app.poll_events();
        assert_eq!(app.device().temperature, Some(34.0));
        assert_eq!(app.device().output_voltage, Some(5.0));
    }

    #[test]
    fn relogio_conta_congela_e_zera() {
        let (mut app, _cmds, events) = AppState::with_test_session();
        let t0 = Instant::now();

        events
            .send(update(|u| u.output_opened = Some(false)))
            .unwrap();
        app.poll_events();
        app.tick(t0);
        assert_eq!(
            app.current_elapsed(t0 + Duration::from_secs(5)),
            Duration::from_secs(5)
        );

        events
            .send(update(|u| u.output_opened = Some(true)))
            .unwrap();
        app.poll_events();
        app.tick(t0 + Duration::from_secs(5));
        assert_eq!(
            app.current_elapsed(t0 + Duration::from_secs(60)),
            Duration::from_secs(5)
        );

        app.disconnect();
        assert_eq!(
            app.current_elapsed(t0 + Duration::from_secs(60)),
            Duration::ZERO
        );
    }

    #[test]
    fn historico_so_grava_conectado() {
        let mut app = AppState::new();
        app.tick(Instant::now());
        assert!(app.history().is_empty());

        let (mut app, _cmds, _events) = AppState::with_test_session();
        app.tick(Instant::now());
        assert_eq!(app.history().len(), 1);
    }

    #[test]
    fn erro_da_sessao_volta_ao_estado_ocioso() {
        let (mut app, _cmds, events) = AppState::with_test_session();
        app.tick(Instant::now());
        events.send(SerialEvent::Error("x".to_owned())).unwrap();
        app.poll_events();

        assert_eq!(app.connection(), &ConnectionState::Error("x".to_owned()));
        assert!(!app.is_connected());
        assert!(app.is_idle());
        assert!(app.history().is_empty());
    }

    #[test]
    fn porta_selecionada_some_da_lista() {
        let mut app = AppState::new();
        *app.selected_port_mut() = "/dev/ttyACM1".to_owned();
        app.set_available_ports(vec!["/dev/ttyACM0".to_owned()]);
        assert_eq!(app.selected_port(), "/dev/ttyACM0");
        app.set_available_ports(Vec::new());
        assert_eq!(app.selected_port(), "");
    }

    #[test]
    fn intervalo_de_leitura_limitado_e_enviado() {
        let (mut app, cmds, _events) = AppState::with_test_session();
        app.set_update_ms(5000);
        assert_eq!(app.update_ms(), 1000);
        assert_eq!(
            cmds.try_recv(),
            Ok(AppCommand::SetPollInterval(Duration::from_millis(1000)))
        );
    }
}
