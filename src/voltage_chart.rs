use crate::{
    adc_reading_to_voltage, remote_data::RemoteData, time_interval::TimeInterval, Message,
};
use canvas::{Frame, Geometry};
use iced::widget::canvas::Cache;
use iced::widget::*;
use iced::*;
use plotters_iced::{Chart, ChartWidget};
use std::{collections::VecDeque, ops::Range};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ChartType {
    #[default]
    Voltage,
    Power,
}

#[derive(Debug)]
pub struct CustomChart {
    pub title: String,
    pub data: VecDeque<f32>,
    pub display_data: VecDeque<(f32, f32)>,
    pub cache: Cache,
    /// bottom border of the displayed chart
    pub min_y: f32,
    /// top border of the displayed chart
    pub max_y: f32,
    /// left border of the displayed chart
    pub min_time: f32,
    /// right border of the displayed chart
    pub max_time: f32,
    /// indices into data for actually visible data
    pub data_range: Range<usize>,
    /// time between 2 voltage measurements
    pub tick_len: f32,
    pub chart_type: ChartType,
}

impl Default for CustomChart {
    fn default() -> Self {
        Self {
            title: Default::default(),
            data: Default::default(),
            display_data: Default::default(),
            // drawing cache should be cleared if new data arrives
            cache: Default::default(),
            min_y: 0.0,
            max_y: 100.0,
            min_time: -100.0,
            max_time: 0.0,
            data_range: (0..0),
            tick_len: 0.02,
            chart_type: Default::default(),
        }
    }
}

impl CustomChart {
    pub fn update_voltages_from_remote(&mut self, remote_data: &mut RemoteData, num_acc: usize) {
        let adc_readings = remote_data.take_adc_readings();
        let voltages: VecDeque<f32> = adc_readings
            .iter()
            .map(|adc_reading| adc_reading_to_voltage(*adc_reading))
            .collect();
        self.data.try_reserve(voltages.len()).ok();
        for voltage in voltages {
            self.data.push_back(voltage);
        }

        self.accumulate_into_view_buffer(num_acc);
        self.cache.clear();
    }

    pub fn update_power_from_remote(&mut self, remote_data: &mut RemoteData, num_acc: usize) {
        let power_readings = remote_data.take_power_readings();
        let power_values: VecDeque<f32> = power_readings
            .iter()
            .map(|&power_reading| power_reading as f32)
            .collect();
        if !power_values.is_empty() {
            println!("new power values : {:?}", power_values);
        }
        self.data.try_reserve(power_values.len()).ok();
        for power_value in power_values {
            self.data.push_back(power_value);
        }

        self.accumulate_into_view_buffer(num_acc);
        self.cache.clear();
    }

    fn update_data_range(&mut self) {
        self.data_range.start = self.index_for_time(self.min_time);
        self.data_range.end = (1 + self.index_for_time(self.max_time)).min(self.data.len());
    }

    fn time_for_index(&self, ix: usize) -> f32 {
        let max_ix = if self.data.is_empty() {
            0
        } else {
            self.data.len() - 1
        };
        let b = max_ix as f32 * -self.tick_len;
        let m = self.tick_len;
        m * (ix as f32) + b
    }

    fn index_for_time(&self, time: f32) -> usize {
        if self.data.is_empty() {
            return 0;
        }
        let max_ix = self.data.len() - 1;
        let b = max_ix as u32;
        let m = 1.0 / self.tick_len;
        let res = m * time + b as f32;
        (res as u32).clamp(0, max_ix as u32) as usize
    }

    fn range_time(&self, Range { start, end }: Range<usize>) -> f32 {
        let mid = start + ((end - start) / 2);
        self.time_for_index(mid)
    }

    pub fn accumulate_into_view_buffer(&mut self, num_acc: usize) {
        self.update_data_range();
        self.display_data.clear();
        let mut acc_voltage = 0.0;
        let mut acc_count = 1;
        let mut time = 0.0;
        for (offset, val) in self.data.range(self.data_range.clone()).enumerate() {
            if acc_count == 1 {
                time = self.range_time(
                    (self.data_range.start + offset)..(self.data_range.start + offset + num_acc),
                );
            }

            acc_voltage += val;

            if acc_count == num_acc {
                self.display_data
                    .push_back((time, acc_voltage / (num_acc as f32)));
                acc_count = 1;
                acc_voltage = 0.0;
            } else {
                acc_count += 1;
            }
        }
    }

    pub fn view(&self, _idx: usize, chart_height: f32) -> Element<Message> {
        Column::new()
            .width(Length::Fill)
            .height(Length::Shrink)
            .spacing(5)
            .align_items(Alignment::Center)
            .push(Text::new(self.title.clone()))
            .push(ChartWidget::new(self).height(Length::Fixed(chart_height)))
            .into()
    }

    pub fn adjust_time_interval(&mut self, time_interval: TimeInterval) {
        let interval_seconds = time_interval.to_seconds();
        self.max_time = self.max_time.min(0.0);
        self.min_time = self.max_time - interval_seconds;
        match self.chart_type {
            ChartType::Voltage => {
                self.accumulate_into_view_buffer(time_interval.accumulations());
            }
            ChartType::Power => {
                self.accumulate_into_view_buffer(1);
            }
        }
        self.cache.clear();
    }
}

impl Chart<Message> for CustomChart {
    type State = ();

    #[inline]
    fn draw<R: plotters_iced::Renderer, F: Fn(&mut Frame)>(
        &self,
        renderer: &R,
        bounds: Size,
        draw_fn: F,
    ) -> Geometry {
        renderer.draw_cache(&self.cache, bounds, draw_fn)
    }

    fn build_chart<DB: plotters::prelude::DrawingBackend>(
        &self,
        _state: &Self::State,
        mut builder: plotters::prelude::ChartBuilder<DB>,
    ) {
        use plotters::prelude::*;
        const PLOT_LINE_COLOR: RGBColor = RGBColor(0, 175, 255);

        let y_unit_text = match self.chart_type {
            ChartType::Voltage => "V",
            ChartType::Power => "W",
        };

        let mut chart = builder
            .x_label_area_size(28)
            .y_label_area_size(50)
            .margin(20)
            .build_cartesian_2d(self.min_time..self.max_time, self.min_y..self.max_y)
            .expect("failed to build chart");

        chart
            .configure_mesh()
            .bold_line_style(plotters::style::colors::BLUE.mix(0.1))
            .light_line_style(plotters::style::colors::BLUE.mix(0.05))
            .axis_style(ShapeStyle::from(plotters::style::colors::BLUE.mix(0.45)).stroke_width(1))
            .x_labels(10)
            .y_labels(20)
            .y_label_style(
                ("mono", 15.0)
                    .into_font()
                    .color(&plotters::style::colors::GREEN.mix(0.9))
                    .transform(FontTransform::Rotate90),
            )
            .x_label_style(
                ("mono", 15.0)
                    .into_font()
                    .color(&plotters::style::colors::CYAN),
            )
            .x_label_formatter(&|x| {
                if self.max_time - self.min_time <= 300.0 {
                    format!("{:.1}s", x)
                } else {
                    format!("{:.0}m", x / 60.0)
                }
            })
            .y_label_formatter(&|y| format!("{:.1} {}", y, y_unit_text))
            .draw()
            .expect("failed to draw chart mesh");

        chart
            .draw_series(
                AreaSeries::new(
                    self.display_data.iter().cloned(),
                    0.0,
                    PLOT_LINE_COLOR.mix(0.175),
                )
                .border_style(ShapeStyle::from(PLOT_LINE_COLOR).stroke_width(2)),
            )
            .expect("failed to draw chart data");
    }
}
