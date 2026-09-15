use super::*;

/// Handles IMU messages (msg_type 164).
/// Format: TimeUS,I,GyrX,GyrY,GyrZ,AccX,AccY,AccZ,EG,EA,T,GH,AH,GHz,AHz
pub struct ImuSchemaProvider;

impl MessageSchemaProvider for ImuSchemaProvider {
    fn handles_message(&self, msg_name: &str) -> bool {
        msg_name == "IMU"
    }

    fn format_field(&self, label: String, value: FieldValue) -> Result<LogField, ParseError> {
        let field_value = match label.as_str() {
            TIME_US_LABEL => match value {
                FieldValue::Uint(val) => LogValue::Time(Duration::from_micros(val)),
                _ => return Err(ParseError::InvalidLabelType { label, value }),
            },
            // Gyro and Accelerometer values are typically Float32 in logs, stored as f64
            "GyrX" | "GyrY" | "GyrZ" | "AccX" | "AccY" | "AccZ" | "EG" | "EA" | "T" | "GH"
            | "AH" | "GHz" | "AHz" => match value {
                FieldValue::Float(val) => LogValue::F64(val as f64),
                FieldValue::Int(val) => LogValue::F64(val as f64),
                FieldValue::Uint(val) => LogValue::F64(val as f64),
                _ => return Err(ParseError::InvalidLabelType { label, value }),
            },
            "I" => match value {
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
