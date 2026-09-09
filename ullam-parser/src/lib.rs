use std::collections::HashMap;
use std::io::{BufReader, Read};
use std::time::Duration;

use crate::types::{FieldType, LogFormat};
use crate::utils::{
    decode_values, extract_timestamp, field_type, fixed_string, parsed_fields, payload_size,
    read_exact_or_eof, timestamp_type,
};

mod message;
mod types;
mod utils;

pub use self::message::*;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("unexpected end of DataFlash record")]
    UnexpectedEof,
    #[error("invalid DataFlash record header")]
    InvalidHeader,
    #[error("invalid format for {message}: {format}")]
    InvalidFormat { message: String, format: String },
    #[error("invalid {message} record")]
    InvalidRecord { message: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub value: LogValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogValue {
    I8(i8),
    U8(u8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    F32(f32),
    F64(f64),
    Latitude(i32),
    Longitude(i32),
    FlightMode(u8),
    Scaled(f64),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogEntry {
    pub id: u8,
    pub name: String,

    pub fields: Vec<Field>,
    /// Timestamp in microseconds since boot (from the first Q-typed field).
    pub timestamp: Option<Duration>,
    pub values: Vec<LogValue>,
}

impl LogEntry {
    pub fn get(&self, name: &str) -> Option<&LogValue> {
        self.fields
            .iter()
            .position(|field| field.name == name)
            .and_then(|index| self.fields.get(index))
            .map(|field| &field.value)
    }
}

/// Lazy iterator over entries in an ArduPilot DataFlash BIN log.
///
/// The underlying parser reads only the next record when [`Iterator::next`]
/// is called. `FMT` records are consumed as they appear and automatically
/// define the schemas used by subsequent records.
pub struct DataFlashReader<R: Read, M> {
    reader: BufReader<R>,
    formats: HashMap<u8, LogFormat>,
    schema_provider: M,
}

impl<R: Read, M: ArduPilotMessage> DataFlashReader<R, M> {
    /// Create a reader using a custom ArduPilot message-definition type.
    pub fn new_with_message_type(reader: R, schema_provider: M) -> Self {
        Self {
            reader: BufReader::new(reader),
            formats: HashMap::new(),
            schema_provider,
        }
    }

    /// Return the message schemas discovered while iterating.
    #[must_use]
    pub fn formats(&self) -> &HashMap<u8, LogFormat> {
        &self.formats
    }

    fn next_record(&mut self) -> Result<Option<LogEntry>, ParseError> {
        let mut header = [0u8; 3];
        if !read_exact_or_eof(&mut self.reader, &mut header)? {
            return Ok(None);
        }
        if header[..2] != [0xA3, 0x95] {
            return Err(ParseError::InvalidHeader);
        }

        let id = header[2];
        if id == 0x80 {
            return self.read_fmt();
        }

        let format = self
            .formats
            .get(&id)
            .cloned()
            .ok_or(ParseError::InvalidRecord {
                message: format!("unknown message type {id}"),
            })?;
        let payload_length = payload_size(&format.fields);
        let mut payload = vec![0; payload_length];
        self.reader.read_exact(&mut payload)?;
        let values = decode_values(&format.fields, &payload)?;
        let timestamp = extract_timestamp(&format, &payload);
        
        Ok(Some(LogEntry {
            id: format.id,
            name: format.name.clone(),
            fields: parsed_fields(&format.fields, &values),
            timestamp,
            values,
        }))
    }

    fn read_fmt(&mut self) -> Result<Option<LogEntry>, ParseError> {
        let mut payload = [0u8; 86];
        self.reader.read_exact(&mut payload)?;
        let id = payload[0];
        let length = payload[1] as usize;
        let name = fixed_string(&payload[2..6]);
        let encoding = fixed_string(&payload[6..22]);
        let labels = fixed_string(&payload[22..86])
            .split(',')
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let format = self.schema_provider.format(id, &name, &encoding, &labels)?;
        if length != payload_size(&format.fields) + 3 {
            return Err(ParseError::InvalidRecord {
                message: "FMT".to_owned(),
            });
        }
        self.formats.insert(id, format.clone());
        let values = decode_values(&format.fields, &payload)?;
        Ok(Some(LogEntry {
            id: format.id,
            name: format.name.clone(),
            fields: parsed_fields(&format.fields, &values),
            timestamp: None,
            values,
        }))
    }
}

impl<R: Read, M: ArduPilotMessage> Iterator for DataFlashReader<R, M> {
    type Item = Result<LogEntry, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_record().transpose()
    }
}

#[cfg(test)]
mod tests {
    use crate::DefaultArduPilotMessage;

    use super::{DataFlashReader, LogValue};
    use std::io::Cursor;

    fn fmt_record(
        message_type: u8,
        length: u8,
        name: &[u8],
        format: &[u8],
        labels: &[u8],
    ) -> Vec<u8> {
        let mut record = vec![0xA3, 0x95, 0x80];
        let mut payload = [0u8; 86];
        payload[0] = message_type;
        payload[1] = length;
        payload[2..2 + name.len()].copy_from_slice(name);
        payload[6..6 + format.len()].copy_from_slice(format);
        payload[22..22 + labels.len()].copy_from_slice(labels);
        record.extend_from_slice(&payload);
        record
    }

    #[test]
    fn parses_fmt_and_data_records() {
        let mut data = fmt_record(
            0x80,
            89,
            b"FMT\0",
            b"BBnNZ",
            b"Type,Length,Name,Format,Labels",
        );
        data.extend(fmt_record(0x81, 11, b"TST\0", b"Q", b"TimeUS"));
        data.extend_from_slice(&[0xA3, 0x95, 0x81]);
        data.extend_from_slice(&42u64.to_le_bytes());

        let mut reader = DataFlashReader::new_with_message_type(
            Cursor::new(data),
            DefaultArduPilotMessage::default(),
        );

        assert_eq!(reader.next().unwrap().unwrap().name, "FMT");
        assert_eq!(reader.next().unwrap().unwrap().name, "TST");
        let entry = reader.next().unwrap().unwrap();

        assert_eq!(entry.name, "TST");
        assert_eq!(entry.timestamp, Some(std::time::Duration::from_micros(42)));
        assert_eq!(entry.get("TimeUS"), Some(&LogValue::U64(42)));
        assert!(reader.formats().contains_key(&0x81));
        assert!(reader.next().is_none());
    }
}
