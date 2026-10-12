//! Sessão de comunicação com a fonte, numa thread própria.

use super::protocol::{DPS150, DPSUpdate, commands};
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
    SaveProfile(u8, f32, f32),
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
                notify();
            }
        }

        if last_poll.elapsed() >= poll_interval {
            let _ = port.write_all(&power_supply.get_all());
            last_poll = Instant::now();
        }
    }
}
