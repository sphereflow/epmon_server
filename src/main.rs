use all_charts::AllCharts;
use iced::{
    executor, font,
    widget::{Column, Container},
    Alignment, Application, Command, Length, Settings, Subscription,
};
use remote_data::RemoteData;
use std::{
    sync::{mpsc::*, Arc, Mutex},
    thread,
    time::Instant,
};
use time_interval::TimeInterval;
use udp_broadcast_task::udp_broadcast;

pub mod all_charts;
pub mod command;
pub mod remote_data;
pub mod server_task;
pub mod time_interval;
pub mod tracer_an;
pub mod udp_broadcast_task;
pub mod voltage_chart;

pub const CHART_HEIGHT: f32 = 400.0;

fn main() {
    let connected = Arc::new(Mutex::new(false));
    let connected_bc = connected.clone();
    let (remote_data_sender, remote_data_receiver) = channel();
    let (command_sender, command_receiver) = channel();

    thread::spawn(move || udp_broadcast(connected_bc));
    thread::spawn(move || {
        server_task::server_task(connected, remote_data_sender, command_receiver)
    });
    State::run(Settings {
        flags: (remote_data_receiver, command_sender),
        id: Default::default(),
        window: Default::default(),
        fonts: Default::default(),
        default_font: Default::default(),
        default_text_size: iced::Pixels(16.0),
        antialiasing: Default::default(),
    })
    .expect("Error: State::run");
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    TimeIntervallSelected(TimeInterval),
    MaxTimeDaySelected(f32),
    MaxTimeSelected(f32),
    MaxTimeFineSelected(f32),
    MaxVoltageSelected(f32),
    FontLoaded(Result<(), font::Error>),
    AddressInput(String),
    ReadHoldings { register_address: u16, size: u8 },
    ReadRegisters { register_address: u16, size: u8 },
    PauseUnpause,
}

struct State {
    charts: AllCharts,
    start_instant: Instant,
    voltage_buffer_size: usize,
    remote_data_receiver: Receiver<RemoteData>,
    command_sender: Sender<command::Command>,
}

impl State {
    fn tick_update(&mut self) {
        // receive all the remote data in the channel in a loop
        while let Ok(remote_data) = self.remote_data_receiver.try_recv() {
            self.update_remote_data(remote_data);
        }
        self.charts.time_correctness = self.charts.pv.tick_len
            * self.charts.pv.voltages.len() as f32
            / (self.voltage_buffer_size as f32 * self.charts.pv.tick_len
                + (Instant::now() - self.start_instant).as_secs() as f32);
    }

    fn update_remote_data(&mut self, mut remote_data: RemoteData) {
        let time_interval = self.charts.selected_time_interval;
        let mut bupdate_battery2 = false;
        match remote_data {
            RemoteData::NoData => {}
            RemoteData::BatteryVoltage(_) => {
                self.charts
                    .battery1
                    .update_from_remote(&mut remote_data, time_interval.accumulations());
                bupdate_battery2 = true;
            }
            RemoteData::BatteryPackVoltage(_) => {
                self.charts
                    .battery_pack
                    .update_from_remote(&mut remote_data, time_interval.accumulations());
                bupdate_battery2 = true;
            }
            RemoteData::PVVoltage(_) => {
                self.charts
                    .pv
                    .update_from_remote(&mut remote_data, time_interval.accumulations());
            }
            RemoteData::VoltageBufferSize(s) => self.voltage_buffer_size = s,
            RemoteData::Intervalms(interval) => {
                let tick_len = interval as f32 / 1000.0;
                self.charts.battery1.tick_len = tick_len;
                self.charts.battery2.tick_len = tick_len;
                self.charts.battery_pack.tick_len = tick_len;
                self.charts.pv.tick_len = tick_len;
            }
            RemoteData::Holdings(val) => self.charts.modbus_val = val,
            RemoteData::InputRegisters(val) => self.charts.modbus_val = val,
        }
        if bupdate_battery2 {
            self.charts.update_battery2();
        }
    }
}

impl Application for State {
    type Executor = executor::Default;

    type Message = Message;

    type Theme = iced::Theme;

    type Flags = (Receiver<RemoteData>, Sender<command::Command>);

    fn new(
        (remote_data_receiver, command_sender): Self::Flags,
    ) -> (Self, iced::Command<Self::Message>) {
        (
            Self {
                charts: Default::default(),
                start_instant: Instant::now(),
                voltage_buffer_size: 0,
                remote_data_receiver,
                command_sender,
            },
            iced::Command::none(),
        )
    }

    fn title(&self) -> String {
        "EpMon Server".to_string()
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::GruvboxDark
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Tick => self.tick_update(),
            Message::TimeIntervallSelected(interval) => self.charts.adjust_time_interval(interval),
            Message::MaxTimeDaySelected(t) => {
                self.charts.max_time_day = t;
                self.charts.adjust_max_time();
            }
            Message::MaxTimeSelected(t) => {
                self.charts.max_time = t;
                self.charts.adjust_max_time();
            }
            Message::MaxTimeFineSelected(t) => {
                self.charts.max_time_fine = t;
                self.charts.adjust_max_time();
            }
            Message::MaxVoltageSelected(max_voltage) => {
                self.charts.max_voltage = max_voltage;
                self.charts.adjust_max_voltage();
            }
            Message::PauseUnpause => self.charts.paused = !self.charts.paused,
            Message::AddressInput(s) => {
                if let Ok(address) = u16::from_str_radix(&s, 16) {
                    self.charts.register_address = address;
                    self.charts.register_address_string = s;
                }
            }
            Message::ReadHoldings {
                register_address,
                size,
            } => self
                .command_sender
                .send(command::Command::ModbusGetHoldings {
                    register_address,
                    size,
                })
                .expect("command_sender: could not send command"),
            Message::ReadRegisters {
                register_address,
                size,
            } => self
                .command_sender
                .send(command::Command::ModbusGetInputRegisters {
                    register_address,
                    size,
                })
                .expect("command_sender: could not send command"),
            Message::FontLoaded(_) => {}
        }
        self.charts.clear_caches();
        Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        let content = Column::new()
            .spacing(20)
            .align_items(Alignment::Start)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(self.charts.view());

        Container::new(content)
            //.style(style::Container)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(5)
            .center_x()
            .center_y()
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        if !self.charts.paused {
            iced::time::every(iced::time::Duration::from_millis(100)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }
}

fn adc_reading_to_voltage(adc_reading: u16) -> f32 {
    (20700.0 / 124.0) * 1.1 * adc_reading as f32 / 4081.0
}
