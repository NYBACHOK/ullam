use std::time::Duration;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct LogEntry {
    pub id: u8, // msg_type
    pub name: String,
    /// Timestamp in microseconds since boot (from the first Q-typed field).
    pub timestamp: Option<Duration>,

    pub fields: Vec<LogField>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct LogField {
    pub name: String,
    pub value: LogValue,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum LogValue {
    I64(i64),
    U64(u64),
    F64(f64),
    Array(Vec<i16>),

    // ArduPilot Specific
    Time(Duration),
    Latitude(i32),  // Degrees
    Longitude(i32), // Degrees
    Altitude(f64),  // Meters
    FlightMode(u8),
    ScaledI16(f64), // int16_t * scale
    ScaledI32(f64), // int32_t * scale
    String(String),
}

impl LogValue {
    #[must_use]
    pub fn try_into_float(self) -> Option<f64> {
        match self {
            LogValue::I64(value) => Some(value as f64),
            LogValue::U64(value) => Some(value as f64),
            LogValue::F64(value)
            | LogValue::Altitude(value)
            | LogValue::ScaledI16(value)
            | LogValue::ScaledI32(value) => Some(value),
            LogValue::Latitude(value) | LogValue::Longitude(value) => Some(f64::from(value)),
            LogValue::Time(value) => Some(value.as_secs_f64()),
            LogValue::FlightMode(value) => Some(f64::from(value)),
            LogValue::Array(_) | LogValue::String(_) => None,
        }
    }
}
