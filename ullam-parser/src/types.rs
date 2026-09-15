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
    Latitude(i32),  // Degrees * 1e7
    Longitude(i32), // Degrees * 1e7
    Altitude(f64),  // Meters
    FlightMode(u8),
    ScaledI16(f64), // int16_t * scale
    ScaledI32(f64), // int32_t * scale
    String(String),
}
