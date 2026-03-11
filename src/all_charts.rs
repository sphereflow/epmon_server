use std::sync::{Arc, Mutex};

use crate::{
    inverter::Inverter,
    server_task::ServerMessage,
    time_interval::TimeInterval,
    tracer_an::{
        two_bytes_to_f32, BatteryType, Rated, Realtime, RealtimeStatus, Stats, VoltageSettings,
    },
    voltage_chart::{ChartType, CustomChart},
    Message, CHART_HEIGHT,
};
use iced::{widget::*, Alignment, Element, Length};
use iced_aw::{TabBar, TabLabel};

#[derive(Debug)]
pub struct AllCharts {
    pub selected_tab: SelectedTab,
    pub battery1: CustomChart,
    pub battery2: CustomChart,
    pub battery_pack: CustomChart,
    pub pv: CustomChart,
    pub pv_power: CustomChart,
    pub inverter_input_power: CustomChart,
    pub inverter_output_power: CustomChart,
    pub selected_time_interval: TimeInterval,
    pub time_correctness: f32,
    pub max_time_day: f32,
    pub max_time: f32,
    pub max_time_fine: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub register_address_string: String,
    pub register_address: u16,
    pub modbus_val: Vec<u8>,
    pub realtime_data: Realtime,
    pub realtime_status_data: RealtimeStatus,
    pub rated_data: Rated,
    pub stats: Stats,
    pub inverter_stats: Inverter,
    pub voltage_settings: VoltageSettings,
    pub change_voltage_settings: VoltageSettings,
    pub chart_controls: bool,
    pub paused: bool,
    pub connected: Arc<Mutex<bool>>,
    pub log: Vec<String>,
}

impl Default for AllCharts {
    fn default() -> Self {
        let battery1 = CustomChart {
            title: "Battery1".to_string(),
            ..Default::default()
        };
        let battery2 = CustomChart {
            title: "Battery2".to_string(),
            ..Default::default()
        };
        let battery_pack = CustomChart {
            title: "Battery Pack".to_string(),
            ..Default::default()
        };
        let pv = CustomChart {
            title: "PV".to_string(),
            ..Default::default()
        };
        let pv_power = CustomChart {
            title: "PV Power".to_string(),
            max_y: 1200.0,
            chart_type: ChartType::Power,
            ..Default::default()
        };
        let inverter_input_power = CustomChart {
            title: "Inverter Input Power".to_string(),
            max_y: 3000.0,
            chart_type: ChartType::Power,
            ..Default::default()
        };
        let inverter_output_power = CustomChart {
            title: "Inverter Output Power".to_string(),
            max_y: 3000.0,
            chart_type: ChartType::Power,
            ..Default::default()
        };
        let mut all_charts = AllCharts {
            selected_tab: SelectedTab::VoltageCharts,
            battery1,
            battery2,
            battery_pack,
            pv,
            pv_power,
            inverter_input_power,
            inverter_output_power,
            selected_time_interval: Default::default(),
            max_time_day: 0.0,
            max_time: 0.0,
            max_time_fine: 0.0,
            min_y: 0.0,
            max_y: 100.0,
            time_correctness: 1.0,
            chart_controls: true,
            paused: false,
            register_address: 0,
            register_address_string: String::new(),
            modbus_val: Vec::new(),
            realtime_data: Default::default(),
            realtime_status_data: Default::default(),
            voltage_settings: Default::default(),
            change_voltage_settings: Default::default(),
            rated_data: Default::default(),
            stats: Default::default(),
            inverter_stats: Default::default(),
            connected: Arc::new(Mutex::new(false)),
            log: Vec::new(),
        };
        all_charts.adjust_min_max_y();
        all_charts
    }
}

impl AllCharts {
    pub fn view(&self) -> Element<'_, Message> {
        let tab_bar = TabBar::new(Message::TabSelected)
            .push(0, TabLabel::Text(String::from("Voltage Charts")))
            .push(1, TabLabel::Text(String::from("Power Charts")))
            .push(2, TabLabel::Text(String::from("Stats")))
            .push(3, TabLabel::Text(String::from("Settings")))
            .push(4, TabLabel::Text(String::from("Log")))
            .set_active_tab(&(self.selected_tab as i32));

        let connected = *self.connected.lock().expect("could not lock mutex");
        let mut main_contents = Column::new();
        if !connected {
            main_contents = main_contents.push(Text::new("No connection !!!").size(36));
        }

        main_contents = main_contents.push(match self.selected_tab {
            SelectedTab::VoltageCharts => self.view_voltage_charts(),
            SelectedTab::PowerCharts => self.view_power_charts(),
            SelectedTab::Stats => self.view_modbus(),
            SelectedTab::Settings => self.view_settings(),
            SelectedTab::Log => self.view_log(),
        });
        Scrollable::new(
            Column::new()
                .push(tab_bar)
                .push(spacer())
                .push(main_contents),
        )
        .height(Length::Shrink)
        .into()
    }

    fn view_voltage_charts(&self) -> Element<'_, Message> {
        let control_row = self.view_chart_controls();
        let row1 = Row::new()
            .spacing(15)
            .padding(20)
            .width(Length::Fill)
            .height(Length::Shrink)
            .align_y(Alignment::Center)
            .push(self.battery_pack.view(0, CHART_HEIGHT))
            .push(self.battery1.view(1, CHART_HEIGHT));
        let row2 = Row::new()
            .spacing(15)
            .padding(20)
            .width(Length::Fill)
            .height(Length::Shrink)
            .align_y(Alignment::Center)
            .push(self.pv.view(0, CHART_HEIGHT))
            .push(self.battery2.view(1, CHART_HEIGHT));

        Column::new()
            .width(Length::Fill)
            .height(Length::Shrink)
            .align_x(Alignment::Start)
            .push(control_row)
            .push(row1)
            .push(row2)
            .into()
    }

    fn view_power_charts(&self) -> Element<'_, Message> {
        let control_row = self.view_chart_controls();
        let pv_row = Row::new()
            .spacing(15)
            .padding(20)
            .width(Length::Fill)
            .height(Length::Shrink)
            .align_y(Alignment::Center)
            .push(self.pv_power.view(0, CHART_HEIGHT));
        let inverter_row = Row::new()
            .spacing(15)
            .padding(20)
            .width(Length::Fill)
            .height(Length::Shrink)
            .align_y(Alignment::Center)
            .push(self.inverter_input_power.view(0, CHART_HEIGHT))
            .push(self.inverter_output_power.view(1, CHART_HEIGHT));
        Column::new()
            .width(Length::Fill)
            .height(Length::Shrink)
            .align_x(Alignment::Start)
            .push(control_row)
            .push(pv_row)
            .push(inverter_row)
            .into()
    }

    fn view_chart_controls(&self) -> Row<'_, Message> {
        let selected = self.selected_time_interval;
        let control_row = Row::new();
        let toggle_chart_controls = Button::new(if self.chart_controls { "-" } else { "+" })
            .on_press(Message::ToggleChartControls);
        let make_radio = |&(label, value)| {
            Radio::new(label, value, Some(selected), Message::TimeIntervallSelected).into()
        };

        let radio_data = [
            ("second", TimeInterval::Second),
            ("ten seconds", TimeInterval::TenSeconds),
            ("thirty seconds", TimeInterval::ThirtySeconds),
            ("minute", TimeInterval::Minute),
            ("5 minutes", TimeInterval::FiveMinutes),
            ("30 minutes", TimeInterval::ThirtyMinutes),
            ("hour", TimeInterval::Hour),
            ("3 hours", TimeInterval::ThreeHours),
            ("6 hours", TimeInterval::SixHours),
            ("12 hours", TimeInterval::TwelveHours),
            ("day", TimeInterval::Day),
        ];
        let radios1 = radio_data[..6].iter().map(make_radio);
        let radios2 = radio_data[6..].iter().map(make_radio);
        let time_intervall_column1 = Column::with_children(radios1);
        let time_intervall_column2 = Column::with_children(radios2);

        let max_time_slider_day = Slider::new(
            (-3600.0 * 24.0)..=0.0,
            self.max_time_day,
            Message::MaxTimeDaySelected,
        )
        .width(500);
        let max_time_slider = Slider::new(-3600.0..=0.0, self.max_time, Message::MaxTimeSelected)
            .step(1.0)
            .shift_step(0.1)
            .width(500);
        let max_time_slider_fine = Slider::new(
            -100.0..=0.0,
            self.max_time_fine,
            Message::MaxTimeFineSelected,
        )
        .step(0.1)
        .width(500);
        let min_voltage_slider = VerticalSlider::new(
            0.0..=(self.max_y - 1.0),
            self.min_y,
            Message::MinVoltageSelected,
        )
        .step(1.0)
        .height(200.0);
        let max_voltage_slider = VerticalSlider::new(
            (self.min_y + 1.0)..=200.0,
            self.max_y,
            Message::MaxVoltageSelected,
        )
        .step(1.0)
        .height(200.0);
        let pause_button = if self.paused {
            Button::new("unpause")
        } else {
            Button::new("pause")
        }
        .on_press(Message::PauseUnpause);

        let si = self.selected_time_interval.interval();
        let i = *si.start()..=self.pv_power.integration_sub_range.end;
        let min_sub_interval_slider = Slider::new(
            i,
            self.pv_power.integration_sub_range.start,
            Message::MinIntegrationSubRange,
        )
        .width(200);
        let i = self.pv_power.integration_sub_range.start..=*si.end();
        let max_sub_interval_slider = Slider::new(
            i,
            self.pv_power.integration_sub_range.end,
            Message::MaxIntegrationSubRange,
        )
        .width(200);

        let integration_col = Column::new()
            .push(min_sub_interval_slider)
            .push(spacer())
            .push(max_sub_interval_slider)
            .push(Text::new(format!(
                "pv: {:.3} kWh",
                self.pv_power.kilo_watt_hours()
            )))
            .push(Text::new(format!(
                "inverter in: {:.3} kWh",
                self.inverter_input_power.kilo_watt_hours()
            )))
            .push(Text::new(format!(
                "inverter out: {:.3} kWh",
                self.inverter_output_power.kilo_watt_hours()
            )))
            .push(Text::new(format!(
                "{} s ..= {} s",
                self.pv_power.integration_sub_range.start, self.pv_power.integration_sub_range.end
            )));

        if self.chart_controls {
            control_row
                .push(toggle_chart_controls)
                .push(time_intervall_column1)
                .push(time_intervall_column2)
                .push(Space::new(30., 30.))
                .push(iced::widget::column![
                    max_time_slider_day,
                    Space::new(30.0, 30.0),
                    max_time_slider,
                    Space::new(30.0, 30.0),
                    max_time_slider_fine,
                ])
                .push(spacer())
                .push(min_voltage_slider)
                .push(Space::new(30.0, 30.0))
                .push(max_voltage_slider)
                .push(Space::new(30.0, 10.0))
                .push(text(format!(
                    "time correctness: {:.2} %",
                    self.time_correctness * 100.0
                )))
                .push(Space::new(30., 30.))
                .push(pause_button)
                .push(integration_col)
        } else {
            control_row.push(toggle_chart_controls)
        }
    }

    fn view_modbus(&self) -> Element<'_, Message> {
        let register_text_input = text_input(
            "enter address of holding/register",
            &self.register_address_string,
        )
        .width(140)
        .on_input(Message::AddressInput);
        let holding_button = Button::new("get holding val").on_press(Message::ReadHoldings {
            register_address: self.register_address,
            size: 1,
        });
        let register_button = Button::new("get input register").on_press(Message::ReadRegisters {
            register_address: self.register_address,
            size: 1,
        });
        let read_realtime_button =
            Button::new("read realtime data").on_press(Message::ReadRealtime);
        let read_realtime_status_button =
            Button::new("read realtime status data").on_press(Message::ReadRealtimeStatus);
        let register_numeric_text =
            text(format!("numeric register value: {}", self.register_address));
        let holding_text = if self.modbus_val.len() >= 2 {
            text(format!(
                "received value: {:?}",
                two_bytes_to_f32([self.modbus_val[0], self.modbus_val[1]])
            ))
        } else {
            text("no value")
        };
        let realtime_text = text(format!("{}", self.realtime_data));
        let realtime_status_text = text(format!("{}", self.realtime_status_data));
        let rated_col = self.view_rated();
        let stats_col = self.view_stats();
        let register_col = Column::new()
            .push(register_text_input)
            .push(spacer())
            .push(holding_button)
            .push(register_button)
            .push(spacer())
            .push(register_numeric_text)
            .push(holding_text);
        let realtime_col = Column::new().push(read_realtime_button).push(realtime_text);
        let realtime_status_col = Column::new()
            .push(read_realtime_status_button)
            .push(realtime_status_text);
        let get_inverter_button =
            Button::new("read inverter status").on_press(Message::ReadInverter);
        let inverter_text = text(format!("{}", self.inverter_stats));
        let inverter_col = Column::new().push(get_inverter_button).push(inverter_text);
        let row1 = Row::new()
            .push(Space::new(100, 10))
            .push(register_col)
            .push(realtime_col)
            .push(realtime_status_col)
            .push(inverter_col)
            .spacing(100);
        let row2 = Row::new()
            .push(Space::new(100, 10))
            .push(rated_col)
            .push(stats_col)
            .spacing(100);
        Column::new()
            .push(spacer())
            .push(row1)
            .push(row2)
            .spacing(50)
            .into()
    }

    fn view_settings(&self) -> Element<'_, Message> {
        Row::new()
            .push(spacer())
            .push(self.view_voltage_settings())
            .into()
    }

    fn view_voltage_settings(&self) -> Element<'_, Message> {
        let s = self.voltage_settings;
        let cs = self.change_voltage_settings;
        let get_voltage_settings_button =
            Button::new("get voltage settings").on_press(Message::ReadVoltageSettings);
        let set_buttons_col = Column::new()
            .push(Space::new(1, 0))
            .push(Text::new("battery type"))
            .push(Text::new("over voltage disconnect"))
            .push(Text::new("charging limit voltage"))
            .push(Text::new("over voltage reconnect"))
            .push(Text::new("equalization voltage"))
            .push(Text::new("boost voltage"))
            .push(Text::new("float voltage"))
            .push(Text::new("boost reconnect voltage"))
            .push(Text::new("low voltage reconnect"))
            .push(Text::new("under voltage recover"))
            .push(Text::new("under voltage warning"))
            .push(Text::new("low voltage disconnect"))
            .push(Text::new("discharging limit voltage"))
            .push(Text::new("Settings valid?"))
            .spacing(10);
        let display_voltage_settings_col = Column::new()
            .push(Space::new(1, 0.))
            .push(Text::new(format!("{}", s.battery_type)))
            .push(Text::new(format!("{}", s.over_voltage_disconnect)))
            .push(Text::new(format!("{}", s.charging_limit_voltage)))
            .push(Text::new(format!("{}", s.over_voltage_reconnect)))
            .push(Text::new(format!("{}", s.equalization_voltage)))
            .push(Text::new(format!("{}", s.boost_voltage)))
            .push(Text::new(format!("{}", s.float_voltage)))
            .push(Text::new(format!("{}", s.boost_reconnect_voltage)))
            .push(Text::new(format!("{}", s.low_voltage_reconnect_voltage)))
            .push(Text::new(format!("{}", s.under_voltage_recover_voltage)))
            .push(Text::new(format!("{}", s.under_voltage_warning_voltage)))
            .push(Text::new(format!("{}", s.low_voltage_disconnect_voltage)))
            .push(Text::new(format!("{}", s.discharging_limit_voltage)))
            .push(Text::new(format!("{:?}", s.check_settings_lifepo4())).width(200))
            .spacing(10);
        let battery_type_options = [
            BatteryType::UserDefined,
            BatteryType::Sealed,
            BatteryType::Gel,
            BatteryType::Flooded,
            BatteryType::LFP8S,
        ];
        fn voltage_text_input(f: f32) -> TextInput<'static, Message> {
            TextInput::new("", &format!("{:.2}", f))
        }
        let set_voltages_button = Button::new("SET").on_press(Message::SendServerMessage(
            ServerMessage::SetVoltageSettings(cs),
        ));
        let voltage_settings_input_col = Column::new()
            .push(PickList::new(
                battery_type_options,
                Some(self.change_voltage_settings.battery_type),
                Message::BatteryTypeSelected,
            ))
            .push(
                voltage_text_input(self.change_voltage_settings.over_voltage_disconnect)
                    .on_input(Message::InputOverVoltageDisconnect),
            )
            .push(
                voltage_text_input(self.change_voltage_settings.charging_limit_voltage)
                    .on_input(Message::InputChargingLimitVoltage),
            )
            .push(
                voltage_text_input(self.change_voltage_settings.over_voltage_reconnect)
                    .on_input(Message::InputOverVoltageReconnect),
            )
            .push(
                voltage_text_input(self.change_voltage_settings.equalization_voltage)
                    .on_input(Message::InputEqualizationVoltage),
            )
            .push(
                voltage_text_input(self.change_voltage_settings.boost_voltage)
                    .on_input(Message::InputBoostVoltage),
            )
            .push(
                voltage_text_input(self.change_voltage_settings.float_voltage)
                    .on_input(Message::InputFloatVoltage),
            )
            .push(
                voltage_text_input(self.change_voltage_settings.boost_reconnect_voltage)
                    .on_input(Message::InputBoostReconnectVoltage),
            )
            .push(
                voltage_text_input(self.change_voltage_settings.low_voltage_reconnect_voltage)
                    .on_input(Message::InputLowVoltageReconnectVoltage),
            )
            .push(
                voltage_text_input(self.change_voltage_settings.under_voltage_recover_voltage)
                    .on_input(Message::InputUnderVoltageRecoverVoltage),
            )
            .push(
                voltage_text_input(self.change_voltage_settings.under_voltage_warning_voltage)
                    .on_input(Message::InputUnderVoltageWarningVoltage),
            )
            .push(
                voltage_text_input(self.change_voltage_settings.low_voltage_disconnect_voltage)
                    .on_input(Message::InputLowVoltageDisconnectVoltage),
            )
            .push(
                voltage_text_input(self.change_voltage_settings.discharging_limit_voltage)
                    .on_input(Message::InputDischargingLimitVoltage),
            )
            .push(
                Text::new(format!(
                    "{:?}",
                    self.change_voltage_settings.check_settings_lifepo4()
                ))
                .width(200),
            )
            .width(200);
        let row = Row::new()
            .push(set_buttons_col)
            .push(spacer())
            .push(display_voltage_settings_col)
            .push(spacer())
            .push(voltage_settings_input_col)
            .push(spacer())
            .push(set_voltages_button);
        Column::new()
            .push(spacer())
            .push(get_voltage_settings_button)
            .push(row)
            .into()
    }

    fn view_log(&self) -> Element<'_, Message> {
        let mut concatenated_log = String::new();
        for line in self.log.iter() {
            concatenated_log.push_str(line);
            concatenated_log.push('\n');
        }
        let log_text = text(concatenated_log);
        let scrollable_log_text = scrollable(log_text);
        let get_log_button = Button::new("get last log line")
            .on_press(Message::SendServerMessage(ServerMessage::ReadLastLog));
        Column::new()
            .push(scrollable_log_text)
            .push(get_log_button)
            .into()
    }

    fn view_rated(&self) -> Element<'_, Message> {
        let read_rated_button = Button::new("read rated").on_press(Message::ReadRated);
        let rated_text = Text::new(format!("{}", self.rated_data));
        Column::new()
            .push(read_rated_button)
            .push(spacer())
            .push(rated_text)
            .into()
    }

    fn view_stats(&self) -> Element<'_, Message> {
        let read_stats_button = Button::new("read stats").on_press(Message::ReadStats);
        let stats_text = Text::new(format!("{}", self.stats));
        Column::new()
            .push(read_stats_button)
            .push(spacer())
            .push(stats_text)
            .into()
    }

    pub fn update_battery2(&mut self) {
        let voltages = self
            .battery_pack
            .data
            .iter()
            .zip(self.battery1.data.iter())
            .map(|(bp_voltage, b1_voltage)| bp_voltage - b1_voltage);
        self.battery2.data = voltages.collect();
        self.battery2.accumulate_into_view_buffer();
    }

    pub fn adjust_time_interval(&mut self, time_interval: TimeInterval) {
        self.selected_time_interval = time_interval;
        self.map_charts(|vc| vc.adjust_time_interval(time_interval));
    }

    pub fn adjust_max_time(&mut self) {
        let max_time = self.max_time_day + self.max_time + self.max_time_fine;
        self.map_charts(|vc| vc.max_time = max_time);
        self.adjust_time_interval(self.selected_time_interval);
    }

    pub fn adjust_min_max_y(&mut self) {
        self.battery1.min_y = 0.15 * self.min_y;
        self.battery1.max_y = 0.15 * self.max_y;
        self.battery2.min_y = 0.15 * self.min_y;
        self.battery2.max_y = 0.15 * self.max_y;
        self.battery_pack.min_y = 0.35 * self.min_y;
        self.battery_pack.max_y = 0.35 * self.max_y;
        self.pv.min_y = self.min_y;
        self.pv.max_y = self.max_y;
        self.pv_power.min_y = self.min_y * 6.0;
        self.pv_power.max_y = self.max_y * 6.0;
        self.inverter_input_power.min_y = self.min_y * 6.0;
        self.inverter_input_power.max_y = self.max_y * 6.0;
        self.inverter_output_power.min_y = self.min_y * 6.0;
        self.inverter_output_power.max_y = self.max_y * 6.0;
        self.clear_caches();
    }

    pub fn clear_caches(&mut self) {
        self.map_charts(|chart| chart.cache.clear());
    }

    fn map_charts<F: FnMut(&mut CustomChart)>(&mut self, mut f: F) {
        f(&mut self.battery_pack);
        f(&mut self.battery1);
        f(&mut self.battery2);
        f(&mut self.pv);
        f(&mut self.pv_power);
        f(&mut self.inverter_input_power);
        f(&mut self.inverter_output_power);
    }
}

#[derive(Debug, Copy, Clone)]
pub enum SelectedTab {
    VoltageCharts,
    PowerCharts,
    Stats,
    Settings,
    Log,
}

fn spacer() -> Space {
    Space::new(30, 30)
}
