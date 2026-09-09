use crate::types::FormatField;

use super::*;

pub fn field_type(code: char, label: Option<&str>) -> Result<FieldType, ParseError> {
    let field_type = match code {
        'b' => FieldType::I8,
        'B' => FieldType::U8,
        'h' => FieldType::I16,
        'H' => FieldType::U16,
        'i' => FieldType::I32,
        'I' => FieldType::U32,
        'q' => FieldType::I64,
        'Q' => FieldType::U64,
        'f' => FieldType::F32,
        'd' => FieldType::F64,
        'L' if matches!(label, Some("Lat" | "Latitude")) => FieldType::Latitude,
        'L' if matches!(label, Some("Lng" | "Lon" | "Longitude")) => FieldType::Longitude,
        'L' => FieldType::I32,
        'M' => FieldType::FlightMode,
        'c' => FieldType::ScaledI16(100.0),
        'C' => FieldType::ScaledI16(100.0),
        'e' => FieldType::ScaledI32(100.0),
        'E' => FieldType::ScaledI32(100.0),
        'n' => FieldType::String(4),
        'N' => FieldType::String(16),
        'Z' => FieldType::String(64),
        _ => {
            return Err(ParseError::InvalidFormat {
                message: String::new(),
                format: code.to_string(),
            });
        }
    };
    Ok(field_type)
}

pub fn timestamp_type(encoding: &str, labels: &[String]) -> Option<Duration> {
    match (encoding.chars().next(), labels.first().map(String::as_str)) {
        (Some('Q'), _) | (Some('I'), Some("TimeUS")) => Some(Duration::from_micros(0)),
        (Some('I'), Some("TimeMS")) => Some(Duration::from_millis(0)),
        _ => None,
    }
}

pub fn read_exact_or_eof<R: Read>(reader: &mut R, buffer: &mut [u8]) -> Result<bool, ParseError> {
    match reader.read(&mut buffer[..1])? {
        0 => Ok(false),
        _ => {
            reader.read_exact(&mut buffer[1..])?;
            Ok(true)
        }
    }
}

pub fn fixed_string(bytes: &[u8]) -> String {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).trim_end().to_owned()
}

pub fn field_size(field_type: &FieldType) -> usize {
    match field_type {
        FieldType::I8 | FieldType::U8 | FieldType::FlightMode => 1,
        FieldType::I16 | FieldType::U16 | FieldType::ScaledI16(_) => 2,
        FieldType::I32
        | FieldType::U32
        | FieldType::F32
        | FieldType::Latitude
        | FieldType::Longitude
        | FieldType::ScaledI32(_) => 4,
        FieldType::I64 | FieldType::U64 | FieldType::F64 => 8,
        FieldType::String(length) => *length,
    }
}

pub fn payload_size(fields: &[FormatField]) -> usize {
    fields.iter().map(|field| field_size(&field.ty)).sum()
}

pub fn decode_values(fields: &[FormatField], payload: &[u8]) -> Result<Vec<LogValue>, ParseError> {
    let mut offset = 0;
    fields
        .iter()
        .map(|field| {
            let size = field_size(&field.ty);
            let bytes = payload
                .get(offset..offset + size)
                .ok_or(ParseError::UnexpectedEof)?;
            offset += size;
            Ok(decode_value(&field.ty, bytes))
        })
        .collect()
}

pub fn parsed_fields(fields: &[FormatField], values: &[LogValue]) -> Vec<Field> {
    fields
        .iter()
        .zip(values.iter().cloned())
        .map(|(field, value)| Field {
            name: field.name.clone(),
            value,
        })
        .collect()
}

pub fn decode_value(field_type: &FieldType, bytes: &[u8]) -> LogValue {
    match field_type {
        FieldType::I8 => LogValue::I8(bytes[0] as i8),
        FieldType::U8 => LogValue::U8(bytes[0]),
        FieldType::I16 => LogValue::I16(i16::from_le_bytes([bytes[0], bytes[1]])),
        FieldType::U16 => LogValue::U16(u16::from_le_bytes([bytes[0], bytes[1]])),
        FieldType::I32 => LogValue::I32(i32::from_le_bytes(bytes.try_into().expect("field size"))),
        FieldType::U32 => LogValue::U32(u32::from_le_bytes(bytes.try_into().expect("field size"))),
        FieldType::I64 => LogValue::I64(i64::from_le_bytes(bytes.try_into().expect("field size"))),
        FieldType::U64 => LogValue::U64(u64::from_le_bytes(bytes.try_into().expect("field size"))),
        FieldType::F32 => LogValue::F32(f32::from_le_bytes(bytes.try_into().expect("field size"))),
        FieldType::F64 => LogValue::F64(f64::from_le_bytes(bytes.try_into().expect("field size"))),
        FieldType::Latitude => {
            LogValue::Latitude(i32::from_le_bytes(bytes.try_into().expect("field size")))
        }
        FieldType::Longitude => {
            LogValue::Longitude(i32::from_le_bytes(bytes.try_into().expect("field size")))
        }
        FieldType::FlightMode => LogValue::FlightMode(bytes[0]),
        FieldType::ScaledI16(scale) => {
            LogValue::Scaled(i16::from_le_bytes([bytes[0], bytes[1]]) as f64 / scale)
        }
        FieldType::ScaledI32(scale) => LogValue::Scaled(
            i32::from_le_bytes(bytes.try_into().expect("field size")) as f64 / scale,
        ),
        FieldType::String(_) => LogValue::String(fixed_string(bytes)),
    }
}

pub fn extract_timestamp(format: &LogFormat, payload: &[u8]) -> Option<Duration> {
    match format.timestamp {
        Some(_)
            if matches!(
                format.fields.first().map(|field| &field.ty),
                Some(FieldType::U64)
            ) =>
        {
            Some(Duration::from_micros(u64::from_le_bytes(
                payload[..8].try_into().ok()?,
            )))
        }
        Some(_) => Some(Duration::from_millis(
            u32::from_le_bytes(payload[..4].try_into().ok()?) as u64,
        )),
        None => None,
    }
}
