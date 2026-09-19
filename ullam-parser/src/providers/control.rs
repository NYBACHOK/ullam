use super::{
    Duration, FieldValue, LogField, LogValue, MessageSchemaProvider, ParseError, TIME_US_LABEL,
    default_parse,
};

/// Handles Control/PID messages (PIDR, PIDP, CTUN, NTUN).
pub struct ControlSchemaProvider;

impl MessageSchemaProvider for ControlSchemaProvider {
    fn handles_message(&self, msg_name: &str) -> bool {
        msg_name.starts_with("PID")
            || msg_name == "CTUN"
            || msg_name == "NTUN"
            || msg_name == "RATE"
    }

    fn format_field(&self, label: String, value: FieldValue) -> Result<LogField, ParseError> {
        let field_value = match label.as_str() {
            TIME_US_LABEL => match value {
                FieldValue::Uint(val) => LogValue::Time(Duration::from_micros(val)),
                _ => return Err(ParseError::InvalidLabelType { label, value }),
            },
            // PID terms (P, I, D, FF) and Targets/Actuals are floats
            "Tar" | "Act" | "Err" | "P" | "I" | "D" | "FF" | "DFF" | "DesRoll" | "Roll"
            | "DesPitch" | "Pitch" | "DesYaw" | "Yaw" => match value {
                FieldValue::Float(val) => LogValue::F64(val),
                FieldValue::Int(val) => LogValue::F64(val as f64),
                _ => return Err(ParseError::InvalidLabelType { label, value }),
            },
            "Flags" => match value {
                FieldValue::Int(val) => LogValue::U64(val as u64),
                FieldValue::Uint(val) => LogValue::U64(val),
                _ => return Err(ParseError::InvalidLabelType { label, value }),
            },
            "SRate" => match value {
                FieldValue::Float(val) => LogValue::F64(val),
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
