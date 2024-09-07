use crate::command::Command;
use std::fmt::Display;

#[derive(Debug, Copy, Clone)]
pub struct Tracer {
    pub rated: Rated,
    pub realtime: Realtime,
    pub realtime_status: RealtimeStatus,
    pub stats: Stats,
    pub settings: Settings,
}

pub const RATED_BASE_ADDRESS: u16 = 0x3000;

#[derive(Default, Debug, Copy, Clone)]
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

impl Rated {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() >= Rated::data_len());
        Self {
            array_rated_voltage: two_bytes_to_f32([bytes[0], bytes[1]]),
            array_rated_current: two_bytes_to_f32([bytes[2], bytes[3]]),
            array_rated_power: four_bytes_to_f32([bytes[4], bytes[5], bytes[6], bytes[7]]),
            battery_rated_voltage: two_bytes_to_f32([bytes[8], bytes[9]]),
            battery_rated_current: two_bytes_to_f32([bytes[10], bytes[11]]),
            battery_rated_power: four_bytes_to_f32([bytes[12], bytes[13], bytes[14], bytes[15]]),
            charging_mode: match bytes[17] {
                0 => ChargingMode::ConnectDisconnect,
                1 => ChargingMode::PWM,
                _ => ChargingMode::MPPT,
            },
            rated_current_load: two_bytes_to_f32([bytes[18], bytes[19]]),
        }
    }

    pub fn generate_commands() -> [Command; 2] {
        [
            Command::ModbusGetInputRegisters {
                register_address: RATED_BASE_ADDRESS,
                size: 9,
            },
            Command::ModbusGetInputRegisters {
                register_address: RATED_BASE_ADDRESS + 0x0E,
                size: 1,
            },
        ]
    }

    pub fn data_len() -> usize {
        20
    }
}

impl Display for Rated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Rated: \n")?;
        writeln!(f, "rated pv voltage: {} V", self.array_rated_voltage)?;
        writeln!(f, "rated pv current: {} A", self.array_rated_current)?;
        writeln!(f, "rated pv power: {} W", self.array_rated_power)?;
        writeln!(f, "rated battery voltage: {} V", self.battery_rated_voltage)?;
        writeln!(f, "rated battery current: {} A", self.battery_rated_current)?;
        writeln!(f, "rated battery power: {} W", self.battery_rated_power)?;
        writeln!(f, "charging mode: {:?}", self.charging_mode)?;
        writeln!(f, "rated load current: {} A", self.rated_current_load)
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub enum ChargingMode {
    ConnectDisconnect = 0x00,
    PWM,
    #[default]
    MPPT,
}

pub const REALTIME_BASE_ADDRESS: u16 = 0x3100;

#[derive(Default, Debug, Copy, Clone)]
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
    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() >= Realtime::data_len());
        let mut buf4: [u8; 4] = [0; 4];
        buf4.copy_from_slice(&bytes[4..8]);
        let pv_power = four_bytes_to_f32(buf4);
        buf4.copy_from_slice(&bytes[8..12]);
        let battery_power = four_bytes_to_f32(buf4);
        buf4.copy_from_slice(&bytes[16..20]);
        let load_power = four_bytes_to_f32(buf4);
        Realtime {
            pv_voltage: two_bytes_to_f32([bytes[0], bytes[1]]),
            pv_current: two_bytes_to_f32([bytes[2], bytes[3]]),
            pv_power,
            battery_power,
            load_voltage: two_bytes_to_f32([bytes[12], bytes[13]]),
            load_current: two_bytes_to_f32([bytes[14], bytes[15]]),
            load_power,
            battery_temperature: two_bytes_to_f32([bytes[20], bytes[21]]),
            equipment_temperature: two_bytes_to_f32([bytes[22], bytes[23]]),
            remaining_battery_capacity: two_bytes_to_f32([bytes[24], bytes[25]]),
            remote_battery_temperature: two_bytes_to_f32([bytes[26], bytes[27]]),
            battery_real_rated_power: two_bytes_to_f32([bytes[28], bytes[29]]),
        }
    }

    pub fn data_len() -> usize {
        30
    }

    pub fn generate_commands() -> [Command; 5] {
        [
            Command::ModbusGetInputRegisters {
                register_address: REALTIME_BASE_ADDRESS,
                size: 4,
            },
            Command::ModbusGetInputRegisters {
                register_address: REALTIME_BASE_ADDRESS + 0x06,
                size: 2,
            },
            Command::ModbusGetInputRegisters {
                register_address: REALTIME_BASE_ADDRESS + 0x0C,
                size: 6,
            },
            Command::ModbusGetInputRegisters {
                register_address: REALTIME_BASE_ADDRESS + 0x1A,
                size: 2,
            },
            Command::ModbusGetInputRegisters {
                register_address: REALTIME_BASE_ADDRESS + 0x1D,
                size: 1,
            },
        ]
    }
}

impl Display for Realtime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Realtime: \n")?;
        writeln!(f, "pv voltage: {} V", self.pv_voltage)?;
        writeln!(f, "pv current: {} A", self.pv_current)?;
        writeln!(f, "pv power: {} W", self.pv_power)?;
        writeln!(f, "battery power: {} W", self.battery_power)?;
        writeln!(f, "load voltage: {} V", self.load_voltage)?;
        writeln!(f, "load current: {} A", self.load_current)?;
        writeln!(f, "load power: {} W", self.load_power)?;
        writeln!(f, "battery temperature: {} C", self.battery_temperature)?;
        writeln!(
            f,
            "remote battery temperature: {} C",
            self.remote_battery_temperature
        )?;
        writeln!(f, "equipment temperature: {} C", self.equipment_temperature)?;
        writeln!(
            f,
            "remaining battery capacity: {} %",
            self.remaining_battery_capacity
        )?;
        writeln!(
            f,
            "battery real rated power: {} V",
            self.battery_real_rated_power
        )
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

#[derive(Default, Debug, Copy, Clone)]
pub struct RealtimeStatus {
    battery_status: BatteryStatus,
    charging_equipment_status: ChargingEquipmentStatus,
    discharging_equipment_status: DischargingEquipmentStatus,
}

impl RealtimeStatus {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() >= Self::data_len());
        RealtimeStatus {
            battery_status: BatteryStatus(u16::from_be_bytes([bytes[0], bytes[1]])),
            charging_equipment_status: ChargingEquipmentStatus(u16::from_be_bytes([
                bytes[2], bytes[3],
            ])),
            discharging_equipment_status: DischargingEquipmentStatus(u16::from_be_bytes([
                bytes[4], bytes[5],
            ])),
        }
    }

    pub fn data_len() -> usize {
        6
    }

    pub fn generate_command() -> Command {
        Command::ModbusGetInputRegisters {
            register_address: REALTIME_STATUS_BASE_ADDRESS,
            size: 3,
        }
    }
}

impl Display for RealtimeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "realtime status:")?;
        write!(f, "{}", self.battery_status)?;
        write!(f, "{}", self.charging_equipment_status)?;
        write!(f, "{}", self.discharging_equipment_status)
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct BatteryStatus(u16);

impl BatteryStatus {
    pub fn is_inner_resistance_abnormal(&self) -> bool {
        (0b100000000 & self.0) != 0
    }

    pub fn is_wrong_rated_voltage(&self) -> bool {
        (0b1000_0000_0000_0000 & self.0) != 0
    }
}

impl Display for BatteryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "battery voltage status: {:?}",
            BatteryVoltageStatus::from(*self)
        )?;
        writeln!(
            f,
            "battery temperature status: {:?}",
            BatteryTemperatureStatus::from(*self)
        )?;
        if self.is_inner_resistance_abnormal() {
            writeln!(f, "inner resistance is abnormal")?;
        } else {
            writeln!(f, "inner resistance is normal")?;
        }
        if self.is_wrong_rated_voltage() {
            writeln!(
                f,
                "rated battery voltage differs from actual battery voltage"
            )
        } else {
            writeln!(
                f,
                "rated battery voltage and actual battery voltage are in line"
            )
        }
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub enum BatteryVoltageStatus {
    #[default]
    Normal,
    OverVolt,
    UnderVolt,
    LowVoltDisconnect,
    Fault,
}

impl From<BatteryStatus> for BatteryVoltageStatus {
    fn from(BatteryStatus(b): BatteryStatus) -> Self {
        match b & 0b00001111 {
            0x00 => BatteryVoltageStatus::Normal,
            0x01 => BatteryVoltageStatus::OverVolt,
            0x02 => BatteryVoltageStatus::UnderVolt,
            0x03 => BatteryVoltageStatus::LowVoltDisconnect,
            _ => BatteryVoltageStatus::Fault,
        }
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub enum BatteryTemperatureStatus {
    #[default]
    Normal,
    OverTemp,
    LowTemp,
}

impl From<BatteryStatus> for BatteryTemperatureStatus {
    fn from(BatteryStatus(b): BatteryStatus) -> Self {
        match (b >> 4) & 0b1111 {
            0x00 => BatteryTemperatureStatus::Normal,
            0x01 => BatteryTemperatureStatus::OverTemp,
            _ => BatteryTemperatureStatus::LowTemp,
        }
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct ChargingEquipmentStatus(u16);

impl ChargingEquipmentStatus {
    pub fn is_running(&self) -> bool {
        (1 & self.0) != 0
    }

    pub fn has_fault(&self) -> bool {
        (2 & self.0) != 0
    }

    pub fn is_pv_input_short(&self) -> bool {
        // is D4 (the 5fth bit) set
        (1 & (self.0 >> 4)) != 0
    }

    pub fn is_load_mosfet_short(&self) -> bool {
        // is D7 (the 8th bit) set
        (1 & (self.0 >> 7)) != 0
    }

    pub fn is_load_short(&self) -> bool {
        // is D8 (the 9th bit) set
        (1 & (self.0 >> 8)) != 0
    }

    pub fn is_load_over_current(&self) -> bool {
        // is D9 (the 10th bit) set
        (1 & (self.0) >> 9) != 0
    }

    pub fn is_input_over_current(&self) -> bool {
        // is D10 (the 11th bit) set
        (1 & (self.0) >> 10) != 0
    }

    pub fn is_anti_reverse_mosfet_short(&self) -> bool {
        // is D11 (the 12th bit) set
        (1 & (self.0) >> 11) != 0
    }

    pub fn is_charging_or_anti_reverse_mosfet_short(&self) -> bool {
        // is D12 (the 13th bit) set
        (1 & (self.0) >> 12) != 0
    }

    pub fn is_charging_mosfet_short(&self) -> bool {
        // is D13 (the 14th bit) set
        (1 & (self.0) >> 13) != 0
    }
}

impl Display for ChargingEquipmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "charging equipment status:")?;
        if self.is_running() {
            writeln!(f, "running")?;
        } else {
            writeln!(f, "on standby")?;
        }
        writeln!(f, "{:?}", ChargingStatus::from(*self))?;
        writeln!(f, "{:?}", InputVoltStatus::from(*self))?;
        if self.has_fault() {
            writeln!(f, "fault detected")?;
        }
        if self.is_pv_input_short() {
            writeln!(f, "pv input is shorted")?;
        }
        if self.is_load_mosfet_short() {
            writeln!(f, "load MOSFET is shorted")?;
        }
        if self.is_load_short() {
            writeln!(f, "load is shorted")?;
        }
        if self.is_load_over_current() {
            writeln!(f, "load overcurrent detected")?;
        }
        if self.is_input_over_current() {
            writeln!(f, "input overcurrent detected")?;
        }
        if self.is_anti_reverse_mosfet_short() {
            writeln!(f, "anti reverse MOSFET is shorted")?;
        }
        if self.is_charging_or_anti_reverse_mosfet_short() {
            writeln!(f, "charging or anti reverse MOSFET is shorted")?;
        }
        if self.is_charging_mosfet_short() {
            writeln!(f, "charging MOSFET is shorted")?;
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ChargingStatus {
    Off,
    Float,
    Boost,
    Equalization,
}

impl From<ChargingEquipmentStatus> for ChargingStatus {
    fn from(ChargingEquipmentStatus(val): ChargingEquipmentStatus) -> Self {
        match val >> 2 {
            0 => ChargingStatus::Off,
            1 => ChargingStatus::Float,
            2 => ChargingStatus::Boost,
            _ => ChargingStatus::Equalization,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum InputVoltStatus {
    Normal,
    NoPowerConnected,
    HigherVoltInput,
    Error,
}

impl From<ChargingEquipmentStatus> for InputVoltStatus {
    fn from(ChargingEquipmentStatus(val): ChargingEquipmentStatus) -> Self {
        match val >> 2 {
            0 => InputVoltStatus::Normal,
            1 => InputVoltStatus::NoPowerConnected,
            2 => InputVoltStatus::HigherVoltInput,
            _ => InputVoltStatus::Error,
        }
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct DischargingEquipmentStatus(u16);

impl DischargingEquipmentStatus {
    pub fn is_running(&self) -> bool {
        (1 & self.0) != 0
    }

    pub fn has_fault(&self) -> bool {
        (2 & self.0) != 0
    }

    pub fn is_output_overpressure(&self) -> bool {
        // is D4 (the 5fth bit) set
        (1 & (self.0 >> 4)) != 0
    }

    pub fn is_boost_overpressure(&self) -> bool {
        // is D5 (the 6th bit) set
        (1 & (self.0 >> 5)) != 0
    }

    pub fn is_high_voltage_side_short_circuit(&self) -> bool {
        // is D6 (the 7th bit) set
        (1 & (self.0 >> 6)) != 0
    }

    pub fn is_input_over_pressure(&self) -> bool {
        // is D7 (the 8th bit) set
        (1 & (self.0 >> 7)) != 0
    }

    pub fn is_output_voltage_abnormal(&self) -> bool {
        // is D8 (the 9th bit) set
        (1 & (self.0 >> 8)) != 0
    }

    pub fn is_unable_to_stop_discharging(&self) -> bool {
        // is D9 (the 10th bit) set
        (1 & (self.0) >> 9) != 0
    }

    pub fn is_unable_to_discharge(&self) -> bool {
        // is D10 (the 11th bit) set
        (1 & (self.0) >> 10) != 0
    }

    pub fn is_short_circuit(&self) -> bool {
        // is D11 (the 12th bit) set
        (1 & (self.0) >> 11) != 0
    }
}

impl Display for DischargingEquipmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "discharging equipment status")?;
        if self.is_running() {
            writeln!(f, "status: running")?;
        } else {
            writeln!(f, "status: on standby")?;
        }
        if self.has_fault() {
            writeln!(f, "fault detected")?;
        }
        if self.is_output_overpressure() {
            writeln!(f, "output overpressure")?;
        }
        if self.is_boost_overpressure() {
            writeln!(f, "boost overpressure")?;
        }
        if self.is_high_voltage_side_short_circuit() {
            writeln!(f, "high voltage side short circuit detected")?;
        }
        if self.is_input_over_pressure() {
            writeln!(f, "input overpressure")?;
        }
        if self.is_output_voltage_abnormal() {
            writeln!(f, "abnormal output voltage")?;
        }
        if self.is_unable_to_stop_discharging() {
            writeln!(f, "unable to stop discharging")?;
        }
        if self.is_unable_to_discharge() {
            writeln!(f, "unable to discharge")?;
        }
        if self.is_short_circuit() {
            writeln!(f, "short circuit detected")?;
        }
        writeln!(f, "output power status: {:?}", OutputPower::from(*self))?;
        writeln!(f, "discharge status: {:?}", DischargeStatus::from(*self))?;
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
pub enum OutputPower {
    LightLoad,
    Moderate,
    Rated,
    OverLoad,
}

impl From<DischargingEquipmentStatus> for OutputPower {
    fn from(DischargingEquipmentStatus(val): DischargingEquipmentStatus) -> Self {
        match (val >> 12) & 0b11 {
            0 => Self::LightLoad,
            1 => Self::Moderate,
            2 => Self::Rated,
            _ => Self::OverLoad,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum DischargeStatus {
    Normal,
    Low,
    High,
    NoAccessInputVoltError,
}

impl From<DischargingEquipmentStatus> for DischargeStatus {
    fn from(DischargingEquipmentStatus(val): DischargingEquipmentStatus) -> Self {
        match (val >> 12) & 0b11 {
            0 => Self::Normal,
            1 => Self::Low,
            2 => Self::High,
            _ => Self::NoAccessInputVoltError,
        }
    }
}

pub const STATS_BASE_ADDRESS: u16 = 0x3300;

#[derive(Default, Debug, Copy, Clone)]
pub struct Stats {}

pub const SETTINGS_BASE_ADDRESS: u16 = 0x9000;

#[derive(Debug, Copy, Clone)]
pub struct Settings {}
