pub mod commands {
    pub const PACKET_HEADER_SIZE: usize = 5; // bytes fixos antes do payload.
    pub const BUFFER_MAX_SIZE: usize = 1024; // tamanho do buffer do protocolo.
    pub const CHECKSUM_MODULUS: u32 = 0x100; // Modulo do checksum.

    pub const HEADER_INPUT: u8 = 0xf0; // 240
    pub const HEADER_OUTPUT: u8 = 0xf1; // 241
    pub const CMD_GET: u8 = 0xa1; // 161
    pub const CMD_BAUD: u8 = 0xb0; // 176
    pub const CMD_SET: u8 = 0xb1; // 177
    pub const CMD_XXX_192: u8 = 0xc0; // 192
    pub const CMD_SESSION: u8 = 0xc1; // 193

    // floatc) as
    pub const VOLTAGE_SET: u8 = 193;
    pub const CURRENT_SET: u8 = 194;
    pub const TEMPERATURE: u8 = 196;
    pub const GROUP1_VOLTAGE_SET: u8 = 197;
    pub const GROUP1_CURRENT_SET: u8 = 198;
    pub const GROUP2_VOLTAGE_SET: u8 = 199;
    pub const GROUP2_CURRENT_SET: u8 = 200;
    pub const GROUP3_VOLTAGE_SET: u8 = 201;
    pub const GROUP3_CURRENT_SET: u8 = 202;
    pub const GROUP4_VOLTAGE_SET: u8 = 203;
    pub const GROUP4_CURRENT_SET: u8 = 204;
    pub const GROUP5_VOLTAGE_SET: u8 = 205;
    pub const GROUP5_CURRENT_SET: u8 = 206;
    pub const GROUP6_VOLTAGE_SET: u8 = 207;
    pub const GROUP7_CURRENT_SET: u8 = 208;

    // Proteções.
    pub const OVP: u8 = 209;
    pub const OCP: u8 = 210;
    pub const OPP: u8 = 211;
    pub const OTP: u8 = 212;
    pub const LVP: u8 = 213;

    pub const METERING_ENABLE: u8 = 216;
    pub const OUTPUT_ENABLE: u8 = 219;
    pub const PROTECTION_STATE_DATA: u8 = 220; // tipos de proteção.
    // byte.
    pub const BRIGHTNESS: u8 = 214;
    pub const VOLUME: u8 = 215;
    pub const MODEL_NAME: u8 = 222;
    pub const HARDWARE_VERSION: u8 = 223;
    pub const FIRMWARE_VERSION: u8 = 224;

    // Baude rate.
    pub const BAUD_9600: u8 = 1;
    pub const BAUD_19200: u8 = 2;
    pub const BAUD_38400: u8 = 3;
    pub const BAUD_57600: u8 = 4;
    pub const BAUD_115200: u8 = 5;
    pub const ALL: u8 = 255;
}
use std::u8;

use commands::*;

#[derive(Debug, Default)]
pub struct DPSUpdate {
    pub input_voltage: Option<f32>,
    pub output_voltage: Option<f32>,
    pub output_current: Option<f32>,
    pub output_power: Option<f32>,
    pub temperature: Option<f32>,
    pub model_name: Option<String>,
    pub protection_state: Option<String>,
    pub vset: Option<f32>,
    pub cset: Option<f32>,
    pub g1_vset: Option<f32>,
    pub g1_cset: Option<f32>,
    pub g2_vset: Option<f32>,
    pub g2_cset: Option<f32>,
    pub g3_vset: Option<f32>,
    pub g3_cset: Option<f32>,
    pub g4_vset: Option<f32>,
    pub g4_cset: Option<f32>,
    pub g5_vset: Option<f32>,
    pub g5_cset: Option<f32>,
    pub g6_vset: Option<f32>,
    pub g6_cset: Option<f32>,
    pub ovp: Option<f32>,
    pub ocp: Option<f32>,
    pub opp: Option<f32>,
    pub otp: Option<f32>,
    pub lvp: Option<f32>,
    pub brightness: Option<u8>,
    pub volume: Option<u8>,
    pub metering: Option<u8>,
    pub output_capacity: Option<f32>,
    pub output_energy: Option<f32>,
    pub cc_cv: Option<String>, // cc ou cv.
    pub upper_limit_voltage: Option<f32>,
    pub upper_limit_current: Option<f32>,
    pub output_closed: bool,
    pub firmware_version: Option<String>,
    pub hardware_version: Option<String>,
}

const PROTECTION_STATE_LABELS: [&str; 7] = ["", "OVP", "OCP", "OPP", "OTP", "LVP", "REP"];
const CC_CV_LABELS: [&str; 2] = ["CC", "CV"];

pub struct DPS150 {
    buffer: Vec<u8>,
}

impl DPS150 {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn init_command(&self) -> Vec<Vec<u8>> {
        vec![
            self.build_command(HEADER_OUTPUT, CMD_SESSION, 0, &[1]), // Unlock.
            self.build_command(HEADER_OUTPUT, CMD_BAUD, 0, &[BAUD_115200]), // Baudrate rate.
            self.build_command(HEADER_OUTPUT, CMD_GET, TEMPERATURE, &[0]), // Temperature.
            self.build_command(HEADER_OUTPUT, CMD_GET, MODEL_NAME, &[0]), // Model name.
            self.build_command(HEADER_OUTPUT, CMD_GET, HARDWARE_VERSION, &[0]), // Get hardware version.
            self.build_command(HEADER_OUTPUT, CMD_GET, FIRMWARE_VERSION, &[0]),
            self.get_all(),
        ]
    }

    pub fn get_protection(&self) -> Vec<u8> {
        self.build_command(HEADER_OUTPUT, CMD_GET, PROTECTION_STATE_DATA, &[0])
    }

    pub fn get_all(&self) -> Vec<u8> {
        self.build_command(HEADER_OUTPUT, CMD_GET, ALL, &[0])
    }

    pub fn enable_output(&self, enable: bool) -> Vec<u8> {
        let val = if enable { 1 } else { 0 };
        self.build_command(HEADER_OUTPUT, CMD_SET, OUTPUT_ENABLE, &[val])
    }

    pub fn set_float_value(&self, type_id: u8, value: f32) -> Vec<u8> {
        let bytes = value.to_le_bytes();
        self.build_command(HEADER_OUTPUT, CMD_SET, type_id, &bytes)
    }

    fn build_command(&self, c1: u8, c2: u8, c3: u8, payload: &[u8]) -> Vec<u8> {
        let mut packet = Vec::with_capacity(PACKET_HEADER_SIZE + payload.len());
        packet.push(c1);
        packet.push(c2);
        packet.push(c3);
        packet.push(payload.len() as u8);
        let mut checksum = c3 as u32 + payload.len() as u32;
        for &byte in payload {
            packet.push(byte);
            checksum += byte as u32;
        }
        packet.push((checksum % CHECKSUM_MODULUS) as u8);
        packet
    }

    pub fn push_serial_data(&mut self, data: &[u8]) -> Vec<DPSUpdate> {
        self.buffer.extend_from_slice(data);
        self.process_buffer()
    }

    fn process_buffer(&mut self) -> Vec<DPSUpdate> {
        let mut updates = Vec::new();

        loop {
            if self.buffer.len() < 6 {
                break;
            } // Proteção contra pânico em buffers pequenos

            let mut found_packet = false;
            // Range exclusivo (..) evita tentar acessar index 0 em buffer vazio
            for i in 0..self.buffer.len().saturating_sub(5) {
                if self.buffer[i] == HEADER_INPUT && self.buffer[i + 1] == CMD_GET {
                    let type_id = self.buffer[i + 2];
                    let len = self.buffer[i + 3] as usize;

                    if i + PACKET_HEADER_SIZE + len > self.buffer.len() {
                        return updates; // Aguarda o resto do pacote
                    }

                    let received_chk = self.buffer[i + 4 + len];
                    let mut calc_chk = type_id as u32 + len as u32;
                    for j in 0..len {
                        calc_chk += self.buffer[i + 4 + j] as u32;
                    }

                    if (calc_chk % 256) == received_chk as u32 {
                        let payload = &self.buffer[i + 4..i + 4 + len];
                        if let Some(update) = self.parse_data(type_id, payload) {
                            updates.push(update);
                        }
                        self.buffer.drain(0..i + 5 + len);
                        found_packet = true;
                        break;
                    }
                }
            }

            if !found_packet {
                if self.buffer.len() > BUFFER_MAX_SIZE {
                    self.buffer.clear();
                }
                break;
            }
        }
        updates
    }

    fn parse_data(&self, type_id: u8, payload: &[u8]) -> Option<DPSUpdate> {
        let mut update = DPSUpdate::default();
        match type_id {
            192 => update.input_voltage = Some(self.read_float(payload, 0)),
            195 => {
                update.output_voltage = Some(self.read_float(payload, 0));
                update.output_current = Some(self.read_float(payload, 4));
                update.output_power = Some(self.read_float(payload, 8));
            }
            196 => update.temperature = Some(self.read_float(payload, 0)), // Temp Interna.
            217 => update.output_capacity = Some(self.read_float(payload, 0)),
            218 => update.output_energy = Some(self.read_float(payload, 0)),
            219 => update.output_closed = payload[0] == 1,
            220 => {
                update.protection_state =
                    Some(PROTECTION_STATE_LABELS[payload[0] as usize].to_owned())
            }
            221 => update.cc_cv = Some(CC_CV_LABELS[payload[0] as usize].to_owned()),
            222 => update.model_name = Some(String::from_utf8_lossy(payload).into_owned()),
            223 => update.hardware_version = Some(String::from_utf8_lossy(payload).into_owned()),
            224 => update.firmware_version = Some(String::from_utf8_lossy(payload).into_owned()),
            255 => {
                update.input_voltage = Some(self.read_float(payload, 0));
                update.vset = Some(self.read_float(payload, 4));
                update.cset = Some(self.read_float(payload, 8));
                update.output_voltage = Some(self.read_float(payload, 12));
                update.output_current = Some(self.read_float(payload, 16));
                update.output_power = Some(self.read_float(payload, 20));
                update.temperature = Some(self.read_float(payload, 24));
                update.g1_vset = Some(self.read_float(payload, 28));
                update.g1_cset = Some(self.read_float(payload, 32));
                update.g2_vset = Some(self.read_float(payload, 36));
                update.g2_cset = Some(self.read_float(payload, 40));
                update.g3_vset = Some(self.read_float(payload, 44));
                update.g3_cset = Some(self.read_float(payload, 48));
                update.g4_vset = Some(self.read_float(payload, 52));
                update.g4_cset = Some(self.read_float(payload, 56));
                update.g5_vset = Some(self.read_float(payload, 60));
                update.g5_cset = Some(self.read_float(payload, 64));
                update.g6_vset = Some(self.read_float(payload, 68));
                update.g6_cset = Some(self.read_float(payload, 72));
                update.ovp = Some(self.read_float(payload, 76));
                update.ocp = Some(self.read_float(payload, 80));
                update.opp = Some(self.read_float(payload, 84));
                update.otp = Some(self.read_float(payload, 88));
                update.lvp = Some(self.read_float(payload, 82))

                // TODO: precisa implementar todos os dados aqui.
            }
            226 => update.upper_limit_voltage = Some(self.read_float(payload, 0)),
            227 => update.upper_limit_current = Some(self.read_float(payload, 0)),
            _ => return None,
        }
        Some(update)
    }

    fn read_float(&self, payload: &[u8], offset: usize) -> f32 {
        if offset + 4 > payload.len() {
            return 0.0;
        }
        let bytes: [u8; 4] = payload[offset..offset + 4].try_into().unwrap_or([0; 4]);
        f32::from_le_bytes(bytes)
    }
}
