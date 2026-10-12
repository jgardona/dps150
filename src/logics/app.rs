//! Estado da aplicação e regras de negócio do controlador.

use super::{
    history::{History, Sample},
    memory_setup::{MemorySetup, SetupError, SetupField, SetupValues},
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

/// Leituras completas (com proteções) que a conferência espera depois de salvar o setup.
const SAVE_VERIFY_READS: u8 = 3;

/// Situação da última gravação do setup de memória.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SaveStatus {
    Idle,
    /// Enviado; esperando uma leitura completa da fonte para comparar.
    Verifying {
        expected: SetupValues,
        reads_left: u8,
    },
    /// A fonte devolveu os valores enviados.
    Confirmed,
    /// As leituras não bateram com o que foi enviado.
    NotConfirmed,
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
    /// Setup (Vset, Iset e proteções) em edição no quadro da memória selecionada.
    memory_setup: MemorySetup,
    /// Memória pedida enquanto há edição pendente: espera o usuário confirmar o descarte.
    pending_switch: Option<u8>,
    save_status: SaveStatus,

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
            memory_setup: MemorySetup::default(),
            pending_switch: None,
            save_status: SaveStatus::Idle,
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

    /// Setup em edição no quadro da memória selecionada.
    pub fn memory_setup(&self) -> &MemorySetup {
        &self.memory_setup
    }

    pub fn save_status(&self) -> SaveStatus {
        self.save_status
    }

    /// Memória pedida que aguarda a confirmação de descarte da edição pendente.
    pub fn pending_switch(&self) -> Option<u8> {
        self.pending_switch
    }

    /// Valores V/I salvos no grupo de memória `n` (1..6), se ambos disponíveis.
    pub fn group_values(&self, n: u8) -> Option<(f32, f32)> {
        group_of(&self.state, n)
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
                    self.check_save_status(&update);
                    self.merge(*update);
                    self.sync_edit_buffers();
                    self.sync_memory_setup();
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

    /// Seleciona a memória `n` (1..6) e mostra os valores dela no quadro de setup, descartando
    /// qualquer edição pendente. Não envia nada ao aparelho.
    pub fn select_memory(&mut self, n: u8) {
        if !(1..=6).contains(&n) {
            return;
        }
        self.selected_profile = n;
        self.pending_switch = None;
        self.save_status = SaveStatus::Idle;
        self.memory_setup.clear_dirty();
        self.sync_memory_setup();
    }

    /// Pede a troca para a memória `n`. Com edição pendente, não troca: guarda o pedido para o
    /// usuário confirmar o descarte (`confirm_switch`) ou continuar editando (`cancel_switch`).
    pub fn request_select_memory(&mut self, n: u8) {
        if n == self.selected_profile {
            return;
        }
        if self.memory_setup.is_dirty() {
            self.pending_switch = Some(n);
        } else {
            self.select_memory(n);
        }
    }

    /// Confirma a troca pedida e descarta a edição pendente.
    pub fn confirm_switch(&mut self) {
        if let Some(n) = self.pending_switch.take() {
            self.select_memory(n);
        }
    }

    /// Mantém a edição e esquece o pedido de troca.
    pub fn cancel_switch(&mut self) {
        self.pending_switch = None;
    }

    /// Edição de um campo do quadro de setup.
    pub fn set_setup_field(&mut self, field: SetupField, value: f32) {
        self.memory_setup.set(field, value);
        self.save_status = SaveStatus::Idle;
    }

    /// Descarta a edição do quadro e volta aos valores da fonte.
    pub fn discard_memory_setup_edits(&mut self) {
        self.memory_setup.clear_dirty();
        self.pending_switch = None;
        self.sync_memory_setup();
    }

    /// Carrega Vset/Iset da memória `n` (1..6) no card Setpoint, sem enviá-los ao aparelho:
    /// o envio fica a cargo de "Aplicar".
    pub fn use_memory_in_setpoint(&mut self, n: u8) {
        if let Some((v, c)) = self.group_values(n) {
            self.setpoint.vset_input = v;
            self.setpoint.cset_input = c;
            self.setpoint.dirty = true;
        }
    }

    /// Valida e envia os seis valores do quadro para a memória selecionada.
    ///
    /// Sem conexão ou com algum valor inválido, nada é enviado e os erros voltam para a tela.
    /// A escrita de OVP/OCP/OPP/OTP é experimental (ainda não confirmada em aparelho real);
    /// por isso a gravação é conferida pela leitura seguinte (`check_save_status`).
    pub fn save_memory_setup(&mut self) -> Result<(), Vec<SetupError>> {
        if !self.is_connected() {
            return Err(Vec::new());
        }
        let values = self.memory_setup.validate(self.selected_profile)?;
        self.send(AppCommand::SaveMemorySetup(values));
        self.memory_setup.clear_dirty();
        self.pending_switch = None;
        self.save_status = SaveStatus::Verifying {
            expected: values,
            reads_left: SAVE_VERIFY_READS,
        };
        Ok(())
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
        self.memory_setup = MemorySetup::default();
        self.pending_switch = None;
        self.save_status = SaveStatus::Idle;
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

    /// Sem edição pendente, o quadro acompanha a fonte: Vset/Iset da memória selecionada e as
    /// proteções do aparelho. Durante a conferência de uma gravação ele não muda, para não voltar
    /// aos valores antigos antes de a fonte devolver os novos.
    fn sync_memory_setup(&mut self) {
        if self.memory_setup.is_dirty() || matches!(self.save_status, SaveStatus::Verifying { .. })
        {
            return;
        }
        let group = self.group_values(self.selected_profile);
        let s = &self.state;
        let values = [
            (SetupField::Vset, group.map(|g| g.0)),
            (SetupField::Iset, group.map(|g| g.1)),
            (SetupField::Ovp, s.ovp),
            (SetupField::Ocp, s.ocp),
            (SetupField::Opp, s.opp),
            (SetupField::Otp, s.otp),
        ];
        for (field, value) in values {
            self.memory_setup.load(field, value);
        }
    }

    /// Compara uma leitura completa (a que traz as proteções) com o que foi salvo.
    /// Leituras parciais não contam; depois de `SAVE_VERIFY_READS` leituras diferentes, desiste.
    fn check_save_status(&mut self, update: &DPSUpdate) {
        let SaveStatus::Verifying {
            expected,
            reads_left,
        } = self.save_status
        else {
            return;
        };
        if update.ovp.is_none() {
            return;
        }
        let group = group_of(update, expected.group);
        let read = |field: SetupField| match field {
            SetupField::Vset => group.map(|g| g.0),
            SetupField::Iset => group.map(|g| g.1),
            SetupField::Ovp => update.ovp,
            SetupField::Ocp => update.ocp,
            SetupField::Opp => update.opp,
            SetupField::Otp => update.otp,
        };
        let matches = SetupField::ALL
            .iter()
            .all(|&f| read(f).is_some_and(|v| (v - expected.get(f)).abs() <= f.tolerance()));
        self.save_status = if matches {
            SaveStatus::Confirmed
        } else if reads_left <= 1 {
            SaveStatus::NotConfirmed
        } else {
            SaveStatus::Verifying {
                expected,
                reads_left: reads_left - 1,
            }
        };
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

/// Vset/Iset do grupo de memória `n` (1..6) numa leitura, se ambos vieram.
fn group_of(update: &DPSUpdate, n: u8) -> Option<(f32, f32)> {
    match n {
        1 => Some((update.g1_vset?, update.g1_cset?)),
        2 => Some((update.g2_vset?, update.g2_cset?)),
        3 => Some((update.g3_vset?, update.g3_cset?)),
        4 => Some((update.g4_vset?, update.g4_cset?)),
        5 => Some((update.g5_vset?, update.g5_cset?)),
        6 => Some((update.g6_vset?, update.g6_cset?)),
        _ => None,
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

    /// Leitura completa de uma fonte com a memória 2 em 5 V / 0,5 A e as proteções do aparelho.
    fn full_read(f: impl FnOnce(&mut DPSUpdate)) -> SerialEvent {
        update(|u| {
            u.g2_vset = Some(5.0);
            u.g2_cset = Some(0.5);
            u.g3_vset = Some(12.0);
            u.g3_cset = Some(1.5);
            u.ovp = Some(36.0);
            u.ocp = Some(2.5);
            u.opp = Some(295.2);
            u.otp = Some(60.0);
            f(u);
        })
    }

    fn fill_valid(app: &mut AppState) {
        for (f, v) in [
            (SetupField::Vset, 5.0),
            (SetupField::Iset, 0.5),
            (SetupField::Ovp, 36.0),
            (SetupField::Ocp, 2.5),
            (SetupField::Opp, 295.2),
            (SetupField::Otp, 60.0),
        ] {
            app.set_setup_field(f, v);
        }
    }

    #[test]
    fn selecionar_memoria_mostra_os_seis_valores_sem_enviar_nada() {
        let (mut app, cmds, events) = AppState::with_test_session();
        events.send(full_read(|_| {})).unwrap();
        app.poll_events();

        app.select_memory(2);
        let m = app.memory_setup();
        assert_eq!(m.get(SetupField::Vset), Some(5.0));
        assert_eq!(m.get(SetupField::Iset), Some(0.5));
        assert_eq!(m.get(SetupField::Ovp), Some(36.0));
        assert_eq!(m.get(SetupField::Ocp), Some(2.5));
        assert_eq!(m.get(SetupField::Opp), Some(295.2));
        assert_eq!(m.get(SetupField::Otp), Some(60.0));
        assert_eq!(app.selected_profile(), 2);

        app.select_memory(3);
        assert_eq!(app.memory_setup().get(SetupField::Vset), Some(12.0));
        assert_eq!(app.memory_setup().get(SetupField::Ovp), Some(36.0));
        assert!(cmds.try_recv().is_err());
    }

    #[test]
    fn memoria_sem_dado_deixa_vset_e_iset_vazios() {
        let (mut app, _cmds, events) = AppState::with_test_session();
        events.send(full_read(|_| {})).unwrap();
        app.poll_events();
        app.select_memory(4);
        assert_eq!(app.memory_setup().get(SetupField::Vset), None);
        assert_eq!(app.memory_setup().get(SetupField::Iset), None);
        assert_eq!(app.memory_setup().get(SetupField::Otp), Some(60.0));
    }

    #[test]
    fn memoria_invalida_e_ignorada() {
        let mut app = AppState::new();
        app.select_memory(0);
        app.select_memory(7);
        assert_eq!(app.selected_profile(), 1);
    }

    #[test]
    fn usar_memoria_no_setpoint_nao_envia_comandos() {
        let (mut app, cmds, events) = AppState::with_test_session();
        events
            .send(update(|u| {
                u.g2_vset = Some(12.0);
                u.g2_cset = Some(1.5);
            }))
            .unwrap();
        app.poll_events();

        app.use_memory_in_setpoint(2);
        assert_eq!(app.setpoint().vset_input, 12.0);
        assert_eq!(app.setpoint().cset_input, 1.5);
        assert!(app.setpoint().is_dirty());
        assert!(cmds.try_recv().is_err());
    }

    #[test]
    fn edicao_do_setup_e_preservada_e_desfazer_volta_a_fonte() {
        let (mut app, cmds, events) = AppState::with_test_session();
        events.send(full_read(|_| {})).unwrap();
        app.poll_events();
        app.select_memory(2);

        app.set_setup_field(SetupField::Ovp, 30.0);
        assert!(app.memory_setup().is_dirty());
        events.send(full_read(|_| {})).unwrap();
        app.poll_events();
        assert_eq!(app.memory_setup().get(SetupField::Ovp), Some(30.0));

        app.discard_memory_setup_edits();
        assert!(!app.memory_setup().is_dirty());
        assert_eq!(app.memory_setup().get(SetupField::Ovp), Some(36.0));
        assert!(cmds.try_recv().is_err());
    }

    #[test]
    fn trocar_de_memoria_com_pendencia_pede_confirmacao() {
        let (mut app, cmds, events) = AppState::with_test_session();
        events.send(full_read(|_| {})).unwrap();
        app.poll_events();
        app.select_memory(2);
        app.set_setup_field(SetupField::Vset, 7.0);

        app.request_select_memory(3);
        assert_eq!(app.pending_switch(), Some(3));
        assert_eq!(app.selected_profile(), 2);
        assert_eq!(app.memory_setup().get(SetupField::Vset), Some(7.0));

        app.cancel_switch();
        assert_eq!(app.pending_switch(), None);
        assert!(app.memory_setup().is_dirty());

        app.request_select_memory(3);
        app.confirm_switch();
        assert_eq!(app.selected_profile(), 3);
        assert_eq!(app.memory_setup().get(SetupField::Vset), Some(12.0));
        assert!(!app.memory_setup().is_dirty());

        // Sem pendência, a troca é direta.
        app.request_select_memory(2);
        assert_eq!(app.selected_profile(), 2);
        assert_eq!(app.pending_switch(), None);
        assert!(cmds.try_recv().is_err());
    }

    #[test]
    fn salvar_valido_envia_um_comando_e_limpa_a_pendencia() {
        let (mut app, cmds, _events) = AppState::with_test_session();
        app.select_memory(2);
        fill_valid(&mut app);

        assert_eq!(app.save_memory_setup(), Ok(()));
        let esperado = SetupValues {
            group: 2,
            vset: 5.0,
            iset: 0.5,
            ovp: 36.0,
            ocp: 2.5,
            opp: 295.2,
            otp: 60.0,
        };
        assert_eq!(cmds.try_recv(), Ok(AppCommand::SaveMemorySetup(esperado)));
        assert!(cmds.try_recv().is_err());
        assert!(!app.memory_setup().is_dirty());
        assert!(matches!(app.save_status(), SaveStatus::Verifying { .. }));
    }

    #[test]
    fn salvar_invalido_ou_sem_conexao_nao_envia_nada() {
        let (mut app, cmds, _events) = AppState::with_test_session();
        fill_valid(&mut app);
        app.set_setup_field(SetupField::Vset, 12.0);
        app.set_setup_field(SetupField::Ovp, 10.0);
        assert_eq!(app.save_memory_setup(), Err(vec![SetupError::VsetAboveOvp]));
        assert!(cmds.try_recv().is_err());
        assert!(app.memory_setup().is_dirty());

        app.set_setup_field(SetupField::Vset, 36.5);
        assert_eq!(
            app.save_memory_setup(),
            Err(vec![SetupError::AboveMax(SetupField::Vset)])
        );
        assert!(cmds.try_recv().is_err());

        let mut offline = AppState::new();
        fill_valid(&mut offline);
        assert!(offline.save_memory_setup().is_err());
    }

    #[test]
    fn salvar_aceita_os_seis_maximos() {
        let (mut app, cmds, _events) = AppState::with_test_session();
        for f in SetupField::ALL {
            app.set_setup_field(f, f.max());
        }
        assert_eq!(app.save_memory_setup(), Ok(()));
        assert!(matches!(
            cmds.try_recv(),
            Ok(AppCommand::SaveMemorySetup(v)) if v.vset == 36.0 && v.iset == 8.2 && v.opp == 295.2
        ));
    }

    #[test]
    fn leitura_igual_confirma_a_gravacao() {
        let (mut app, _cmds, events) = AppState::with_test_session();
        app.select_memory(2);
        fill_valid(&mut app);
        app.save_memory_setup().unwrap();

        // Leitura parcial (sem proteções) não conta.
        events.send(update(|u| u.temperature = Some(30.0))).unwrap();
        app.poll_events();
        assert!(matches!(
            app.save_status(),
            SaveStatus::Verifying { reads_left: 3, .. }
        ));

        // Dentro da tolerância de meio passo (0,004 V < 0,005 V).
        events.send(full_read(|u| u.g2_vset = Some(5.004))).unwrap();
        app.poll_events();
        assert_eq!(app.save_status(), SaveStatus::Confirmed);
        assert_eq!(app.memory_setup().get(SetupField::Ovp), Some(36.0));
    }

    #[test]
    fn tres_leituras_diferentes_nao_confirmam() {
        let (mut app, _cmds, events) = AppState::with_test_session();
        app.select_memory(2);
        fill_valid(&mut app);
        app.save_memory_setup().unwrap();

        for _ in 0..3 {
            events.send(full_read(|u| u.ovp = Some(30.0))).unwrap();
            app.poll_events();
        }
        assert_eq!(app.save_status(), SaveStatus::NotConfirmed);
        // Sem pendência, o quadro volta a mostrar o que a fonte tem.
        assert_eq!(app.memory_setup().get(SetupField::Ovp), Some(30.0));
    }

    #[test]
    fn durante_a_conferencia_o_quadro_nao_volta_ao_valor_antigo() {
        let (mut app, _cmds, events) = AppState::with_test_session();
        events.send(full_read(|_| {})).unwrap();
        app.poll_events();
        app.select_memory(2);
        app.set_setup_field(SetupField::Ocp, 3.0);
        app.save_memory_setup().unwrap();

        events.send(full_read(|_| {})).unwrap(); // leitura antiga (ocp = 2,5)
        app.poll_events();
        assert_eq!(app.memory_setup().get(SetupField::Ocp), Some(3.0));
    }

    #[test]
    fn aplicar_envia_os_maximos_do_setpoint() {
        let (mut app, cmds, _events) = AppState::with_test_session();
        app.setpoint_mut().vset_input = 36.0;
        app.setpoint_mut().cset_input = 8.2;
        app.apply_setpoints();
        assert_eq!(cmds.try_recv(), Ok(AppCommand::SetVoltage(36.0)));
        assert_eq!(cmds.try_recv(), Ok(AppCommand::SetCurrent(8.2)));
    }

    #[test]
    fn aplicar_envia_os_valores_dos_campos() {
        let (mut app, cmds, _events) = AppState::with_test_session();
        app.setpoint_mut().vset_input = 5.0;
        app.setpoint_mut().cset_input = 0.2;
        app.mark_setpoint_edited();

        app.apply_setpoints();
        assert_eq!(cmds.try_recv(), Ok(AppCommand::SetVoltage(5.0)));
        assert_eq!(cmds.try_recv(), Ok(AppCommand::SetCurrent(0.2)));
        assert!(!app.setpoint().is_dirty());
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
