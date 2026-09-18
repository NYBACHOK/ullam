use std::time::Duration;

use ardupilot_binlog::FieldValue;

use crate::{LogField, LogValue, ParseError};

mod common;
mod control;
mod ekf;
mod gps;
mod imu;
mod rc;

pub use self::{common::*, control::*, ekf::*, gps::*, imu::*, rc::*};

const TIME_US_LABEL: &str = "TimeUS";

/// Trait for parsing individual fields within a specific message context.
pub trait MessageSchemaProvider: Send + Sync {
    /// Returns true if this provider handles the given message name (e.g., "IMU", "NKF1").
    fn handles_message(&self, msg_name: &str) -> bool;

    /// Formats a specific field value based on the message context.
    ///
    /// # Arguments
    /// * `label` - The field label (e.g., "Lat", "Roll", "TimeUS").
    /// * `value` - The raw parsed value from the binary log.
    fn format_field(&self, label: String, value: FieldValue) -> Result<LogField, ParseError>;
}

/// Helper function for default parsing of unknown fields.
fn default_parse(value: FieldValue, label: &str) -> Result<LogValue, ParseError> {
    let value = match label {
        TIME_US_LABEL => match value {
            FieldValue::Uint(val) => LogValue::Time(Duration::from_micros(val)),
            _ => {
                return Err(ParseError::InvalidLabelType {
                    label: label.to_owned(),
                    value,
                });
            }
        },
        "I" => match value {
            FieldValue::Int(val) => LogValue::I64(val),
            FieldValue::Uint(val) => LogValue::U64(val),
            _ => {
                return Err(ParseError::InvalidLabelType {
                    label: label.to_owned(),
                    value,
                });
            }
        },
        _ => match value {
            FieldValue::Int(val) => LogValue::I64(val),
            FieldValue::Uint(val) => LogValue::U64(val),
            FieldValue::Float(val) => LogValue::F64(val as f64),
            FieldValue::String(val) => LogValue::String(val),
            FieldValue::Array(items) => LogValue::Array(items),
        },
    };

    Ok(value)
}
