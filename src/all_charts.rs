use std::sync::{Arc, Mutex};

use crate::{
    time_interval::TimeInterval,
    tracer_an::{two_bytes_to_f32, Rated, Realtime, RealtimeStatus, VoltageSettings},
    voltage_chart::VoltageChart,
    Message, CHART_HEIGHT,
};
use iced::{widget::*, Alignment, Element, Length};
use iced_aw::{TabBar, TabLabel};

#[derive(Debug)]
pub struct AllCharts {
    pub selected_tab: SelectedTab,
    pub battery1: VoltageChart,
    pub battery2: VoltageChart,
    pub battery_pack: VoltageChart,
    pub pv: VoltageChart,
    pub selected_time_interval: TimeInterval,
    pub time_correctness: f32,
    pub max_time_day: f32,
    pub max_time: f32,
    pub max_time_fine: f32,
    pub max_voltage: f32,
    pub register_address_string: String,
    pub register_address: u16,
    pub modbus_val: Vec<u8>,
    pub realtime_data: Realtime,
    pub realtime_status_data: RealtimeStatus,
    pub rated_data: Rated,
    pub voltage_settings: VoltageSettings,
    pub chart_controls: bool,
    pub paused: bool,
    pub connected: Arc<Mutex<bool>>,
}

impl Default for AllCharts {
    fn default() -> Self {
        let battery1 = VoltageChart {
            title: "Battery1".to_string(),
            ..Default::default()
        };
        let battery2 = VoltageChart {
            title: "Battery2".to_string(),
            ..Default::default()
        };
        let battery_pack = VoltageChart {
            title: "Battery Pack".to_string(),
            ..Default::default()
        };
        let pv = VoltageChart {
            title: "PV".to_string(),
            ..Default::default()
        };
        AllCharts {
            selected_tab: SelectedTab::VoltageCharts,
            battery1,
            battery2,
            battery_pack,
            pv,
            selected_time_interval: Default::default(),
            max_time_day: 0.0,
            max_time: 0.0,
            max_time_fine: 0.0,
            max_voltage: 100.0,
            time_correctness: 1.0,
            chart_controls: true,
            paused: false,
            register_address: 0,
            register_address_string: String::new(),
            modbus_val: Vec::new(),
            realtime_data: Default::default(),
            realtime_status_data: Default::default(),
            voltage_settings: Default::default(),
            rated_data: Default::default(),
            connected: Arc::new(Mutex::new(false)),
        }
    }
}

impl AllCharts {
    pub fn view(&self) -> Element<Message> {
        let tab_bar = TabBar::new(Message::TabSelected)
            .push(0, TabLabel::Text(String::from("Voltage Charts")))
            .push(1, TabLabel::Text(String::from("Modbus")));

        let connected = *self.connected.lock().expect("could not lock mutex");
        let mut main_contents = Column::new();
        if !connected {
            main_contents = main_contents.push(Text::new("No connection !!!").size(36));
        }

        main_contents = main_contents.push(match self.selected_tab {
            SelectedTab::VoltageCharts => self.view_charts(),
            SelectedTab::Modbus => self.view_modbus(),
        });
        Scrollable::new(Column::new().push(tab_bar).push(main_contents))
            .height(Length::Shrink)
            .into()
    }

    fn view_charts(&self) -> Element<Message> {
        let control_row = self.view_chart_controls();
        let row1 = Row::new()
            .spacing(15)
            .padding(20)
            .width(Length::Fill)
            .height(Length::Shrink)
            .align_items(Alignment::Center)
            .push(self.battery_pack.view(0, CHART_HEIGHT))
            .push(self.battery1.view(1, CHART_HEIGHT));
        let row2 = Row::new()
            .spacing(15)
            .padding(20)
            .width(Length::Fill)
            .height(Length::Shrink)
            .align_items(Alignment::Center)
            .push(self.pv.view(0, CHART_HEIGHT))
            .push(self.battery2.view(1, CHART_HEIGHT));

        Column::new()
            .width(Length::Fill)
            .height(Length::Shrink)
            .align_items(Alignment::Start)
            .push(control_row)
            .push(row1)
            .push(row2)
            .into()
    }

    fn view_chart_controls(&self) -> Row<Message> {
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
        let max_voltage_slider =
            VerticalSlider::new(10.0..=200.0, self.max_voltage, Message::MaxVoltageSelected)
                .step(1.0)
                .height(200.0);
        let pause_button = if self.paused {
            Button::new("unpause")
        } else {
            Button::new("pause")
        }
        .on_press(Message::PauseUnpause);

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
                .push(Space::new(30.0, 30.0))
                .push(max_voltage_slider)
                .push(Space::new(30.0, 10.0))
                .push(text(format!(
                    "time correctness: {:.2} %",
                    self.time_correctness * 100.0
                )))
                .push(Space::new(30., 30.))
                .push(pause_button)
        } else {
            control_row.push(toggle_chart_controls)
        }
    }

    fn view_modbus(&self) -> Element<Message> {
        let register_text_input = text_input(
            "enter register address of holding",
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
        let row1 = Row::new()
            .push(register_col)
            .push(realtime_col)
            .push(realtime_status_col)
            .push(rated_col)
            .spacing(30);
        let row2 = Row::new().push(self.view_settings());
        Column::new().push(spacer()).push(row1).push(row2).into()
    }

    fn view_settings(&self) -> Element<Message> {
        Row::new()
            .push(spacer())
            .push(self.view_voltage_settings())
            .into()
    }

    fn view_voltage_settings(&self) -> Element<Message> {
        let s = self.voltage_settings;
        let get_voltage_settings_button =
            Button::new("get voltage settings").on_press(Message::ReadVoltageSettings);
        let set_buttons_col = Column::new()
            .push(Space::new(1, 4))
            .push(Button::new("set battery type"))
            .push(Button::new("set over voltage disconnect"))
            .push(Button::new("set charging limit voltage"))
            .push(Button::new("set over voltage reconnect"))
            .push(Button::new("set equalization voltage"))
            .push(Button::new("set boost voltage"))
            .push(Button::new("set float voltage"))
            .push(Button::new("set boost reconnect voltage"))
            .push(Button::new("set low voltage reconnect"))
            .push(Button::new("set under voltage recover"))
            .push(Button::new("set under voltage warning"))
            .push(Button::new("set low voltage disconnect"))
            .push(Button::new("set discharging limit voltage"))
            .push(Text::new("Settings valid?"));
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
            .push(Text::new(format!("{:?}", s.check_settings_lifepo4())))
            .spacing(10);
        let voltage_settings_input_col = Column::new();
        let row = Row::new()
            .push(set_buttons_col)
            .push(spacer())
            .push(display_voltage_settings_col)
            .push(spacer())
            .push(voltage_settings_input_col);
        Column::new()
            .push(spacer())
            .push(get_voltage_settings_button)
            .push(row)
            .into()
    }

    fn view_rated(&self) -> Element<Message> {
        let read_rated_button = Button::new("read rated").on_press(Message::ReadRated);
        let rated_text = Text::new(format!("{}", self.rated_data));
        Column::new()
            .push(read_rated_button)
            .push(spacer())
            .push(rated_text)
            .into()
    }

    pub fn update_battery2(&mut self) {
        let voltages = self
            .battery_pack
            .voltages
            .iter()
            .zip(self.battery1.voltages.iter())
            .map(|(bp_voltage, b1_voltage)| bp_voltage - b1_voltage);
        self.battery2.voltages = voltages.collect();
        self.battery2
            .accumulate_into_view_buffer(self.selected_time_interval.accumulations());
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

    pub fn adjust_max_voltage(&mut self) {
        self.battery1.max_voltage = 0.25 * self.max_voltage;
        self.battery2.max_voltage = 0.25 * self.max_voltage;
        self.battery_pack.max_voltage = 0.5 * self.max_voltage;
        self.pv.max_voltage = self.max_voltage;
    }

    pub fn clear_caches(&mut self) {
        self.map_charts(|vc| vc.cache.clear());
    }

    fn map_charts<F: FnMut(&mut VoltageChart)>(&mut self, f: F) {
        [
            &mut self.battery_pack,
            &mut self.battery1,
            &mut self.battery2,
            &mut self.pv,
        ]
        .map(f);
    }
}

#[derive(Debug, Copy, Clone)]
pub enum SelectedTab {
    VoltageCharts,
    Modbus,
}

fn spacer() -> Space {
    Space::new(30, 30)
}
