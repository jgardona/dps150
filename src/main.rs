use std::{
    io::{Read, Write},
    thread,
    time::{Duration, Instant},
};

mod dps150;
use dps150::DPS150;

fn main() {
    let port_name = "/dev/ttyACM0";
    let baud_rate = 115200;

    let mut port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(100))
        .open()
        .expect("Erro ao abrir porta");

    // Configurações lógicas de sinal
    let _ = port.write_data_terminal_ready(true);
    let _ = port.write_request_to_send(true);

    println!("Aguardando estabilização...");
    thread::sleep(Duration::from_secs(2));

    let mut power_supply = DPS150::new();

    // 1. Sequência de Inicialização (Unlock, Baud, Model, Get All)
    println!("Enviando comandos iniciais...");
    for cmd in power_supply.init_command() {
        port.write_all(&cmd).unwrap();
        thread::sleep(Duration::from_millis(50));
    }

    // 2. Configuração de Trabalho
    let setup = [
        power_supply.set_float_value(193, 2.33), // 5V
        power_supply.enable_output(true),        // Ligar
    ];

    for cmd in setup {
        port.write_all(&cmd).unwrap();
        thread::sleep(Duration::from_millis(50));
    }

    port.flush().unwrap();
    println!("Monitoramento Ativo.");

    let mut serial_buf = vec![0u8; 2048];
    let mut last_poll = Instant::now();

    loop {
        // Polling a cada 1 segundo
        if last_poll.elapsed() >= Duration::from_secs(1) {
            let _ = port.write_all(&power_supply.get_all());
            let _ = port.flush();
            last_poll = Instant::now();
        }

        if let Ok(n) = port.read(&mut serial_buf) {
            if n > 0 {
                let updates = power_supply.push_serial_data(&serial_buf[..n]);
                for state in updates {
                    if let Some(v) = state.output_voltage {
                        let i = state.output_current.unwrap_or(0.0);
                        let temp = state.temperature.unwrap_or(0.0);
                        println!("V: {:.2}V | A: {:.2}A | Temp: {:.1}ºC", v, i, temp);
                    }
                    if let Some(model) = state.model_name {
                        println!("Conectado a: {}", model);
                    }
                }
            }
        }
    }
}
