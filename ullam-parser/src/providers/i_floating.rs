use super::{FieldValue, LogField, LogValue, MessageSchemaProvider, ParseError, default_parse};

pub struct IFloatingSchemaProvider;

impl MessageSchemaProvider for IFloatingSchemaProvider {
    fn handles_message(&self, msg_name: &str) -> bool {
        msg_name.starts_with("TEC2")
            || msg_name.starts_with("TEC3")
            || msg_name.starts_with("PLNX")
            || msg_name.starts_with("PLNV")
            || msg_name.starts_with("PLNY")
    }

    fn format_field(
        &self,
        msg_name: &str,
        label: String,
        value: FieldValue,
    ) -> Result<LogField, ParseError> {
        let field_value = match label.as_str() {
            "I" => match value {
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
