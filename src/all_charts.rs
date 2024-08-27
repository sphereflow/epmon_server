use crate::{time_interval::TimeInterval, voltage_chart::VoltageChart, Message, CHART_HEIGHT};
use iced::{widget::*, Alignment, Element, Length};

#[derive(Debug)]
pub struct AllCharts {
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
    pub holding_val: Vec<u8>,
    pub paused: bool,
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
            paused: false,
            register_address: 0,
            register_address_string: String::new(),
            holding_val: Vec::new(),
        }
    }
}

impl AllCharts {
    pub fn view(&self) -> Element<Message> {
        let mut control_row = Row::new();
        let selected = self.selected_time_interval;

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
        let spacer = iced::widget::Space::new(30.0, 30.0);
        control_row = control_row
            .push(time_intervall_column1)
            .push(time_intervall_column2)
            .push(spacer);

        let max_time_slider_day = Slider::new(
            (-3600.0 * 24.0)..=0.0,
            self.max_time_day,
            Message::MaxTimeDaySelected,
        )
        .width(1000);
        let max_time_slider = Slider::new(-3600.0..=0.0, self.max_time, Message::MaxTimeSelected)
            .step(1.0)
            .shift_step(0.1)
            .width(1000);
        let max_time_slider_fine = Slider::new(
            -100.0..=0.0,
            self.max_time_fine,
            Message::MaxTimeFineSelected,
        )
        .step(0.1)
        .width(1000);
        control_row = control_row
            .push(iced::widget::column![
                max_time_slider_day,
                Space::new(30.0, 30.0),
                max_time_slider,
                Space::new(30.0, 30.0),
                max_time_slider_fine,
            ])
            .push(Space::new(30.0, 30.0));

        let max_voltage_slider =
            VerticalSlider::new(10.0..=200.0, self.max_voltage, Message::MaxVoltageSelected)
                .step(1.0)
                .height(200.0);
        control_row = control_row
            .push(max_voltage_slider)
            .push(Space::new(30.0, 10.0))
            .push(text(format!(
                "time correctness: {:.2} %",
                self.time_correctness * 100.0
            )));

        let pause_button = if self.paused {
            Button::new("unpause")
        } else {
            Button::new("pause")
        }
        .on_press(Message::PauseUnpause);
        let spacer = iced::widget::Space::new(30.0, 30.0);
        let mut pause_holding_col = Column::new();

        let register_text_input = text_input(
            "enter register address of holding",
            &self.register_address_string,
        )
        .on_input(Message::AddressInput);
        let holding_button = Button::new("get holding val").on_press(Message::ReadHoldings {
            register_address: self.register_address,
            size: 1,
        });
        let register_button = Button::new("get").on_press(Message::ReadRegisters {
            register_address: self.register_address,
            size: 1,
        });
        let register_numeric_text =
            text(format!("numeric register value: {}", self.register_address));
        let holding_text = text(format!("holding val: {:?}", self.holding_val));
        pause_holding_col = pause_holding_col
            .push(pause_button)
            .push(register_text_input)
            .push(holding_button)
            .push(register_button)
            .push(register_numeric_text)
            .push(holding_text);

        control_row = control_row.push(spacer).push(pause_holding_col);

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
        let col = Column::new()
            .width(Length::Fill)
            .height(Length::Shrink)
            .align_items(Alignment::Center)
            .push(control_row)
            .push(row1)
            .push(row2);

        Scrollable::new(col).height(Length::Shrink).into()
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
