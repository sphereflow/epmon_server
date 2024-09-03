pub struct Tracer {
    rated: Rated,
    realtime: Realtime,
    realtime_status: RealtimeStatus,
    stats: Stats,
    settings: Settings,
}

pub struct Rated {
    array_rated_voltage: f32,
    array_rated_current: f32,
    array_rated_power: f32,
    battery_rated_voltage: f32,
    battery_rated_current: f32,
    battery_rated_power: f32,
    charging_mode: ChargingMode,
    rated_current_load: f32,
}

pub enum ChargingMode {
    ConnectDisconnect = 0x00,
    PWM,
    MPPT,
}

pub struct Realtime {
    pv_voltage: f32,
    pv_current: f32,
    pv_power: f32,
    battery_power: f32,
    load_voltage: f32,
    load_current: f32,
    load_power: f32,
    battery_temperature: f32,
    equipment_temperature: f32,
    remaining_battery_capacity: f32,
    remote_battery_temperature: f32,
    battery_real_rated_power: f32,
}

pub struct RealtimeStatus {
    battery_status: BatteryStatus,
    charging_equipment_status: ChargingEquipmentStatus,
    discharging_equipment_status: DischargingEquipmentStatus,
}

pub enum BatteryStatus {}

pub enum ChargingEquipmentStatus {}

pub enum DischargingEquipmentStatus {}

pub struct Stats {}

pub struct Settings {}
