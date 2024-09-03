use crate::{
    command::{self, Command},
    remote_data::RemoteData,
};
use mpsc::{Receiver, Sender};
use std::{io::Write, net::TcpListener, sync::*, thread, time::Duration};

pub fn server_task(
    connected: Arc<Mutex<bool>>,
    remote_data_sender: Sender<RemoteData>,
    command_receiver: Receiver<Command>,
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
                if let Ok(battery_voltage) = RemoteData::read_battery_voltage(&mut tcp_stream) {
                    remote_data_sender.send(battery_voltage).unwrap();
                } else {
                    if let Ok(mut mgc) = connected.lock() {
                        *mgc = false;
                    }
                    continue 'outer;
                }
                if let Ok(battery_pack_voltage) =
                    RemoteData::read_battery_pack_voltage(&mut tcp_stream)
                {
                    remote_data_sender.send(battery_pack_voltage).unwrap();
                } else {
                    if let Ok(mut mgc) = connected.lock() {
                        *mgc = false;
                    }
                    continue 'outer;
                }
                if let Ok(pv_voltage) = RemoteData::read_pv_voltage(&mut tcp_stream) {
                    remote_data_sender.send(pv_voltage).unwrap();
                } else {
                    if let Ok(mut mgc) = connected.lock() {
                        *mgc = false;
                    }
                    continue 'outer;
                }
                while let Ok(command) = command_receiver.try_recv() {
                    if let command::Command::ModbusGetHoldings { .. } = command {
                        if let Ok(val) = RemoteData::get_holdings(&mut tcp_stream, command) {
                            remote_data_sender.send(val).unwrap();
                        } else {
                            if let Ok(mut mgc) = connected.lock() {
                                *mgc = false;
                            }
                            continue 'outer;
                        }
                    }
                    if let command::Command::ModbusGetInputRegisters { .. } = command {
                        println!("getting_input_registers");
                        if let Ok(val) = RemoteData::get_input_registers(&mut tcp_stream, command) {
                            println!("Input reg val: {:?}", &val);
                            remote_data_sender.send(val).unwrap();
                        } else {
                            if let Ok(mut mgc) = connected.lock() {
                                *mgc = false;
                            }
                            continue 'outer;
                        }
                    }
                }

                thread::sleep(Duration::from_millis(500));
            }
        }
    }
}
