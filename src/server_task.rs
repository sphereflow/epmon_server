use crate::{
    command::{self, Command},
    remote_data::RemoteData,
};
use mpsc::{Receiver, SendError, Sender};
use std::{
    io::Write,
    net::{TcpListener, TcpStream},
    sync::*,
    thread,
    time::Duration,
};

pub fn server_task(
    connected: Arc<Mutex<bool>>,
    remote_data_sender: Sender<RemoteData>,
    server_message_receiver: Receiver<ServerMessage>,
) {
    if let Ok(tcp_listener) = TcpListener::bind("0.0.0.0:8900") {
        'outer: for result in tcp_listener.incoming() {
            let mut tcp_stream = result.expect("tcp_stream error");
            if let Ok(mut mgc) = connected.lock() {
                *mgc = true;
            }

            println!("connection established");
            if let Ok(intervalms) = RemoteData::read_interval_ms(&mut tcp_stream) {
                println!("Interval : {intervalms:?} ms");
                remote_data_sender.send(intervalms).unwrap();
            } else {
                println!("could not read intervalms => closing socket");
                if let Ok(mut mgc) = connected.lock() {
                    *mgc = false;
                }
                continue;
            }

            if let Ok(voltage_buffer_size) = RemoteData::read_voltage_buffer_size(&mut tcp_stream) {
                println!("voltage_buffer_size: {voltage_buffer_size:?}");
                remote_data_sender.send(voltage_buffer_size).unwrap();
            } else {
                println!("could not read voltage_buffer_size => closing socket");
                if let Ok(mut mgc) = connected.lock() {
                    *mgc = false;
                }
                continue;
            }
            let command_bytes = command::Command::RetransmitBuffers.to_bytes();
            if let Err(e) = tcp_stream.write_all(&command_bytes) {
                println!("Command::RetransmitBuffers failed Error: {e}");
                if let Ok(mut mgc) = connected.lock() {
                    *mgc = false;
                }
                continue;
            }
            // get buffers in a loop
            loop {
                if server_loop(
                    &remote_data_sender,
                    &server_message_receiver,
                    &mut tcp_stream,
                )
                .is_err()
                {
                    if let Ok(mut mgc) = connected.lock() {
                        *mgc = false;
                    }
                    continue 'outer;
                }
            }
        }
    }
}

fn server_loop(
    remote_data_sender: &Sender<RemoteData>,
    server_message_receiver: &Receiver<ServerMessage>,
    tcp_stream: &mut TcpStream,
) -> Result<(), ServerError> {
    let battery_voltage = RemoteData::read_battery_voltage(tcp_stream)?;
    remote_data_sender.send(battery_voltage)?;
    let battery_pack_voltage = RemoteData::read_battery_pack_voltage(tcp_stream)?;
    remote_data_sender.send(battery_pack_voltage)?;
    if let Ok(pv_voltage) = RemoteData::read_pv_voltage(tcp_stream) {
        remote_data_sender.send(pv_voltage)?;
    }
    while let Ok(message) = server_message_receiver.try_recv() {
        match message {
            ServerMessage::Command(command) => {
                serve_command(command, remote_data_sender, tcp_stream)?;
            }
            ServerMessage::ReadRealtime => {
                let remote_data = RemoteData::read_realtime(tcp_stream)?;
                remote_data_sender.send(remote_data)?;
            }
            ServerMessage::ReadRealtimeStatus => {
                let remote_data = RemoteData::read_realtime_status(tcp_stream)?;
                remote_data_sender.send(remote_data)?;
            }
            ServerMessage::ReadVoltageSettings => {
                let remote_data = RemoteData::read_voltage_settings(tcp_stream)?;
                remote_data_sender.send(remote_data)?;
            }
        }
    }

    thread::sleep(Duration::from_millis(500));
    Ok(())
}

pub fn serve_command(
    command: Command,
    remote_data_sender: &Sender<RemoteData>,
    tcp_stream: &mut TcpStream,
) -> Result<(), ServerError> {
    match command {
        command::Command::ModbusGetHoldings { .. } => {
            let val = RemoteData::get_holdings(tcp_stream, command)?;
            println!("holding val: {:?}", &val);
            remote_data_sender.send(val)?;
        }
        command::Command::ModbusGetInputRegisters { .. } => {
            println!("getting_input_registers");
            let val = RemoteData::get_input_registers(tcp_stream, command)?;
            println!("input reg val: {:?}", &val);
            remote_data_sender.send(val)?;
        }
        _ => {}
    }
    Ok(())
}

pub enum ServerError {
    IoError,
    SendError,
}

impl From<std::io::Error> for ServerError {
    fn from(_value: std::io::Error) -> Self {
        ServerError::IoError
    }
}

impl From<SendError<RemoteData>> for ServerError {
    fn from(_value: SendError<RemoteData>) -> Self {
        ServerError::SendError
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ServerMessage {
    Command(Command),
    ReadRealtime,
    ReadRealtimeStatus,
    ReadVoltageSettings,
}
