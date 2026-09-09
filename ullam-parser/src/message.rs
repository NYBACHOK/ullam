use crate::types::FormatField;

use super::*;

/// Supplies the schema interpretation for a DataFlash message.
pub trait ArduPilotMessage {
    fn format(
        &self,
        id: u8,
        name: &str,
        encoding: &str,
        labels: &[String],
    ) -> Result<LogFormat, ParseError>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultArduPilotMessage;

impl ArduPilotMessage for DefaultArduPilotMessage {
    fn format(
        &self,
        id: u8,
        name: &str,
        encoding: &str,
        labels: &[String],
    ) -> Result<LogFormat, ParseError> {
        let fields = encoding
            .chars()
            .enumerate()
            .map(|(index, code)| {
                Ok::<FormatField, ParseError>(FormatField {
                    name: labels
                        .get(index)
                        .cloned()
                        .unwrap_or_else(|| format!("field_{index}")),
                    ty: field_type(code, labels.get(index).map(String::as_str))?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(LogFormat {
            id,
            name: name.to_owned(),
            timestamp: timestamp_type(encoding, labels),
            fields,
        })
    }
}
