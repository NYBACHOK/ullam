use super::{FieldValue, LogField, LogValue, MessageSchemaProvider, ParseError, default_parse};

/// Handles IMU messages (`msg_type` 164).
/// Format: TimeUS,I,GyrX,GyrY,GyrZ,AccX,AccY,AccZ,EG,EA,T,GH,AH,GHz,AHz
pub struct ImuSchemaProvider;

impl MessageSchemaProvider for ImuSchemaProvider {
    fn handles_message(&self, msg_name: &str) -> bool {
        msg_name == "IMU"
    }

    fn format_field(
        &self,
        msg_name: &str,
        label: String,
        value: FieldValue,
    ) -> Result<LogField, ParseError> {
        let field_value = match label.as_str() {
            // Gyro and Accelerometer values are typically Float32 in logs, stored as f64
            "GyrX" | "GyrY" | "GyrZ" | "AccX" | "AccY" | "AccZ" | "EG" | "EA" | "T" | "GH"
            | "AH" | "GHz" | "AHz" => match value {
                FieldValue::Float(val) => LogValue::F64(val),
                FieldValue::Int(val) => LogValue::F64(val as f64),
                FieldValue::Uint(val) => LogValue::F64(val as f64),
                _ => {
                    return Err(ParseError::InvalidLabelType {
                        label,
                        value,
                        message: msg_name.to_owned(),
                    });
                }
            },
            "I" => match value {
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
