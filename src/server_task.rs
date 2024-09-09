use crate::{
    command::{self, Command},
    remote_data::RemoteData,
    tracer_an::VoltageSettings,
};
use mpsc::{Receiver, SendError, Sender};
use std::{
    io::Write,
    net::{TcpListener, TcpStream},
    sync::*,
    thread,
    time::Duration,
};

pub struct Server {
    connected: Arc<Mutex<bool>>,
    remote_data_sender: Sender<RemoteData>,
    server_message_receiver: Receiver<ServerMessage>,
}
impl Server {
    pub fn new(
        connected: Arc<Mutex<bool>>,
        remote_data_sender: Sender<RemoteData>,
        server_message_receiver: Receiver<ServerMessage>,
    ) -> Self {
        Server {
            connected,
            remote_data_sender,
            server_message_receiver,
        }
    }

    pub fn run(mut self) {
        if let Ok(tcp_listener) = TcpListener::bind("0.0.0.0:8900") {
            for result in tcp_listener.incoming() {
                let mut tcp_stream = result.expect("tcp_stream error");
                if let Ok(mut mgc) = self.connected.lock() {
                    *mgc = true;
                }
                if self.connection_established(&mut tcp_stream).is_err() {
                    if let Ok(mut mgc) = self.connected.lock() {
                        *mgc = false;
                    }
                    continue;
                }

                // get buffers in a loop
                loop {
                    if self.server_loop(&mut tcp_stream).is_err() {
                        if let Ok(mut mgc) = self.connected.lock() {
                            *mgc = false;
                        }
                        break;
                    }
                }
            }
        }
    }

    fn connection_established(&self, tcp_stream: &mut TcpStream) -> Result<(), ServerError> {
        let remote_data_sender = &self.remote_data_sender;
        println!("connection established");
        if let Ok(intervalms) = RemoteData::read_interval_ms(tcp_stream) {
            println!("Interval : {intervalms:?} ms");
            remote_data_sender.send(intervalms).unwrap();
        }

        let voltage_buffer_size = RemoteData::read_voltage_buffer_size(tcp_stream)?;
        remote_data_sender.send(voltage_buffer_size)?;

        let command_bytes = command::Command::RetransmitBuffers.to_bytes();
        tcp_stream.write_all(&command_bytes)?;
        Ok(())
    }

    fn server_loop(&mut self, tcp_stream: &mut TcpStream) -> Result<(), ServerError> {
        let battery_voltage = RemoteData::read_battery_voltage(tcp_stream)?;
        self.remote_data_sender.send(battery_voltage)?;
        let battery_pack_voltage = RemoteData::read_battery_pack_voltage(tcp_stream)?;
        self.remote_data_sender.send(battery_pack_voltage)?;
        if let Ok(pv_voltage) = RemoteData::read_pv_voltage(tcp_stream) {
            self.remote_data_sender.send(pv_voltage)?;
        }
        while let Ok(message) = self.server_message_receiver.try_recv() {
            match message {
                ServerMessage::Command(command) => {
                    self.serve_command(command, tcp_stream)?;
                }
                ServerMessage::ReadRealtime => {
                    let remote_data = RemoteData::read_realtime(tcp_stream)?;
                    self.remote_data_sender.send(remote_data)?;
                }
                ServerMessage::ReadRealtimeStatus => {
                    let remote_data = RemoteData::read_realtime_status(tcp_stream)?;
                    self.remote_data_sender.send(remote_data)?;
                }
                ServerMessage::ReadVoltageSettings => {
                    let remote_data = RemoteData::read_voltage_settings(tcp_stream)?;
                    self.remote_data_sender.send(remote_data)?;
                }
                ServerMessage::ReadRated => {
                    let remote_data = RemoteData::read_rated(tcp_stream)?;
                    self.remote_data_sender.send(remote_data)?;
                }
                ServerMessage::SetVoltageSettings(cs) => {
                    Self::send_command(cs.generate_set_command(), tcp_stream)?
                }
            }
        }
        thread::sleep(Duration::from_millis(500));
        Ok(())
    }

    pub fn send_command(
        command: Command,
        tcp_stream: &mut TcpStream,
    ) -> Result<(), std::io::Error> {
        tcp_stream.write_all(&command.to_bytes())
    }

    pub fn serve_command(
        &mut self,
        command: Command,
        tcp_stream: &mut TcpStream,
    ) -> Result<(), ServerError> {
        match command {
            command::Command::ModbusGetHoldings { .. } => {
                let val = RemoteData::get_holdings(tcp_stream, command)?;
                println!("holding val: {:?}", &val);
                self.remote_data_sender.send(val)?;
            }
            command::Command::ModbusGetInputRegisters { .. } => {
                println!("getting_input_registers");
                let val = RemoteData::get_input_registers(tcp_stream, command)?;
                println!("input reg val: {:?}", &val);
                self.remote_data_sender.send(val)?;
            }
            _ => {}
        }
        Ok(())
    }
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
    ReadRated,
    SetVoltageSettings(VoltageSettings),
}
