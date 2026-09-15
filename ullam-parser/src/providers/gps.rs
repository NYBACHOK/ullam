use super::*;

/// Handles GPS messages (GPS, GPA, GRAW).
pub struct GpsSchemaProvider;

impl MessageSchemaProvider for GpsSchemaProvider {
    fn handles_message(&self, msg_name: &str) -> bool {
        msg_name == "GPS" || msg_name == "GPA" || msg_name == "GRAW"
    }

    fn format_field(&self, label: String, value: FieldValue) -> Result<LogField, ParseError> {
        let field_value = match label.as_str() {
            TIME_US_LABEL => match value {
                FieldValue::Uint(val) => LogValue::Time(Duration::from_micros(val)),
                _ => return Err(ParseError::InvalidLabelType { label, value }),
            },
            "Lat" | "Lng" => {
                // ArduPilot GPS coordinates are Int32, degrees * 1e7
                match value {
                    FieldValue::Int(val) => LogValue::Latitude(val as i32),
                    FieldValue::Uint(val) => LogValue::Longitude(val as i32),
                    _ => return Err(ParseError::InvalidLabelType { label, value }),
                }
            }
            "Alt" | "Spd" | "HDop" | "VDop" => match value {
                FieldValue::Float(val) => LogValue::F64(val as f64),
                FieldValue::Int(val) => LogValue::F64(val as f64),
                _ => return Err(ParseError::InvalidLabelType { label, value }),
            },
            "Status" | "NSats" | "I" => match value {
                FieldValue::Int(val) => LogValue::U64(val as u64),
                FieldValue::Uint(val) => LogValue::U64(val),
                _ => return Err(ParseError::InvalidLabelType { label, value }),
            },
            _ => default_parse(value, &label)?,
        };

        Ok(LogField {
            name: label,
            value: field_value,
        })
    }
}
