use std::time::Duration;


#[derive(Debug, Clone, PartialEq)]
pub struct LogFormat {
    pub id: u8,
    pub name: String,
    pub timestamp: Option<Duration>,
    pub fields: Vec<FormatField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FormatField {
    pub name: String,
    pub ty: FieldType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,

    // ArduPilot-specific
    Latitude,
    Longitude,
    FlightMode,
    ScaledI16(f64),
    ScaledI32(f64),
    String(usize),
}