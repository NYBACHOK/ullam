use super::{FieldValue, LogField, LogValue, MessageSchemaProvider, ParseError, default_parse};

/// Handles Control/PID messages (PIDR, PIDP, CTUN, NTUN).
pub struct ControlSchemaProvider;

impl MessageSchemaProvider for ControlSchemaProvider {
    fn handles_message(&self, msg_name: &str) -> bool {
        msg_name.starts_with("PID")
            || msg_name == "CTUN"
            || msg_name == "NTUN"
            || msg_name == "RATE"
    }

    fn format_field(
        &self,
        msg_name: &str,
        label: String,
        value: FieldValue,
    ) -> Result<LogField, ParseError> {
        let field_value = match label.as_str() {
            // PID terms (P, I, D, FF) and Targets/Actuals are floats
            "Tar" | "Act" | "Err" | "P" | "I" | "D" | "FF" | "DFF" | "DesRoll" | "Roll"
            | "DesPitch" | "Pitch" | "DesYaw" | "Yaw" => match value {
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
            "Flags" => match value {
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
            "SRate" => match value {
                FieldValue::Float(val) => LogValue::F64(val),
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
