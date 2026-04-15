use std::{
    io::{Read, Write},
    thread,
    time::Duration,
};

mod dps150;
use dps150::DPS150;

use crate::dps150::DPSUpdate;

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
        thread::sleep(Duration::from_millis(100));
    }

    // let pstate = power_supply.get_protection();
    // port.write_all(&pstate).unwrap();

    // 2. Configuração de Trabalho
    // let setup = [
    //     power_supply.set_float_value(VOLTAGE_SET, 5.33), // 5V
    //     power_supply.enable_output(false),        // Ligar
    // ];

    // for cmd in setup {
    //     port.write_all(&cmd).unwrap();
    //     thread::sleep(Duration::from_millis(50));
    // }

    port.flush().unwrap();
    println!("Monitoramento Ativo.");

    let mut serial_buf = vec![0u8; 2048];
    // let mut last_poll = Instant::now();

    // Polling a cada 1 segundo
    // if last_poll.elapsed() >= Duration::from_secs(1) {
    //     //let _ = port.write_all(&power_supply.get_all());
    //     let _ = port.flush();
    //     last_poll = Instant::now();
    // }

    // Pega a resposta da fonte, de acordo com o que foi pedido.
    loop {
        if let Ok(n) = port.read(&mut serial_buf)
            && n > 0
        {
            let updates = power_supply.push_serial_data(&serial_buf[..n]);
            for state in updates {
                select_data_to_print(state);
            }
        }
    }
}

fn select_data_to_print(state: DPSUpdate) {
    if let Some(model) = state.model_name.as_ref() {
        println!("Model Name: {model}");
    }
    if let Some(pstate) = state.protection_state.as_ref() {
        println!("Protection State: {pstate}")
    }
    if let Some(v) = state.output_voltage {
        println!("V: {:.2}V", v);
    }
    if let Some(c) = state.output_current {
        println!("I: {:.2}A", c);
    }
    if let Some(t) = state.temperature {
        println!("Temperature: {:.3} graus celsius.", t);
    }
    if let Some(fv) = state.firmware_version {
        println!("The firmware version is {fv}.")
    }
    if let Some(hv) = state.hardware_version {
        println!("The hardware version is {hv}.")
    }
    if let Some(p) = state.output_power {
        println!("The dissipate power is {:.3}.", p);
    }
    if let Some(vset) = state.vset {
        println!("The vset is : {:.3}.", vset);
    }
    if let Some(cset) = state.cset {
        println!("The cset is : {:.3}.", cset);
    }
    if let Some(e) = state.output_energy {
        println!("The output energy is {:.3} Wh.", e);
    }
}
