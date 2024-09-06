use crate::command::Command;

pub struct Tracer {
    rated: Rated,
    realtime: Realtime,
    realtime_status: RealtimeStatus,
    stats: Stats,
    settings: Settings,
}

pub const RATED_BASE_ADDRESS: u16 = 0x3000;

#[derive(Debug, Copy, Clone)]
pub struct Rated {
    array_rated_voltage: f32,
    array_rated_current: f32,
    array_rated_power: f32,
    battery_rated_voltage: f32,
    battery_rated_current: f32,
    battery_rated_power: f32,
    charging_mode: ChargingMode,
    rated_current_load: f32,
}

#[derive(Debug, Copy, Clone)]
pub enum ChargingMode {
    ConnectDisconnect = 0x00,
    PWM,
    MPPT,
}

pub const REALTIME_BASE_ADDRESS: u16 = 0x3100;

#[derive(Debug, Copy, Clone)]
pub struct Realtime {
    pv_voltage: f32,
    pv_current: f32,
    pv_power: f32,
    battery_power: f32,
    load_voltage: f32,
    load_current: f32,
    load_power: f32,
    battery_temperature: f32,
    equipment_temperature: f32,
    remaining_battery_capacity: f32,
    remote_battery_temperature: f32,
    battery_real_rated_power: f32,
}

impl Realtime {
    fn from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() >= 60);
        let mut buf4: [u8; 4] = [0; 4];
        buf4.copy_from_slice(&bytes[4..8]);
        let pv_power = four_bytes_to_f32(buf4);
        buf4.copy_from_slice(&bytes[12..16]);
        let battery_power = four_bytes_to_f32(buf4);
        buf4.copy_from_slice(&bytes[28..32]);
        let load_power = four_bytes_to_f32(buf4);
        Realtime {
            pv_voltage: two_bytes_to_f32([bytes[0], bytes[1]]),
            pv_current: two_bytes_to_f32([bytes[2], bytes[3]]),
            pv_power,
            battery_power,
            load_voltage: two_bytes_to_f32([bytes[24], bytes[25]]),
            load_current: two_bytes_to_f32([bytes[26], bytes[27]]),
            load_power,
            battery_temperature: two_bytes_to_f32([bytes[32], bytes[33]]),
            equipment_temperature: two_bytes_to_f32([bytes[34], bytes[35]]),
            remaining_battery_capacity: two_bytes_to_f32([bytes[52], bytes[53]]),
            remote_battery_temperature: two_bytes_to_f32([bytes[54], bytes[55]]),
            battery_real_rated_power: two_bytes_to_f32([bytes[58], bytes[59]]),
        }
    }

    fn generate_command() -> Command {
        Command::ModbusGetInputRegisters {
            register_address: REALTIME_BASE_ADDRESS,
            size: 30,
        }
    }
}

// [b0, b1, b2, b3] => u32 => f32 => / 100
fn four_bytes_to_f32(bytes: [u8; 4]) -> f32 {
    let integer: u32 = u32::from_be_bytes(bytes);
    (integer as f32) / 100.0
}

// [b0, b1, b2, b3] => u16 => f32 => / 100
pub fn two_bytes_to_f32(bytes: [u8; 2]) -> f32 {
    let integer: u16 = u16::from_be_bytes(bytes);
    (integer as f32) / 100.0
}

pub const REALTIME_STATUS_BASE_ADDRESS: u16 = 0x3200;

#[derive(Debug, Copy, Clone)]
pub struct RealtimeStatus {
    battery_status: BatteryStatus,
    charging_equipment_status: ChargingEquipmentStatus,
    discharging_equipment_status: DischargingEquipmentStatus,
}

#[derive(Debug, Copy, Clone)]
pub enum BatteryStatus {}

#[derive(Debug, Copy, Clone)]
pub enum ChargingEquipmentStatus {}

#[derive(Debug, Copy, Clone)]
pub enum DischargingEquipmentStatus {}

pub const STATS_BASE_ADDRESS: u16 = 0x3300;

#[derive(Debug, Copy, Clone)]
pub struct Stats {}

pub const SETTINGS_BASE_ADDRESS: u16 = 0x9000;

#[derive(Debug, Copy, Clone)]
pub struct Settings {}
