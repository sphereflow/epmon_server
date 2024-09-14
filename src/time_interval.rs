use std::ops::RangeInclusive;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeInterval {
    Second,
    TenSeconds,
    ThirtySeconds,
    #[default]
    Minute,
    FiveMinutes,
    ThirtyMinutes,
    Hour,
    ThreeHours,
    SixHours,
    TwelveHours,
    Day,
}

impl TimeInterval {
    pub fn to_seconds(self) -> f32 {
        match self {
            TimeInterval::Second => 1.0,
            TimeInterval::TenSeconds => 10.0,
            TimeInterval::ThirtySeconds => 30.0,
            TimeInterval::Minute => 60.0,
            TimeInterval::FiveMinutes => 300.0,
            TimeInterval::ThirtyMinutes => 1800.0,
            TimeInterval::Hour => 3600.0,
            TimeInterval::ThreeHours => 3.0 * 3600.0,
            TimeInterval::SixHours => 6.0 * 3600.0,
            TimeInterval::TwelveHours => 12.0 * 3600.0,
            TimeInterval::Day => 24.0 * 3600.0,
        }
    }

    pub fn interval(&self) -> RangeInclusive<f32> {
        0.0..=self.to_seconds()
    }
}
