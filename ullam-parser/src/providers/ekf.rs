use super::{FieldValue, LogField, LogValue, MessageSchemaProvider, ParseError, default_parse};

/// Handles EKF messages (NKF1-NKF5, XKF1-XKF5).
/// Contains Lat/Lng, Velocities, Attitudes, and Innovation data.
pub struct EkfSchemaProvider;

impl MessageSchemaProvider for EkfSchemaProvider {
    fn handles_message(&self, msg_name: &str) -> bool {
        msg_name.starts_with("NKF") || msg_name.starts_with("XKF")
    }

    fn format_field(
        &self,
        msg_name: &str,
        label: String,
        value: FieldValue,
    ) -> Result<LogField, ParseError> {
        let field_value = match label.as_str() {
            // Latitude/Longitude are often stored as Int32 (degrees * 1e7) in ArduPilot logs
            "Lat" | "Lng" | "PN" | "PE" | "PD" | "PVN" | "PVE" | "PVD" => {
                match value {
                    FieldValue::Int(val) => LogValue::Latitude(val as i32), // Simplified mapping
                    FieldValue::Uint(val) => LogValue::Longitude(val as i32),
                    FieldValue::Float(val) => LogValue::F64(val),
                    _ => {
                        return Err(ParseError::InvalidLabelType {
                            label,
                            value,
                            message: msg_name.to_owned(),
                        });
                    }
                }
            }
            // Velocity and Position errors
            "VN" | "VE" | "VD" | "IVN" | "IVE" | "IVD" | "RErr" | "ErSc" => match value {
                FieldValue::Float(val) => LogValue::F64(val),
                FieldValue::Int(val) => LogValue::F64(val as f64),
                _ => {
                    return Err(ParseError::InvalidLabelType {
                        label,
                        value,
                        message: msg_name.to_owned(),
                    });
                }
            },
            "C" => match value {
                FieldValue::Int(val) => LogValue::U64(val as u64),
                FieldValue::Uint(val) => LogValue::U64(val),
                _ => {
                    return Err(ParseError::InvalidLabelType {
                        label,
                        value,
                        message: msg_name.to_owned(),
                    });
                }
            },
            _ => default_parse(msg_name, value, &label)?,
        };

        Ok(LogField {
            name: label,
            value: field_value,
        })
    }
}
