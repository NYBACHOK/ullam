use super::{FieldValue, LogField, LogValue, MessageSchemaProvider, ParseError, default_parse};

/// Handles RC Input/Output messages (RCIN, RCOU).
pub struct RcSchemaProvider;

impl MessageSchemaProvider for RcSchemaProvider {
    fn handles_message(&self, msg_name: &str) -> bool {
        msg_name == "RCIN" || msg_name == "RCOU" || msg_name == "RCI2" || msg_name == "RCO2"
    }

    fn format_field(&self, label: String, value: FieldValue) -> Result<LogField, ParseError> {
        fn is_rc_channel(label: &str) -> bool {
            let Some(num_part) = label.strip_prefix('C') else {
                return false;
            };
            num_part.parse::<u32>().is_ok_and(|n| (1..=32).contains(&n))
        }

        let field_value = match label.as_str() {
            // RC channels are typically u16 (1000-2000)
            s if is_rc_channel(s) => match value {
                FieldValue::Uint(val) => LogValue::U64(val),
                FieldValue::Int(val) => LogValue::I64(val),
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
