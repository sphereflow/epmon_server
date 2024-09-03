use std::{
    io::{ErrorKind, Read, Write},
    net::TcpStream,
};

use crate::command;

#[derive(PartialEq, Debug, Clone, Default)]
pub enum RemoteData {
    #[default]
    NoData,
    BatteryVoltage(Vec<u16>),
    BatteryPackVoltage(Vec<u16>),
    PVVoltage(Vec<u16>),
    VoltageBufferSize(usize),
    Intervalms(u16),
    Holdings(Vec<u8>),
    InputRegisters(Vec<u8>),
}

impl RemoteData {
    pub fn read_battery_voltage(tcp_stream: &mut TcpStream) -> std::io::Result<RemoteData> {
        tcp_stream
            .write_all(&command::Command::GetBattery1Buffer.to_bytes())
            .expect("could not write to tcp_stream");
        let voltages = Self::read_voltage(tcp_stream)?;
        Ok(RemoteData::BatteryVoltage(voltages))
    }

    pub fn read_battery_pack_voltage(tcp_stream: &mut TcpStream) -> std::io::Result<RemoteData> {
        tcp_stream
            .write_all(&command::Command::GetBatteryPackBuffer.to_bytes())
            .expect("could not write to tcp_stream");
        let voltages = Self::read_voltage(tcp_stream)?;
        Ok(RemoteData::BatteryPackVoltage(voltages))
    }

    pub fn read_pv_voltage(tcp_stream: &mut TcpStream) -> std::io::Result<RemoteData> {
        tcp_stream
            .write_all(&command::Command::GetPVBuffer.to_bytes())
            .expect("could not write to tcp_stream");
        let voltages = Self::read_voltage(tcp_stream)?;
        Ok(RemoteData::PVVoltage(voltages))
    }

    pub fn read_voltage(tcp_stream: &mut TcpStream) -> std::io::Result<Vec<u16>> {
        let mut size_buf = [0; 4];
        tcp_stream.read_exact(&mut size_buf)?;
        let voltage_buffer_size = u32::from_be_bytes(size_buf) as usize;
        let mut buf = vec![0; voltage_buffer_size];
        tcp_stream.read_exact(&mut buf)?;
        Ok(bytemuck::cast_slice(&buf).to_vec())
    }

    pub fn read_interval_ms(tcp_stream: &mut TcpStream) -> std::io::Result<RemoteData> {
        let write_buf = command::Command::GetIntervalms.to_bytes();
        println!("tcp_stream.write({write_buf:?})");
        tcp_stream.write_all(&write_buf)?;
        let mut read_buf = [0; 2];
        println!("tcp_stream read GetIntervalms");
        tcp_stream.read_exact(&mut read_buf)?;
        println!("GetIntervalms :: {read_buf:?}");
        Ok(RemoteData::Intervalms(u16::from_be_bytes(read_buf)))
    }

    pub fn read_voltage_buffer_size(tcp_stream: &mut TcpStream) -> std::io::Result<RemoteData> {
        let write_buf = command::Command::GetVoltageBufferSize.to_bytes();
        tcp_stream.write_all(&write_buf)?;
        let mut read_buf = [0; 4];
        tcp_stream.read_exact(&mut read_buf)?;
        Ok(RemoteData::VoltageBufferSize(
            u32::from_be_bytes(read_buf) as usize
        ))
    }

    pub fn get_holdings(
        tcp_stream: &mut TcpStream,
        command: command::Command,
    ) -> std::io::Result<RemoteData> {
        let write_buf = command.to_bytes();
        if let command::Command::ModbusGetHoldings {
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
        command: command::Command,
    ) -> std::io::Result<RemoteData> {
        let write_buf = command.to_bytes();
        if let command::Command::ModbusGetInputRegisters {
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
}
