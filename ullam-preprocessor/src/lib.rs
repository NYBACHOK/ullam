use std::{collections::BTreeMap, time::Duration};

use serde::{Deserialize, Serialize};
use ullam_parser::{LogEntry, LogValue};

mod config;

pub use self::config::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NumericStats {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub stddev: f64,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventCode {
    #[serde(rename = "HIGH_ANGULAR_RATE")]
    HighAngularRate,
    #[serde(rename = "BROWNOUT")]
    Brownout,
    #[serde(rename = "SIGNAL_LOSS")]
    SignalLoss,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventDetails {
    HighAngularRate { field: String, rate: f64 },
    Brownout { field: String, voltage: f64 },
    SignalLoss { field: String, value: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum IrRecord {
    #[serde(rename = "WINDOW")]
    Window {
        ts: f64,
        msg: String,
        stats: BTreeMap<String, NumericStats>,
        duration_s: f64,
    },
    #[serde(rename = "EVENT")]
    Event {
        ts: f64,
        msg: String,
        code: EventCode,
        details: EventDetails,
    },
    #[serde(rename = "SNAPSHOT")]
    Snapshot {
        ts: f64,
        msg: String,
        fields: BTreeMap<String, LogValue>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PreprocessedLog {
    pub records: Vec<IrRecord>,
}

#[derive(Debug, Clone)]
struct StatsAccumulator {
    values: Vec<f64>,
}

impl StatsAccumulator {
    fn add(&mut self, value: f64) {
        self.values.push(value);
    }

    fn finish(&self) -> NumericStats {
        let count = self.values.len();
        let min = self.values.iter().copied().fold(f64::INFINITY, f64::min);
        let max = self
            .values
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let mean = self.values.iter().sum::<f64>() / count as f64;
        let variance = self
            .values
            .iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>()
            / count as f64;

        NumericStats {
            min,
            max,
            mean,
            stddev: variance.sqrt(),
            count,
        }
    }
}

pub fn process(source: impl Iterator<Item = LogEntry>) -> PreprocessedLog {
    process_with_config(source, PreprocessorConfig::default())
}

pub fn process_with_config(
    source: impl Iterator<Item = LogEntry>,
    config: PreprocessorConfig,
) -> PreprocessedLog {
    assert!(config.window_duration > Duration::ZERO);
    let mut windows: BTreeMap<(u128, String), BTreeMap<String, StatsAccumulator>> = BTreeMap::new();
    let mut records = Vec::new();

    for entry in source {
        if config
            .formatting_messages
            .iter()
            .any(|message| message == &entry.name)
        {
            continue;
        }

        let timestamp = entry.timestamp.unwrap_or_default();
        let timestamp_s = timestamp.as_secs_f64();
        let window_start = timestamp.as_nanos() / config.window_duration.as_nanos();
        let key = (window_start, entry.name.clone());
        let window = windows.entry(key).or_default();

        for field in &entry.fields {
            if let Some(value) = match &field.value {
                LogValue::I64(value) => Some(*value as f64),
                LogValue::U64(value) => Some(*value as f64),
                LogValue::F64(value)
                | LogValue::Altitude(value)
                | LogValue::ScaledI16(value)
                | LogValue::ScaledI32(value) => Some(*value),
                LogValue::Latitude(value) | LogValue::Longitude(value) => Some(*value as f64),
                LogValue::Time(value) => Some(value.as_secs_f64()),
                LogValue::FlightMode(value) => Some(*value as f64),
                LogValue::Array(_) | LogValue::String(_) => None,
            } {
                window
                    .entry(field.name.clone())
                    .or_insert_with(|| StatsAccumulator { values: Vec::new() })
                    .add(value);
                detect_events(&mut records, &entry, field, value, timestamp_s, &config);
            }
        }

        if config
            .gps_messages
            .iter()
            .any(|message| message == &entry.name)
        {
            let fields = entry
                .fields
                .iter()
                .map(|field| (field.name.clone(), field.value.clone()))
                .collect();
            records.push(IrRecord::Snapshot {
                ts: timestamp_s,
                msg: entry.name.clone(),
                fields,
            });
        }
    }

    for ((window_start, msg), fields) in windows {
        let stats = fields
            .into_iter()
            .map(|(name, values)| (name, values.finish()))
            .collect();
        let duration_s = config.window_duration.as_secs_f64();
        records.push(IrRecord::Window {
            ts: window_start as f64 * duration_s,
            msg,
            stats,
            duration_s,
        });
    }

    records.sort_by(|left, right| record_timestamp(left).total_cmp(&record_timestamp(right)));
    PreprocessedLog { records }
}

fn detect_events(
    records: &mut Vec<IrRecord>,
    entry: &LogEntry,
    field: &ullam_parser::LogField,
    value: f64,
    timestamp: f64,
    config: &PreprocessorConfig,
) {
    if config
        .angular_rate_fields
        .iter()
        .any(|name| name == &field.name)
        && value.abs() > config.angular_rate_threshold
    {
        records.push(IrRecord::Event {
            ts: timestamp,
            msg: entry.name.clone(),
            code: EventCode::HighAngularRate,
            details: EventDetails::HighAngularRate {
                field: field.name.clone(),
                rate: value,
            },
        });
    }
    if config
        .battery_voltage_fields
        .iter()
        .any(|name| name == &field.name)
        && value < config.brownout_voltage_threshold
    {
        records.push(IrRecord::Event {
            ts: timestamp,
            msg: entry.name.clone(),
            code: EventCode::Brownout,
            details: EventDetails::Brownout {
                field: field.name.clone(),
                voltage: value,
            },
        });
    }
    if field.name == config.rc_channel_field && value == config.signal_loss_value {
        records.push(IrRecord::Event {
            ts: timestamp,
            msg: entry.name.clone(),
            code: EventCode::SignalLoss,
            details: EventDetails::SignalLoss {
                field: field.name.clone(),
                value,
            },
        });
    }
}

fn record_timestamp(record: &IrRecord) -> f64 {
    match record {
        IrRecord::Window { ts, .. }
        | IrRecord::Event { ts, .. }
        | IrRecord::Snapshot { ts, .. } => *ts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(timestamp: u64, name: &str, fields: Vec<(&str, LogValue)>) -> LogEntry {
        LogEntry {
            id: 1,
            name: name.to_owned(),
            timestamp: Some(Duration::from_secs(timestamp)),
            fields: fields
                .into_iter()
                .map(|(name, value)| ullam_parser::LogField {
                    name: name.to_owned(),
                    value,
                })
                .collect(),
        }
    }

    #[test]
    fn aggregates_windows_and_detects_events() {
        let result = process(
            [
                entry(0, "IMU", vec![("GyrY", LogValue::F64(10.0))]),
                entry(0, "RATE", vec![("GyrY", LogValue::F64(200.0))]),
                entry(0, "BAT", vec![("Volt", LogValue::F64(10.0))]),
                entry(0, "RCIN", vec![("C1", LogValue::U64(0))]),
            ]
            .into_iter(),
        );

        assert!(result.records.iter().any(|record| matches!(
            record,
            IrRecord::Event {
                code: EventCode::HighAngularRate,
                ..
            }
        )));
        assert!(result.records.iter().any(|record| matches!(
            record,
            IrRecord::Event {
                code: EventCode::Brownout,
                ..
            }
        )));
        assert!(result.records.iter().any(|record| matches!(
            record,
            IrRecord::Event {
                code: EventCode::SignalLoss,
                ..
            }
        )));
        assert!(result.records.iter().any(|record| matches!(
			record,
			IrRecord::Window { stats, .. } if stats.get("GyrY").is_some_and(|value| value.mean == 10.0)
		)));
    }

    #[test]
    fn filters_formatting_messages_but_keeps_string_fields_and_events() {
        let result = process(
            [
                entry(
                    0,
                    "FMTU",
                    vec![
                        ("FmtType", LogValue::U64(1)),
                        ("GyrY", LogValue::F64(300.0)),
                    ],
                ),
                entry(
                    0,
                    "GPS",
                    vec![
                        ("Status", LogValue::String("3D_FIX".to_owned())),
                        ("GyrY", LogValue::F64(200.0)),
                    ],
                ),
            ]
            .into_iter(),
        );

        assert!(!result.records.iter().any(|record| matches!(
            record,
            IrRecord::Window { msg, .. } if msg == "FMTU"
        )));
        assert!(!result.records.iter().any(|record| matches!(
            record,
            IrRecord::Event { msg, .. } if msg == "FMTU"
        )));
        assert!(result.records.iter().any(|record| matches!(
            record,
            IrRecord::Event {
                msg,
                code: EventCode::HighAngularRate,
                ..
            } if msg == "GPS"
        )));
        assert!(result.records.iter().any(|record| matches!(
            record,
            IrRecord::Snapshot { fields, .. }
                if fields.get("Status") == Some(&LogValue::String("3D_FIX".to_owned()))
        )));
    }
}
