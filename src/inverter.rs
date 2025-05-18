use std::fmt::Display;

use crate::{
    command::Command,
    tracer_an::{four_bytes_to_f32, two_bytes_to_f32},
};

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct Inverter {
    pub registers: InverterRegisters,
    pub load_status: LoadStatus,
    pub holdings: InverterHoldings,
    // these holding registers are separate (not enough to warrant a new struct)
    pub output_ac_voltage: f32,
    pub output_ac_frequency: f32,
}

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct InverterRegisters {
    pub load_input_voltage: f32,
    pub load_input_current: f32,
    pub load_input_power: f32,
    pub load_output_voltage: f32,
    pub load_output_current: f32,
    pub load_output_power: f32,
    pub device_temperature: f32,
    pub heatsink_temperature: f32,
}

const INVERTER_REGISTERS_BASE_ADDRESS: u16 = 0x3108;

impl InverterRegisters {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() >= InverterRegisters::data_len());
        let mut buf4: [u8; 4] = [0; 4];
        buf4.copy_from_slice(&bytes[4..8]);
        let load_input_power = four_bytes_to_f32(buf4);
        buf4.copy_from_slice(&bytes[12..16]);
        let load_output_power = four_bytes_to_f32(buf4);
        InverterRegisters {
            load_input_voltage: two_bytes_to_f32([bytes[0], bytes[1]]),
            load_input_current: two_bytes_to_f32([bytes[2], bytes[3]]),
            load_input_power,
            load_output_voltage: two_bytes_to_f32([bytes[8], bytes[9]]),
            load_output_current: two_bytes_to_f32([bytes[10], bytes[11]]),
            load_output_power,
            device_temperature: two_bytes_to_f32([bytes[16], bytes[17]]),
            heatsink_temperature: two_bytes_to_f32([bytes[18], bytes[19]]),
        }
    }

    pub fn data_len() -> usize {
        20
    }
    pub fn generate_get_command() -> Command {
        Command::ModbusInverterGetInputRegisters {
            register_address: INVERTER_REGISTERS_BASE_ADDRESS,
            size: 10,
        }
    }
}

impl Display for InverterRegisters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Inverter Registers:")?;
        writeln!(f, "    load input voltage: {} V", self.load_input_voltage)?;
        writeln!(f, "    load input current: {} A", self.load_input_current)?;
        writeln!(f, "    load input power: {} W", self.load_input_power)?;
        writeln!(f, "    load output voltage: {} V", self.load_output_voltage)?;
        writeln!(f, "    load output current: {} A", self.load_output_current)?;
        writeln!(f, "    load output power: {} W", self.load_output_power)?;
        writeln!(f, "    device temperature: {} C", self.device_temperature)?;
        writeln!(
            f,
            "    heatsink temperature: {} C",
            self.heatsink_temperature
        )
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct LoadStatus(u16);

impl LoadStatus {
    pub fn is_output_fail(&self) -> bool {
        (self.0 >> 5) & 0x01 != 0
    }
    pub fn is_high_voltage_side_shorted(&self) -> bool {
        (self.0 >> 6) & 0x01 != 0
    }
    pub fn is_input_over_current(&self) -> bool {
        (self.0 >> 7) & 0x01 != 0
    }
    pub fn is_abnormal_output_voltage(&self) -> bool {
        (self.0 >> 8) & 0x01 != 0
    }
    pub fn is_unable_to_stop_discharging(&self) -> bool {
        (self.0 >> 9) & 0x01 != 0
    }
    pub fn is_unable_to_discharge(&self) -> bool {
        (self.0 >> 10) & 0x01 != 0
    }
    pub fn is_shorted(&self) -> bool {
        (self.0 >> 11) & 0x01 != 0
    }
    pub fn running_or_standby(&self) -> StandbyStatus {
        match self.0 & 0x01 {
            0x00 => StandbyStatus::Standby,
            0x01 => StandbyStatus::Running,
            _ => panic!("LoadStatus::running_or_standby: unexpected value"),
        }
    }
}

impl Display for LoadStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "LoadStatus:")?;
        match self.running_or_standby() {
            StandbyStatus::Standby => writeln!(f, "    device is on standby")?,
            StandbyStatus::Running => writeln!(f, "    device is running")?,
        }
        if self.is_output_fail() {
            writeln!(f, "    no output")?;
        }
        if self.is_high_voltage_side_shorted() {
            writeln!(f, "    high voltage side is shorted")?;
        }
        if self.is_input_over_current() {
            writeln!(f, "    input over current")?;
        }
        if self.is_abnormal_output_voltage() {
            writeln!(f, "    output voltage is abnormal")?;
        }
        if self.is_unable_to_stop_discharging() {
            writeln!(f, "    unable to stop discharging")?;
        }
        if self.is_unable_to_discharge() {
            writeln!(f, "    unable to discharge")?;
        }
        if self.is_shorted() {
            writeln!(f, "    (input maybe?) is shorted")?;
        }
        writeln!(
            f,
            "    load voltage status: {:?}",
            LoadVoltageStatus::from(*self)
        )?;
        writeln!(
            f,
            "    load output power status: {:?}",
            LoadOutputPowerStatus::from(*self)
        )
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub enum LoadVoltageStatus {
    #[default]
    Normal,
    Low,
    High,
    NoInputPower,
}

impl From<LoadStatus> for LoadVoltageStatus {
    fn from(LoadStatus(b): LoadStatus) -> Self {
        match (b >> 14) & 0b00000011 {
            0x00 => LoadVoltageStatus::Normal,
            0x01 => LoadVoltageStatus::Low,
            0x02 => LoadVoltageStatus::High,
            0x03 => LoadVoltageStatus::NoInputPower,
            _ => panic!("Could not convert from LoadStatus to LoadVoltageStatus"),
        }
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub enum LoadOutputPowerStatus {
    #[default]
    Light,
    Medium,
    Nominal,
    Overload,
}

impl From<LoadStatus> for LoadOutputPowerStatus {
    fn from(LoadStatus(b): LoadStatus) -> Self {
        match (b >> 12) & 0b00000011 {
            0x00 => LoadOutputPowerStatus::Light,
            0x01 => LoadOutputPowerStatus::Medium,
            0x02 => LoadOutputPowerStatus::Nominal,
            0x03 => LoadOutputPowerStatus::Overload,
            _ => panic!("Could not convert from LoadStatus to LoadOutputPowerStatus"),
        }
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub enum StandbyStatus {
    #[default]
    Standby,
    Running,
}

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct InverterHoldings {
    pub low_input_voltage: f32,
    pub low_input_voltage_5s: f32,
    pub low_input_voltage_recovery_voltage: f32,
    pub high_input_voltage_recovery_voltage: f32,
    pub high_input_voltage_5s: f32,
    pub high_input_voltage: f32,
    pub high_input_current: f32,
    pub high_input_current_recovery_voltage: f32,
}

pub const INVERTER_HOLDINGS_BASE_ADDRESS: u16 = 0x902F;

impl InverterHoldings {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() >= InverterHoldings::data_len());
        InverterHoldings {
            low_input_voltage: two_bytes_to_f32([bytes[0], bytes[1]]),
            low_input_voltage_5s: two_bytes_to_f32([bytes[2], bytes[3]]),
            low_input_voltage_recovery_voltage: two_bytes_to_f32([bytes[4], bytes[5]]),
            high_input_voltage_recovery_voltage: two_bytes_to_f32([bytes[6], bytes[7]]),
            high_input_voltage_5s: two_bytes_to_f32([bytes[8], bytes[9]]),
            high_input_voltage: two_bytes_to_f32([bytes[10], bytes[11]]),
            high_input_current: two_bytes_to_f32([bytes[12], bytes[13]]),
            high_input_current_recovery_voltage: two_bytes_to_f32([bytes[14], bytes[15]]),
        }
    }
    pub fn data_len() -> usize {
        16
    }
    pub fn generate_get_command() -> Command {
        Command::ModbusInverterGetHoldings {
            register_address: INVERTER_HOLDINGS_BASE_ADDRESS,
            size: 8,
        }
    }

    /// WARNING: this functionality has not been validated
    pub fn generate_set_command(&self) -> Command {
        let holding_values = [
            0, // the inverter does not allow low_input_voltage to be modified => default value
            (self.low_input_voltage_5s / 0.01) as u16,
            (self.low_input_voltage_recovery_voltage / 0.01) as u16,
            (self.high_input_voltage_recovery_voltage / 0.01) as u16,
            (self.high_input_voltage_5s / 0.01) as u16,
            0, // the inverter does not allow high_input_voltage to be modified => default value
            0, // the inverter does not allow high_input_current to be modified => default value
            0, // the inverter does not allow high_input_voltage_recovery_voltage to be modified =>
               // default value
        ];
        Command::ModbusInverterSetHoldings {
            register_address: INVERTER_HOLDINGS_BASE_ADDRESS,
            new_holding_values: holding_values,
        }
    }
}

impl Display for InverterHoldings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "InverterHoldings:")?;
        writeln!(f, "    low input voltage: {} V", self.low_input_voltage)?;
        writeln!(
            f,
            "    low input voltage 5s: {} V",
            self.low_input_voltage_5s
        )?;
        writeln!(
            f,
            "    low input voltage recovery voltage: {} V",
            self.low_input_voltage_recovery_voltage
        )?;
        writeln!(f, "    high input voltage: {} V", self.high_input_voltage)?;
        writeln!(
            f,
            "    high input voltage 5s: {} V",
            self.high_input_voltage_5s
        )?;
        writeln!(
            f,
            "    high input voltage recovery voltage: {} V",
            self.high_input_voltage_recovery_voltage
        )?;
        writeln!(f, "    high input current: {}", self.low_input_voltage_5s)?;
        writeln!(
            f,
            "    high input current recovery voltage: {} V",
            self.high_input_current_recovery_voltage
        )
    }
}
