//! Sessão de comunicação com a fonte, numa thread própria.

use super::{
    memory_setup::SetupValues,
    protocol::{DPS150, DPSUpdate, commands},
};
use std::{
    io::{Read, Write},
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

const BAUD_RATE: u32 = 115200;

/// Comando enviado da GUI para a thread serial.
#[derive(Debug, PartialEq)]
pub enum AppCommand {
    SetVoltage(f32),
    SetCurrent(f32),
    EnableOutput(bool),
    /// Grava OVP/OCP/OPP/OTP e Vset/Iset do grupo, e relê a fonte.
    SaveMemorySetup(SetupValues),
    SetPollInterval(Duration),
    Disconnect,
}

/// Evento enviado da thread serial para a GUI.
#[derive(Debug)]
pub enum SerialEvent {
    Update(Box<DPSUpdate>),
    Connected,
    Error(String),
}

/// Inicia a sessão em `port_name` numa thread própria e devolve os canais de comando e de eventos.
/// `notify` é chamado sempre que houver evento novo, para a interface se redesenhar.
pub fn spawn_session(
    port_name: String,
    poll_interval: Duration,
    notify: impl Fn() + Send + 'static,
) -> (Sender<AppCommand>, Receiver<SerialEvent>) {
    let (evt_tx, evt_rx) = mpsc::channel::<SerialEvent>();
    let (cmd_tx, cmd_rx) = mpsc::channel::<AppCommand>();
    thread::spawn(move || session_main(port_name, poll_interval, evt_tx, cmd_rx, notify));
    (cmd_tx, evt_rx)
}

/// Nomes das portas seriais disponíveis (vazio em caso de erro).
pub fn available_ports() -> Vec<String> {
    serialport::available_ports()
        .map(|ports| ports.into_iter().map(|p| p.port_name).collect())
        .unwrap_or_default()
}

/// Pacotes que gravam o setup de uma memória, na ordem de envio: proteções (OVP, OCP, OPP, OTP),
/// Vset e Iset do grupo, e uma releitura (`GET ALL`) para a interface conferir a gravação.
///
/// A escrita das proteções é experimental: ainda não foi confirmada em aparelho real.
fn memory_setup_packets(power_supply: &DPS150, v: &SetupValues) -> Vec<Vec<u8>> {
    let mut packets = vec![
        power_supply.set_float_value(commands::OVP, v.ovp),
        power_supply.set_float_value(commands::OCP, v.ocp),
        power_supply.set_float_value(commands::OPP, v.opp),
        power_supply.set_float_value(commands::OTP, v.otp),
    ];
    if let Some(id) = DPS150::group_voltage_id(v.group) {
        packets.push(power_supply.set_float_value(id, v.vset));
    }
    if let Some(id) = DPS150::group_current_id(v.group) {
        packets.push(power_supply.set_float_value(id, v.iset));
    }
    packets.push(power_supply.get_all());
    packets
}

fn session_main(
    port_name: String,
    mut poll_interval: Duration,
    tx: Sender<SerialEvent>,
    cmd_rx: Receiver<AppCommand>,
    notify: impl Fn(),
) {
    let mut port = match serialport::new(port_name.as_str(), BAUD_RATE)
        .timeout(Duration::from_millis(50))
        .open()
    {
        Ok(p) => p,
        Err(e) => {
            let _ = tx.send(SerialEvent::Error(format!(
                "Falha ao abrir {port_name}: {e}"
            )));
            notify();
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
    notify();

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
                AppCommand::SaveMemorySetup(values) => {
                    for packet in memory_setup_packets(&power_supply, &values) {
                        let _ = port.write_all(&packet);
                        thread::sleep(Duration::from_millis(100));
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
                notify();
            }
        }

        if last_poll.elapsed() >= poll_interval {
            let _ = port.write_all(&power_supply.get_all());
            last_poll = Instant::now();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values(group: u8) -> SetupValues {
        SetupValues {
            group,
            vset: 5.0,
            iset: 0.5,
            ovp: 36.0,
            ocp: 2.5,
            opp: 295.2,
            otp: 60.0,
        }
    }

    #[test]
    fn pacotes_do_setup_seguem_a_ordem_do_contrato() {
        let ps = DPS150::new();
        let packets = memory_setup_packets(&ps, &values(2));
        let esperado = vec![
            ps.set_float_value(commands::OVP, 36.0),
            ps.set_float_value(commands::OCP, 2.5),
            ps.set_float_value(commands::OPP, 295.2),
            ps.set_float_value(commands::OTP, 60.0),
            ps.set_float_value(commands::GROUP2_VOLTAGE_SET, 5.0),
            ps.set_float_value(commands::GROUP2_CURRENT_SET, 0.5),
            ps.get_all(),
        ];
        assert_eq!(packets, esperado);
    }

    #[test]
    fn grupo_sem_type_id_nao_envia_vset_e_iset() {
        let ps = DPS150::new();
        let packets = memory_setup_packets(&ps, &values(9));
        assert_eq!(packets.len(), 5);
        assert_eq!(packets.last(), Some(&ps.get_all()));
    }
}
