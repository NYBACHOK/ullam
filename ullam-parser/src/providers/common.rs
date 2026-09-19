use super::{FieldValue, LogField, MessageSchemaProvider, ParseError, default_parse};

/// Handles generic messages that don't have a specialized provider.
/// Implements the `MessageSchemaProvider` trait to act as the default router/fallback.
pub struct CommonSchemaProvider;

impl MessageSchemaProvider for CommonSchemaProvider {
    fn handles_message(&self, _msg_name: &str) -> bool {
        // This provider acts as the catch-all. It will be called if no other provider matches.
        true
    }

    fn format_field(&self, label: String, value: FieldValue) -> Result<LogField, ParseError> {
        let field_value = match label.as_str() {
            _ => default_parse(value, &label)?,
        };

        Ok(LogField {
            name: label,
            value: field_value,
        })
    }
}
