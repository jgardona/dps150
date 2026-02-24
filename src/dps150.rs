pub mod consts {
    pub const HEADER_INPUT: u8 = 0xf0; // 240
    pub const HEADER_OUTPUT: u8 = 0xf1; // 241
    pub const CMD_GET: u8 = 0xa1; // 161
    pub const CMD_XXX_176: u8 = 0xb0; // 176 
    pub const CMD_SET: u8 = 0xb1; // 177
    pub const CMD_XXX_192: u8 = 0xc0; // 192
    pub const CMD_XXX_193: u8 = 0xc1; // 193

    // float
    pub const VOLTAGE_SET: u8 = 193;
    pub const CURRENT_SET: u8 = 194;
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

    // Proteções
    pub const OVP: u8 = 209;
    pub const OCP: u8 = 210;
    pub const OPP: u8 = 211;
    pub const OTP: u8 = 212;
    pub const LVP: u8 = 213;

    pub const METERING_ENABLE: u8 = 216;
    pub const OUTPUT_ENABLE: u8 = 219;

    // byte
    pub const BRIGHTNESS: u8 = 214;
    pub const VOLUME: u8 = 215;
    pub const MODEL_NAME: u8 = 222;
    pub const HARDWARE_VERSION: u8 = 223;
    pub const FIRMWARE_VERSION: u8 = 224;

    pub const ALL: u8 = 255;
}
use consts::*;

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

}

pub struct DPS150 {
    buffer: Vec<u8>,
}

impl DPS150 {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn init_command(&self) -> Vec<Vec<u8>> {
        vec![
            self.build_command(HEADER_OUTPUT, 0xc1, 0, &[1]), // Unlock
            self.build_command(HEADER_OUTPUT, 0xb0, 0, &[5]), // Baudrate
            self.build_command(HEADER_OUTPUT, CMD_GET, 222, &[]), // Model
            self.get_all(),                                   // Full Data
        ]
    }

    pub fn get_all(&self) -> Vec<u8> {
        self.build_command(HEADER_OUTPUT, CMD_GET, ALL, &[])
    }

    pub fn enable_output(&self, enable: bool) -> Vec<u8> {
        let val = if enable { 1 } else { 0 };
        self.build_command(HEADER_OUTPUT, CMD_SET, 219, &[val])
    }

    pub fn set_float_value(&self, type_id: u8, value: f32) -> Vec<u8> {
        let bytes = value.to_le_bytes();
        self.build_command(HEADER_OUTPUT, CMD_SET, type_id, &bytes)
    }

    fn build_command(&self, c1: u8, c2: u8, c3: u8, payload: &[u8]) -> Vec<u8> {
        let mut packet = Vec::with_capacity(5 + payload.len());
        packet.push(c1);
        packet.push(c2);
        packet.push(c3);
        packet.push(payload.len() as u8);
        let mut checksum = c3 as u32 + payload.len() as u32;
        for &byte in payload {
            packet.push(byte);
            checksum += byte as u32;
        }
        packet.push((checksum % 0x100) as u8);
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

                    if i + 5 + len > self.buffer.len() {
                        return updates; // Aguarda o resto do pacote
                    }

                    let received_chk = self.buffer[i + 4 + len];
                    let mut calc_chk = type_id as u32 + len as u32;
                    for j in 0..len {
                        calc_chk += self.buffer[i + 4 + j] as u32;
                    }

                    if (calc_chk % 256) == received_chk as u32 {
                        let payload = &self.buffer[i + 4..i + 4 + len];
                        if let Some(update) = Self::parse_data(type_id, payload) {
                            updates.push(update);
                        }
                        self.buffer.drain(0..i + 5 + len);
                        found_packet = true;
                        break;
                    }
                }
            }

            if !found_packet {
                if self.buffer.len() > 1024 {
                    self.buffer.clear();
                }
                break;
            }
        }
        updates
    }

    fn parse_data(type_id: u8, payload: &[u8]) -> Option<DPSUpdate> {
        let mut update = DPSUpdate::default();
        match type_id {
            192 => update.input_voltage = Some(Self::read_float(payload, 0)),
            195 => {
                update.output_voltage = Some(Self::read_float(payload, 0));
                update.output_current = Some(Self::read_float(payload, 4));
                update.output_power = Some(Self::read_float(payload, 8));
            }
            196 => update.temperature = Some(Self::read_float(payload, 0)), // Temp Interna.
            222 => update.model_name = Some(String::from_utf8_lossy(payload).into_owned()),
            255 if payload.len() >= 135 => {
                update.input_voltage = Some(Self::read_float(payload, 0));
                update.output_voltage = Some(Self::read_float(payload, 12));
                update.output_current = Some(Self::read_float(payload, 16));
            }
            _ => return None,
        }
        Some(update)
    }

    fn read_float(payload: &[u8], offset: usize) -> f32 {
        if offset + 4 > payload.len() {
            return 0.0;
        }
        let bytes: [u8; 4] = payload[offset..offset + 4].try_into().unwrap_or([0; 4]);
        f32::from_le_bytes(bytes)
    }
}
