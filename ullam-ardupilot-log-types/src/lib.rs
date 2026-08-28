#![doc = "Generated binrw and serde bindings for ArduPilot DataFlash log messages."]
#![allow(non_snake_case)]

mod fixed_bytes {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S, const N: usize>(value: &[u8; N], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value.as_slice().serialize(serializer)
    }

    pub fn deserialize<'de, D, const N: usize>(deserializer: D) -> Result<[u8; N], D::Error>
    where
        D: Deserializer<'de>,
    {
        let value =
            serde_json::Value::deserialize(deserializer).map_err(serde::de::Error::custom)?;
        let bytes = match value {
            serde_json::Value::String(value) => value.into_bytes(),
            serde_json::Value::Array(values) => values
                .into_iter()
                .map(|value| {
                    value
                        .as_u64()
                        .ok_or_else(|| {
                            serde::de::Error::custom("fixed byte array contains a non-byte value")
                        })
                        .and_then(|value| {
                            u8::try_from(value).map_err(|_| {
                                serde::de::Error::custom(
                                    "fixed byte array contains an out-of-range value",
                                )
                            })
                        })
                })
                .collect::<Result<Vec<_>, _>>()?,
            _ => return Err(serde::de::Error::custom("expected fixed bytes or a string")),
        };
        if bytes.len() > N {
            return Err(serde::de::Error::custom("fixed byte value is too long"));
        }
        let mut result = [0; N];
        result[..bytes.len()].copy_from_slice(&bytes);
        Ok(result)
    }
}

include!(concat!(env!("OUT_DIR"), "/messages.rs"));

#[cfg(test)]
mod tests {
    use super::ACC;
    use binrw::{BinRead, BinWrite};
    use std::io::Cursor;

    #[test]
    fn generated_message_round_trips_with_binrw_and_serde() {
        let value = ACC {
            TimeUS: 42,
            I: 1,
            SampleUS: 41,
            AccX: 1.0,
            AccY: -2.0,
            AccZ: 9.81,
        };
        let mut bytes = Cursor::new(Vec::new());
        value.write_le(&mut bytes).unwrap();
        assert_eq!(
            ACC::read_le(&mut Cursor::new(bytes.into_inner())).unwrap(),
            value
        );

        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(serde_json::from_str::<ACC>(&json).unwrap(), value);
    }
}
