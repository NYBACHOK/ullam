#![doc = "Generated binrw and serde bindings for ArduPilot DataFlash log messages."]
#![allow(non_snake_case)]

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
