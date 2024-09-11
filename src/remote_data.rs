use crate::{
    command::{BufferType, Command},
    tracer_an::{Rated, Realtime, RealtimeStatus, Stats, VoltageSettings},
};
use std::{
    io::{ErrorKind, Read, Write},
    net::TcpStream,
};

#[derive(PartialEq, Debug, Clone, Default)]
pub enum RemoteData {
    #[default]
    NoData,
    BatteryVoltage(Vec<u16>),
    BatteryPackVoltage(Vec<u16>),
    PVVoltage(Vec<u16>),
    PVPower(Vec<u16>),
    VoltageBufferSize(usize),
    VoltageIntervalms(u16),
    PowerIntervalms(u16),
    Holdings(Vec<u8>),
    InputRegisters(Vec<u8>),
    Realtime(Realtime),
    RealtimeStatus(RealtimeStatus),
    VoltageSettings(VoltageSettings),
    Rated(Rated),
    Stats(Stats),
}

impl RemoteData {
    pub fn read_battery_voltage(tcp_stream: &mut TcpStream) -> std::io::Result<RemoteData> {
        tcp_stream.write_all(&Command::GetBuffer(BufferType::Battery1Voltage).to_bytes())?;
        let voltages = Self::read_buffer(tcp_stream)?;
        Ok(RemoteData::BatteryVoltage(voltages))
    }

    pub fn read_battery_pack_voltage(tcp_stream: &mut TcpStream) -> std::io::Result<RemoteData> {
        tcp_stream.write_all(&Command::GetBuffer(BufferType::BatteryPackVoltage).to_bytes())?;
        let voltages = Self::read_buffer(tcp_stream)?;
        Ok(RemoteData::BatteryPackVoltage(voltages))
    }

    pub fn read_pv_voltage(tcp_stream: &mut TcpStream) -> std::io::Result<RemoteData> {
        tcp_stream.write_all(&Command::GetBuffer(BufferType::PVVoltage).to_bytes())?;
        let voltages = Self::read_buffer(tcp_stream)?;
        Ok(RemoteData::PVVoltage(voltages))
    }

    pub fn read_pv_power(tcp_stream: &mut TcpStream) -> std::io::Result<RemoteData> {
        tcp_stream.write_all(&Command::GetBuffer(BufferType::PVPower).to_bytes())?;
        println!("getting power_data");
        let power_data = Self::read_buffer(tcp_stream)?;
        Ok(RemoteData::PVPower(power_data))
    }

    pub fn read_buffer(tcp_stream: &mut TcpStream) -> std::io::Result<Vec<u16>> {
        let mut size_buf = [0; 4];
        tcp_stream.read_exact(&mut size_buf)?;
        let buffer_size = u32::from_be_bytes(size_buf) as usize;
        if buffer_size == 0 {
            return Ok(Vec::new());
        }
        println!("buffer_size: {}", buffer_size);
        let mut buf = vec![0; buffer_size];
        tcp_stream.read_exact(&mut buf)?;
        Ok(bytemuck::cast_slice(&buf).to_vec())
    }

    pub fn read_interval_ms_voltage(tcp_stream: &mut TcpStream) -> std::io::Result<RemoteData> {
        let write_buf = Command::GetVoltageIntervalms.to_bytes();
        println!("tcp_stream.write({write_buf:?})");
        tcp_stream.write_all(&write_buf)?;
        let mut read_buf = [0; 2];
        println!("tcp_stream read GetIntervalms");
        tcp_stream.read_exact(&mut read_buf)?;
        println!("GetIntervalms :: {read_buf:?}");
        Ok(RemoteData::VoltageIntervalms(u16::from_be_bytes(read_buf)))
    }

    pub fn read_interval_ms_power(tcp_stream: &mut TcpStream) -> std::io::Result<RemoteData> {
        let write_buf = Command::GetPowerIntervalms.to_bytes();
        println!("tcp_stream.write({write_buf:?})");
        tcp_stream.write_all(&write_buf)?;
        let mut read_buf = [0; 2];
        println!("tcp_stream read GetIntervalms");
        tcp_stream.read_exact(&mut read_buf)?;
        println!("GetIntervalms :: {read_buf:?}");
        Ok(RemoteData::PowerIntervalms(u16::from_be_bytes(read_buf)))
    }

    pub fn read_voltage_buffer_size(tcp_stream: &mut TcpStream) -> std::io::Result<RemoteData> {
        let write_buf = Command::GetVoltageBufferSize.to_bytes();
        tcp_stream.write_all(&write_buf)?;
        let mut read_buf = [0; 4];
        tcp_stream.read_exact(&mut read_buf)?;
        Ok(RemoteData::VoltageBufferSize(
            u32::from_be_bytes(read_buf) as usize
        ))
    }

    pub fn get_holdings(
        tcp_stream: &mut TcpStream,
        command: Command,
    ) -> std::io::Result<RemoteData> {
        let write_buf = command.to_bytes();
        if let Command::ModbusGetHoldings {
            register_address: _,
            size,
        } = command
        {
            tcp_stream.write_all(&write_buf)?;
            let mut read_buf = vec![0; (size * 2) as usize];
            tcp_stream.read_exact(&mut read_buf)?;
            Ok(RemoteData::Holdings(read_buf))
        } else {
            Err(ErrorKind::InvalidInput.into())
        }
    }

    pub fn get_input_registers(
        tcp_stream: &mut TcpStream,
        command: Command,
    ) -> std::io::Result<RemoteData> {
        let write_buf = command.to_bytes();
        println!("Sending Command: {:?}", command);
        if let Command::ModbusGetInputRegisters {
            register_address: _,
            size,
        } = command
        {
            tcp_stream.write_all(&write_buf)?;
            let mut read_buf = vec![0; (size * 2) as usize];
            tcp_stream.read_exact(&mut read_buf)?;
            Ok(RemoteData::InputRegisters(read_buf))
        } else {
            Err(ErrorKind::InvalidInput.into())
        }
    }

    pub fn read_realtime(tcp_stream: &mut TcpStream) -> std::io::Result<RemoteData> {
        let mut write_buf;
        let mut bytes = Vec::new();
        for command in Realtime::generate_commands() {
            println!("read_realtime => sending command: {:?}", command);
            write_buf = command.to_bytes();
            println!("write_buf: {:?}", write_buf);
            tcp_stream.write_all(&write_buf)?;
            let mut read_buf = vec![0; (command.size() * 2) as usize];
            tcp_stream.read_exact(&mut read_buf)?;
            bytes.extend_from_slice(&read_buf[..]);
        }
        Ok(Self::Realtime(Realtime::from_bytes(&bytes)))
    }

    pub fn read_realtime_status(tcp_stream: &mut TcpStream) -> std::io::Result<RemoteData> {
        let command = RealtimeStatus::generate_command();
        tcp_stream.write_all(&command.to_bytes())?;
        let mut read_buf = vec![0; (command.size() * 2) as usize];
        tcp_stream.read_exact(&mut read_buf)?;
        Ok(Self::RealtimeStatus(RealtimeStatus::from_bytes(&read_buf)))
    }

    pub fn read_voltage_settings(tcp_stream: &mut TcpStream) -> std::io::Result<RemoteData> {
        let command = VoltageSettings::generate_get_command();
        tcp_stream.write_all(&command.to_bytes())?;
        let mut read_buf = vec![0; (command.size() * 2) as usize];
        tcp_stream.read_exact(&mut read_buf)?;
        Ok(Self::VoltageSettings(VoltageSettings::from_bytes(
            &read_buf,
        )))
    }

    pub fn read_rated(tcp_stream: &mut TcpStream) -> std::io::Result<RemoteData> {
        let mut write_buf;
        let mut bytes = Vec::new();
        for command in Rated::generate_commands() {
            write_buf = command.to_bytes();
            tcp_stream.write_all(&write_buf)?;
            let mut read_buf = vec![0; (command.size() * 2) as usize];
            tcp_stream.read_exact(&mut read_buf)?;
            bytes.extend_from_slice(&read_buf[..]);
        }
        Ok(Self::Rated(Rated::from_bytes(&bytes)))
    }

    pub fn read_stats(tcp_stream: &mut TcpStream) -> std::io::Result<RemoteData> {
        let mut write_buf;
        let commands = Stats::generate_get_commands();
        write_buf = commands[0].to_bytes();
        tcp_stream.write_all(&write_buf)?;
        let mut ev_buf = vec![0; (commands[0].size() * 2) as usize];
        tcp_stream.read_exact(&mut ev_buf)?;
        write_buf = commands[1].to_bytes();
        tcp_stream.write_all(&write_buf)?;
        let mut battery_buf = vec![0; (commands[1].size() * 2) as usize];
        tcp_stream.read_exact(&mut battery_buf)?;
        let mut energy_voltage_data: [u16; 20] = [0; 20];
        for (ix, chunk) in ev_buf.chunks(2).enumerate() {
            energy_voltage_data[ix] = u16::from_be_bytes([chunk[0], chunk[1]]);
        }
        let mut battery_data: [u16; 3] = [0; 3];
        for (ix, chunk) in battery_buf.chunks(2).enumerate() {
            battery_data[ix] = u16::from_be_bytes([chunk[0], chunk[1]]);
        }
        dbg!(commands);

        Ok(Self::Stats(Stats {
            energy_voltage_data,
            battery_data,
        }))
    }

    pub fn take_adc_readings(&mut self) -> Vec<u16> {
        let mut res = Vec::new();
        match self {
            RemoteData::NoData => {}
            RemoteData::BatteryVoltage(v) => res = std::mem::take(v),
            RemoteData::BatteryPackVoltage(v) => res = std::mem::take(v),
            RemoteData::PVVoltage(v) => res = std::mem::take(v),
            _ => {}
        }
        *self = RemoteData::NoData;
        res
    }
    pub fn take_power_readings(&mut self) -> Vec<u16> {
        let mut res = Vec::new();
        if let RemoteData::PVPower(v) = self {
            res = std::mem::take(v);
        }
        *self = RemoteData::NoData;
        res
    }
}
