mod providers;
mod types;

use std::{io::Read, time::Duration};

use ardupilot_binlog::FieldValue;

pub use self::{providers::*, types::*};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error(
        "schema provider found invalid relation to field label({label}) and its value: {value:#?}"
    )]
    InvalidLabelType { label: String, value: FieldValue },
    #[error(transparent)]
    Binlog(#[from] ardupilot_binlog::BinlogError),
}

pub struct LogReader<R: Read> {
    inner: ardupilot_binlog::Reader<R>,
    providers: Vec<Box<dyn MessageSchemaProvider>>,
}

impl<R: Read> LogReader<R> {
    pub fn new(reader: R) -> Self {
        let mut providers: Vec<Box<dyn MessageSchemaProvider>> = Vec::new();

        // Register specialized providers first
        providers.push(Box::new(ImuSchemaProvider));
        providers.push(Box::new(EkfSchemaProvider));
        providers.push(Box::new(GpsSchemaProvider));
        providers.push(Box::new(ControlSchemaProvider));
        providers.push(Box::new(RcSchemaProvider));

        // Register the fallback parser last
        providers.push(Box::new(CommonSchemaProvider));

        Self {
            inner: ardupilot_binlog::Reader::new(reader),
            providers,
        }
    }

    /// Parses the next entry from the log, applying the correct schema provider.
    pub fn next_entry(&mut self) -> Option<Result<LogEntry, ParseError>> {
        let raw_entry = match self.inner.next() {
            Some(Ok(e)) => e,
            Some(Err(e)) => return Some(Err(ParseError::Binlog(e))),
            None => return None,
        };

        // Find the appropriate provider based on message name
        let provider = self
            .providers
            .iter()
            .find(|p| p.handles_message(&raw_entry.name))
            .expect("CommonMessageParser should always match");

        let mut fields = Vec::new();

        for (label, value) in raw_entry.labels().iter().zip(raw_entry.values().iter()) {
            let formatted_field = match provider.format_field(label.clone(), value.clone()) {
                Ok(f) => f,
                Err(e) => return Some(Err(e)),
            };
            fields.push(formatted_field);
        }

        let entry = LogEntry {
            id: raw_entry.msg_type,
            name: raw_entry.name,
            timestamp: raw_entry.timestamp_usec.map(Duration::from_micros),
            fields,
        };

        Some(Ok(entry))
    }
}

impl<R: Read> Iterator for LogReader<R> {
    type Item = Result<LogEntry, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_entry()
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use crate::LogReader;

    const EXAMPLE_LOG_FILE: &str = "../assets/example_logs.BIN";

    #[test]
    fn test() {
        use std::path::PathBuf;

        let log_file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(EXAMPLE_LOG_FILE);

        let mut reader = LogReader::new(File::open(log_file).expect("failed to open log file"));

        assert!(!reader.any(|this| this.is_err()))

        // let debvug = File::create("/home/ghuba/debug.json").unwrap();
        // serde_json::ser::to_writer_pretty(
        //     debvug,
        //     &reader
        //         .into_iter()
        //         .inspect(|this| {
        //             if let Err(this) = this {
        //                 eprintln!("{this}");
        //             }
        //         })
        //         .filter_map(|this| this.ok())
        //         .collect::<Vec<_>>(),
        // )
        // .unwrap();
    }
}
