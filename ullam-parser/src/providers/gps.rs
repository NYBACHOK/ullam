use super::{FieldValue, LogField, LogValue, MessageSchemaProvider, ParseError, default_parse};

/// Handles GPS messages (GPS, GPA, GRAW).
pub struct GpsSchemaProvider;

impl MessageSchemaProvider for GpsSchemaProvider {
    fn handles_message(&self, msg_name: &str) -> bool {
        msg_name == "GPS" || msg_name == "GPA" || msg_name == "GRAW"
    }

    fn format_field(
        &self,
        msg_name: &str,
        label: String,
        value: FieldValue,
    ) -> Result<LogField, ParseError> {
        let field_value = match label.as_str() {
            "Lat" | "Lng" => {
                // ArduPilot GPS coordinates are Int32, degrees * 1e7
                match value {
                    FieldValue::Int(val) => LogValue::F64(val as f64 / 10_000_000.0),
                    FieldValue::Uint(val) => LogValue::F64(val as f64 / 10_000_000.0),
                    _ => {
                        return Err(ParseError::InvalidLabelType {
                            label,
                            value,
                            message: msg_name.to_owned(),
                        });
                    }
                }
            }
            "Alt" | "Spd" | "HDop" | "VDop" => match value {
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
            "Status" | "NSats" => match value {
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
