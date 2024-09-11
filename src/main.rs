use all_charts::{AllCharts, SelectedTab};
use command::Command;
use iced::{
    executor, font,
    widget::{Column, Container},
    Alignment, Application, Length, Settings, Subscription,
};
use remote_data::RemoteData;
use server_task::{Server, ServerMessage};
use std::{
    sync::{mpsc::*, Arc, Mutex},
    thread,
    time::Instant,
};
use time_interval::TimeInterval;
use tracer_an::BatteryType;
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
    let connected_main_app = connected.clone();
    let (remote_data_sender, remote_data_receiver) = channel();
    let (command_sender, command_receiver) = channel();

    thread::spawn(move || udp_broadcast(connected_bc));
    thread::spawn(move || {
        Server::run(Server::new(connected, remote_data_sender, command_receiver))
    });
    State::run(Settings {
        flags: (remote_data_receiver, command_sender, connected_main_app),
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
    MinVoltageSelected(f32),
    MaxVoltageSelected(f32),
    FontLoaded(Result<(), font::Error>),
    AddressInput(String),
    ReadHoldings { register_address: u16, size: u8 },
    ReadRegisters { register_address: u16, size: u8 },
    ReadRealtime,
    ReadRealtimeStatus,
    PauseUnpause,
    TabSelected(i32),
    ToggleChartControls,
    ReadVoltageSettings,
    ReadRated,
    ReadStats,
    BatteryTypeSelected(BatteryType),
    InputOverVoltageDisconnect(String),
    InputChargingLimitVoltage(String),
    InputOverVoltageReconnect(String),
    InputEqualizationVoltage(String),
    InputBoostVoltage(String),
    InputFloatVoltage(String),
    InputBoostReconnectVoltage(String),
    InputLowVoltageReconnectVoltage(String),
    InputUnderVoltageRecoverVoltage(String),
    InputUnderVoltageWarningVoltage(String),
    InputLowVoltageDisconnectVoltage(String),
    InputDischargingLimitVoltage(String),
    SendServerMessage(ServerMessage),
}

struct State {
    charts: AllCharts,
    start_instant: Instant,
    voltage_buffer_size: usize,
    remote_data_receiver: Receiver<RemoteData>,
    server_message_sender: Sender<ServerMessage>,
}

impl State {
    fn tick_update(&mut self) {
        // receive all the remote data in the channel in a loop
        while let Ok(remote_data) = self.remote_data_receiver.try_recv() {
            self.update_remote_data(remote_data);
        }
        self.charts.time_correctness = self.charts.pv.tick_len * self.charts.pv.data.len() as f32
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
                    .update_voltages_from_remote(&mut remote_data, time_interval.accumulations());
                bupdate_battery2 = true;
            }
            RemoteData::BatteryPackVoltage(_) => {
                self.charts
                    .battery_pack
                    .update_voltages_from_remote(&mut remote_data, time_interval.accumulations());
                bupdate_battery2 = true;
            }
            RemoteData::PVVoltage(_) => {
                self.charts
                    .pv
                    .update_voltages_from_remote(&mut remote_data, time_interval.accumulations());
            }
            RemoteData::PVPower(_) => {
                self.charts
                    .pv_power
                    .update_power_from_remote(&mut remote_data, time_interval.accumulations());
            }
            RemoteData::VoltageBufferSize(s) => self.voltage_buffer_size = s,
            RemoteData::VoltageIntervalms(interval) => {
                let tick_len = interval as f32 / 1000.0;
                self.charts.battery1.tick_len = tick_len;
                self.charts.battery2.tick_len = tick_len;
                self.charts.battery_pack.tick_len = tick_len;
                self.charts.pv.tick_len = tick_len;
            }
            RemoteData::PowerIntervalms(interval) => {
                let tick_len = interval as f32 / 1000.0;
                self.charts.pv_power.tick_len = tick_len;
                self.charts.inverter_power.tick_len = tick_len;
            }
            RemoteData::Holdings(val) | RemoteData::InputRegisters(val) => {
                self.charts.modbus_val = val;
            }
            RemoteData::Realtime(realtime) => self.charts.realtime_data = realtime,
            RemoteData::RealtimeStatus(realtime_status) => {
                self.charts.realtime_status_data = realtime_status
            }
            RemoteData::VoltageSettings(voltage_settings) => {
                self.charts.voltage_settings = voltage_settings;
                self.charts.change_voltage_settings = voltage_settings;
            }
            RemoteData::Rated(rated) => {
                self.charts.rated_data = rated;
            }
            RemoteData::Stats(stats) => {
                self.charts.stats = stats;
            }
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

    type Flags = (
        Receiver<RemoteData>,
        Sender<ServerMessage>,
        Arc<Mutex<bool>>,
    );

    fn new(
        (remote_data_receiver, command_sender, connected): Self::Flags,
    ) -> (Self, iced::Command<Self::Message>) {
        (
            Self {
                charts: AllCharts {
                    connected,
                    ..Default::default()
                },
                start_instant: Instant::now(),
                voltage_buffer_size: 0,
                remote_data_receiver,
                server_message_sender: command_sender,
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

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
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
                self.charts.adjust_min_max_voltage();
            }
            Message::MinVoltageSelected(min_voltage) => {
                self.charts.min_voltage = min_voltage;
                self.charts.adjust_min_max_voltage();
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
            } => {
                self.server_message_sender
                    .send(ServerMessage::Command(Command::ModbusGetHoldings {
                        register_address,
                        size,
                    }))
                    .expect("command_sender: could not send command");
            }
            Message::ReadRegisters {
                register_address,
                size,
            } => {
                self.server_message_sender
                    .send(ServerMessage::Command(Command::ModbusGetInputRegisters {
                        register_address,
                        size,
                    }))
                    .expect("command_sender: could not send command");
            }
            Message::ReadRealtime => {
                self.server_message_sender
                    .send(ServerMessage::ReadRealtime)
                    .expect("command sender: could not send command");
            }
            Message::ReadRealtimeStatus => {
                self.server_message_sender
                    .send(ServerMessage::ReadRealtimeStatus)
                    .expect("command sender: could not send command");
            }
            Message::ReadVoltageSettings => {
                self.server_message_sender
                    .send(ServerMessage::ReadVoltageSettings)
                    .expect("command sender: could not send command");
            }
            Message::ReadRated => {
                self.server_message_sender
                    .send(ServerMessage::ReadRated)
                    .expect("command sender: could not send command");
            }
            Message::ReadStats => {
                self.server_message_sender
                    .send(ServerMessage::ReadStats)
                    .expect("command sender: could not send command");
            }
            Message::TabSelected(ix) => match ix {
                0 => self.charts.selected_tab = SelectedTab::VoltageCharts,
                _ => self.charts.selected_tab = SelectedTab::Modbus,
            },
            Message::ToggleChartControls => {
                self.charts.chart_controls = !self.charts.chart_controls
            }
            Message::BatteryTypeSelected(battery_type) => {
                self.charts.change_voltage_settings.battery_type = battery_type;
            }
            Message::InputOverVoltageDisconnect(s) => {
                if let Ok(f) = s.parse::<f32>() {
                    let f = (f / 0.01).round() / 100.0;
                    self.charts.change_voltage_settings.over_voltage_disconnect = f;
                }
            }
            Message::InputChargingLimitVoltage(s) => {
                if let Ok(f) = s.parse::<f32>() {
                    let f = (f / 0.01).round() / 100.0;
                    self.charts.change_voltage_settings.charging_limit_voltage = f;
                }
            }

            Message::InputOverVoltageReconnect(s) => {
                if let Ok(f) = s.parse::<f32>() {
                    let f = (f / 0.01).round() / 100.0;
                    self.charts.change_voltage_settings.over_voltage_reconnect = f;
                }
            }

            Message::InputEqualizationVoltage(s) => {
                if let Ok(f) = s.parse::<f32>() {
                    let f = (f / 0.01).round() / 100.0;
                    self.charts.change_voltage_settings.equalization_voltage = f;
                }
            }

            Message::InputBoostVoltage(s) => {
                if let Ok(f) = s.parse::<f32>() {
                    let f = (f / 0.01).round() / 100.0;
                    self.charts.change_voltage_settings.boost_voltage = f;
                }
            }

            Message::InputFloatVoltage(s) => {
                if let Ok(f) = s.parse::<f32>() {
                    let f = (f / 0.01).round() / 100.0;
                    self.charts.change_voltage_settings.float_voltage = f;
                }
            }

            Message::InputBoostReconnectVoltage(s) => {
                if let Ok(f) = s.parse::<f32>() {
                    let f = (f / 0.01).round() / 100.0;
                    self.charts.change_voltage_settings.boost_reconnect_voltage = f;
                }
            }

            Message::InputLowVoltageReconnectVoltage(s) => {
                if let Ok(f) = s.parse::<f32>() {
                    let f = (f / 0.01).round() / 100.0;
                    self.charts
                        .change_voltage_settings
                        .low_voltage_reconnect_voltage = f;
                }
            }

            Message::InputUnderVoltageRecoverVoltage(s) => {
                if let Ok(f) = s.parse::<f32>() {
                    let f = (f / 0.01).round() / 100.0;
                    self.charts
                        .change_voltage_settings
                        .under_voltage_recover_voltage = f;
                }
            }

            Message::InputUnderVoltageWarningVoltage(s) => {
                if let Ok(f) = s.parse::<f32>() {
                    let f = (f / 0.01).round() / 100.0;
                    self.charts
                        .change_voltage_settings
                        .under_voltage_warning_voltage = f;
                }
            }

            Message::InputLowVoltageDisconnectVoltage(s) => {
                if let Ok(f) = s.parse::<f32>() {
                    let f = (f / 0.01).round() / 100.0;
                    self.charts
                        .change_voltage_settings
                        .low_voltage_disconnect_voltage = f;
                }
            }

            Message::InputDischargingLimitVoltage(s) => {
                if let Ok(f) = s.parse::<f32>() {
                    let f = (f / 0.01).round() / 100.0;
                    self.charts
                        .change_voltage_settings
                        .discharging_limit_voltage = f;
                }
            }
            Message::SendServerMessage(message) => self
                .server_message_sender
                .send(message)
                .expect("could not send server message"),

            Message::FontLoaded(_) => {}
        }
        self.charts.clear_caches();
        iced::Command::none()
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
