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

    pub fn accumulations(self) -> usize {
        match self {
            TimeInterval::Second => 1,
            TimeInterval::TenSeconds => 1,
            TimeInterval::ThirtySeconds => 3,
            TimeInterval::Minute => 6,
            TimeInterval::FiveMinutes => 30,
            TimeInterval::ThirtyMinutes => 180,
            TimeInterval::Hour => 360,
            TimeInterval::ThreeHours => 360 * 3,
            TimeInterval::SixHours => 360 * 6,
            TimeInterval::TwelveHours => 360 * 12,
            TimeInterval::Day => 360 * 24,
        }
    }
}
